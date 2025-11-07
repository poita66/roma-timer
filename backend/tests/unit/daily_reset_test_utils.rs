//! Test Utilities for Daily Session Reset Unit Tests
//!
//! Provides common test utilities, fixtures, and mock time support for daily reset testing:
//! - Mock time provider integration
//! - Test data fixtures and factories
//! - Database test setup and cleanup
//! - Assertion helpers and test matchers

use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::Tz;
use sqlx::SqlitePool;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use crate::models::{UserConfiguration, DailyResetTime};
use crate::services::{TimeProvider, MockTimeProvider};
use crate::database::{DatabaseManager, DailyResetDatabaseExtensions};

/// Test context for daily reset unit tests
#[derive(Debug, Clone)]
pub struct DailyResetTestContext {
    /// Mock time provider
    pub time_provider: Arc<MockTimeProvider>,

    /// Test database manager
    pub db_manager: Arc<DatabaseManager>,

    /// Test user ID
    pub test_user_id: String,

    /// Test device ID
    pub test_device_id: String,

    /// Temporary directory for test files
    #[allow(dead_code)]
    temp_dir: Arc<TempDir>,
}

impl DailyResetTestContext {
    /// Create a new test context with all components initialized
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = Arc::new(TempDir::new()?);
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Initialize mock time provider
        let start_time = Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap();
        let time_provider = Arc::new(MockTimeProvider::new(start_time));

        // Initialize test database
        let db_manager = Arc::new(DatabaseManager::new(&db_url).await?);
        db_manager.migrate().await?;

        // Create test user and device IDs
        let test_user_id = Uuid::new_v4().to_string();
        let test_device_id = Uuid::new_v4().to_string();

        Ok(Self {
            time_provider,
            db_manager,
            test_user_id,
            test_device_id,
            temp_dir,
        })
    }

    /// Create a test user configuration with daily reset settings
    pub async fn create_test_user_config(
        &self,
        timezone: &str,
        reset_time: DailyResetTime,
        reset_enabled: bool,
    ) -> Result<UserConfiguration, Box<dyn std::error::Error>> {
        let mut config = UserConfiguration::with_id(self.test_user_id.clone());
        config.set_timezone(timezone.to_string())?;
        config.set_daily_reset_time(reset_time)?;
        config.set_daily_reset_enabled(reset_enabled);

        // Save to database
        self.db_manager.save_user_configuration(&config).await?;
        Ok(config)
    }

    /// Create a default test user configuration
    pub async fn create_default_test_user_config(
        &self,
    ) -> Result<UserConfiguration, Box<dyn std::error::Error>> {
        self.create_test_user_config(
            "UTC",
            DailyResetTime::midnight(),
            false,
        ).await
    }

    /// Advance the mock time by the specified duration
    pub fn advance_time(&self, hours: i64, minutes: i64, seconds: i64) {
        let duration = chrono::Duration::hours(hours) +
            chrono::Duration::minutes(minutes) +
            chrono::Duration::seconds(seconds);
        self.time_provider.advance(duration);
    }

    /// Set the mock time to a specific datetime
    pub fn set_time(&self, datetime: DateTime<Utc>) {
        self.time_provider.set_time(datetime);
    }

    /// Get current mock time
    pub fn current_time(&self) -> DateTime<Utc> {
        self.time_provider.now_utc()
    }

    /// Create mock session data for testing
    pub fn create_mock_session_data(
        &self,
        work_sessions: u32,
        work_seconds: u64,
        break_seconds: u64,
    ) -> MockSessionData {
        MockSessionData {
            user_id: self.test_user_id.clone(),
            date: self.time_provider.now_utc().format("%Y-%m-%d").to_string(),
            timezone: "UTC".to_string(),
            work_sessions_completed: work_sessions,
            total_work_seconds: work_seconds,
            total_break_seconds: break_seconds,
            manual_overrides: 0,
            final_session_count: work_sessions,
        }
    }
}

/// Mock session data for testing
#[derive(Debug, Clone)]
pub struct MockSessionData {
    pub user_id: String,
    pub date: String,
    pub timezone: String,
    pub work_sessions_completed: u32,
    pub total_work_seconds: u64,
    pub total_break_seconds: u64,
    pub manual_overrides: u32,
    pub final_session_count: u32,
}

/// Factory methods for creating test data
pub mod factories {
    use super::*;
    use crate::models::{DailyResetTimeType};

    /// Create a test daily reset time at midnight
    pub fn midnight_reset_time() -> DailyResetTime {
        DailyResetTime::midnight()
    }

    /// Create a test daily reset time at specific hour
    pub fn hourly_reset_time(hour: u8) -> Result<DailyResetTime, crate::models::UserConfigurationError> {
        DailyResetTime::hour(hour)
    }

    /// Create a test daily reset time with custom time
    pub fn custom_reset_time(time: &str) -> Result<DailyResetTime, crate::models::UserConfigurationError> {
        DailyResetTime::custom(time.to_string())
    }

    /// Create a test user configuration with basic settings
    pub fn test_user_configuration(user_id: String) -> UserConfiguration {
        let mut config = UserConfiguration::with_id(user_id);
        config.timezone = "UTC".to_string();
        config.daily_reset_enabled = false;
        config.today_session_count = 0;
        config
    }

    /// Create a test user configuration with daily reset enabled
    pub fn test_user_configuration_with_daily_reset(
        user_id: String,
        timezone: &str,
        reset_time: DailyResetTime,
    ) -> Result<UserConfiguration, crate::models::UserConfigurationError> {
        let mut config = test_user_configuration(user_id);
        config.set_timezone(timezone.to_string())?;
        config.set_daily_reset_time(reset_time)?;
        config.set_daily_reset_enabled(true);
        Ok(config)
    }

    /// Create test timezone data
    pub fn test_timezone_data() -> Vec<(&'static str, bool)> {
        vec![
            ("UTC", true),
            ("America/New_York", true),
            ("Europe/London", true),
            ("Asia/Tokyo", true),
            ("Australia/Sydney", true),
            ("Invalid/Timezone", false),
            ("", false),
        ]
    }

    /// Create test reset time scenarios
    pub fn test_reset_time_scenarios() -> Vec<(DailyResetTime, &'static str)> {
        vec![
            (DailyResetTime::midnight(), "midnight"),
            (DailyResetTime::hour(7).unwrap(), "7am"),
            (DailyResetTime::hour(18).unwrap(), "6pm"),
            (DailyResetTime::custom("09:30".to_string()).unwrap(), "9:30am"),
            (DailyResetTime::custom("23:45".to_string()).unwrap(), "11:45pm"),
        ]
    }
}

/// Assertion helpers for testing
pub mod assertions {
    use super::*;
    use crate::models::{UserConfigurationError};

    /// Assert that a user configuration has specific daily reset settings
    pub fn assert_daily_reset_config(
        config: &UserConfiguration,
        expected_timezone: &str,
        expected_enabled: bool,
        expected_session_count: u32,
    ) {
        assert_eq!(config.timezone, expected_timezone);
        assert_eq!(config.daily_reset_enabled, expected_enabled);
        assert_eq!(config.today_session_count, expected_session_count);
    }

    /// Assert that a daily reset time is valid
    pub fn assert_valid_reset_time(reset_time: &DailyResetTime) {
        let validation_result = reset_time.validate();
        assert!(validation_result.is_ok(),
            "Expected valid reset time, got error: {:?}", validation_result);
    }

    /// Assert that a daily reset time is invalid
    pub fn assert_invalid_reset_time(reset_time: &DailyResetTime, expected_error: &str) {
        let validation_result = reset_time.validate();
        assert!(validation_result.is_err(),
            "Expected invalid reset time, but validation passed");

        match validation_result.unwrap_err() {
            UserConfigurationError::InvalidResetHour(hour) => {
                assert!(expected_error.contains(&hour.to_string()));
            }
            UserConfigurationError::InvalidResetTime(time) => {
                assert!(expected_error.contains(&time));
            }
            _ => panic!("Unexpected error type: {:?}", validation_result),
        }
    }

    /// Assert timezone validation results
    pub fn assert_timezone_validation(timezone: &str, should_be_valid: bool) {
        let result: Result<Tz, _> = timezone.parse();

        if should_be_valid {
            assert!(result.is_ok(), "Expected timezone '{}' to be valid", timezone);
        } else {
            assert!(result.is_err(), "Expected timezone '{}' to be invalid", timezone);
        }
    }

    /// Assert that session count operations work correctly
    pub fn assert_session_count_operations(
        initial_count: u32,
        increment_count: u32,
        expected_final_count: u32,
    ) {
        assert!(initial_count + increment_count == expected_final_count,
            "Session count arithmetic failed: {} + {} != {}",
            initial_count, increment_count, expected_final_count);
    }

    /// Assert time-based calculations
    pub fn assert_time_calculations(
        current_time: DateTime<Utc>,
        hours_to_advance: i64,
        expected_future_time: DateTime<Utc>,
    ) {
        let actual_future_time = current_time + chrono::Duration::hours(hours_to_advance);
        assert_eq!(actual_future_time, expected_future_time,
            "Time calculation failed: {} + {} hours != {}",
            current_time, hours_to_advance, expected_future_time);
    }
}

/// Database test helpers
pub mod database {
    use super::*;

    /// Create a test database in memory
    pub async fn create_test_database() -> Result<DatabaseManager, Box<dyn std::error::Error>> {
        let db_manager = DatabaseManager::new("sqlite::memory:").await?;
        db_manager.migrate().await?;
        Ok(db_manager)
    }

    /// Create a test database with file
    pub async fn create_test_database_with_file() -> Result<(DatabaseManager, TempDir), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let db_manager = DatabaseManager::new(&db_url).await?;
        db_manager.migrate().await?;

        Ok((db_manager, temp_dir))
    }

    /// Cleanup test database records
    pub async fn cleanup_test_records(
        db_manager: &DatabaseManager,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Clean up user configuration
        sqlx::query("DELETE FROM user_configurations WHERE id = ?1")
            .bind(user_id)
            .execute(match db_manager.pool {
                crate::database::DatabasePool::Sqlite(pool) => pool,
            })
            .await?;

        // Clean up related records
        sqlx::query("DELETE FROM daily_session_stats WHERE user_configuration_id = ?1")
            .bind(user_id)
            .execute(match db_manager.pool {
                crate::database::DatabasePool::Sqlite(pool) => pool,
            })
            .await?;

        sqlx::query("DELETE FROM session_reset_events WHERE user_configuration_id = ?1")
            .bind(user_id)
            .execute(match db_manager.pool {
                crate::database::DatabasePool::Sqlite(pool) => pool,
            })
            .await?;

        sqlx::query("DELETE FROM scheduled_tasks WHERE user_configuration_id = ?1")
            .bind(user_id)
            .execute(match db_manager.pool {
                crate::database::DatabasePool::Sqlite(pool) => pool,
            })
            .await?;

        Ok(())
    }
}

/// Time-based test helpers
pub mod time {
    use super::*;

    /// Create a mock time provider with a specific start time
    pub fn create_mock_time_provider(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<MockTimeProvider, chrono::ParseError> {
        let start_time = Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .ok_or(chrono::ParseError::OutOfRange)?;
        Ok(MockTimeProvider::new(start_time))
    }

    /// Create mock time provider with current time
    pub fn create_current_time_provider() -> MockTimeProvider {
        MockTimeProvider::new_from_now()
    }

    /// Test time zone conversion scenarios
    pub fn test_timezone_conversion_scenarios() -> Vec<(DateTime<Utc>, &'static str, &'static str)> {
        vec![
            // (UTC time, timezone, expected local time)
            (
                Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap(),
                "America/New_York",
                "2025-01-07 07:00:00 EST", // UTC-5 in January
            ),
            (
                Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap(),
                "Europe/London",
                "2025-01-07 12:00:00 GMT", // UTC+0 in January
            ),
            (
                Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap(),
                "Asia/Tokyo",
                "2025-01-07 21:00:00 JST", // UTC+9
            ),
        ]
    }

    /// Test daily reset scenarios across different timezones
    pub fn test_daily_reset_scenarios() -> Vec<DailyResetScenario> {
        vec![
            DailyResetScenario {
                timezone: "UTC",
                reset_hour: Some(0),
                current_utc_hour: 23,
                expected_reset_soon: true,
                description: "Midnight reset in UTC, 1 hour away",
            },
            DailyResetScenario {
                timezone: "America/New_York",
                reset_hour: Some(7),
                current_utc_hour: 11,
                expected_reset_soon: true,
                description: "7 AM reset in New York, 1 hour away (UTC)",
            },
            DailyResetScenario {
                timezone: "Asia/Tokyo",
                reset_hour: Some(9),
                current_utc_hour: 0,
                expected_reset_soon: false,
                description: "9 AM reset in Tokyo, many hours away",
            },
        ]
    }

    /// Scenario for testing daily reset timing
    #[derive(Debug, Clone)]
    pub struct DailyResetScenario {
        pub timezone: &'static str,
        pub reset_hour: Option<u8>,
        pub current_utc_hour: u32,
        pub expected_reset_soon: bool,
        pub description: &'static str,
    }
}

/// Common test macros
#[macro_export]
macro_rules! assert_daily_reset_eq {
    ($left:expr, $right:expr, $user_id:expr) => {
        assert_eq!($left, $right,
            "Daily reset assertion failed for user {}: expected {:?}, got {:?}",
            $user_id, $right, $left);
    };
}

#[macro_export]
macro_rules! assert_time_provider_eq {
    ($provider:expr, $expected:expr) => {
        assert_eq!($provider.now_utc(), $expected,
            "Mock time provider mismatch: expected {}, got {}",
            $expected, $provider.now_utc());
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use factories::*;

    #[tokio::test]
    async fn test_daily_reset_test_context_creation() {
        let context = DailyResetTestContext::new().await.unwrap();

        assert!(!context.test_user_id.is_empty());
        assert!(!context.test_device_id.is_empty());
        assert_eq!(context.current_time(),
            Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap());
    }

    #[tokio::test]
    async fn test_create_test_user_config() {
        let context = DailyResetTestContext::new().await.unwrap();

        let reset_time = hourly_reset_time(7).unwrap();
        let config = context.create_test_user_config(
            "UTC",
            reset_time,
            true
        ).await.unwrap();

        assert_eq!(config.timezone, "UTC");
        assert_eq!(config.daily_reset_enabled, true);
        assert_eq!(config.daily_reset_time_hour, Some(7));
    }

    #[test]
    fn test_factory_methods() {
        let midnight = midnight_reset_time();
        assert_eq!(midnight.time_type, DailyResetTimeType::Midnight);

        let hourly = hourly_reset_time(7).unwrap();
        assert_eq!(hourly.time_type, DailyResetTimeType::Hour);
        assert_eq!(hourly.hour, Some(7));

        let custom = custom_reset_time("14:30").unwrap();
        assert_eq!(custom.time_type, DailyResetTimeType::Custom);
        assert_eq!(custom.time, Some("14:30".to_string()));
    }

    #[test]
    fn test_assertion_helpers() {
        use assertions::*;

        let reset_time = midnight_reset_time();
        assert_valid_reset_time(&reset_time);

        assert_timezone_validation("UTC", true);
        assert_timezone_validation("America/New_York", true);
        assert_timezone_validation("Invalid/Timezone", false);

        assert_session_count_operations(5, 3, 8);
    }

    #[test]
    fn test_time_helpers() {
        use time::*;

        let provider = create_mock_time_provider(2025, 1, 7, 10, 0, 0).unwrap();
        assert_eq!(provider.now_utc(),
            Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap());

        let scenarios = test_daily_reset_scenarios();
        assert!(!scenarios.is_empty());
        assert_eq!(scenarios[0].timezone, "UTC");
    }
}