//! Scheduled Task Model
//!
//! Represents background tasks that need to be executed at specific times.
//! Used for daily reset scheduling and other time-based operations.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Task types for scheduled operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ScheduledTaskType {
    #[serde(rename = "daily_reset")]
    #[sqlx(rename = "daily_reset")]
    DailyReset,

    #[serde(rename = "cleanup")]
    #[sqlx(rename = "cleanup")]
    Cleanup,

    #[serde(rename = "analytics")]
    #[sqlx(rename = "analytics")]
    Analytics,

    #[serde(rename = "backup")]
    #[sqlx(rename = "backup")]
    Backup,

    #[serde(rename = "notification")]
    #[sqlx(rename = "notification")]
    Notification,
}

impl Default for ScheduledTaskType {
    fn default() -> Self {
        ScheduledTaskType::DailyReset
    }
}

impl ScheduledTaskType {
    /// Get display name for the task type
    pub fn display_name(&self) -> &'static str {
        match self {
            ScheduledTaskType::DailyReset => "Daily Reset",
            ScheduledTaskType::Cleanup => "Cleanup",
            ScheduledTaskType::Analytics => "Analytics",
            ScheduledTaskType::Backup => "Backup",
            ScheduledTaskType::Notification => "Notification",
        }
    }

    /// Get default cron expression for this task type
    pub fn default_cron_expression(&self) -> &'static str {
        match self {
            ScheduledTaskType::DailyReset => "0 0 * * *", // Midnight daily
            ScheduledTaskType::Cleanup => "0 2 * * 0",     // 2 AM on Sundays
            ScheduledTaskType::Analytics => "0 1 * * *",   // 1 AM daily
            ScheduledTaskType::Backup => "0 3 * * 0",      // 3 AM on Sundays
            ScheduledTaskType::Notification => "* * * * *", // Every minute (for testing)
        }
    }
}

/// Scheduled task for background execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ScheduledTask {
    /// Unique identifier for the task
    pub id: String,

    /// Type of scheduled task
    #[sqlx(rename = "task_type")]
    pub task_type: ScheduledTaskType,

    /// User configuration this task belongs to (optional for system tasks)
    #[sqlx(rename = "user_configuration_id")]
    pub user_configuration_id: Option<String>,

    /// Cron expression for scheduling
    #[sqlx(rename = "cron_expression")]
    pub cron_expression: String,

    /// Timezone for scheduling (default: UTC)
    #[sqlx(rename = "timezone")]
    pub timezone: String,

    /// Next scheduled execution time (Unix timestamp UTC)
    #[sqlx(rename = "next_run_utc")]
    pub next_run_utc: i64,

    /// Last execution time (Unix timestamp UTC)
    #[sqlx(rename = "last_run_utc")]
    pub last_run_utc: Option<i64>,

    /// Whether the task is currently active
    #[sqlx(rename = "is_active")]
    pub is_active: bool,

    /// Number of successful runs
    #[sqlx(rename = "run_count")]
    pub run_count: i64,

    /// Number of failed executions
    #[sqlx(rename = "failure_count")]
    pub failure_count: i64,

    /// Additional task data (JSON string)
    #[sqlx(rename = "task_data")]
    pub task_data: Option<String>,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: i64,

    /// Last update timestamp (Unix timestamp)
    #[sqlx(rename = "updated_at")]
    pub updated_at: i64,
}

impl ScheduledTask {
    /// Create a new scheduled task
    pub fn new(
        task_type: ScheduledTaskType,
        cron_expression: String,
        timezone: String,
    ) -> Self {
        let now = Utc::now().timestamp();
        let id = format!("task_{}_{}", task_type.display_name().to_lowercase().replace(' ', '_'), Uuid::new_v4());

        Self {
            id,
            task_type,
            user_configuration_id: None,
            cron_expression,
            timezone,
            next_run_utc: now, // Will be calculated properly later
            last_run_utc: None,
            is_active: true,
            run_count: 0,
            failure_count: 0,
            task_data: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a daily reset task for a specific user
    pub fn daily_reset_task(
        user_configuration_id: String,
        cron_expression: String,
        timezone: String,
    ) -> Self {
        let mut task = Self::new(
            ScheduledTaskType::DailyReset,
            cron_expression,
            timezone,
        );
        task.user_configuration_id = Some(user_configuration_id);
        task
    }

    /// Create a system task (not tied to a specific user)
    pub fn system_task(task_type: ScheduledTaskType, timezone: String) -> Self {
        let cron_expression = task_type.default_cron_expression().to_string();
        Self::new(
            task_type,
            cron_expression,
            timezone,
        )
    }

    /// Create with custom task data
    pub fn with_task_data(mut self, task_data: String) -> Self {
        self.task_data = Some(task_data);
        self
    }

    /// Check if the task is due for execution
    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        self.next_run_utc <= now && self.is_active
    }

    /// Mark task as executed successfully
    pub fn mark_success(&mut self) {
        let now = Utc::now().timestamp();
        self.last_run_utc = Some(now);
        self.run_count += 1;
        self.touch();
    }

    /// Mark task execution as failed
    pub fn mark_failure(&mut self) {
        let now = Utc::now().timestamp();
        self.last_run_utc = Some(now);
        self.failure_count += 1;
        self.touch();
    }

    /// Activate the task
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }

    /// Deactivate the task
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }

    /// Calculate next run time based on cron expression
    pub fn calculate_next_run(&mut self, base_time: DateTime<Utc>) -> Result<(), ScheduledTaskError> {
        // Simple implementation - in production, use a proper cron library
        let next_run = self.calculate_next_run_simple(base_time)?;
        self.next_run_utc = next_run.timestamp();
        Ok(())
    }

    /// Simple next run calculation (placeholder - should use cron library)
    fn calculate_next_run_simple(&self, base_time: DateTime<Utc>) -> Result<DateTime<Utc>, ScheduledTaskError> {
        // Parse cron expression: minute hour day month day_of_week
        let parts: Vec<&str> = self.cron_expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(ScheduledTaskError::InvalidCronExpression);
        }

        let (minute, hour, day, month, day_of_week) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

        // For now, handle simple daily reset cases
        if minute == "0" && hour == "0" && day == "*" && month == "*" && day_of_week == "*" {
            // Daily at midnight
            let today_midnight = base_time.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            if today_midnight > base_time {
                return Ok(today_midnight);
            } else {
                return Ok((base_time.date_naive() + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc());
            }
        }

        // Handle hourly resets
        if minute == "0" && hour != "*" && day == "*" && month == "*" && day_of_week == "*" {
            if let Ok(hour_val) = hour.parse::<u32>() {
                if hour_val < 24 {
                    let today_at_hour = base_time.date_naive().and_hms_opt(hour_val, 0, 0).unwrap().and_utc();
                    if today_at_hour > base_time {
                        return Ok(today_at_hour);
                    } else {
                        return Ok((base_time.date_naive() + chrono::Duration::days(1))
                            .and_hms_opt(hour_val, 0, 0)
                            .unwrap()
                            .and_utc());
                    }
                }
            }
        }

        // Default to next hour for unknown patterns
        Ok(base_time + chrono::Duration::hours(1))
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp();
    }

    /// Get task type as string
    pub fn task_type_str(&self) -> &str {
        match self.task_type {
            ScheduledTaskType::DailyReset => "daily_reset",
            ScheduledTaskType::Cleanup => "cleanup",
            ScheduledTaskType::Analytics => "analytics",
            ScheduledTaskType::Backup => "backup",
            ScheduledTaskType::Notification => "notification",
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let total_runs = self.run_count + self.failure_count;
        if total_runs == 0 {
            100.0
        } else {
            (self.run_count as f64 / total_runs as f64) * 100.0
        }
    }

    /// Check if task should be disabled due to too many failures
    pub fn should_disable_due_to_failures(&self, max_failures: u32) -> bool {
        self.failure_count >= max_failures as i64 && self.success_rate() < 50.0
    }

    /// Validate the scheduled task
    pub fn validate(&self) -> Result<(), ScheduledTaskError> {
        if self.id.is_empty() {
            return Err(ScheduledTaskError::InvalidId);
        }

        if self.cron_expression.is_empty() {
            return Err(ScheduledTaskError::InvalidCronExpression);
        }

        if self.timezone.is_empty() {
            return Err(ScheduledTaskError::InvalidTimezone);
        }

        if self.next_run_utc < 0 {
            return Err(ScheduledTaskError::InvalidNextRunTime);
        }

        if self.run_count < 0 {
            return Err(ScheduledTaskError::InvalidRunCount);
        }

        if self.failure_count < 0 {
            return Err(ScheduledTaskError::InvalidFailureCount);
        }

        if self.updated_at < self.created_at {
            return Err(ScheduledTaskError::InvalidTimestamps);
        }

        // Validate timezone format
        if let Err(_) = self.timezone.parse::<chrono_tz::Tz>() {
            return Err(ScheduledTaskError::InvalidTimezone);
        }

        // Basic cron expression validation
        let cron_parts: Vec<&str> = self.cron_expression.split_whitespace().collect();
        if cron_parts.len() != 5 {
            return Err(ScheduledTaskError::InvalidCronExpression);
        }

        Ok(())
    }

    /// Get last run time as DateTime
    pub fn last_run_time(&self) -> Option<DateTime<Utc>> {
        self.last_run_utc.and_then(|ts| DateTime::from_timestamp(ts, 0))
    }

    /// Get next run time as DateTime
    pub fn next_run_time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.next_run_utc, 0).unwrap_or_else(|| Utc::now())
    }

    /// Clone with new ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// Clone with different cron expression
    pub fn with_cron_expression(mut self, cron_expression: String) -> Self {
        self.cron_expression = cron_expression;
        self
    }

    /// Clone with different timezone
    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = timezone;
        self
    }
}

impl Default for ScheduledTask {
    fn default() -> Self {
        Self::new(
            ScheduledTaskType::DailyReset,
            "0 0 * * *".to_string(),
            "UTC".to_string(),
        )
    }
}

/// Scheduled task validation errors
#[derive(Debug, thiserror::Error)]
pub enum ScheduledTaskError {
    #[error("Invalid task ID")]
    InvalidId,

    #[error("Invalid cron expression")]
    InvalidCronExpression,

    #[error("Invalid timezone")]
    InvalidTimezone,

    #[error("Invalid next run time")]
    InvalidNextRunTime,

    #[error("Invalid run count")]
    InvalidRunCount,

    #[error("Invalid failure count")]
    InvalidFailureCount,

    #[error("Invalid timestamps")]
    InvalidTimestamps,
}

/// DTO for creating scheduled tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskRequest {
    pub task_type: ScheduledTaskType,
    pub user_configuration_id: Option<String>,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub task_data: Option<String>,
    pub is_active: Option<bool>,
}

impl CreateScheduledTaskRequest {
    /// Convert to ScheduledTask model
    pub fn to_model(self) -> ScheduledTask {
        let cron_expression = self.cron_expression
            .unwrap_or_else(|| self.task_type.default_cron_expression().to_string());

        let mut task = ScheduledTask::new(self.task_type, cron_expression, self.timezone);
        task.user_configuration_id = self.user_configuration_id;

        if let Some(task_data) = self.task_data {
            task.task_data = Some(task_data);
        }

        if let Some(is_active) = self.is_active {
            task.is_active = is_active;
        }

        task
    }
}

/// DTO for updating scheduled tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskRequest {
    pub cron_expression: Option<String>,
    pub timezone: Option<String>,
    pub next_run_utc: Option<i64>,
    pub is_active: Option<bool>,
    pub task_data: Option<String>,
}

impl UpdateScheduledTaskRequest {
    /// Apply updates to existing task
    pub fn apply_to(&self, task: &mut ScheduledTask) {
        if let Some(cron_expr) = &self.cron_expression {
            task.cron_expression = cron_expr.clone();
        }

        if let Some(timezone) = &self.timezone {
            task.timezone = timezone.clone();
        }

        if let Some(next_run) = self.next_run_utc {
            task.next_run_utc = next_run;
        }

        if let Some(is_active) = self.is_active {
            task.is_active = is_active;
        }

        if let Some(task_data) = &self.task_data {
            task.task_data = Some(task_data.clone());
        }

        task.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        assert_eq!(task.task_type, ScheduledTaskType::DailyReset);
        assert_eq!(task.cron_expression, "0 8 * * *");
        assert_eq!(task.timezone, "UTC");
        assert!(task.is_active);
        assert_eq!(task.run_count, 0);
        assert_eq!(task.failure_count, 0);
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_daily_reset_task() {
        let task = ScheduledTask::daily_reset_task(
            "user-123".to_string(),
            "0 7 * * *".to_string(),
            "America/New_York".to_string(),
        );

        assert_eq!(task.task_type, ScheduledTaskType::DailyReset);
        assert_eq!(task.user_configuration_id, Some("user-123".to_string()));
        assert_eq!(task.cron_expression, "0 7 * * *");
        assert_eq!(task.timezone, "America/New_York");
    }

    #[test]
    fn test_system_task() {
        let task = ScheduledTask::system_task(
            ScheduledTaskType::Cleanup,
            "UTC".to_string(),
        );

        assert_eq!(task.task_type, ScheduledTaskType::Cleanup);
        assert_eq!(task.user_configuration_id, None);
        assert_eq!(task.cron_expression, "0 2 * * 0"); // Default cleanup schedule
    }

    #[test]
    fn test_task_execution_tracking() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        // Mark successful execution
        task.mark_success();
        assert_eq!(task.run_count, 1);
        assert_eq!(task.failure_count, 0);
        assert!(task.last_run_utc.is_some());

        // Mark failed execution
        task.mark_failure();
        assert_eq!(task.run_count, 1);
        assert_eq!(task.failure_count, 1);

        // Calculate success rate
        assert_eq!(task.success_rate(), 50.0);
    }

    #[test]
    fn test_task_activation() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        assert!(task.is_active);

        task.deactivate();
        assert!(!task.is_active);

        task.activate();
        assert!(task.is_active);
    }

    #[test]
    fn test_next_run_calculation() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 0 * * *".to_string(), // Midnight daily
            "UTC".to_string(),
        );

        // Test calculation for time before midnight
        let before_midnight = Utc.with_ymd_and_hms(2025, 1, 7, 23, 0, 0).single().unwrap();
        task.calculate_next_run(before_midnight).unwrap();

        let expected_next = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).single().unwrap();
        assert_eq!(task.next_run_time(), expected_next);

        // Test calculation for time after midnight
        let after_midnight = Utc.with_ymd_and_hms(2025, 1, 7, 1, 0, 0).single().unwrap();
        task.calculate_next_run(after_midnight).unwrap();

        let expected_next = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).single().unwrap();
        assert_eq!(task.next_run_time(), expected_next);
    }

    #[test]
    fn test_hourly_reset_calculation() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 9 * * *".to_string(), // 9 AM daily
            "UTC".to_string(),
        );

        // Test before 9 AM
        let before_nine = Utc.with_ymd_and_hms(2025, 1, 7, 8, 0, 0).single().unwrap();
        task.calculate_next_run(before_nine).unwrap();

        let expected_next = Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap();
        assert_eq!(task.next_run_time(), expected_next);

        // Test after 9 AM
        let after_nine = Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap();
        task.calculate_next_run(after_nine).unwrap();

        let expected_next = Utc.with_ymd_and_hms(2025, 1, 8, 9, 0, 0).single().unwrap();
        assert_eq!(task.next_run_time(), expected_next);
    }

    #[test]
    fn test_task_due_check() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        // Set next run to past time
        task.next_run_utc = Utc::now().timestamp() - 3600; // 1 hour ago
        assert!(task.is_due());

        // Set next run to future time
        task.next_run_utc = Utc::now().timestamp() + 3600; // 1 hour in future
        assert!(!task.is_due());

        // Deactivate task
        task.is_active = false;
        assert!(!task.is_due()); // Inactive tasks are never due
    }

    #[test]
    fn test_failure_handling() {
        let mut task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        // Simulate multiple failures
        for _ in 0..5 {
            task.mark_failure();
        }

        assert_eq!(task.failure_count, 5);
        assert_eq!(task.run_count, 0);
        assert_eq!(task.success_rate(), 0.0);

        // Check if task should be disabled (with threshold of 3 failures and <50% success rate)
        assert!(task.should_disable_due_to_failures(3));
    }

    #[test]
    fn test_create_request_to_model() {
        let request = CreateScheduledTaskRequest {
            task_type: ScheduledTaskType::DailyReset,
            user_configuration_id: Some("user-123".to_string()),
            cron_expression: Some("0 7 * * *".to_string()),
            timezone: "America/New_York".to_string(),
            task_data: Some(r#"{"test": "data"}"#.to_string()),
            is_active: Some(false),
        };

        let task = request.to_model();

        assert_eq!(task.task_type, ScheduledTaskType::DailyReset);
        assert_eq!(task.user_configuration_id, Some("user-123".to_string()));
        assert_eq!(task.cron_expression, "0 7 * * *");
        assert_eq!(task.timezone, "America/New_York");
        assert_eq!(task.task_data, Some(r#"{"test": "data"}"#.to_string()));
        assert!(!task.is_active);
    }

    #[test]
    fn test_task_validation() {
        let task = ScheduledTask::new(
            ScheduledTaskType::DailyReset,
            "0 8 * * *".to_string(),
            "UTC".to_string(),
        );

        assert!(task.validate().is_ok());

        // Test invalid cron expression
        let mut invalid_task = task.clone();
        invalid_task.cron_expression = "invalid".to_string();
        assert!(invalid_task.validate().is_err());

        // Test invalid timezone
        let mut invalid_task = task.clone();
        invalid_task.timezone = "Invalid/Timezone".to_string();
        assert!(invalid_task.validate().is_err());
    }

    #[test]
    fn test_task_type_display() {
        assert_eq!(ScheduledTaskType::DailyReset.display_name(), "Daily Reset");
        assert_eq!(ScheduledTaskType::Cleanup.display_name(), "Cleanup");
        assert_eq!(ScheduledTaskType::Analytics.display_name(), "Analytics");
        assert_eq!(ScheduledTaskType::Backup.display_name(), "Backup");
        assert_eq!(ScheduledTaskType::Notification.display_name(), "Notification");
    }

    #[test]
    fn test_default_cron_expressions() {
        assert_eq!(ScheduledTaskType::DailyReset.default_cron_expression(), "0 0 * * *");
        assert_eq!(ScheduledTaskType::Cleanup.default_cron_expression(), "0 2 * * 0");
        assert_eq!(ScheduledTaskType::Analytics.default_cron_expression(), "0 1 * * *");
        assert_eq!(ScheduledTaskType::Backup.default_cron_expression(), "0 3 * * 0");
        assert_eq!(ScheduledTaskType::Notification.default_cron_expression(), "* * * * *");
    }
}