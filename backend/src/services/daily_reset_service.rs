//! Daily Reset Service
//!
//! Core service for managing daily session reset functionality including:
//! - Timezone-aware reset scheduling and execution
//! - Session count management and validation
//! - Background task coordination
//! - Analytics and audit trail
//! - Real-time synchronization

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::models::{
    user_configuration::UserConfiguration,
    user_configuration::DailyResetTime,
    daily_session_stats::DailySessionStats,
    scheduled_task::ScheduledTask,
    session_reset_event::SessionResetEvent,
    session_reset_event::SessionResetEventType,
    session_reset_event::SessionResetTriggerSource,
};
use crate::services::time_provider::TimeProvider;
// use crate::database::DailyResetDatabaseExtensions; // Temporarily disabled for compilation
use crate::error::AppError;

use tracing::{debug, info, warn, error, instrument};

/// Daily Reset Service
///
/// Provides comprehensive daily session reset functionality with timezone awareness
/// and real-time synchronization capabilities.
#[derive(Debug, Clone)]
pub struct DailyResetService {
    /// Time provider for deterministic testing
    time_provider: Arc<dyn TimeProvider>,
}

impl DailyResetService {
    /// Create a new daily reset service
    pub fn new(
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            time_provider,
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
            .map_err(|e| AppError::TimezoneValidation(format!("Invalid timezone: {}", e)))?;

        // Get current time in user's timezone
        let current_local = current_time.with_timezone(&user_timezone);
        let current_date = current_local.date_naive();

        // Calculate reset time for today
        let reset_time = match user_config.daily_reset_time_type {
            crate::models::DailyResetTimeType::Midnight => {
                current_date.and_hms_opt(0, 0, 0)
            }
            crate::models::DailyResetTimeType::Hour => {
                if let Some(hour) = user_config.daily_reset_time_hour {
                    current_date.and_hms_opt(hour as u32, 0, 0)
                } else {
                    current_date.and_hms_opt(0, 0, 0) // Default to midnight
                }
            }
            crate::models::DailyResetTimeType::Custom => {
                if let Some(ref time_str) = user_config.daily_reset_time_custom {
                    parse_time_to_naive_time(time_str)
                        .map(|(hour, minute)| current_date.and_hms_opt(hour, minute, 0))
                        .unwrap_or_else(|| current_date.and_hms_opt(0, 0, 0))
                } else {
                    current_date.and_hms_opt(0, 0, 0) // Default to midnight
                }
            }
        };

        let reset_local = user_timezone.from_local_datetime(&reset_time.unwrap())
            .ok_or_else(|| {
                warn!("Failed to create local datetime for timezone {}, using UTC", user_timezone);
                reset_time.unwrap().and_utc()
            });

        let reset_utc = reset_local.with_timezone(&Utc);

        // If reset time has passed today, schedule for tomorrow
        if reset_utc <= current_time {
            let tomorrow_date = current_date.succ_opt()
                .ok_or_else(|| {
                    warn!("Failed to get tomorrow date, using current date");
                    current_date
                });

            let tomorrow_reset_time = match user_config.daily_reset_time_type {
                crate::models::DailyResetTimeType::Midnight => {
                    tomorrow_date.and_hms_opt(0, 0, 0)
                }
                crate::models::DailyResetType::Hour => {
                    if let Some(hour) = user_config.daily_reset_time_hour {
                        tomorrow_date.and_hms_opt(hour as u32, 0, 0)
                    } else {
                        tomorrow_date.and_hms_opt(0, 0, 0)
                    }
                }
                crate::models::DailyResetTimeType::Custom => {
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
                .ok_or_else(|| {
                    warn!("Failed to create local datetime for tomorrow, using current");
                    tomorrow_reset_time.unwrap().and_utc()
                });

            return Ok(tomorrow_local.with_timezone(&Utc));
        }

        Ok(reset_utc)
    }

    /// Check if a daily reset is due for the given user configuration
    #[instrument(skip(self, user_config))]
    pub fn is_reset_due(
        &self,
        user_config: &UserConfiguration,
    ) -> bool {
        if !user_config.daily_reset_enabled {
            return false;
        }

        let current_time = self.time_provider.now_utc();
        user_config.is_daily_reset_due(current_time.timestamp() as u64)
    }

    /// Perform a daily session reset for a user
    #[instrument(skip(self, user_config))]
    pub async fn perform_daily_reset(
        &self,
        user_config: &UserConfiguration,
        trigger_source: SessionResetTriggerSource,
        device_id: Option<String>,
    ) -> Result<(), AppError> {
        let current_time = self.time_provider.now_utc();
        let session_count_before = user_config.get_current_session_count();

        info!(
            user_id = %user_config.id,
            previous_count = session_count_before,
            trigger_source = %trigger_source.display_name(),
            "Performing daily reset for user"
        );

        // Record the reset event
        let reset_event = SessionResetEvent::new(
            user_config.id.clone(),
            SessionResetEventType::ScheduledDaily,
            session_count_before,
            0, // Reset to zero
            current_time,
            user_config.timezone.clone(),
            trigger_source,
        ).with_device_id(device_id.unwrap_or_else(|| "system".to_string()));

        self.db_manager.record_session_reset_event(
            &crate::database::SessionResetEventData {
                id: reset_event.id,
                user_configuration_id: reset_event.user_configuration_id,
                reset_type: reset_event.reset_type.display_name().to_string(),
                previous_count: reset_event.previous_count as u32,
                new_count: reset_event.new_count as u32,
                reset_timestamp_utc: reset_event.reset_timestamp_utc,
                user_timezone: reset_event.user_timezone,
                local_reset_time: reset_event.local_reset_time,
                device_id: reset_event.device_id,
                trigger_source: reset_event.trigger_source.display_name().to_string(),
                context: None,
            },
        ).await?;

        // Update user configuration with reset session count
        let mut updated_config = user_config.clone();
        updated_config.reset_session_count();

        self.db_manager.save_user_configuration(&updated_config).await?;

        // Record daily statistics
        let today = current_time.format("%Y-%m-%d").to_string();
        let work_sessions_completed = session_count_before as u32;
        let total_work_seconds = work_sessions_completed * 25 * 60; // Assume 25 minutes per session

        self.db_manager.record_daily_session_stat(
            &user_config.id,
            &today,
            &user_config.timezone,
            work_sessions_completed,
            total_work_seconds,
            0, // No break time for simple reset
            0, // No manual overrides for scheduled reset
            0, // Final count is 0
        ).await?;

        // Send real-time notification
        self.send_reset_notification(&updated_config, session_count_before, 0).await?;

        info!(
            user_id = %user_config.id,
            "Daily reset completed successfully"
        );

        Ok(())
    }

    /// Increment session count for a user
    #[instrument(skip(self, user_config))]
    pub async fn increment_session_count(
        &self,
        user_config: &UserConfiguration,
    ) -> Result<u32, AppError> {
        let current_count = user_config.get_current_session_count();

        // Don't increment if there's a manual override
        if user_config.manual_session_override.is_some() {
            return Ok(current_count);
        }

        let new_count = current_count + 1;

        // Validate count bounds
        if new_count > 1000 {
            return Err(AppError::ManualSessionOverrideInvalid(
                format!("Session count {} exceeds maximum allowed (1000)", new_count)
            ));
        }

        // Update user configuration
        let mut updated_config = user_config.clone();
        updated_config.today_session_count = new_count;

        self.db_manager.save_user_configuration(&updated_config).await?;

        // Send real-time notification
        self.send_session_count_notification(&updated_config, current_count, new_count).await?;

        debug!(
            user_id = %user_config.id,
            "Session count incremented from {} to {}",
            current_count,
            new_count
        );

        Ok(new_count)
    }

    /// Set manual session override
    #[instrument(skip(self, user_config, override_count))]
    pub async fn set_manual_session_override(
        &self,
        user_config: &UserConfiguration,
        override_count: Option<u32>,
    ) -> Result<u32, AppError> {
        let current_count = user_config.get_current_session_count();

        // Validate override count bounds
        if let Some(count) = override_count {
            if count > 1000 {
                return Err(AppError::ManualSessionOverrideInvalid(
                    format!("Manual override count {} exceeds maximum allowed (1000)", count)
                ));
            }
        }

        // Update user configuration
        let mut updated_config = user_config.clone();
        updated_config.set_manual_session_override(override_count)?;

        self.db_manager.save_user_configuration(&updated_config).await?;

        let final_count = updated_config.get_current_session_count();

        // Send real-time notification if there was an override
        if override_count.is_some() && final_count != current_count {
            self.send_manual_override_notification(&updated_config, current_count, final_count).await?;
        }

        debug!(
            user_id = %user_config.id,
            "Manual session override set from {} to {}",
            current_count,
            final_count
        );

        Ok(final_count)
    }

    /// Get session count for a user
    #[instrument(skip(self, user_id))]
    pub async fn get_session_count(
        &self,
        user_id: &str,
    ) -> Result<u32, AppError> {
        let config = self.db_manager.get_user_configuration(user_id).await?
            .ok_or_else(|| AppError::ConfigurationNotFound)?;

        Ok(config.get_current_session_count())
    }

    /// Get daily statistics for a user
    #[instrument(skip(self, user_id, start_date, end_date))]
    pub async fn get_daily_statistics(
        &self,
        user_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<crate::database::DailySessionStatRow>, AppError> {
        let stats = self.db_manager
            .get_daily_session_stats(user_id, start_date, end_date)
            .await?;

        Ok(stats)
    }

    /// Get next reset time for a user
    #[instrument(skip(self, user_config))]
    pub async fn get_next_reset_time(
        &self,
        user_config: &UserConfiguration,
    ) -> Result<Option<DateTime<Utc>>, AppError> {
        if !user_config.daily_reset_enabled {
            return Ok(None);
        }

        let next_reset = user_config.get_next_reset_time_utc()
            .map(|ts| DateTime::from_timestamp(ts as i64, 0));

        Ok(next_reset)
    }

    /// Update daily reset configuration
    #[instrument(skip(self, user_config, reset_time, timezone))]
    pub async fn update_daily_reset_configuration(
        &self,
        user_config: &UserConfiguration,
        reset_time: DailyResetTime,
        timezone: &str,
    ) -> Result<(), AppError> {
        // Validate timezone
        timezone.parse::<Tz>()
            .map_err(|_| AppError::TimezoneValidation(timezone.to_string()))?;

        // Validate reset time
        reset_time.validate()?;

        // Update user configuration
        let mut updated_config = user_config.clone();
        updated_config.set_timezone(timezone.to_string())?;
        updated_config.set_daily_reset_time(reset_time)?;

        self.db_manager.save_user_configuration(&updated_config).await?;

        // If configuration changed significantly, record a reset event
        let current_count = updated_config.get_current_session_count();
        if current_count > 0 {
            let reset_event = SessionResetEvent::configuration_change_reset(
                user_config.id.clone(),
                current_count,
                self.time_provider.now_utc(),
                user_config.timezone.clone(),
                &serde_json::json!({
                    "old_timezone": user_config.timezone,
                    "new_timezone": timezone,
                    "old_reset_time": user_config.get_daily_reset_time().display_name(),
                    "new_reset_time": reset_time.display_name(),
                }),
            );

            self.db_manager.record_session_reset_event(
                &crate::database::SessionResetEventData {
                    id: reset_event.id,
                    user_configuration_id: reset_event.user_configuration_id,
                    reset_type: reset_event.reset_type.display_name().to_string(),
                    previous_count: reset_event.previous_count as u32,
                    new_count: reset_event.new_count as u32,
                    reset_timestamp_utc: reset_event.reset_timestamp_utc,
                    user_timezone: reset_event.user_timezone,
                    local_reset_time: reset_event.local_reset_time,
                    device_id: reset_event.device_id,
                    trigger_source: reset_event.trigger_source.display_name().to_string(),
                    context: reset_event.context,
                },
            ).await?;
        }

        // Send configuration update notification
        self.send_configuration_change_notification(&updated_config).await?;

        info!(
            user_id = %user_config.id,
            "Daily reset configuration updated"
        );

        Ok(())
    }

    /// Create and schedule a daily reset task
    #[instrument(skip(self, user_config))]
    pub async fn schedule_daily_reset_task(
        &self,
        user_config: &UserConfiguration,
    ) -> Result<String, AppError> {
        if !user_config.daily_reset_enabled {
            return Err(AppError::DailyResetScheduling(
                "Cannot schedule task: daily reset is disabled".to_string()
            ));
        }

        let task_id = format!("daily_reset_task_{}", user_config.id);
        let next_run_utc = self.calculate_next_reset_time(user_config)?.timestamp() as u64;

        let task_data = crate::database::ScheduledTaskData {
            id: task_id.clone(),
            task_type: "daily_reset".to_string(),
            user_configuration_id: Some(user_config.id.clone()),
            cron_expression: user_config.get_daily_reset_cron_expression(),
            timezone: user_config.timezone.clone(),
            next_run_utc,
            last_run_utc: user_config.last_daily_reset_utc.map(|ts| ts as u64),
            is_active: user_config.daily_reset_enabled,
            run_count: 0,
            failure_count: 0,
            task_data: None,
        };

        self.db_manager.upsert_scheduled_task(&task_data).await?;

        info!(
            user_id = %user_config.id,
            task_id = %task_id,
            next_run_utc = %next_run_utc,
            "Daily reset task scheduled"
        );

        Ok(task_id)
    }

    /// Cancel a scheduled daily reset task
    #[instrument(skip(self, task_id))]
    pub async fn cancel_scheduled_task(
        &self,
        task_id: &str,
    ) -> Result<(), AppError> {
        let active_tasks = self.db_manager.get_active_scheduled_tasks().await?;

        let task = active_tasks.iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::NotFound)?;

        // Deactivate the task
        let mut task_data = crate::database::ScheduledTaskData {
            id: task.id.clone(),
            task_type: task.task_type.clone(),
            user_configuration_id: task.user_configuration_id.clone(),
            cron_expression: task.cron_expression.clone(),
            timezone: task.timezone.clone(),
            next_run_utc: task.next_run_utc as u64,
            last_run_utc: task.last_run_utc.map(|ts| ts as u64),
            is_active: false,
            run_count: task.run_count as u32,
            failure_count: task.failure_count as u32,
            task_data: task.task_data.clone(),
        };

        self.db_manager.upsert_scheduled_task(&task_data).await?;

        info!(
            task_id = %task_id,
            "Daily reset task cancelled"
        );

        Ok(())
    }

    /// Process all pending daily reset tasks
    #[instrument(skip(self))]
    pub async fn process_pending_tasks(&self) -> Result<u32, Vec<(String, AppError)>> {
        let active_tasks = self.db_manager.get_active_scheduled_tasks().await?;
        let current_time = self.time_provider.now_utc().timestamp() as i64;

        let mut processed_count = 0;
        let mut errors = Vec::new();

        for task in active_tasks {
            if task.next_run_utc <= current_time {
                let task_id = task.id.clone();
                match self.process_task(&task).await {
                    Ok(_) => {
                        processed_count += 1;
                    }
                    Err(e) => {
                        errors.push((task_id, e));
                    }
                }
            }
        }

        if processed_count > 0 {
            info!(
                processed_count = %processed_count,
                errors = %errors.len(),
                "Processed {} daily reset tasks ({} errors)",
                processed_count,
                errors.len()
            );
        }

        Ok(processed_count)
    }

    /// Process a single scheduled task
    #[instrument(skip(self, task))]
    async fn process_task(&self, task: &crate::database::ScheduledTaskRow) -> Result<(), AppError> {
        let task_id = task.id.clone();
        let current_time = self.time_provider.now_utc();
        let execution_time = current_time.timestamp() as u64;

        match task.task_type.as_str() {
            "daily_reset" => {
                if let Some(user_id) = &task.user_configuration_id {
                    let user_config = self.db_manager.get_user_configuration(user_id).await?
                        .ok_or_else(|| AppError::ConfigurationNotFound)?;

                    // Verify task is still valid for this user
                    if user_config.daily_reset_enabled &&
                       self.is_reset_due(&user_config) &&
                       task.next_run_utc == user_config.get_next_reset_time_utc().unwrap_or(0) as i64 {

                        // Process the reset
                        self.perform_daily_reset(
                            &user_config,
                            SessionResetTriggerSource::BackgroundService,
                            Some("system".to_string()),
                        ).await?;

                        // Update task execution status
                        self.db_manager.update_scheduled_task_execution(
                            &task_id,
                            execution_time,
                            task.run_count as u32 + 1,
                            task.failure_count as u32,
                        ).await?;
                    } else {
                        // Task is no longer valid, deactivate it
                        self.cancel_scheduled_task(&task_id).await?;
                    }
                }
            }
            _ => {
                return Err(AppError::BackgroundTaskFailed(
                    format!("Unsupported task type: {}", task.task_type)
                ));
            }
        }

        Ok(())
    }

    /// Send WebSocket notification for configuration changes
    async fn send_configuration_change_notification(
        &self,
        config: &UserConfiguration,
    ) -> Result<(), AppError> {
        // TODO: Implement WebSocket notification sending
        // This would connect to the WebSocket service and send a message
        debug!(
            "Configuration change notification for user {}: timezone={}, daily_reset_enabled={}",
            config.id,
            config.timezone,
            config.daily_reset_enabled
        );
        Ok(())
    }

    /// Send WebSocket notification for session count changes
    async fn send_session_count_notification(
        &self,
        config: &UserConfiguration,
        previous_count: u32,
        new_count: u32,
    ) -> Result<(), AppError> {
        // TODO: Implement WebSocket notification sending
        debug!(
            "Session count notification for user {}: {} -> {}",
            config.id,
            previous_count,
            new_count
        );
        Ok(())
    }

    /// Send WebSocket notification for session resets
    async fn send_reset_notification(
        &self,
        config: &UserConfiguration,
        previous_count: u32,
        new_count: u32,
    ) -> Result<(), AppError> {
        // TODO: Implement WebSocket notification sending
        debug!(
            "Reset notification for user {}: {} -> {}",
            config.id,
            previous_count,
            new_count
        );
        Ok(())
    }

    /// Send WebSocket notification for manual overrides
    async fn send_manual_override_notification(
        &self,
        config: &UserConfiguration,
        previous_count: u32,
        new_count: u32,
    ) -> Result<(), AppError> {
        // TODO: Implement WebSocket notification sending
        debug!(
            "Manual override notification for user {}: {} -> {}",
            config.id,
            previous_count,
            new_count
        );
        Ok(())
    }

    /// Cleanup old daily reset records
    #[instrument(skip(self, days_to_keep))]
    pub async fn cleanup_old_records(&self, days_to_keep: u32) -> Result<(), AppError> {
        self.db_manager.cleanup_old_records(days_to_keep).await
    }
}

/// Parse time string in HH:MM format to naive time
fn parse_time_to_naive_time(time_str: &str) -> Result<(u32, u32), ()> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(());
    }

    let hour: u32 = parts[0].parse().map_err(|_| ())?;
    let minute: u32 = parts[1].parse().map_err(|_| ())?;

    if hour > 23 || minute > 59 {
        return Err(());
    }

    Ok((hour, minute))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::time_provider::MockTimeProvider;
    use crate::database::DatabaseManager;

    async fn create_test_service() -> Result<(DailyResetService, ()), Box<dyn std::error::Error>> {
        let time_provider = Arc::new(MockTimeProvider::new_from_now());
        let service = DailyResetService::new(time_provider);

        Ok((service, ()))
    }

    #[tokio::test]
    async fn test_calculate_next_reset_time_midnight() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("UTC".to_string())?;
        config.set_daily_reset_time(DailyResetTime::midnight())?;
        config.set_daily_reset_enabled(true);

        // Test time before midnight (should reset at next midnight)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 23, 30, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        // Test time after midnight (should reset tomorrow)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 0, 30, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_next_reset_time_hourly() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("UTC".to_string())?;
        config.set_daily_reset_time(DailyResetTime::hour(8)?)?;
        config.set_daily_reset_enabled(true);

        // Test time before 8 AM (should reset at 8 AM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 7, 30, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 8, 0, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        // Test time after 8 AM (should reset tomorrow at 8 AM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 8, 0, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_next_reset_time_custom() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("UTC".to_string())?;
        config.set_daily_reset_time(DailyResetTime::custom("14:30".to_string())?)?;
        config.set_daily_reset_enabled(true);

        // Test time before 2:30 PM (should reset at 2:30 PM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 14, 0, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 14, 30, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        // Test time after 2:30 PM (should reset tomorrow at 2:30 PM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 15, 0, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 14, 30, 0).single().unwrap();
        assert_eq!(next_reset, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_timezone_aware_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("America/New_York".to_string())?;
        config.set_daily_reset_time(DailyResetTime::hour(7)?); // 7 AM NY time
        config.set_daily_reset_enabled(true);

        // Set time to 11:30 UTC (6:30 AM NY time - should reset at 7:00 AM NY)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 11, 30, 0).single().unwrap()
        );

        let next_reset = service.calculate_next_reset_time(&config)?;
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap(); // 7:00 AM NY = 12:00 UTC

        assert_eq!(next_reset, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_is_reset_due() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let mut config = UserConfiguration::new();
        config.set_timezone("UTC".to_string())?;
        config.set_daily_reset_time(DailyResetTime::hour(8)?)?;
        config.set_daily_reset_enabled(true);
        config.last_daily_reset_utc = Some(
            Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).single().unwrap().timestamp() as u64
        );

        // Should not be due (last reset was yesterday, current time is before 8 AM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 7, 0, 0).single().unwrap()
        );

        assert!(!service.is_reset_due(&config));

        // Should be due (last reset was yesterday, current time is after 8 AM)
        service.time_provider.set_time(
            Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap()
        );

        assert!(service.is_reset_due(&config));

        // Should not be due when disabled
        config.set_daily_reset_enabled(false);
        assert!(!service.is_reset_due(&config));

        Ok(())
    }

    #[tokio::test]
    async fn test_increment_session_count() -> Result<(), Box<dyn std::error::Error>> {
        let (service, db_manager) = create_test_service().await?;

        let config = db_manager.get_or_create_user_config("test-user").await?;

        // Initial count should be 0
        let count = service.increment_session_count(&config).await?;
        assert_eq!(count, 1);

        // Should increment again
        let count = service.increment_session_count(&config).await?;
        assert_eq!(count, 2);

        // Get count should return current count
        let count = service.get_session_count("test-user").await?;
        assert_eq!(count, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_manual_session_override() -> Result<(), Box<dyn std::error::Error>> {
        let (service, db_manager) = create_test_service().await?;

        let config = db_manager.get_or_create_user_config("test-user").await?;

        // Set manual override
        let count = service.set_manual_session_override(&config, Some(10))?;
        assert_eq!(count, 10);

        // Increment should not work with manual override
        let count = service.increment_session_count(&config).await?;
        assert_eq!(count, 10); // Should still be 10

        // Clear override
        let count = service.set_manual_session_override(&config, None)?;
        assert_eq!(count, 0); // Should return to original count

        Ok(())
    }

    #[tokio::test]
    async fn test_maximum_session_count_validation() -> Result<(), Box<dyn std::error::Error>> {
        let (service, _db_manager) = create_test_service().await?;

        let config = UserConfiguration::new();

        // Should fail with count > 1000
        let result = service.set_manual_session_override(&config, Some(1001));
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::ManualSessionOverrideInvalid(_))));

        // Should succeed with count = 1000
        let result = service.set_manual_session_override(&config, Some(1000));
        assert!(result.is_ok());

        Ok(())
    }
}