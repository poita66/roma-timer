//! Unit Tests for Timezone Service
//!
//! Tests timezone validation, conversion, and handling including:
//! - IANA timezone validation
//! - Timezone-aware time conversion
//! - DST transition handling
//! - Timezone database updates
//! - Invalid timezone error handling

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};
use chrono_tz::Tz;

use crate::services::time_provider::{TimeProvider, MockTimeProvider};

use super::daily_reset_test_utils::{DailyResetTestContext, factories, assertions};

#[cfg(test)]
mod timezone_service_tests {
    use super::*;

    /// Test valid timezone validation
    #[tokio::test]
    async fn test_valid_timezone_validation() {
        let valid_timezones = vec![
            "UTC",
            "America/New_York",
            "Europe/London",
            "Asia/Tokyo",
            "Australia/Sydney",
            "America/Los_Angeles",
            "Europe/Paris",
            "Asia/Shanghai",
            "America/Chicago",
            "Europe/Berlin",
        ];

        for timezone in valid_timezones {
            let result: Result<Tz, _> = timezone.parse();
            assert!(result.is_ok(), "Expected '{}' to be a valid timezone", timezone);

            let tz = result.unwrap();
            assert!(!tz.name().is_empty());
        }
    }

    /// Test invalid timezone validation
    #[tokio::test]
    async fn test_invalid_timezone_validation() {
        let invalid_timezones = vec![
            "Invalid/Timezone",
            "America/CityThatDoesNotExist",
            "Europe/NonExistent",
            "Asia/FakeCity",
            "",
            "NotATimezone",
            "UTC-5", // Should use proper IANA format
            "GMT+2", // Should use proper IANA format
            "12345",
            "America/New York", // Space in name
        ];

        for timezone in invalid_timezones {
            let result: Result<Tz, _> = timezone.parse();
            assert!(result.is_err(), "Expected '{}' to be an invalid timezone", timezone);
        }
    }

    /// Test timezone time conversion
    #[tokio::test]
    async fn test_timezone_time_conversion() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Test specific UTC time
        let utc_time = Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap();

        // Test conversion to different timezones
        let test_cases = vec![
            ("UTC", "2025-01-07 12:00:00 UTC"),
            ("America/New_York", "2025-01-07 07:00:00 EST"), // UTC-5 in January
            ("Europe/London", "2025-01-07 12:00:00 GMT"), // UTC+0 in January
            ("Asia/Tokyo", "2025-01-07 21:00:00 JST"), // UTC+9
            ("Australia/Sydney", "2025-01-07 23:00:00 AEDT"), // UTC+11 (DST)
            ("America/Los_Angeles", "2025-01-07 04:00:00 PST"), // UTC-8
        ];

        for (timezone_str, expected_local) in test_cases {
            let tz: Tz = timezone_str.parse()?;
            let local_time = utc_time.with_timezone(&tz);

            // Verify the conversion worked
            assert_ne!(utc_time, local_time.with_timezone(&Utc));

            // Parse expected time and compare
            let expected_parts = extract_datetime_parts(expected_local);
            assert_eq!(local_time.year(), expected_parts.year);
            assert_eq!(local_time.month(), expected_parts.month);
            assert_eq!(local_time.day(), expected_parts.day);
            assert_eq!(local_time.hour(), expected_parts.hour);
            assert_eq!(local_time.minute(), expected_parts.minute);
        }

        Ok(())
    }

    /// Test reverse timezone conversion
    #[tokio::test]
    async fn test_reverse_timezone_conversion() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create local time in New York
        let ny_tz: Tz = "America/New_York".parse()?;
        let ny_local_time = ny_tz.with_ymd_and_hms(2025, 1, 7, 9, 30, 0).single().unwrap();

        // Convert to UTC and back
        let utc_time = ny_local_time.with_timezone(&Utc);
        let converted_back = utc_time.with_timezone(&ny_tz);

        assert_eq!(ny_local_time, converted_back);

        Ok(())
    }

    /// Test DST transition handling - winter to summer
    #[tokio::test]
    async fn test_dst_transition_winter_to_summer() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let ny_tz: Tz = "America/New_York".parse()?;

        // Test in standard time (January) - UTC-5
        let winter_utc = Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap();
        let winter_ny = winter_utc.with_timezone(&ny_tz);
        assert_eq!(winter_ny.hour(), 7); // 12:00 UTC = 7:00 EST (UTC-5)

        // Test in daylight time (July) - UTC-4
        let summer_utc = Utc.with_ymd_and_hms(2025, 7, 7, 12, 0, 0).single().unwrap();
        let summer_ny = summer_utc.with_timezone(&ny_tz);
        assert_eq!(summer_ny.hour(), 8); // 12:00 UTC = 8:00 EDT (UTC-4)

        // Verify the UTC offset difference
        assert_eq!(winter_ny.offset().fix().local_minus_utc(), -5 * 3600);
        assert_eq!(summer_ny.offset().fix().local_minus_utc(), -4 * 3600);

        Ok(())
    }

    /// Test DST transition handling - spring forward
    #[tokio::test]
    async fn test_dst_spring_forward() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let ny_tz: Tz = "America/New_York".parse()?;

        // Test around spring forward (2nd Sunday in March 2025 is March 9)
        // 2:00 AM becomes 3:00 AM (skipped)
        let before_spring = ny_tz.with_ymd_and_hms(2025, 3, 9, 1, 59, 59).single().unwrap();
        let after_spring = ny_tz.with_ymd_and_hms(2025, 3, 9, 3, 0, 0).single().unwrap();

        let before_utc = before_spring.with_timezone(&Utc);
        let after_utc = after_spring.with_timezone(&Utc);

        // Should be about 1 hour apart (with some time for the transition)
        let duration = after_utc.signed_duration_since(before_utc);
        assert!(duration.num_minutes() > 58 && duration.num_minutes() < 62);

        Ok(())
    }

    /// Test DST transition handling - fall back
    #[tokio::test]
    async fn test_dst_fall_back() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let ny_tz: Tz = "America/New_York".parse()?;

        // Test around fall back (1st Sunday in November 2025 is November 2)
        // 2:00 AM becomes 1:00 AM (repeated)
        let before_fall = ny_tz.with_ymd_and_hms(2025, 11, 2, 0, 59, 59).single().unwrap();
        let after_fall = ny_tz.with_ymd_and_hms(2025, 11, 2, 2, 0, 0).single().unwrap();

        let before_utc = before_fall.with_timezone(&Utc);
        let after_utc = after_fall.with_timezone(&Utc);

        // Should be about 3 hours apart (due to the repeated hour)
        let duration = after_utc.signed_duration_since(before_utc);
        assert!(duration.num_minutes() > 178 && duration.num_minutes() < 182);

        Ok(())
    }

    /// Test timezone for scheduling daily resets
    #[tokio::test]
    async fn test_timezone_scheduling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let test_cases = vec![
            ("UTC", 7, "2025-01-07 07:00:00 UTC"), // 7 AM UTC
            ("America/New_York", 7, "2025-01-07 12:00:00 UTC"), // 7 AM EST = 12:00 UTC
            ("Europe/London", 9, "2025-01-07 09:00:00 UTC"), // 9 AM GMT = 9:00 UTC
            ("Asia/Tokyo", 9, "2025-01-07 00:00:00 UTC"), // 9 AM JST = 00:00 UTC previous day
            ("Australia/Sydney", 8, "2025-01-06 21:00:00 UTC"), // 8 AM AEDT = 21:00 UTC previous day
        ];

        for (timezone, local_hour, expected_utc) in test_cases {
            let tz: Tz = timezone.parse()?;

            // Create local time at specified hour
            let local_date = NaiveDate::from_ymd_opt(2025, 1, 7).unwrap();
            let local_time = if timezone == "Asia/Tokyo" || timezone == "Australia/Sydney" {
                // These cases cross to previous day in UTC
                tz.with_ymd_and_hms(2025, 1, 8, local_hour, 0, 0).single().unwrap()
            } else {
                tz.with_ymd_and_hms(2025, 1, 7, local_hour, 0, 0).single().unwrap()
            };

            let utc_time = local_time.with_timezone(&Utc);
            let utc_str = utc_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();

            assert_eq!(utc_str, expected_utc,
                "Timezone {} at {}:00 should be {}, got {}",
                timezone, local_hour, expected_utc, utc_str);
        }

        Ok(())
    }

    /// Test timezone database completeness
    #[tokio::test]
    async fn test_timezone_database_completeness() {
        // Test that commonly used timezones are available
        let essential_timezones = vec![
            "UTC",
            "GMT",
            "America/New_York",
            "America/Chicago",
            "America/Denver",
            "America/Los_Angeles",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Asia/Kolkata",
            "Australia/Sydney",
            "Australia/Melbourne",
        ];

        for timezone in essential_timezones {
            let result: Result<Tz, _> = timezone.parse();
            assert!(result.is_ok(), "Essential timezone '{}' not available", timezone);
        }
    }

    /// Test timezone formatting and display
    #[tokio::test]
    async fn test_timezone_formatting() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let utc_time = Utc.with_ymd_and_hms(2025, 1, 7, 12, 30, 45).single().unwrap();
        let ny_tz: Tz = "America/New_York".parse()?;
        let ny_time = utc_time.with_timezone(&ny_tz);

        // Test different formatting options
        let iso_format = ny_time.format("%Y-%m-%dT%H:%M:%S%z").to_string();
        assert!(!iso_format.is_empty());

        let readable_format = ny_time.format("%Y-%m-%d %H:%M:%S %Z").to_string();
        assert!(readable_format.contains("EST")); // Should contain timezone abbreviation

        let date_only = ny_time.format("%Y-%m-%d").to_string();
        assert_eq!(date_only, "2025-01-07");

        let time_only = ny_time.format("%H:%M:%S").to_string();
        assert_eq!(time_only, "07:30:45");

        Ok(())
    }

    /// Test timezone edge cases
    #[tokio::test]
    async fn test_timezone_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Test midnight boundary
        let utc_midnight = Utc.with_ymd_and_hms(2025, 1, 7, 0, 0, 0).single().unwrap();
        let tz: Tz = "Pacific/Auckland".parse()?; // UTC+13, ahead of UTC
        let local_time = utc_midnight.with_timezone(&tz);

        // Should be January 7th in Auckland (ahead of UTC)
        assert_eq!(local_time.month(), 1);
        assert_eq!(local_time.day(), 7);
        assert_eq!(local_time.hour(), 13);

        // Test far ahead timezone
        let utc_time = Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap();
        let local_time = utc_time.with_timezone(&tz);
        assert_eq!(local_time.date().month(), 1);
        assert_eq!(local_time.date().day(), 8); // Next day in Auckland

        // Test far behind timezone
        let us_tz: Tz = "America/Samoa".parse()?; // UTC-11, behind UTC
        let local_time = utc_midnight.with_timezone(&us_tz);
        assert_eq!(local_time.month(), 1);
        assert_eq!(local_time.day(), 6); // Previous day in Samoa

        Ok(())
    }

    /// Test timezone validation in user configuration
    #[tokio::test]
    async fn test_timezone_validation_in_config() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Test setting valid timezone
        let mut config = context.create_default_test_user_config().await?;

        assert!(config.set_timezone("America/New_York".to_string()).is_ok());
        assert_eq!(config.timezone, "America/New_York");

        // Test setting invalid timezone
        let result = config.set_timezone("Invalid/Timezone".to_string());
        assert!(result.is_err());
        // Should still have the previous valid timezone
        assert_eq!(config.timezone, "America/New_York");

        // Test setting empty timezone
        let result = config.set_timezone("".to_string());
        assert!(result.is_err());

        Ok(())
    }
}

/// Helper function to extract datetime parts from a formatted string
fn extract_datetime_parts(datetime_str: &str) -> DateTimeParts {
    // Parse format like "2025-01-07 07:00:00 EST"
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    let date_parts: Vec<u32> = parts[0].split('-')
        .map(|s| s.parse().unwrap())
        .collect();
    let time_parts: Vec<u32> = parts[1].split(':')
        .map(|s| s.parse().unwrap())
        .collect();

    DateTimeParts {
        year: date_parts[0],
        month: date_parts[1],
        day: date_parts[2],
        hour: time_parts[0],
        minute: time_parts[1],
        second: time_parts[2],
    }
}

/// Helper struct for datetime comparison
#[derive(Debug, PartialEq)]
struct DateTimeParts {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

#[cfg(test)]
mod timezone_validation_tests {
    use super::*;

    #[test]
    fn test_extract_datetime_parts() {
        let input = "2025-01-07 07:30:45 EST";
        let expected = DateTimeParts {
            year: 2025,
            month: 1,
            day: 7,
            hour: 7,
            minute: 30,
            second: 45,
        };

        let result = extract_datetime_parts(input);
        assert_eq!(result, expected);
    }
}