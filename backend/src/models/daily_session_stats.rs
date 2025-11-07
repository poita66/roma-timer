//! Daily Session Statistics Model
//!
//! Represents aggregated daily statistics for user pomodoro sessions.
//! Tracks work sessions, break times, manual overrides, and analytics.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Daily session statistics for analytics and reporting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DailySessionStats {
    /// Unique identifier for the statistics record
    pub id: String,

    /// User configuration ID this statistics belongs to
    #[sqlx(rename = "user_configuration_id")]
    pub user_configuration_id: String,

    /// Date in YYYY-MM-DD format (UTC)
    #[sqlx(rename = "date")]
    pub date: String,

    /// User's timezone for this statistics record
    #[sqlx(rename = "timezone")]
    pub timezone: String,

    /// Number of work sessions completed
    #[sqlx(rename = "work_sessions_completed")]
    pub work_sessions_completed: i64,

    /// Total time spent in work sessions (seconds)
    #[sqlx(rename = "total_work_seconds")]
    pub total_work_seconds: i64,

    /// Total time spent in breaks (seconds)
    #[sqlx(rename = "total_break_seconds")]
    pub total_break_seconds: i64,

    /// Number of manual session overrides
    #[sqlx(rename = "manual_overrides")]
    pub manual_overrides: i64,

    /// Final session count for the day
    #[sqlx(rename = "final_session_count")]
    pub final_session_count: i64,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: i64,

    /// Last update timestamp (Unix timestamp)
    #[sqlx(rename = "updated_at")]
    pub updated_at: i64,
}

impl DailySessionStats {
    /// Create a new daily session statistics record
    pub fn new(
        user_configuration_id: String,
        date: String,
        timezone: String,
    ) -> Self {
        let now = Utc::now().timestamp();
        let id = format!("daily_stats_{}_{}_{}", user_configuration_id, date, Uuid::new_v4());

        Self {
            id,
            user_configuration_id,
            date,
            timezone,
            work_sessions_completed: 0,
            total_work_seconds: 0,
            total_break_seconds: 0,
            manual_overrides: 0,
            final_session_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with initial session count
    pub fn with_initial_count(
        user_configuration_id: String,
        date: String,
        timezone: String,
        initial_count: u32,
    ) -> Self {
        let mut stats = Self::new(user_configuration_id, date, timezone);
        stats.final_session_count = initial_count as i64;
        stats
    }

    /// Update work session statistics
    pub fn add_work_session(&mut self, work_duration_seconds: u32) {
        self.work_sessions_completed += 1;
        self.total_work_seconds += work_duration_seconds as i64;
        self.final_session_count = self.work_sessions_completed; // Update final count
        self.touch();
    }

    /// Add break time statistics
    pub fn add_break_time(&mut self, break_duration_seconds: u32) {
        self.total_break_seconds += break_duration_seconds as i64;
        self.touch();
    }

    /// Record a manual override
    pub fn add_manual_override(&mut self) {
        self.manual_overrides += 1;
        self.touch();
    }

    /// Set final session count manually
    pub fn set_final_session_count(&mut self, count: u32) {
        self.final_session_count = count as i64;
        self.touch();
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp();
    }

    /// Get work sessions as u32
    pub fn work_sessions_completed(&self) -> u32 {
        self.work_sessions_completed as u32
    }

    /// Get total work seconds as u32
    pub fn total_work_seconds(&self) -> u32 {
        self.total_work_seconds as u32
    }

    /// Get total break seconds as u32
    pub fn total_break_seconds(&self) -> u32 {
        self.total_break_seconds as u32
    }

    /// Get manual overrides as u32
    pub fn manual_overrides(&self) -> u32 {
        self.manual_overrides as u32
    }

    /// Get final session count as u32
    pub fn final_session_count(&self) -> u32 {
        self.final_session_count as u32
    }

    /// Get work duration in minutes
    pub fn total_work_minutes(&self) -> u32 {
        self.total_work_seconds / 60
    }

    /// Get break duration in minutes
    pub fn total_break_minutes(&self) -> u32 {
        self.total_break_seconds / 60
    }

    /// Get average work session duration in minutes
    pub fn average_work_session_minutes(&self) -> f64 {
        if self.work_sessions_completed > 0 {
            self.total_work_seconds as f64 / (self.work_sessions_completed as f64 * 60.0)
        } else {
            0.0
        }
    }

    /// Get work to break ratio
    pub fn work_to_break_ratio(&self) -> f64 {
        if self.total_break_seconds > 0 {
            self.total_work_seconds as f64 / self.total_break_seconds as f64
        } else {
            0.0
        }
    }

    /// Get productivity score (0-100)
    pub fn productivity_score(&self) -> u32 {
        if self.work_sessions_completed == 0 {
            return 0;
        }

        // Base score from completed sessions (max 50 points)
        let session_score = (self.work_sessions_completed.min(10) as f64 * 5.0) as u32;

        // Bonus score for long work sessions (max 30 points)
        let duration_bonus = if self.total_work_minutes() >= 200 { // 3+ hours
            30
        } else if self.total_work_minutes() >= 120 { // 2+ hours
            20
        } else if self.total_work_minutes() >= 60 { // 1+ hour
            10
        } else {
            0
        };

        // Penalty for too many manual overrides (max 20 points penalty)
        let override_penalty = (self.manual_overrides().min(4) * 5) as u32;

        let total_score = session_score + duration_bonus - override_penalty;
        total_score.min(100).max(0)
    }

    /// Validate the daily session statistics
    pub fn validate(&self) -> Result<(), DailySessionStatsError> {
        if self.user_configuration_id.is_empty() {
            return Err(DailySessionStatsError::InvalidUserId);
        }

        if self.date.is_empty() {
            return Err(DailySessionStatsError::InvalidDate);
        }

        if self.timezone.is_empty() {
            return Err(DailySessionStatsError::InvalidTimezone);
        }

        if self.work_sessions_completed < 0 {
            return Err(DailySessionStatsError::InvalidSessionCount);
        }

        if self.total_work_seconds < 0 {
            return Err(DailySessionStatsError::InvalidWorkTime);
        }

        if self.total_break_seconds < 0 {
            return Err(DailySessionStatsError::InvalidBreakTime);
        }

        if self.manual_overrides < 0 {
            return Err(DailySessionStatsError::InvalidOverrideCount);
        }

        if self.final_session_count < 0 {
            return Err(DailySessionStatsError::InvalidFinalCount);
        }

        if self.updated_at < self.created_at {
            return Err(DailySessionStatsError::InvalidTimestamps);
        }

        // Validate date format (YYYY-MM-DD)
        if !is_valid_date_format(&self.date) {
            return Err(DailySessionStatsError::InvalidDateFormat);
        }

        Ok(())
    }

    /// Merge with another statistics record (for conflict resolution)
    pub fn merge(&mut self, other: &DailySessionStats) {
        // Use the higher values for most fields
        self.work_sessions_completed = self.work_sessions_completed.max(other.work_sessions_completed);
        self.total_work_seconds = self.total_work_seconds.max(other.total_work_seconds);
        self.total_break_seconds = self.total_break_seconds.max(other.total_break_seconds);
        self.manual_overrides = self.manual_overrides.max(other.manual_overrides);
        self.final_session_count = self.final_session_count.max(other.final_session_count);

        // Use the most recent update time
        self.updated_at = self.updated_at.max(other.updated_at);
        self.touch(); // Update to current time
    }
}

impl Default for DailySessionStats {
    fn default() -> Self {
        Self::new("default".to_string(), "2025-01-07".to_string(), "UTC".to_string())
    }
}

/// Daily session statistics validation errors
#[derive(Debug, thiserror::Error)]
pub enum DailySessionStatsError {
    #[error("Invalid user configuration ID")]
    InvalidUserId,

    #[error("Invalid date format")]
    InvalidDate,

    #[error("Invalid timezone")]
    InvalidTimezone,

    #[error("Invalid session count: {0}")]
    InvalidSessionCount(i64),

    #[error("Invalid work time: {0}")]
    InvalidWorkTime(i64),

    #[error("Invalid break time: {0}")]
    InvalidBreakTime(i64),

    #[error("Invalid manual override count: {0}")]
    InvalidOverrideCount(i64),

    #[error("Invalid final session count: {0}")]
    InvalidFinalCount(i64),

    #[error("Invalid timestamps")]
    InvalidTimestamps,

    #[error("Invalid date format (expected YYYY-MM-DD)")]
    InvalidDateFormat,
}

/// Helper function to validate date format YYYY-MM-DD
fn is_valid_date_format(date_str: &str) -> bool {
    if date_str.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    // Check basic format
    if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }

    // Check if all parts are numeric
    parts.iter().all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

/// DTO for creating daily session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDailySessionStatsRequest {
    pub user_configuration_id: String,
    pub date: String,
    pub timezone: String,
    pub work_sessions_completed: Option<u32>,
    pub total_work_seconds: Option<u32>,
    pub total_break_seconds: Option<u32>,
    pub manual_overrides: Option<u32>,
    pub final_session_count: Option<u32>,
}

impl CreateDailySessionStatsRequest {
    /// Convert to DailySessionStats model
    pub fn to_model(self) -> DailySessionStats {
        let mut stats = DailySessionStats::new(
            self.user_configuration_id,
            self.date,
            self.timezone,
        );

        if let Some(count) = self.work_sessions_completed {
            stats.work_sessions_completed = count as i64;
        }

        if let Some(seconds) = self.total_work_seconds {
            stats.total_work_seconds = seconds as i64;
        }

        if let Some(seconds) = self.total_break_seconds {
            stats.total_break_seconds = seconds as i64;
        }

        if let Some(count) = self.manual_overrides {
            stats.manual_overrides = count as i64;
        }

        if let Some(count) = self.final_session_count {
            stats.final_session_count = count as i64;
        }

        stats
    }
}

/// DTO for updating daily session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDailySessionStatsRequest {
    pub work_sessions_completed: Option<u32>,
    pub total_work_seconds: Option<u32>,
    pub total_break_seconds: Option<u32>,
    pub manual_overrides: Option<u32>,
    pub final_session_count: Option<u32>,
}

impl UpdateDailySessionStatsRequest {
    /// Apply updates to existing statistics
    pub fn apply_to(&self, stats: &mut DailySessionStats) {
        if let Some(count) = self.work_sessions_completed {
            stats.work_sessions_completed = count as i64;
        }

        if let Some(seconds) = self.total_work_seconds {
            stats.total_work_seconds = seconds as i64;
        }

        if let Some(seconds) = self.total_break_seconds {
            stats.total_break_seconds = seconds as i64;
        }

        if let Some(count) = self.manual_overrides {
            stats.manual_overrides = count as i64;
        }

        if let Some(count) = self.final_session_count {
            stats.final_session_count = count as i64;
        }

        stats.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_session_stats_creation() {
        let stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        assert_eq!(stats.user_configuration_id, "user-123");
        assert_eq!(stats.date, "2025-01-07");
        assert_eq!(stats.timezone, "UTC");
        assert_eq!(stats.work_sessions_completed, 0);
        assert_eq!(stats.total_work_seconds, 0);
        assert_eq!(stats.final_session_count, 0);
        assert!(!stats.id.is_empty());
    }

    #[test]
    fn test_add_work_session() {
        let mut stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        stats.add_work_session(1500); // 25 minutes

        assert_eq!(stats.work_sessions_completed, 1);
        assert_eq!(stats.total_work_seconds, 1500);
        assert_eq!(stats.final_session_count, 1);
        assert!(stats.updated_at > stats.created_at);
    }

    #[test]
    fn test_add_break_time() {
        let mut stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        stats.add_break_time(300); // 5 minutes

        assert_eq!(stats.total_break_seconds, 300);
        assert!(stats.updated_at > stats.created_at);
    }

    #[test]
    fn test_manual_override() {
        let mut stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        stats.add_manual_override();
        stats.add_manual_override();

        assert_eq!(stats.manual_overrides, 2);
    }

    #[test]
    fn test_calculations() {
        let mut stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        // Add 3 work sessions of 25 minutes each
        for _ in 0..3 {
            stats.add_work_session(1500);
        }

        // Add 2 breaks of 5 minutes each
        stats.add_break_time(300);
        stats.add_break_time(300);

        assert_eq!(stats.total_work_minutes(), 75);
        assert_eq!(stats.total_break_minutes(), 10);
        assert_eq!(stats.average_work_session_minutes(), 25.0);
        assert_eq!(stats.work_to_break_ratio(), 7.5);
        assert_eq!(stats.productivity_score(), 35); // 3 sessions * 5 + 10 duration bonus
    }

    #[test]
    fn test_productivity_score() {
        let mut stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        // Test zero sessions
        assert_eq!(stats.productivity_score(), 0);

        // Test with many sessions
        for _ in 0..15 {
            stats.add_work_session(1500);
        }

        assert!(stats.productivity_score() > 50);
        assert!(stats.productivity_score() <= 100);
    }

    #[test]
    fn test_validation() {
        let stats = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        assert!(stats.validate().is_ok());

        // Test invalid date format
        let mut invalid_stats = stats.clone();
        invalid_stats.date = "invalid-date".to_string();
        assert!(invalid_stats.validate().is_err());
    }

    #[test]
    fn test_merge() {
        let mut stats1 = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        let mut stats2 = DailySessionStats::new(
            "user-123".to_string(),
            "2025-01-07".to_string(),
            "UTC".to_string(),
        );

        stats1.add_work_session(1500);
        stats2.add_work_session(1200);
        stats2.add_break_time(300);

        stats1.merge(&stats2);

        assert_eq!(stats1.work_sessions_completed, 1); // Takes max
        assert_eq!(stats1.total_work_seconds, 1500); // Takes max
        assert_eq!(stats1.total_break_seconds, 300); // Takes max from stats2
    }

    #[test]
    fn test_create_request_to_model() {
        let request = CreateDailySessionStatsRequest {
            user_configuration_id: "user-123".to_string(),
            date: "2025-01-07".to_string(),
            timezone: "UTC".to_string(),
            work_sessions_completed: Some(5),
            total_work_seconds: Some(7500),
            total_break_seconds: Some(1000),
            manual_overrides: Some(1),
            final_session_count: Some(6),
        };

        let stats = request.to_model();

        assert_eq!(stats.user_configuration_id, "user-123");
        assert_eq!(stats.work_sessions_completed, 5);
        assert_eq!(stats.total_work_seconds, 7500);
        assert_eq!(stats.total_break_seconds, 1000);
        assert_eq!(stats.manual_overrides, 1);
        assert_eq!(stats.final_session_count, 6);
    }

    #[test]
    fn test_date_format_validation() {
        assert!(is_valid_date_format("2025-01-07"));
        assert!(is_valid_date_format("2025-12-31"));
        assert!(!is_valid_date_format("2025-1-7"));
        assert!(!is_valid_date_format("25-01-07"));
        assert!(!is_valid_date_format("invalid"));
        assert!(!is_valid_date_format("2025/01/07"));
    }
}