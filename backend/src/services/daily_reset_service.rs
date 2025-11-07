//! Daily Reset Service (Simplified Version)
//!
//! Core service for managing daily session reset functionality without database dependencies.
//! This version provides the essential business logic for timezone-aware reset scheduling.

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::models::{
    user_configuration::{UserConfiguration, DailyResetTimeType, DailyResetTime},
};
use crate::services::time_provider::TimeProvider;
use crate::error::AppError;

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
/// Provides timezone-aware daily session reset functionality.
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

        let current_time = self.time_provider.now();
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
                        crate::models::user_configuration::UserConfigurationError::InvalidResetTimestamp(last_reset_utc)
                    )
                })?;

            let user_timezone: Tz = user_config.timezone.parse()
                .map_err(|_| {
                    crate::models::user_configuration::UserConfigurationError::InvalidTimezone(user_config.timezone.clone())
                })?;

            let last_reset_local = last_reset.with_timezone(&user_timezone);
            let current_local = self.time_provider.now().with_timezone(&user_timezone);

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