//! WebSocket Message Schemas for Daily Session Reset
//!
//! Defines real-time synchronization messages for daily reset functionality including:
//! - Configuration changes (timezone, reset time)
//! - Session count updates and resets
//! - Manual overrides and analytics updates

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// WebSocket message types for daily session reset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DailyResetWebSocketMessage {
    /// Configuration change messages
    ConfigurationChanged(ConfigurationChangedMessage),

    /// Session count updates
    SessionCountUpdated(SessionCountUpdatedMessage),

    /// Session reset notifications
    SessionReset(SessionResetMessage),

    /// Manual override notifications
    ManualSessionOverride(ManualSessionOverrideMessage),

    /// Timezone change notifications
    TimezoneChanged(TimezoneChangedMessage),

    /// Analytics updates
    DailyStatsUpdated(DailyStatsUpdatedMessage),

    /// Error messages
    Error(ErrorMessage),
}

/// Configuration change notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationChangedMessage {
    /// User configuration ID
    pub user_id: String,

    /// Device ID that made the change
    pub device_id: String,

    /// Previous configuration values (for diff)
    pub previous: ConfigurationData,

    /// New configuration values
    pub new: ConfigurationData,

    /// Change timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Source of the change
    pub source: ConfigurationSource,
}

/// Session count update notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCountUpdatedMessage {
    /// User configuration ID
    pub user_id: String,

    /// Device ID that triggered the update
    pub device_id: String,

    /// Previous session count
    pub previous_count: u32,

    /// New session count
    pub new_count: u32,

    /// Whether this was an automatic increment or manual change
    pub change_type: SessionCountChangeType,

    /// Update timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Session type (work/break)
    pub session_type: String,
}

/// Session reset notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionResetMessage {
    /// User configuration ID
    pub user_id: String,

    /// Device ID that triggered or observed the reset
    pub device_id: Option<String>,

    /// Session count before reset
    pub previous_count: u32,

    /// Session count after reset (should be 0)
    pub new_count: u32,

    /// Reset type (scheduled, manual, timezone_change, etc.)
    pub reset_type: ResetType,

    /// Reset timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// User's local time when reset occurred
    pub local_reset_time: String,

    /// User's timezone
    pub timezone: String,

    /// Whether the reset was successful
    pub success: bool,

    /// Error message if reset failed
    pub error: Option<String>,
}

/// Manual session override notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualSessionOverrideMessage {
    /// User configuration ID
    pub user_id: String,

    /// Device ID that set the override
    pub device_id: String,

    /// Previous session count
    pub previous_count: u32,

    /// New manual session count
    pub override_count: u32,

    /// Override timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Reason for override (if provided)
    pub reason: Option<String>,
}

/// Timezone change notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimezoneChangedMessage {
    /// User configuration ID
    pub user_id: String,

    /// Device ID that made the change
    pub device_id: String,

    /// Previous timezone
    pub previous_timezone: String,

    /// New timezone
    pub new_timezone: String,

    /// Change timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Whether the timezone change affected scheduled resets
    pub affected_resets: bool,
}

/// Daily statistics update notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyStatsUpdatedMessage {
    /// User configuration ID
    pub user_id: String,

    /// Date for the statistics (YYYY-MM-DD in UTC)
    pub date: String,

    /// Timezone used for the statistics
    pub timezone: String,

    /// Updated statistics
    pub stats: DailyStats,

    /// Update timestamp (UTC)
    pub timestamp: DateTime<Utc>,
}

/// Error message for daily reset operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// User configuration ID (if applicable)
    pub user_id: Option<String>,

    /// Device ID (if applicable)
    pub device_id: Option<String>,

    /// Error code
    pub code: ErrorCode,

    /// Human-readable error message
    pub message: String,

    /// Error timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Additional context about the error
    pub context: Option<String>,

    /// Whether this is a recoverable error
    pub recoverable: bool,
}

// Supporting data structures

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationData {
    /// Timezone identifier
    pub timezone: String,

    /// Daily reset configuration
    pub daily_reset: DailyResetConfig,

    /// Whether daily reset is enabled
    pub daily_reset_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyResetConfig {
    /// Reset time type
    pub time_type: String,

    /// Hour value (if hourly reset)
    pub hour: Option<u8>,

    /// Custom time string (HH:MM format)
    pub custom_time: Option<String>,

    /// Cron expression for scheduling
    pub cron_expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyStats {
    /// Number of work sessions completed
    pub work_sessions_completed: u32,

    /// Total time spent in work sessions (seconds)
    pub total_work_seconds: u64,

    /// Total time spent in breaks (seconds)
    pub total_break_seconds: u64,

    /// Number of manual overrides
    pub manual_overrides: u32,

    /// Final session count for the day
    pub final_session_count: u32,
}

// Enums

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationSource {
    /// Change made by user in UI
    UserInterface,

    /// Change made via API call
    Api,

    /// Change made by system/automated process
    System,

    /// Change made during migration/upgrade
    Migration,

    /// Change from configuration file/environment
    ConfigFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionCountChangeType {
    /// Automatic increment from completed session
    Automatic,

    /// Manual increase/decrease
    Manual,

    /// Reset to zero
    Reset,

    /// Override to specific value
    Override,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetType {
    /// Scheduled daily reset
    ScheduledDaily,

    /// Manual reset by user
    Manual,

    /// Reset due to timezone change
    TimezoneChange,

    /// Reset due to configuration change
    ConfigurationChange,

    /// System-initiated reset
    System,

    /// Reset on application startup
    Startup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Invalid configuration values
    InvalidConfiguration,

    /// Timezone parsing error
    InvalidTimezone,

    /// Database error
    DatabaseError,

    /// Scheduling error
    SchedulingError,

    /// Network error
    NetworkError,

    /// Permission denied
    PermissionDenied,

    /// Rate limit exceeded
    RateLimitExceeded,

    /// Internal server error
    InternalError,

    /// Validation error
    ValidationError,

    /// Session reset failed
    SessionResetFailed,

    /// Configuration not found
    ConfigurationNotFound,

    /// Device not authorized
    DeviceUnauthorized,
}

// Message validation

impl DailyResetWebSocketMessage {
    /// Validate the message structure and content
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            DailyResetWebSocketMessage::ConfigurationChanged(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::SessionCountUpdated(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::SessionReset(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::ManualSessionOverride(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::TimezoneChanged(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::DailyStatsUpdated(msg) => {
                msg.validate()?;
            }
            DailyResetWebSocketMessage::Error(msg) => {
                msg.validate()?;
            }
        }
        Ok(())
    }

    /// Get the user ID for this message (if applicable)
    pub fn user_id(&self) -> Option<&str> {
        match self {
            DailyResetWebSocketMessage::ConfigurationChanged(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::SessionCountUpdated(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::SessionReset(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::ManualSessionOverride(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::TimezoneChanged(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::DailyStatsUpdated(msg) => Some(&msg.user_id),
            DailyResetWebSocketMessage::Error(msg) => msg.user_id.as_deref(),
        }
    }

    /// Check if this message requires immediate processing
    pub fn is_high_priority(&self) -> bool {
        match self {
            DailyResetWebSocketMessage::SessionReset(_) => true,
            DailyResetWebSocketMessage::ManualSessionOverride(_) => true,
            DailyResetWebSocketMessage::Error(_) => true,
            _ => false,
        }
    }
}

// Validation implementations

impl ConfigurationChangedMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.device_id.is_empty() {
            return Err(ValidationError::MissingField("device_id".to_string()));
        }
        self.new.validate()?;
        Ok(())
    }
}

impl ConfigurationData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.timezone.is_empty() {
            return Err(ValidationError::MissingField("timezone".to_string()));
        }
        self.daily_reset.validate()?;
        Ok(())
    }
}

impl DailyResetConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.time_type.is_empty() {
            return Err(ValidationError::MissingField("time_type".to_string()));
        }
        if self.cron_expression.is_empty() {
            return Err(ValidationError::MissingField("cron_expression".to_string()));
        }
        Ok(())
    }
}

// Validation for other message types
impl SessionCountUpdatedMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.device_id.is_empty() {
            return Err(ValidationError::MissingField("device_id".to_string()));
        }
        if self.session_type.is_empty() {
            return Err(ValidationError::MissingField("session_type".to_string()));
        }
        Ok(())
    }
}

impl SessionResetMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.timezone.is_empty() {
            return Err(ValidationError::MissingField("timezone".to_string()));
        }
        if self.local_reset_time.is_empty() {
            return Err(ValidationError::MissingField("local_reset_time".to_string()));
        }
        Ok(())
    }
}

impl ManualSessionOverrideMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.device_id.is_empty() {
            return Err(ValidationError::MissingField("device_id".to_string()));
        }
        Ok(())
    }
}

impl TimezoneChangedMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.device_id.is_empty() {
            return Err(ValidationError::MissingField("device_id".to_string()));
        }
        if self.previous_timezone.is_empty() || self.new_timezone.is_empty() {
            return Err(ValidationError::MissingField("timezone".to_string()));
        }
        Ok(())
    }
}

impl DailyStatsUpdatedMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.user_id.is_empty() {
            return Err(ValidationError::MissingField("user_id".to_string()));
        }
        if self.date.is_empty() {
            return Err(ValidationError::MissingField("date".to_string()));
        }
        if self.timezone.is_empty() {
            return Err(ValidationError::MissingField("timezone".to_string()));
        }
        Ok(())
    }
}

impl ErrorMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.message.is_empty() {
            return Err(ValidationError::MissingField("message".to_string()));
        }
        Ok(())
    }
}

/// Validation error for WebSocket messages
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {0}")]
    InvalidValue(String),

    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = DailyResetWebSocketMessage::SessionReset(SessionResetMessage {
            user_id: "test-user".to_string(),
            device_id: Some("test-device".to_string()),
            previous_count: 5,
            new_count: 0,
            reset_type: ResetType::ScheduledDaily,
            timestamp: Utc::now(),
            local_reset_time: "2025-01-07 00:00:00".to_string(),
            timezone: "UTC".to_string(),
            success: true,
            error: None,
        });

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: DailyResetWebSocketMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_validation() {
        let valid_msg = SessionResetMessage {
            user_id: "test-user".to_string(),
            device_id: Some("test-device".to_string()),
            previous_count: 5,
            new_count: 0,
            reset_type: ResetType::ScheduledDaily,
            timestamp: Utc::now(),
            local_reset_time: "2025-01-07 00:00:00".to_string(),
            timezone: "UTC".to_string(),
            success: true,
            error: None,
        };

        assert!(valid_msg.validate().is_ok());

        let invalid_msg = SessionResetMessage {
            user_id: "".to_string(), // Invalid: empty user_id
            device_id: Some("test-device".to_string()),
            previous_count: 5,
            new_count: 0,
            reset_type: ResetType::ScheduledDaily,
            timestamp: Utc::now(),
            local_reset_time: "2025-01-07 00:00:00".to_string(),
            timezone: "UTC".to_string(),
            success: true,
            error: None,
        };

        assert!(invalid_msg.validate().is_err());
    }

    #[test]
    fn test_message_priority() {
        let reset_msg = DailyResetWebSocketMessage::SessionReset(SessionResetMessage {
            user_id: "test".to_string(),
            device_id: Some("test".to_string()),
            previous_count: 1,
            new_count: 0,
            reset_type: ResetType::Manual,
            timestamp: Utc::now(),
            local_reset_time: "00:00:00".to_string(),
            timezone: "UTC".to_string(),
            success: true,
            error: None,
        });

        let config_msg = DailyResetWebSocketMessage::ConfigurationChanged(ConfigurationChangedMessage {
            user_id: "test".to_string(),
            device_id: "test".to_string(),
            previous: ConfigurationData {
                timezone: "UTC".to_string(),
                daily_reset: DailyResetConfig {
                    time_type: "midnight".to_string(),
                    hour: None,
                    custom_time: None,
                    cron_expression: "0 0 * * *".to_string(),
                },
                daily_reset_enabled: false,
            },
            new: ConfigurationData {
                timezone: "America/New_York".to_string(),
                daily_reset: DailyResetConfig {
                    time_type: "hour".to_string(),
                    hour: Some(7),
                    custom_time: None,
                    cron_expression: "0 7 * * *".to_string(),
                },
                daily_reset_enabled: true,
            },
            timestamp: Utc::now(),
            source: ConfigurationSource::UserInterface,
        });

        assert!(reset_msg.is_high_priority());
        assert!(!config_msg.is_high_priority());
    }
}