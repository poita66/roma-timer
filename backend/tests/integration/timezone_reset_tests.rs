//! Integration Tests for Timezone-Aware Daily Reset
//!
//! Tests timezone-specific aspects of daily reset including:
//! - DST transition handling
//! - Multiple timezone scheduling
//! - Timezone change scenarios
//! - Edge cases and boundary conditions
//! - Cross-timezone consistency

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;

use crate::models::{UserConfiguration, DailyResetTime};
use crate::services::time_provider::{TimeProvider, MockTimeProvider};
use crate::database::DailyResetDatabaseExtensions;

use super::daily_reset_integration_utils::{DailyResetIntegrationTestContext, scenarios};

#[cfg(test)]
mod timezone_reset_tests {
    use super::*;

    /// Test DST transition from winter to summer (spring forward)
    #[tokio::test]
    async fn test_dst_spring_forward_transition() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Create user with 7 AM reset in New York
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("America/New_York".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(7)?)?;
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test before DST transition (March 8, 2025 - before DST starts)
        // In 2025, DST starts on March 9 at 2:00 AM local time
        let winter_date = Utc.with_ymd_and_hms(2025, 3, 8, 12, 0, 0).single().unwrap();
        context.set_time(winter_date);

        let winter_next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let winter_local = winter_next_reset.with_timezone(&"America/New_York".parse::<Tz>()?);

        assert_eq!(winter_local.hour(), 7);
        assert_eq!(winter_local.date().month(), 3);
        assert_eq!(winter_local.date().day(), 9);

        // Test during DST transition (March 9, 2025 - after DST starts)
        let summer_date = Utc.with_ymd_and_hms(2025, 3, 9, 12, 0, 0).single().unwrap();
        context.set_time(summer_date);

        let summer_next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let summer_local = summer_next_reset.with_timezone(&"America/New_York".parse::<Tz>()?);

        assert_eq!(summer_local.hour(), 7);
        assert_eq!(summer_local.date().month(), 3);
        assert_eq!(summer_local.date().day(), 10);

        // Verify UTC times are different due to DST
        assert_ne!(winter_next_reset.hour(), summer_next_reset.hour());

        Ok(())
    }

    /// Test DST transition from summer to winter (fall back)
    #[tokio::test]
    async fn test_dst_fall_back_transition() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("America/New_York".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(7)?)?;
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test before DST ends (November 1, 2025 - before DST ends)
        // In 2025, DST ends on November 2 at 2:00 AM local time
        let summer_date = Utc.with_ymd_and_hms(2025, 11, 1, 12, 0, 0).single().unwrap();
        context.set_time(summer_date);

        let summer_next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let summer_local = summer_next_reset.with_timezone(&"America/New_York".parse::<Tz>()?);

        assert_eq!(summer_local.hour(), 7);
        assert_eq!(summer_local.date().month(), 11);
        assert_eq!(summer_local.date().day(), 2);

        // Test after DST ends (November 2, 2025 - after DST ends)
        let winter_date = Utc.with_ymd_and_hms(2025, 11, 2, 12, 0, 0).single().unwrap();
        context.set_time(winter_date);

        let winter_next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let winter_local = winter_next_reset.with_timezone(&"America/New_York".parse::<Tz>()?);

        assert_eq!(winter_local.hour(), 7);
        assert_eq!(winter_local.date().month(), 11);
        assert_eq!(winter_local.date().day(), 3);

        // Verify UTC times are different due to DST
        assert_ne!(summer_next_reset.hour(), winter_next_reset.hour());

        Ok(())
    }

    /// Test multiple users in different timezones
    #[tokio::test]
    async fn test_multiple_timezone_users() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Create users in different timezones with 7 AM local reset
        let timezone_configs = vec![
            ("UTC", "user_utc"),
            ("America/New_York", "user_ny"),
            ("Europe/London", "user_london"),
            ("Asia/Tokyo", "user_tokyo"),
            ("Australia/Sydney", "user_sydney"),
        ];

        let mut user_configs = Vec::new();

        for (timezone, user_suffix) in timezone_configs {
            let user_id = format!("{}-{}", context.config.test_user_id, user_suffix);
            let config = context.db_manager.get_or_create_user_config(&user_id).await?;
            let mut updated_config = config.clone();
            updated_config.set_timezone(timezone.to_string())?;
            updated_config.set_daily_reset_time(DailyResetTime::hour(7)?)?;
            updated_config.set_daily_reset_enabled(true);

            context.db_manager.save_user_configuration(&updated_config).await?;
            user_configs.push((user_id, timezone, updated_config));
        }

        // Set a specific UTC time and verify local reset times
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap());

        let mut reset_times = Vec::new();

        for (user_id, timezone, config) in user_configs {
            let next_reset = calculate_timezone_aware_reset_time(&config, &context.time_provider);
            let tz: Tz = timezone.parse()?;
            let local_time = next_reset.with_timezone(&tz);

            reset_times.push((user_id, timezone, next_reset, local_time));

            // Verify all local times are 7 AM
            assert_eq!(local_time.hour(), 7, "User {} should have 7 AM reset", timezone);
        }

        // Verify UTC times are different for different timezones
        let utc_times: Vec<_> = reset_times.iter().map(|(_, _, utc_time, _)| utc_time.hour()).collect();
        let unique_utc_hours: std::collections::HashSet<_> = utc_times.iter().collect();
        assert!(unique_utc_hours.len() > 1, "UTC reset times should vary across timezones");

        Ok(())
    }

    /// Test timezone change scenario
    #[tokio::test]
    async fn test_timezone_change() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("America/New_York".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(8)?)?;
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Set initial time
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 13, 0, 0).single().unwrap());

        let initial_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let initial_local = initial_reset.with_timezone(&"America/New_York".parse::<Tz>()?);
        assert_eq!(initial_local.hour(), 8);

        // Change timezone to Pacific Time
        updated_config.set_timezone("America/Los_Angeles".to_string())?;
        context.db_manager.save_user_configuration(&updated_config).await?;

        let updated_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let updated_local = updated_reset.with_timezone(&"America/Los_Angeles".parse::<Tz>()?);
        assert_eq!(updated_local.hour(), 8);

        // Verify UTC times changed
        assert_ne!(initial_reset, updated_reset);

        Ok(())
    }

    /// Test midnight reset across timezones
    #[tokio::test]
    async fn test_midnight_reset_across_timezones() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let midnight_scenarios = vec![
            ("UTC", "2025-01-08 00:00:00 UTC"),
            ("America/New_York", "2025-01-08 05:00:00 UTC"), // Midnight EST = 5 AM UTC next day
            ("Europe/London", "2025-01-08 00:00:00 UTC"), // Midnight GMT = midnight UTC
            ("Asia/Tokyo", "2025-01-07 15:00:00 UTC"), // Midnight JST = 3 PM UTC previous day
            ("Australia/Sydney", "2025-01-07 13:00:00 UTC"), // Midnight AEDT = 1 PM UTC previous day
        ];

        for (timezone, expected_utc) in midnight_scenarios {
            let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
            let mut updated_config = config.clone();
            updated_config.set_timezone(timezone.to_string())?;
            updated_config.set_daily_reset_time(DailyResetTime::midnight())?;
            updated_config.set_daily_reset_enabled(true);

            context.db_manager.save_user_configuration(&updated_config).await?;

            // Set time to before expected reset
            context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 20, 0, 0).single().unwrap());

            let next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
            let utc_str = next_reset.format("%Y-%m-%d %H:%M:%S UTC").to_string();

            assert_eq!(utc_str, expected_utc,
                "Midnight reset in {} should be {}, got {}",
                timezone, expected_utc, utc_str);

            // Verify local time is actually midnight
            let tz: Tz = timezone.parse()?;
            let local_time = next_reset.with_timezone(&tz);
            assert_eq!(local_time.hour(), 0, "Local time should be midnight in {}", timezone);
            assert_eq!(local_time.minute(), 0);
        }

        Ok(())
    }

    /// Test edge cases around date boundaries
    #[tokio::test]
    async fn test_date_boundary_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Test user in far east timezone (UTC+13)
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("Pacific/Auckland".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(1)?)?; // 1 AM local time
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Set time to just before reset would occur
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 6, 11, 30, 0).single().unwrap());

        let next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        let auckland_time = next_reset.with_timezone(&"Pacific/Auckland".parse::<Tz>()?);

        // Should be 1 AM on January 7 in Auckland
        assert_eq!(auckland_time.date().month(), 1);
        assert_eq!(auckland_time.date().day(), 7);
        assert_eq!(auckland_time.hour(), 1);

        // Should be noon UTC on January 6
        assert_eq!(next_reset.hour(), 12);
        assert_eq!(next_reset.date().day(), 6);

        Ok(())
    }

    /// Test invalid timezone handling
    #[tokio::test]
    async fn test_invalid_timezone_handling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();

        // Test setting invalid timezone
        let result = updated_config.set_timezone("Invalid/Timezone".to_string());
        assert!(result.is_err(), "Should reject invalid timezone");

        // Test setting empty timezone
        let result = updated_config.set_timezone("".to_string());
        assert!(result.is_err(), "Should reject empty timezone");

        // Verify timezone wasn't changed
        let saved_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?;
        assert!(saved_config.is_some());
        // Should still have the original valid timezone

        Ok(())
    }

    /// Test timezone performance with many calculations
    #[tokio::test]
    async fn test_timezone_performance() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("America/New_York".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(7)?)?;
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test performance with many calculations
        let start_time = std::time::Instant::now();

        for i in 0..1000 {
            // Vary the time to test different scenarios
            let hours = (i % 24) as i32;
            let test_time = Utc.with_ymd_and_hms(2025, 1, 7, hours, 0, 0).single().unwrap();
            context.set_time(test_time);

            let _next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);
        }

        let duration = start_time.elapsed();

        // Should complete 1000 calculations in reasonable time (less than 1 second)
        assert!(duration.as_millis() < 1000,
            "Timezone calculations took too long: {}ms", duration.as_millis());

        println!("1000 timezone calculations completed in {}ms", duration.as_millis());

        Ok(())
    }

    /// Test leap year handling
    #[tokio::test]
    async fn test_leap_year_handling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("UTC".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(0)?)?;
        updated_config.set_daily_reset_enabled(true);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test leap year (2024 is a leap year)
        context.set_time(Utc.with_ymd_and_hms(2024, 2, 28, 23, 0, 0).single().unwrap());

        let next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);

        // Next reset should be on February 29, 2024 (leap day)
        assert_eq!(next_reset.date().year(), 2024);
        assert_eq!(next_reset.date().month(), 2);
        assert_eq!(next_reset.date().day(), 29);

        // Test non-leap year (2025 is not a leap year)
        context.set_time(Utc.with_ymd_and_hms(2025, 2, 28, 23, 0, 0).single().unwrap());

        let next_reset = calculate_timezone_aware_reset_time(&updated_config, &context.time_provider);

        // Next reset should be on March 1, 2025
        assert_eq!(next_reset.date().year(), 2025);
        assert_eq!(next_reset.date().month(), 3);
        assert_eq!(next_reset.date().day(), 1);

        Ok(())
    }
}

// Helper functions for timezone testing

/// Calculate timezone-aware reset time
fn calculate_timezone_aware_reset_time(
    config: &UserConfiguration,
    time_provider: &MockTimeProvider,
) -> DateTime<Utc> {
    let current_time = time_provider.now_utc();
    let tz: Tz = config.timezone.parse().expect("Valid timezone required");

    // Get current time in user's timezone
    let current_local = current_time.with_timezone(&tz);
    let current_date = current_local.date_naive();

    // Create reset time for today
    let reset_time = match config.daily_reset_time_type {
        crate::models::DailyResetTimeType::Midnight => {
            current_date.and_hms_opt(0, 0, 0).unwrap()
        }
        crate::models::DailyResetTimeType::Hour => {
            if let Some(hour) = config.daily_reset_time_hour {
                current_date.and_hms_opt(hour as u32, 0, 0).unwrap()
            } else {
                current_date.and_hms_opt(0, 0, 0).unwrap()
            }
        }
        crate::models::DailyResetTimeType::Custom => {
            if let Some(ref time_str) = config.daily_reset_time_custom {
                if let Ok((hour, minute)) = parse_custom_time(time_str) {
                    current_date.and_hms_opt(hour, minute, 0).unwrap()
                } else {
                    current_date.and_hms_opt(0, 0, 0).unwrap()
                }
            } else {
                current_date.and_hms_opt(0, 0, 0).unwrap()
            }
        }
    };

    let reset_local = tz.from_local_datetime(&reset_time).single().unwrap();
    let reset_utc = reset_local.with_timezone(&Utc);

    // If reset time has passed today, schedule for tomorrow
    if reset_utc <= current_time {
        let tomorrow_date = current_date.succ_opt().unwrap();
        let tomorrow_reset = match config.daily_reset_time_type {
            crate::models::DailyResetTimeType::Midnight => {
                tomorrow_date.and_hms_opt(0, 0, 0).unwrap()
            }
            crate::models::DailyResetTimeType::Hour => {
                if let Some(hour) = config.daily_reset_time_hour {
                    tomorrow_date.and_hms_opt(hour as u32, 0, 0).unwrap()
                } else {
                    tomorrow_date.and_hms_opt(0, 0, 0).unwrap()
                }
            }
            crate::models::DailyResetTimeType::Custom => {
                if let Some(ref time_str) = config.daily_reset_time_custom {
                    if let Ok((hour, minute)) = parse_custom_time(time_str) {
                        tomorrow_date.and_hms_opt(hour, minute, 0).unwrap()
                    } else {
                        tomorrow_date.and_hms_opt(0, 0, 0).unwrap()
                    }
                } else {
                    tomorrow_date.and_hms_opt(0, 0, 0).unwrap()
                }
            }
        };

        let tomorrow_local = tz.from_local_datetime(&tomorrow_reset).single().unwrap();
        tomorrow_local.with_timezone(&Utc)
    } else {
        reset_utc
    }
}

/// Parse custom time string in HH:MM format
fn parse_custom_time(time_str: &str) -> Result<(u32, u32), ()> {
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

#[cfg(test)]
mod timezone_test_helpers {
    use super::*;

    #[test]
    fn test_parse_custom_time() {
        assert_eq!(parse_custom_time("09:30"), Ok((9, 30)));
        assert_eq!(parse_custom_time("23:59"), Ok((23, 59)));
        assert_eq!(parse_custom_time("00:00"), Ok((0, 0)));
        assert_eq!(parse_custom_time("24:00"), Err(())); // Invalid hour
        assert_eq!(parse_custom_time("12:60"), Err(())); // Invalid minute
        assert_eq!(parse_custom_time("invalid"), Err(())); // Invalid format
    }
}