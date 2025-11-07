//! Time Provider Trait and Implementations
//!
//! Provides time abstraction for deterministic testing and production use.
//! This enables mocking time in tests while using system time in production.

use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;
use std::sync::Arc;

/// Trait for providing time functionality
/// This enables dependency injection and testing with deterministic time
pub trait TimeProvider: Send + Sync {
    /// Get the current UTC time
    fn now_utc(&self) -> DateTime<Utc>;

    /// Get current time in a specific timezone
    fn now_in_timezone(&self, timezone: Tz) -> DateTime<Tz> {
        self.now_utc().with_timezone(&timezone)
    }

    /// Convert UTC time to a specific timezone
    fn to_timezone(&self, utc_time: &DateTime<Utc>, timezone: Tz) -> DateTime<Tz> {
        utc_time.with_timezone(&timezone)
    }

    /// Get current Unix timestamp (seconds since epoch)
    fn now_timestamp(&self) -> i64 {
        self.now_utc().timestamp()
    }

    /// Get current Unix timestamp in milliseconds
    fn now_timestamp_millis(&self) -> i64 {
        self.now_utc().timestamp_millis()
    }

    /// Parse a string into a DateTime in the specified timezone
    fn parse_in_timezone(
        &self,
        date_str: &str,
        format: &str,
        timezone: Tz,
    ) -> Result<DateTime<Tz>, chrono::ParseError> {
        timezone.datetime_from_str(date_str, format)
    }

    /// Create a DateTime in the specified timezone
    fn datetime_in_timezone(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        timezone: Tz,
    ) -> Result<DateTime<Tz>, chrono::ParseError> {
        timezone.with_ymd_and_hms(year, month, day, hour, min, sec).single()
            .ok_or_else(|| chrono::ParseError::OutOfRange)
    }
}

/// System time provider for production use
#[derive(Debug, Clone)]
pub struct SystemTimeProvider;

impl SystemTimeProvider {
    /// Create a new system time provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Mock time provider for testing
#[derive(Debug, Clone)]
pub struct MockTimeProvider {
    /// Current mock time
    current_time: Arc<std::sync::Mutex<DateTime<Utc>>>,
}

impl MockTimeProvider {
    /// Create a new mock time provider starting from the given time
    pub fn new(start_time: DateTime<Utc>) -> Self {
        Self {
            current_time: Arc::new(std::sync::Mutex::new(start_time)),
        }
    }

    /// Create a mock time provider starting from now
    pub fn new_from_now() -> Self {
        Self::new(Utc::now())
    }

    /// Create a mock time provider starting from a specific date/time
    pub fn new_from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Result<Self, chrono::ParseError> {
        let start_time = Utc.with_ymd_and_hms(year, month, day, hour, min, sec).single()
            .ok_or_else(|| chrono::ParseError::OutOfRange)?;
        Ok(Self::new(start_time))
    }

    /// Set the current mock time
    pub fn set_time(&self, new_time: DateTime<Utc>) {
        if let Ok(mut time) = self.current_time.lock() {
            *time = new_time;
        }
    }

    /// Advance the mock time by the specified duration
    pub fn advance(&self, duration: chrono::Duration) {
        if let Ok(mut time) = self.current_time.lock() {
            *time = *time + duration;
        }
    }

    /// Advance the mock time by the specified number of seconds
    pub fn advance_seconds(&self, seconds: i64) {
        self.advance(chrono::Duration::seconds(seconds));
    }

    /// Advance the mock time by the specified number of minutes
    pub fn advance_minutes(&self, minutes: i64) {
        self.advance(chrono::Duration::minutes(minutes));
    }

    /// Advance the mock time by the specified number of hours
    pub fn advance_hours(&self, hours: i64) {
        self.advance(chrono::Duration::hours(hours));
    }

    /// Advance the mock time by the specified number of days
    pub fn advance_days(&self, days: i64) {
        self.advance(chrono::Duration::days(days));
    }

    /// Get the current mock time
    pub fn current_time(&self) -> DateTime<Utc> {
        if let Ok(time) = self.current_time.lock() {
            *time
        } else {
            Utc::now() // Fallback to system time if lock fails
        }
    }
}

impl Default for MockTimeProvider {
    fn default() -> Self {
        Self::new_from_now()
    }
}

impl TimeProvider for MockTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        self.current_time()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::US::America::New_York;

    #[test]
    fn test_system_time_provider() {
        let provider = SystemTimeProvider::new();
        let now = provider.now_utc();
        let timestamp = provider.now_timestamp();

        // System time should be reasonable (within last minute)
        let system_now = Utc::now();
        assert!((system_now - now).num_seconds().abs() < 60);

        // Timestamp should match the datetime
        assert_eq!(timestamp, now.timestamp());
    }

    #[test]
    fn test_mock_time_provider() {
        let start_time = Utc.with_ymd_and_hms(2025, 1, 7, 10, 30, 0).single().unwrap();
        let provider = MockTimeProvider::new(start_time);

        assert_eq!(provider.now_utc(), start_time);
        assert_eq!(provider.now_timestamp(), start_time.timestamp());
    }

    #[test]
    fn test_mock_time_advance() {
        let start_time = Utc.with_ymd_and_hms(2025, 1, 7, 10, 30, 0).single().unwrap();
        let provider = MockTimeProvider::new(start_time);

        // Advance by 1 hour
        provider.advance_hours(1);
        let expected = start_time + chrono::Duration::hours(1);
        assert_eq!(provider.now_utc(), expected);

        // Advance by 30 minutes
        provider.advance_minutes(30);
        let expected = expected + chrono::Duration::minutes(30);
        assert_eq!(provider.now_utc(), expected);
    }

    #[test]
    fn test_timezone_conversion() {
        let provider = SystemTimeProvider::new();
        let utc_time = provider.now_utc();

        // Convert to New York timezone
        let ny_time = provider.to_timezone(&utc_time, New_York);

        // Should be the same moment, just different timezone
        assert_eq!(utc_time, ny_time.with_timezone(&Utc));
    }

    #[test]
    fn test_create_datetime_in_timezone() {
        let provider = SystemTimeProvider::new();

        // Create 2025-01-07 10:30:00 in New York
        let ny_time = provider.datetime_in_timezone(2025, 1, 7, 10, 30, 0, New_York).unwrap();

        assert_eq!(ny_time.year(), 2025);
        assert_eq!(ny_time.month(), 1);
        assert_eq!(ny_time.day(), 7);
        assert_eq!(ny_time.hour(), 10);
        assert_eq!(ny_time.minute(), 30);
        assert_eq!(ny_time.second(), 0);
        assert_eq!(ny_time.timezone(), New_York);
    }
}