//! Daily Reset Service (Simplified Version)
//!
//! Core service for managing daily session reset functionality without database dependencies.
//! This version provides the essential business logic for timezone-aware reset scheduling.

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;

use crate::models::{
    user_configuration::{UserConfiguration, DailyResetTimeType},
    daily_session_stats::DailySessionStats,
    session_reset_event::SessionResetEvent,
};
use crate::services::time_provider::TimeProvider;
use crate::database::{DatabaseManager, connection::DatabasePool};
use crate::error::AppError;
use sqlx::Row;

use tracing::{debug, info, warn, error, instrument};

/// Parse time string (HH:MM) to naive time components
fn parse_time_to_naive_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;

    if hour > 23 || minute > 59 {
        return None;
    }

    Some((hour, minute))
}

/// Daily Reset Service
///
/// Provides timezone-aware daily session reset functionality with database persistence.
pub struct DailyResetService {
    /// Time provider for deterministic testing
    time_provider: Arc<dyn TimeProvider>,
    /// Database manager for persistence
    database_manager: Arc<DatabaseManager>,
}

impl DailyResetService {
    /// Create a new daily reset service
    pub fn new(
        time_provider: Arc<dyn TimeProvider>,
        database_manager: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            time_provider,
            database_manager,
        }
    }

    /// Calculate the next daily reset time for a user configuration
    #[instrument(skip(self, user_config))]
    pub fn calculate_next_reset_time(
        &self,
        user_config: &UserConfiguration,
    ) -> Result<DateTime<Utc>, AppError> {
        if !user_config.daily_reset_enabled {
            return Ok(Utc::now()); // Return current time if disabled
        }

        let current_time = self.time_provider.now_utc();
        let user_timezone: Tz = user_config.timezone.parse()
            .map_err(|e| AppError::UserConfiguration(
                crate::models::user_configuration::UserConfigurationError::InvalidTimezone(user_config.timezone.clone())
            ))?;

        // Get current time in user's timezone
        let current_local = current_time.with_timezone(&user_timezone);
        let current_date = current_local.date_naive();

        // Calculate reset time for today
        let reset_time = match user_config.daily_reset_time_type {
            DailyResetTimeType::Midnight => {
                current_date.and_hms_opt(0, 0, 0)
            }
            DailyResetTimeType::Hour => {
                if let Some(hour) = user_config.daily_reset_time_hour {
                    current_date.and_hms_opt(hour as u32, 0, 0)
                } else {
                    current_date.and_hms_opt(0, 0, 0) // Default to midnight
                }
            }
            DailyResetTimeType::Custom => {
                if let Some(ref time_str) = user_config.daily_reset_time_custom {
                    parse_time_to_naive_time(time_str)
                        .map(|(hour, minute)| current_date.and_hms_opt(hour, minute, 0))
                        .unwrap_or_else(|| current_date.and_hms_opt(0, 0, 0))
                } else {
                    current_date.and_hms_opt(0, 0, 0)
                }
            }
        };

        let reset_local = user_timezone.from_local_datetime(&reset_time.unwrap())
            .single()
            .ok_or_else(|| {
                warn!("Failed to create local datetime for reset time");
                AppError::UserConfiguration(
                    crate::models::user_configuration::UserConfigurationError::InvalidResetTime("Invalid time".to_string())
                )
            })?;

        // If the reset time has already passed today, calculate for tomorrow
        if reset_local.timestamp() <= current_time.timestamp() {
            let tomorrow_date = current_local.date_naive().succ_opt()
                .ok_or_else(|| {
                    warn!("Failed to get tomorrow date");
                    AppError::UserConfiguration(
                        crate::models::user_configuration::UserConfigurationError::InvalidResetTime("Date calculation failed".to_string())
                    )
                })?;

            let tomorrow_reset_time = match user_config.daily_reset_time_type {
                DailyResetTimeType::Midnight => {
                    tomorrow_date.and_hms_opt(0, 0, 0)
                }
                DailyResetTimeType::Hour => {
                    if let Some(hour) = user_config.daily_reset_time_hour {
                        tomorrow_date.and_hms_opt(hour as u32, 0, 0)
                    } else {
                        tomorrow_date.and_hms_opt(0, 0, 0)
                    }
                }
                DailyResetTimeType::Custom => {
                    if let Some(ref time_str) = user_config.daily_reset_time_custom {
                        parse_time_to_naive_time(time_str)
                            .map(|(hour, minute)| tomorrow_date.and_hms_opt(hour, minute, 0))
                            .unwrap_or_else(|| tomorrow_date.and_hms_opt(0, 0, 0))
                    } else {
                        tomorrow_date.and_hms_opt(0, 0, 0)
                    }
                }
            };

            let tomorrow_local = user_timezone.from_local_datetime(&tomorrow_reset_time.unwrap())
                .single()
                .ok_or_else(|| {
                    warn!("Failed to create local datetime for tomorrow");
                    AppError::UserConfiguration(
                        crate::models::user_configuration::UserConfigurationError::InvalidResetTime("Tomorrow time calculation failed".to_string())
                    )
                })?;

            Ok(tomorrow_local.with_timezone(&Utc))
        } else {
            Ok(reset_local.with_timezone(&Utc))
        }
    }

    /// Check if a daily reset is needed for the given configuration
    #[instrument(skip(self, user_config))]
    pub fn should_reset_today(
        &self,
        user_config: &UserConfiguration,
    ) -> Result<bool, AppError> {
        if !user_config.daily_reset_enabled {
            return Ok(false);
        }

        // Check if last reset was today
        if let Some(last_reset_utc) = user_config.last_daily_reset_utc {
            let last_reset = DateTime::from_timestamp(last_reset_utc as i64, 0)
                .ok_or_else(|| {
                    AppError::UserConfiguration(
                        crate::models::user_configuration::UserConfigurationError::InvalidResetTime(format!("Invalid timestamp: {}", last_reset_utc))
                    )
                })?;

            let user_timezone: Tz = user_config.timezone.parse()
                .map_err(|_| {
                    crate::models::user_configuration::UserConfigurationError::InvalidTimezone(user_config.timezone.clone())
                })?;

            let last_reset_local = last_reset.with_timezone(&user_timezone);
            let current_local = self.time_provider.now_utc().with_timezone(&user_timezone);

            // Check if last reset was on a different day
            Ok(last_reset_local.date_naive() != current_local.date_naive())
        } else {
            // No previous reset, so reset is needed
            Ok(true)
        }
    }

    /// Get current session count for a user configuration
    pub fn get_current_session_count(&self, user_config: &UserConfiguration) -> u32 {
        user_config.manual_session_override
            .or(Some(user_config.today_session_count))
            .unwrap_or(0)
    }

    /// Validate timezone string
    pub fn validate_timezone(&self, timezone: &str) -> Result<(), AppError> {
        timezone.parse::<Tz>()
            .map_err(|_| {
                AppError::UserConfiguration(
                    crate::models::user_configuration::UserConfigurationError::InvalidTimezone(timezone.to_string())
                )
            })?;
        Ok(())
    }

    // ===== Database Operations =====

    /// Perform a complete daily session reset for a user configuration
    /// This is the main method that orchestrates the entire reset process
    #[instrument(skip(self, user_config))]
    pub async fn perform_daily_reset(&self, user_config: &UserConfiguration) -> Result<SessionResetEvent, AppError> {
        let current_time = self.time_provider.now_utc();

        info!("Starting daily session reset for user {}", user_config.id);

        // 1. Get current session count before reset
        let previous_session_count = self.get_current_session_count(user_config);

        // 2. Save today's session stats to database
        let session_stats = self.save_daily_session_stats(user_config, current_time).await?;

        // 3. Reset user configuration in database
        self.reset_user_configuration(user_config, current_time).await?;

        // 4. Create reset event for audit trail
        let reset_event = self.create_reset_event(user_config, previous_session_count, session_stats, current_time).await?;

        info!("Daily session reset completed successfully for user {}", user_config.id);

        Ok(reset_event)
    }

    /// Save today's session statistics to the database
    #[instrument(skip(self, user_config))]
    async fn save_daily_session_stats(&self, user_config: &UserConfiguration, reset_time: DateTime<Utc>) -> Result<DailySessionStats, AppError> {
        let today_date = reset_time.date_naive().to_string();
        let user_timezone: Tz = user_config.timezone.parse()
            .map_err(|_e| AppError::UserConfiguration(
                crate::models::user_configuration::UserConfigurationError::InvalidTimezone(user_config.timezone.clone())
            ))?;

        // Check if stats already exist for today
        let existing_stats = self.get_daily_session_stats(&user_config.id, &today_date).await?;

        if let Some(mut stats) = existing_stats {
            // Update existing stats
            stats.work_sessions_completed = user_config.today_session_count as i64;
            stats.total_work_seconds = (user_config.today_session_count * user_config.work_duration) as i64; // Estimate
            stats.manual_overrides = user_config.manual_session_override.unwrap_or(0) as i64;
            stats.updated_at = reset_time.timestamp();

            // Update in database
            self.update_daily_session_stats(&stats).await?;

            info!("Updated existing daily session stats for user {} on {}", user_config.id, today_date);
            Ok(stats)
        } else {
            // Create new stats
            let stats = DailySessionStats::new(
                user_config.id.clone(),
                today_date.clone(),
                user_timezone.to_string(),
            );

            // Save to database
            let saved_stats = self.insert_daily_session_stats(&stats).await?;

            info!("Created new daily session stats for user {} on {}", user_config.id, today_date);
            Ok(saved_stats)
        }
    }

    /// Get daily session stats for a specific date
    async fn get_daily_session_stats(&self, user_id: &str, date: &str) -> Result<Option<DailySessionStats>, AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        let row = sqlx::query(
            r#"
            SELECT id, user_id, date, timezone, work_sessions_completed,
                   total_work_seconds, total_break_seconds, manual_overrides,
                   created_at, updated_at
            FROM daily_session_stats
            WHERE user_id = ? AND date = ?
            "#
        )
        .bind(user_id)
        .bind(date)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        match row {
            Some(row) => {
                let stats = DailySessionStats {
                    id: row.get("id"),
                    user_configuration_id: row.get("user_configuration_id"),
                    date: row.get("date"),
                    timezone: row.get("timezone"),
                    work_sessions_completed: row.get("work_sessions_completed"),
                    total_work_seconds: row.get("total_work_seconds"),
                    total_break_seconds: row.get("total_break_seconds"),
                    manual_overrides: row.get("manual_overrides"),
                    final_session_count: row.get("final_session_count"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(stats))
            }
            None => Ok(None),
        }
    }

    /// Insert new daily session stats
    async fn insert_daily_session_stats(&self, stats: &DailySessionStats) -> Result<DailySessionStats, AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        let _ = sqlx::query(
            r#"
            INSERT INTO daily_session_stats (
                id, user_configuration_id, date, timezone, work_sessions_completed,
                total_work_seconds, total_break_seconds, manual_overrides,
                final_session_count, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&stats.id)
        .bind(&stats.user_configuration_id)
        .bind(&stats.date)
        .bind(&stats.timezone)
        .bind(stats.work_sessions_completed)
        .bind(stats.total_work_seconds)
        .bind(stats.total_break_seconds)
        .bind(stats.manual_overrides)
        .bind(stats.final_session_count)
        .bind(stats.created_at)
        .bind(stats.updated_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        debug!("Inserted daily session stats with ID: {}", stats.id);
        Ok(stats.clone())
    }

    /// Update existing daily session stats
    async fn update_daily_session_stats(&self, stats: &DailySessionStats) -> Result<(), AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        sqlx::query(
            r#"
            UPDATE daily_session_stats
            SET work_sessions_completed = ?, total_work_seconds = ?,
                total_break_seconds = ?, manual_overrides = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(stats.work_sessions_completed)
        .bind(stats.total_work_seconds)
        .bind(stats.total_break_seconds)
        .bind(stats.manual_overrides)
        .bind(stats.updated_at)
        .bind(&stats.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        debug!("Updated daily session stats with ID: {}", stats.id);
        Ok(())
    }

    /// Reset user configuration session counts
    async fn reset_user_configuration(&self, user_config: &UserConfiguration, reset_time: DateTime<Utc>) -> Result<(), AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        let timestamp = reset_time.timestamp();

        sqlx::query(
            r#"
            UPDATE user_configurations
            SET today_session_count = 0, manual_session_override = NULL,
                last_daily_reset_utc = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(timestamp)
        .bind(timestamp)
        .bind(&user_config.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        info!("Reset user configuration session counts for user {}", user_config.id);
        Ok(())
    }

    /// Create a reset event for audit trail
    async fn create_reset_event(
        &self,
        user_config: &UserConfiguration,
        previous_session_count: u32,
        session_stats: DailySessionStats,
        reset_time: DateTime<Utc>,
    ) -> Result<SessionResetEvent, AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        let event = SessionResetEvent::scheduled_daily_reset(
            user_config.id.clone(),
            previous_session_count,
            reset_time,
            user_config.timezone.clone(),
        );

        sqlx::query(
            r#"
            INSERT INTO session_reset_events (
                id, user_configuration_id, reset_type, previous_count, new_count,
                reset_timestamp_utc, user_timezone, local_reset_time, trigger_source,
                context, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event.id)
        .bind(&event.user_configuration_id)
        .bind(&event.reset_type)
        .bind(event.previous_count)
        .bind(event.new_count)
        .bind(event.reset_timestamp_utc)
        .bind(&event.user_timezone)
        .bind(&event.local_reset_time)
        .bind(&event.trigger_source)
        .bind(&event.context)
        .bind(event.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        info!("Created reset event with ID: {} for user: {}", event.id, user_config.id);
        Ok(event)
    }

    /// Check if any users need daily reset and perform it
    /// This method should be called by the scheduled task
    #[instrument(skip(self))]
    pub async fn process_pending_daily_resets(&self) -> Result<Vec<SessionResetEvent>, AppError> {
        info!("Processing pending daily resets");

        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        // Find all users with daily reset enabled who need reset
        let rows = sqlx::query(
            r#"
            SELECT id, timezone, last_daily_reset_utc, daily_reset_time_type,
                   daily_reset_time_hour, daily_reset_time_custom, today_session_count
            FROM user_configurations
            WHERE daily_reset_enabled = 1
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let mut reset_events = Vec::new();

        for row in rows {
            let user_id: String = row.get("id");
            let timezone: String = row.get("timezone");
            let last_reset: Option<i64> = row.get("last_daily_reset_utc");
            let today_session_count: i32 = row.get("today_session_count");

            // Skip if no sessions today
            if today_session_count == 0 && last_reset.is_none() {
                continue;
            }

            // Check if reset is needed (simplified check - in production, use timezone-aware calculation)
            let current_time = self.time_provider.now_utc();
            let needs_reset = match last_reset {
                Some(last_reset_ts) => {
                    let last_reset = DateTime::from_timestamp(last_reset_ts, 0)
                        .ok_or_else(|| AppError::Database(sqlx::Error::Decode("Invalid timestamp".into())))?;
                    let hours_since_reset = current_time.signed_duration_since(last_reset).num_hours();
                    hours_since_reset >= 24
                }
                None => false, // No previous reset, but no sessions either
            };

            if needs_reset {
                info!("User {} needs daily reset", user_id);

                // Load full user configuration
                let user_config = self.load_user_configuration(&user_id).await?;

                // Perform reset
                match self.perform_daily_reset(&user_config).await {
                    Ok(reset_event) => {
                        reset_events.push(reset_event);
                    }
                    Err(e) => {
                        error!("Failed to perform daily reset for user {}: {}", user_id, e);
                        // Continue with other users
                    }
                }
            }
        }

        info!("Completed processing pending daily resets. Processed {} users.", reset_events.len());
        Ok(reset_events)
    }

    /// Load user configuration from database
    async fn load_user_configuration(&self, user_id: &str) -> Result<UserConfiguration, AppError> {
        let pool = match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        };

        let row = sqlx::query(
            r#"
            SELECT id, work_duration, short_break_duration, long_break_duration,
                   long_break_frequency, notifications_enabled, webhook_url,
                   wait_for_interaction, theme, timezone, daily_reset_time_type,
                   daily_reset_time_hour, daily_reset_time_custom, daily_reset_enabled,
                   last_daily_reset_utc, today_session_count, manual_session_override,
                   created_at, updated_at
            FROM user_configurations
            WHERE id = ?
            "#
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let user_config = UserConfiguration {
            id: row.get("id"),
            work_duration: row.get("work_duration"),
            short_break_duration: row.get("short_break_duration"),
            long_break_duration: row.get("long_break_duration"),
            long_break_frequency: row.get("long_break_frequency"),
            notifications_enabled: row.get("notifications_enabled"),
            webhook_url: row.get("webhook_url"),
            wait_for_interaction: row.get("wait_for_interaction"),
            theme: match row.get::<String, _>("theme").as_str() {
                "Dark" => crate::models::user_configuration::Theme::Dark,
                _ => crate::models::user_configuration::Theme::Light,
            },
            timezone: row.get("timezone"),
            daily_reset_time_type: match row.get::<String, _>("daily_reset_time_type").as_str() {
                "hour" => crate::models::user_configuration::DailyResetTimeType::Hour,
                "custom" => crate::models::user_configuration::DailyResetTimeType::Custom,
                _ => crate::models::user_configuration::DailyResetTimeType::Midnight,
            },
            daily_reset_time_hour: row.get("daily_reset_time_hour"),
            daily_reset_time_custom: row.get("daily_reset_time_custom"),
            daily_reset_enabled: row.get("daily_reset_enabled"),
            last_daily_reset_utc: row.get("last_daily_reset_utc"),
            today_session_count: row.get("today_session_count"),
            manual_session_override: row.get("manual_session_override"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::time_provider::MockTimeProvider;

    async fn create_test_service() -> Result<(DailyResetService, ()), Box<dyn std::error::Error>> {
        let time_provider = Arc::new(MockTimeProvider::new_from_now());
        let service = DailyResetService::new(time_provider);

        Ok((service, ()))
    }

    #[tokio::test]
    async fn test_timezone_validation() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _) = create_test_service().await?;

        // Valid timezone
        assert!(service.validate_timezone("UTC").is_ok());
        assert!(service.validate_timezone("America/New_York").is_ok());

        // Invalid timezone
        assert!(service.validate_timezone("Invalid/Timezone").is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_next_reset_time() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("UTC".to_string())?;
        config.set_daily_reset_time(DailyResetTime::midnight());
        config.set_daily_reset_enabled(true);

        let next_reset = service.calculate_next_reset_time(&config)?;
        // Should be midnight UTC tomorrow
        let expected = service.time_provider.now().date_naive().succ_opt()
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        // Allow some tolerance for time differences
        let diff = (next_reset.timestamp() - expected.timestamp()).abs();
        assert!(diff < 3600); // Within 1 hour

        Ok(())
    }

    #[tokio::test]
    async fn test_session_count_override() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_manual_session_override(Some(5))?;

        assert_eq!(service.get_current_session_count(&config), 5);

        Ok(())
    }
}