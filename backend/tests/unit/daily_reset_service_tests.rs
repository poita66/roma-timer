//! Unit Tests for Daily Reset Service
//!
//! Tests the core daily reset functionality including:
//! - Reset time calculation across different timezones
//! - Session count management and reset logic
//! - Daily reset scheduling and execution
//! - Timezone-aware reset timing

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;

use crate::models::{UserConfiguration, DailyResetTime, DailyResetTimeType};
use crate::services::{TimeProvider, MockTimeProvider};
use crate::database::DailyResetDatabaseExtensions;

use super::daily_reset_test_utils::{DailyResetTestContext, factories, assertions};

#[cfg(test)]
mod daily_reset_service_tests {
    use super::*;

    /// Test daily reset time calculation for midnight reset
    #[tokio::test]
    async fn test_midnight_reset_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with midnight reset in UTC
        let reset_time = DailyResetTime::midnight();
        let config = context.create_test_user_config("UTC", reset_time, true).await?;

        // Set time to 23:30 UTC on Jan 7, 2025
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 23, 30, 0).single().unwrap());

        // Calculate next reset time - should be midnight on Jan 8, 2025
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        // Advance time to past reset time
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 8, 0, 30, 0).single().unwrap());

        // Next reset should be next day
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 9, 0, 0, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        Ok(())
    }

    /// Test daily reset time calculation for hourly reset
    #[tokio::test]
    async fn test_hourly_reset_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with 7 AM reset
        let reset_time = DailyResetTime::hour(7)?;
        let config = context.create_test_user_config("UTC", reset_time, true).await?;

        // Set time to 6:30 AM
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 6, 30, 0).single().unwrap());

        // Next reset should be at 7 AM same day
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 7, 0, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        // Set time to 8:30 AM
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 8, 30, 0).single().unwrap());

        // Next reset should be at 7 AM next day
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 7, 0, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        Ok(())
    }

    /// Test daily reset time calculation for custom time
    #[tokio::test]
    async fn test_custom_reset_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with 14:30 (2:30 PM) custom reset
        let reset_time = DailyResetTime::custom("14:30".to_string())?;
        let config = context.create_test_user_config("UTC", reset_time, true).await?;

        // Set time to 14:00
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 14, 0, 0).single().unwrap());

        // Next reset should be at 14:30 same day
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 14, 30, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        // Set time to 15:00
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 15, 0, 0).single().unwrap());

        // Next reset should be at 14:30 next day
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 8, 14, 30, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        Ok(())
    }

    /// Test timezone-aware reset calculation
    #[tokio::test]
    async fn test_timezone_aware_reset_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with 7 AM reset in New York (UTC-5 in January)
        let reset_time = DailyResetTime::hour(7)?;
        let config = context.create_test_user_config("America/New_York", reset_time, true).await?;

        // Set time to 11:30 UTC (6:30 AM in New York)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 11, 30, 0).single().unwrap());

        // Next reset should be at 12:00 UTC (7:00 AM in New York)
        let next_reset = calculate_next_reset_time(&config, &context.time_provider);
        let expected = Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap();

        assert_eq!(next_reset, expected);

        // Verify timezone conversion
        let tz: Tz = "America/New_York".parse()?;
        let local_reset_time = next_reset.with_timezone(&tz);
        assert_eq!(local_reset_time.hour(), 7);
        assert_eq!(local_reset_time.minute(), 0);

        Ok(())
    }

    /// Test session count reset logic
    #[tokio::test]
    async fn test_session_count_reset() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let config = context.create_default_test_user_config().await?;

        // Set initial session count
        let mut updated_config = config.clone();
        updated_config.today_session_count = 5;
        updated_config.last_daily_reset_utc = Some(
            Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).single().unwrap().timestamp() as u64
        );

        // Save updated configuration
        context.db_manager.save_user_configuration(&updated_config).await?;

        // Set current time to after reset time
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 1, 0, 0).single().unwrap());

        // Check if reset is due
        let current_time = context.current_time().timestamp() as u64;
        let is_reset_due = updated_config.is_daily_reset_due(current_time);
        assert!(is_reset_due);

        // Perform reset
        let reset_time = context.current_time();
        perform_session_reset(&mut updated_config, reset_time).await?;

        // Verify reset
        assert_eq!(updated_config.today_session_count, 0);
        assert_eq!(updated_config.last_daily_reset_utc, Some(reset_time.timestamp() as u64));
        assert_eq!(updated_config.manual_session_override, None);

        // Save reset state
        context.db_manager.save_user_configuration(&updated_config).await?;

        Ok(())
    }

    /// Test session count increment logic
    #[tokio::test]
    async fn test_session_count_increment() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let config = context.create_default_test_user_config().await?;

        // Increment session count
        let mut updated_config = config.clone();
        let initial_count = updated_config.today_session_count;

        updated_config.increment_session_count()?;
        let new_count = updated_config.today_session_count;

        assert_eq!(new_count, initial_count + 1);
        assert!(updated_config.updated_at > updated_config.created_at);

        // Test increment with manual override (should not increment)
        updated_config.set_manual_session_override(Some(10))?;
        let override_count = updated_config.today_session_count;

        updated_config.increment_session_count()?;
        assert_eq!(updated_config.today_session_count, override_count); // Should not change

        Ok(())
    }

    /// Test daily reset across DST transition
    #[tokio::test]
    async fn test_dst_transition_handling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with 7 AM reset in New York (tests DST transition)
        let reset_time = DailyResetTime::hour(7)?;
        let config = context.create_test_user_config("America/New_York", reset_time, true).await?;

        // Test during standard time (January)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap());
        let next_reset_winter = calculate_next_reset_time(&config, &context.time_provider);

        // Test during daylight time (July)
        context.set_time(Utc.with_ymd_and_hms(2025, 7, 7, 11, 0, 0).single().unwrap());
        let next_reset_summer = calculate_next_reset_time(&config, &context.time_provider);

        // Both should result in 7 AM local time in New York
        let tz: Tz = "America/New_York".parse()?;

        let local_winter = next_reset_winter.with_timezone(&tz);
        let local_summer = next_reset_summer.with_timezone(&tz);

        assert_eq!(local_winter.hour(), 7);
        assert_eq!(local_summer.hour(), 7);

        // The UTC offset should be different due to DST
        assert_ne!(next_reset_winter.hour(), next_reset_summer.hour());

        Ok(())
    }

    /// Test disabled daily reset
    #[tokio::test]
    async fn test_disabled_daily_reset() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let config = context.create_test_user_config("UTC", DailyResetTime::midnight(), false).await?;

        // Even if time has passed reset point, disabled reset should not trigger
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 1, 0, 0).single().unwrap());

        let current_time = context.current_time().timestamp() as u64;
        let is_reset_due = config.is_daily_reset_due(current_time);
        assert!(!is_reset_due); // Should not be due when disabled

        // Next reset time should be None when disabled
        let next_reset = config.get_next_reset_time_utc();
        assert_eq!(next_reset, None);

        Ok(())
    }

    /// Test cron expression generation
    #[tokio::test]
    async fn test_cron_expression_generation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Test midnight reset
        let midnight_config = context.create_test_user_config(
            "UTC",
            DailyResetTime::midnight(),
            true
        ).await?;
        assert_eq!(midnight_config.get_daily_reset_cron_expression(), "0 0 * * *");

        // Test hourly reset
        let hourly_config = context.create_test_user_config(
            "UTC",
            DailyResetTime::hour(7)?,
            true
        ).await?;
        assert_eq!(hourly_config.get_daily_reset_cron_expression(), "0 7 * * *");

        // Test custom reset
        let custom_config = context.create_test_user_config(
            "UTC",
            DailyResetTime::custom("14:30".to_string())?,
            true
        ).await?;
        assert_eq!(custom_config.get_daily_reset_cron_expression(), "0 30 14 * * *");

        Ok(())
    }
}

// Helper functions for testing (these would be implemented in the actual service)

/// Calculate next reset time for a given configuration
fn calculate_next_reset_time(
    config: &UserConfiguration,
    time_provider: &MockTimeProvider,
) -> DateTime<Utc> {
    let current_time = time_provider.now_utc();
    let current_date = current_time.date_naive();

    match config.daily_reset_time_type {
        DailyResetTimeType::Midnight => {
            let next_reset = current_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            if next_reset > current_time {
                next_reset
            } else {
                (current_date + chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap().and_utc()
            }
        }
        DailyResetTimeType::Hour => {
            if let Some(hour) = config.daily_reset_time_hour {
                let next_reset = current_date.and_hms_opt(hour as u32, 0, 0).unwrap().and_utc();
                if next_reset > current_time {
                    next_reset
                } else {
                    (current_date + chrono::Duration::days(1)).and_hms_opt(hour as u32, 0, 0).unwrap().and_utc()
                }
            } else {
                current_time + chrono::Duration::hours(1)
            }
        }
        DailyResetTimeType::Custom => {
            if let Some(ref time_str) = config.daily_reset_time_custom {
                if let Ok(time_parts) = parse_time_string(time_str) {
                    let next_reset = current_date.and_hms_opt(time_parts[0], time_parts[1], 0).unwrap().and_utc();
                    if next_reset > current_time {
                        next_reset
                    } else {
                        (current_date + chrono::Duration::days(1)).and_hms_opt(time_parts[0], time_parts[1], 0).unwrap().and_utc()
                    }
                } else {
                    current_time + chrono::Duration::hours(1)
                }
            } else {
                current_time + chrono::Duration::hours(1)
            }
        }
    }
}

/// Parse time string in HH:MM format
fn parse_time_string(time_str: &str) -> Result<(u32, u32), ()> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let hour: u32 = parts[0].parse().map_err(|_| ())?;
        let minute: u32 = parts[1].parse().map_err(|_| ())?;
        if hour < 24 && minute < 60 {
            Ok((hour, minute))
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Perform session reset for a configuration
async fn perform_session_reset(
    config: &mut UserConfiguration,
    reset_time: DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    config.reset_session_count();
    Ok(())
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    #[test]
    fn test_parse_time_string() {
        assert_eq!(parse_time_string("09:30"), Ok((9, 30)));
        assert_eq!(parse_time_string("23:45"), Ok((23, 45)));
        assert_eq!(parse_time_string("00:00"), Ok((0, 0)));
        assert_eq!(parse_time_string("24:00"), Err(())); // Invalid hour
        assert_eq!(parse_time_string("12:60"), Err(())); // Invalid minute
        assert_eq!(parse_time_string("invalid"), Err(())); // Invalid format
    }
}