//! Timezone Service for Roma Timer
//!
//! Provides timezone validation, conversion, and utility functions for the daily reset feature.

use chrono_tz::{Tz, TZ_VARIANTS};
use regex::Regex;
use std::collections::HashSet;

/// Errors that can occur during timezone operations
#[derive(Debug, thiserror::Error)]
pub enum TimezoneError {
    #[error("Invalid timezone identifier: {timezone}")]
    InvalidTimezone { timezone: String },

    #[error("Timezone validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Timezone database not available")]
    DatabaseNotAvailable,
}

/// Result type for timezone operations
pub type TimezoneResult<T> = Result<T, TimezoneError>;

/// Service for handling timezone validation and conversion operations
#[derive(Debug, Clone)]
pub struct TimezoneService {
    /// Cache of valid timezone identifiers for fast lookup
    valid_timezones: HashSet<String>,
}

impl TimezoneService {
    /// Creates a new TimezoneService with an initialized timezone database
    pub fn new() -> Self {
        let mut valid_timezones = HashSet::new();

        // Initialize with all known timezones from the timezone database
        for tz in TZ_VARIANTS.iter() {
            valid_timezones.insert(tz.name().to_string());
        }

        Self { valid_timezones }
    }

    /// Validates if a timezone string is a valid timezone identifier
    ///
    /// # Arguments
    /// * `timezone` - The timezone string to validate
    ///
    /// # Returns
    /// `Ok(())` if the timezone is valid, `Err(TimezoneError)` otherwise
    ///
    /// # Examples
    /// ```
    /// use roma_timer::services::timezone_service::TimezoneService;
    ///
    /// let service = TimezoneService::new();
    /// assert!(service.validate_timezone("UTC").is_ok());
    /// assert!(service.validate_timezone("America/New_York").is_ok());
    /// assert!(service.validate_timezone("Invalid/Timezone").is_err());
    /// ```
    pub fn validate_timezone(&self, timezone: &str) -> TimezoneResult<()> {
        if timezone.is_empty() {
            return Err(TimezoneError::ValidationFailed {
                reason: "Timezone cannot be empty".to_string(),
            });
        }

        // Check if it's in our valid timezone set
        if self.valid_timezones.contains(timezone) {
            return Ok(());
        }

        // Try parsing as a chrono-tz timezone
        match timezone.parse::<Tz>() {
            Ok(_) => {
                // If parsing succeeds, add to cache for future lookups
                return Ok(());
            }
            Err(_) => {
                return Err(TimezoneError::InvalidTimezone {
                    timezone: timezone.to_string(),
                });
            }
        }
    }

    /// Converts a timezone string to a Tz object
    ///
    /// # Arguments
    /// * `timezone` - The timezone string to convert
    ///
    /// # Returns
    /// `Ok(Tz)` if conversion succeeds, `Err(TimezoneError)` otherwise
    pub fn parse_timezone(&self, timezone: &str) -> TimezoneResult<Tz> {
        self.validate_timezone(timezone)?;

        timezone.parse::<Tz>().map_err(|_| TimezoneError::InvalidTimezone {
            timezone: timezone.to_string(),
        })
    }

    /// Gets a list of all valid timezone identifiers
    ///
    /// # Returns
    /// A vector of all valid timezone identifiers, sorted alphabetically
    pub fn get_all_timezones(&self) -> Vec<String> {
        let mut timezones: Vec<String> = self.valid_timezones.iter().cloned().collect();
        timezones.sort_unstable();
        timezones
    }

    /// Gets a list of common timezone identifiers for user selection
    ///
    /// # Returns
    /// A vector of commonly used timezone identifiers
    pub fn get_common_timezones(&self) -> Vec<&'static str> {
        vec![
            "UTC",
            "US/Eastern",
            "US/Central",
            "US/Mountain",
            "US/Pacific",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Europe/Rome",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Asia/Dubai",
            "Australia/Sydney",
            "Australia/Melbourne",
            "America/New_York",
            "America/Los_Angeles",
            "America/Chicago",
            "America/Denver",
            "America/Toronto",
            "America/Vancouver",
            "America/Mexico_City",
            "America/Sao_Paulo",
            "Africa/Cairo",
            "Africa/Johannesburg",
        ]
    }

    /// Checks if a timezone observes Daylight Saving Time
    ///
    /// # Arguments
    /// * `timezone` - The timezone to check
    ///
    /// # Returns
    /// `Ok(true)` if the timezone observes DST, `Ok(false)` if it doesn't, `Err(TimezoneError)` if invalid
    pub fn timezone_observes_dst(&self, timezone: &str) -> TimezoneResult<bool> {
        let tz = self.parse_timezone(timezone)?;

        // Get the current year to check DST offsets
        let current_year = chrono::Utc::now().year();

        // Check offset difference between January and July
        // If they're different, the timezone observes DST
        let jan_offset = tz
            .offset_from_utc_datetime(&chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(current_year, 1, 1)
                    .ok_or_else(|| TimezoneError::ValidationFailed {
                        reason: "Invalid date for January 1st".to_string(),
                    })?,
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
            .fix();

        let jul_offset = tz
            .offset_from_utc_datetime(&chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(current_year, 7, 1)
                    .ok_or_else(|| TimezoneError::ValidationFailed {
                        reason: "Invalid date for July 1st".to_string(),
                    })?,
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
            .fix();

        Ok(jan_offset != jul_offset)
    }

    /// Gets the current offset from UTC for a timezone
    ///
    /// # Arguments
    /// * `timezone` - The timezone to get the offset for
    ///
    /// # Returns
    /// `Ok(i32)` representing the offset in seconds, `Err(TimezoneError)` if invalid
    pub fn get_current_utc_offset(&self, timezone: &str) -> TimezoneResult<i32> {
        let tz = self.parse_timezone(timezone)?;
        let now = chrono::Utc::now();
        let offset = tz.offset_from_utc_datetime(&now.naive_utc()).fix();
        Ok(offset.local_minus_utc())
    }

    /// Validates timezone format using regex patterns
    ///
    /// # Arguments
    /// * `timezone` - The timezone string to validate the format of
    ///
    /// # Returns
    /// `Ok(())` if format is valid, `Err(TimezoneError)` otherwise
    pub fn validate_timezone_format(&self, timezone: &str) -> TimezoneResult<()> {
        if timezone.is_empty() {
            return Err(TimezoneError::ValidationFailed {
                reason: "Timezone cannot be empty".to_string(),
            });
        }

        // Basic timezone format validation
        // Should match patterns like "UTC", "America/New_York", "Europe/London", etc.
        let timezone_regex = Regex::new(r"^[A-Za-z_]+(/[A-Za-z_]+)*$")
            .map_err(|_| TimezoneError::ValidationFailed {
                reason: "Invalid regex pattern".to_string(),
            })?;

        if !timezone_regex.is_match(timezone) {
            return Err(TimezoneError::ValidationFailed {
                reason: format!(
                    "Invalid timezone format: '{}'. Expected format like 'UTC' or 'America/New_York'",
                    timezone
                ),
            });
        }

        Ok(())
    }

    /// Normalizes a timezone string (handles common aliases and formatting issues)
    ///
    /// # Arguments
    /// * `timezone` - The timezone string to normalize
    ///
    /// # Returns
    /// A normalized timezone string if possible, or the original if no normalization needed
    pub fn normalize_timezone(&self, timezone: &str) -> String {
        let trimmed = timezone.trim();

        // Handle common aliases
        match trimmed.to_uppercase().as_str() {
            "ET" | "EASTERN" => "US/Eastern",
            "CT" | "CENTRAL" => "US/Central",
            "MT" | "MOUNTAIN" => "US/Mountain",
            "PT" | "PACIFIC" => "US/Pacific",
            "GMT" => "Europe/London",
            _ => trimmed.to_string(),
        }
        .to_string()
    }

    /// Gets comprehensive timezone information including DST status and current offset
    ///
    /// # Arguments
    /// * `timezone` - The timezone to get information for
    ///
    /// # Returns
    /// `Ok(TimezoneInfo)` with comprehensive timezone data, `Err(TimezoneError)` if invalid
    pub fn get_timezone_info(&self, timezone: &str) -> TimezoneResult<TimezoneInfo> {
        let tz = self.parse_timezone(timezone)?;
        let observes_dst = self.timezone_observes_dst(timezone)?;
        let current_offset = self.get_current_utc_offset(timezone)?;

        // Get current local time in timezone
        let now = chrono::Utc::now();
        let local_time = now.with_timezone(&tz);

        Ok(TimezoneInfo {
            identifier: timezone.to_string(),
            observes_dst,
            current_utc_offset: current_offset,
            current_local_time: local_time.naive_local(),
            is_dst: local_time.offset().dst_offset().is_some(),
        })
    }
}

/// Comprehensive timezone information
#[derive(Debug, Clone)]
pub struct TimezoneInfo {
    /// The timezone identifier (e.g., "America/New_York")
    pub identifier: String,
    /// Whether this timezone observes Daylight Saving Time
    pub observes_dst: bool,
    /// Current offset from UTC in seconds
    pub current_utc_offset: i32,
    /// Current local time in the timezone
    pub current_local_time: chrono::NaiveDateTime,
    /// Whether the timezone is currently in DST
    pub is_dst: bool,
}

impl Default for TimezoneService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_timezone_validation() {
        let service = TimezoneService::new();

        let valid_timezones = vec![
            "UTC",
            "America/New_York",
            "Europe/London",
            "Asia/Tokyo",
            "Australia/Sydney",
        ];

        for timezone in valid_timezones {
            assert!(
                service.validate_timezone(timezone).is_ok(),
                "Expected '{}' to be valid", timezone
            );
        }
    }

    #[test]
    fn test_invalid_timezone_validation() {
        let service = TimezoneService::new();

        let invalid_timezones = vec![
            "",
            "Invalid/Timezone",
            "America/NonExistent",
            "NotATimezone",
            "123/456",
        ];

        for timezone in invalid_timezones {
            assert!(
                service.validate_timezone(timezone).is_err(),
                "Expected '{}' to be invalid", timezone
            );
        }
    }

    #[test]
    fn test_timezone_parsing() {
        let service = TimezoneService::new();

        let result = service.parse_timezone("UTC");
        assert!(result.is_ok());

        let result = service.parse_timezone("Invalid/Timezone");
        assert!(result.is_err());
    }

    #[test]
    fn test_common_timezones_list() {
        let service = TimezoneService::new();
        let common_timezones = service.get_common_timezones();

        assert!(!common_timezones.is_empty());
        assert!(common_timezones.contains(&"UTC"));
        assert!(common_timezones.contains(&"America/New_York"));
    }

    #[test]
    fn test_timezone_normalization() {
        let service = TimezoneService::new();

        assert_eq!(service.normalize_timezone("  UTC  "), "UTC");
        assert_eq!(service.normalize_timezone("ET"), "US/Eastern");
        assert_eq!(service.normalize_timezone("GMT"), "Europe/London");
        assert_eq!(service.normalize_timezone("America/New_York"), "America/New_York");
    }

    #[test]
    fn test_timezone_format_validation() {
        let service = TimezoneService::new();

        assert!(service.validate_timezone_format("UTC").is_ok());
        assert!(service.validate_timezone_format("America/New_York").is_ok());
        assert!(service.validate_timezone_format("").is_err());
        assert!(service.validate_timezone_format("Invalid/Timezone/With/Extra/Slash").is_err());
    }

    #[test]
    fn test_utc_offset() {
        let service = TimezoneService::new();

        let offset = service.get_current_utc_offset("UTC").unwrap();
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_timezone_info() {
        let service = TimezoneService::new();

        let info = service.get_timezone_info("UTC").unwrap();
        assert_eq!(info.identifier, "UTC");
        assert_eq!(info.current_utc_offset, 0);
        assert!(!info.observes_dst);
        assert!(!info.is_dst);
    }
}