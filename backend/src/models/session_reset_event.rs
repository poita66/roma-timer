//! Session Reset Event Model
//!
//! Represents events that occur when session counts are reset.
//! Provides audit trail and analytics for daily reset operations.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Types of session reset events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SessionResetEventType {
    #[serde(rename = "scheduled_daily")]
    #[sqlx(rename = "scheduled_daily")]
    ScheduledDaily,

    #[serde(rename = "manual_reset")]
    #[sqlx(rename = "manual_reset")]
    ManualReset,

    #[serde(rename = "timezone_change")]
    #[sqlx(rename = "timezone_change")]
    TimezoneChange,

    #[serde(rename = "configuration_change")]
    #[sqlx(rename = "configuration_change")]
    ConfigurationChange,

    #[serde(rename = "system")]
    #[sqlx(rename = "system")]
    System,

    #[serde(rename = "startup")]
    #[sqlx(rename = "startup")]
    Startup,
}

impl SessionResetEventType {
    /// Get display name for the reset type
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionResetEventType::ScheduledDaily => "Scheduled Daily Reset",
            SessionResetEventType::ManualReset => "Manual Reset",
            SessionResetEventType::TimezoneChange => "Timezone Change",
            SessionResetEventType::ConfigurationChange => "Configuration Change",
            SessionResetEventType::System => "System Reset",
            SessionResetEventType::Startup => "Startup Reset",
        }
    }

    /// Get description for the reset type
    pub fn description(&self) -> &'static str {
        match self {
            SessionResetEventType::ScheduledDaily => {
                "Automatic daily session reset at configured time"
            }
            SessionResetEventType::ManualReset => {
                "Manual session reset triggered by user"
            }
            SessionResetEventType::TimezoneChange => {
                "Session reset due to timezone configuration change"
            }
            SessionResetEventType::ConfigurationChange => {
                "Session reset due to daily reset configuration change"
            }
            SessionResetEventType::System => {
                "Session reset initiated by system"
            }
            SessionResetEventType::Startup => {
                "Session reset performed during application startup"
            }
        }
    }

    /// Check if this is an automatic reset type
    pub fn is_automatic(&self) -> bool {
        matches!(
            self,
            SessionResetEventType::ScheduledDaily
                | SessionResetEventType::TimezoneChange
                | SessionResetEventType::ConfigurationChange
                | SessionResetEventType::System
                | SessionResetEventType::Startup
        )
    }

    /// Check if this is a user-initiated reset type
    pub fn is_user_initiated(&self) -> bool {
        matches!(self, SessionResetEventType::ManualReset)
    }
}

/// Sources that can trigger session reset events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SessionResetTriggerSource {
    #[serde(rename = "background_service")]
    #[sqlx(rename = "background_service")]
    BackgroundService,

    #[serde(rename = "user_action")]
    #[sqlx(rename = "user_action")]
    UserAction,

    #[serde(rename = "api_call")]
    #[sqlx(rename = "api_call")]
    ApiCall,

    #[serde(rename = "websocket_message")]
    #[sqlx(rename = "websocket_message")]
    WebSocketMessage,

    #[serde(rename = "migration")]
    #[sqlx(rename = "migration")]
    Migration,

    #[serde(rename = "configuration_update")]
    #[sqlx(rename = "configuration_update")]
    ConfigurationUpdate,
}

impl SessionResetTriggerSource {
    /// Get display name for the trigger source
    pub fn display_name(&self) -> &'static str {
        match self {
            SessionResetTriggerSource::BackgroundService => "Background Service",
            SessionResetTriggerSource::UserAction => "User Action",
            SessionResetTriggerSource::ApiCall => "API Call",
            SessionResetTriggerSource::WebSocketMessage => "WebSocket Message",
            SessionResetTriggerSource::Migration => "Migration",
            SessionResetTriggerSource::ConfigurationUpdate => "Configuration Update",
        }
    }
}

/// Session reset event for audit trail and analytics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct SessionResetEvent {
    /// Unique identifier for the event
    pub id: String,

    /// User configuration this event belongs to
    #[sqlx(rename = "user_configuration_id")]
    pub user_configuration_id: String,

    /// Type of reset event
    #[sqlx(rename = "reset_type")]
    pub reset_type: SessionResetEventType,

    /// Session count before reset
    #[sqlx(rename = "previous_count")]
    pub previous_count: i64,

    /// Session count after reset (should be 0 for most reset types)
    #[sqlx(rename = "new_count")]
    pub new_count: i64,

    /// Reset timestamp (Unix timestamp UTC)
    #[sqlx(rename = "reset_timestamp_utc")]
    pub reset_timestamp_utc: i64,

    /// User's timezone at time of reset
    #[sqlx(rename = "user_timezone")]
    pub user_timezone: String,

    /// Local reset time formatted as YYYY-MM-DD HH:MM:SS in user's timezone
    #[sqlx(rename = "local_reset_time")]
    pub local_reset_time: String,

    /// Device ID that triggered the reset (optional)
    #[sqlx(rename = "device_id")]
    pub device_id: Option<String>,

    /// Source that triggered the reset
    #[sqlx(rename = "trigger_source")]
    pub trigger_source: SessionResetTriggerSource,

    /// Additional context data (JSON string)
    #[sqlx(rename = "context")]
    pub context: Option<String>,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: i64,
}

impl SessionResetEvent {
    /// Create a new session reset event
    pub fn new(
        user_configuration_id: String,
        reset_type: SessionResetEventType,
        previous_count: u32,
        new_count: u32,
        reset_timestamp: DateTime<Utc>,
        user_timezone: String,
        trigger_source: SessionResetTriggerSource,
    ) -> Self {
        let now = reset_timestamp.timestamp();
        let id = format!("reset_{}_{}", user_configuration_id, Uuid::new_v4());

        // Format local time in user's timezone
        let local_reset_time = if let Ok(tz) = user_timezone.parse::<chrono_tz::Tz>() {
            reset_timestamp.with_timezone(&tz).format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            // Fallback if timezone parsing fails
            reset_timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        };

        Self {
            id,
            user_configuration_id,
            reset_type,
            previous_count: previous_count as i64,
            new_count: new_count as i64,
            reset_timestamp_utc: now,
            user_timezone,
            local_reset_time,
            device_id: None,
            trigger_source,
            context: None,
            created_at: now,
        }
    }

    /// Create with device ID
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Create with additional context data
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Create a scheduled daily reset event
    pub fn scheduled_daily_reset(
        user_configuration_id: String,
        previous_count: u32,
        reset_timestamp: DateTime<Utc>,
        user_timezone: String,
    ) -> Self {
        Self::new(
            user_configuration_id,
            SessionResetEventType::ScheduledDaily,
            previous_count,
            0, // Reset to 0
            reset_timestamp,
            user_timezone,
            SessionResetTriggerSource::BackgroundService,
        )
    }

    /// Create a manual reset event
    pub fn manual_reset(
        user_configuration_id: String,
        previous_count: u32,
        new_count: u32,
        reset_timestamp: DateTime<Utc>,
        user_timezone: String,
        device_id: String,
    ) -> Self {
        Self::new(
            user_configuration_id,
            SessionResetEventType::ManualReset,
            previous_count,
            new_count,
            reset_timestamp,
            user_timezone,
            SessionResetTriggerSource::UserAction,
        ).with_device_id(device_id)
    }

    /// Create a timezone change reset event
    pub fn timezone_change_reset(
        user_configuration_id: String,
        previous_count: u32,
        reset_timestamp: DateTime<Utc>,
        old_timezone: &str,
        new_timezone: &str,
    ) -> Self {
        let context = serde_json::json!({
            "old_timezone": old_timezone,
            "new_timezone": new_timezone
        }).to_string();

        Self::new(
            user_configuration_id,
            SessionResetEventType::TimezoneChange,
            previous_count,
            0, // Reset to 0
            reset_timestamp,
            new_timezone.to_string(),
            SessionResetTriggerSource::ConfigurationUpdate,
        ).with_context(context)
    }

    /// Create a configuration change reset event
    pub fn configuration_change_reset(
        user_configuration_id: String,
        previous_count: u32,
        reset_timestamp: DateTime<Utc>,
        user_timezone: String,
        configuration_details: &serde_json::Value,
    ) -> Self {
        Self::new(
            user_configuration_id,
            SessionResetEventType::ConfigurationChange,
            previous_count,
            0, // Reset to 0
            reset_timestamp,
            user_timezone,
            SessionResetTriggerSource::ConfigurationUpdate,
        ).with_context(configuration_details.to_string())
    }

    /// Create a startup reset event
    pub fn startup_reset(
        user_configuration_id: String,
        previous_count: u32,
        reset_timestamp: DateTime<Utc>,
        user_timezone: String,
    ) -> Self {
        Self::new(
            user_configuration_id,
            SessionResetEventType::Startup,
            previous_count,
            0, // Reset to 0
            reset_timestamp,
            user_timezone,
            SessionResetTriggerSource::System,
        )
    }

    /// Get reset timestamp as DateTime
    pub fn reset_timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.reset_timestamp_utc, 0).unwrap_or_else(|| Utc::now())
    }

    /// Get previous count as u32
    pub fn previous_count(&self) -> u32 {
        self.previous_count as u32
    }

    /// Get new count as u32
    pub fn new_count(&self) -> u32 {
        self.new_count as u32
    }

    /// Check if this was a reset to zero
    pub fn is_reset_to_zero(&self) -> bool {
        self.new_count == 0
    }

    /// Check if the session count changed
    pub fn count_changed(&self) -> bool {
        self.previous_count != self.new_count
    }

    /// Get the session count difference
    pub fn count_difference(&self) -> i32 {
        (self.new_count - self.previous_count) as i32
    }

    /// Get context as JSON value if available
    pub fn context_as_json(&self) -> Option<serde_json::Value> {
        self.context.as_ref().and_then(|ctx| serde_json::from_str(ctx).ok())
    }

    /// Validate the session reset event
    pub fn validate(&self) -> Result<(), SessionResetEventError> {
        if self.id.is_empty() {
            return Err(SessionResetEventError::InvalidId);
        }

        if self.user_configuration_id.is_empty() {
            return Err(SessionResetEventError::InvalidUserId);
        }

        if self.user_timezone.is_empty() {
            return Err(SessionResetEventError::InvalidTimezone);
        }

        if self.local_reset_time.is_empty() {
            return Err(SessionResetEventError::InvalidLocalTime);
        }

        if self.previous_count < 0 {
            return Err(SessionResetEventError::InvalidPreviousCount);
        }

        if self.new_count < 0 {
            return Err(SessionResetEventError::InvalidNewCount);
        }

        if self.reset_timestamp_utc < 0 {
            return Err(SessionResetEventError::InvalidResetTimestamp);
        }

        // Validate reset timestamp is reasonable (not too far in future)
        let now = Utc::now().timestamp();
        if self.reset_timestamp_utc > now + 86400 { // More than 1 day in future
            return Err(SessionResetEventError::FutureResetTimestamp);
        }

        Ok(())
    }

    /// Check if this event is recent (within last 24 hours)
    pub fn is_recent(&self) -> bool {
        let now = Utc::now().timestamp();
        let one_day_ago = now - 86400;
        self.reset_timestamp_utc >= one_day_ago
    }

    /// Get event age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = Utc::now().timestamp();
        (now - self.reset_timestamp_utc).max(0) as u64
    }

    /// Get human-readable age description
    pub fn age_description(&self) -> String {
        let age_seconds = self.age_seconds();

        if age_seconds < 60 {
            format!("{} seconds ago", age_seconds)
        } else if age_seconds < 3600 {
            let minutes = age_seconds / 60;
            format!("{} minutes ago", minutes)
        } else if age_seconds < 86400 {
            let hours = age_seconds / 3600;
            format!("{} hours ago", hours)
        } else {
            let days = age_seconds / 86400;
            format!("{} days ago", days)
        }
    }
}

/// Session reset event validation errors
#[derive(Debug, thiserror::Error)]
pub enum SessionResetEventError {
    #[error("Invalid event ID")]
    InvalidId,

    #[error("Invalid user configuration ID")]
    InvalidUserId,

    #[error("Invalid timezone")]
    InvalidTimezone,

    #[error("Invalid local reset time")]
    InvalidLocalTime,

    #[error("Invalid previous session count")]
    InvalidPreviousCount,

    #[error("Invalid new session count")]
    InvalidNewCount,

    #[error("Invalid reset timestamp")]
    InvalidResetTimestamp,

    #[error("Reset timestamp is in the future")]
    FutureResetTimestamp,
}

/// DTO for creating session reset events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResetEventRequest {
    pub user_configuration_id: String,
    pub reset_type: SessionResetEventType,
    pub previous_count: u32,
    pub new_count: u32,
    pub reset_timestamp: Option<DateTime<Utc>>,
    pub user_timezone: String,
    pub device_id: Option<String>,
    pub trigger_source: SessionResetTriggerSource,
    pub context: Option<String>,
}

impl CreateSessionResetEventRequest {
    /// Convert to SessionResetEvent model
    pub fn to_model(self) -> SessionResetEvent {
        let reset_timestamp = self.reset_timestamp.unwrap_or_else(Utc::now);

        let mut event = SessionResetEvent::new(
            self.user_configuration_id,
            self.reset_type,
            self.previous_count,
            self.new_count,
            reset_timestamp,
            self.user_timezone,
            self.trigger_source,
        );

        if let Some(device_id) = self.device_id {
            event.device_id = Some(device_id);
        }

        if let Some(context) = self.context {
            event.context = Some(context);
        }

        event
    }
}

/// Query filters for session reset events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResetEventQuery {
    pub user_configuration_id: Option<String>,
    pub reset_type: Option<SessionResetEventType>,
    pub trigger_source: Option<SessionResetTriggerSource>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub device_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SessionResetEventQuery {
    /// Create a new query with all filters optional
    pub fn new() -> Self {
        Self {
            user_configuration_id: None,
            reset_type: None,
            trigger_source: None,
            start_date: None,
            end_date: None,
            device_id: None,
            limit: None,
            offset: None,
        }
    }

    /// Filter by user configuration ID
    pub fn for_user(mut self, user_id: String) -> Self {
        self.user_configuration_id = Some(user_id);
        self
    }

    /// Filter by reset type
    pub fn with_reset_type(mut self, reset_type: SessionResetEventType) -> Self {
        self.reset_type = Some(reset_type);
        self
    }

    /// Filter by date range
    pub fn between_dates(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    /// Set limit for number of results
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset for pagination
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_reset_event_creation() {
        let event = SessionResetEvent::new(
            "user-123".to_string(),
            SessionResetEventType::ScheduledDaily,
            5,
            0,
            Utc::now(),
            "UTC".to_string(),
            SessionResetTriggerSource::BackgroundService,
        );

        assert_eq!(event.user_configuration_id, "user-123");
        assert_eq!(event.reset_type, SessionResetEventType::ScheduledDaily);
        assert_eq!(event.previous_count, 5);
        assert_eq!(event.new_count, 0);
        assert_eq!(event.trigger_source, SessionResetTriggerSource::BackgroundService);
        assert!(!event.id.is_empty());
    }

    #[test]
    fn test_scheduled_daily_reset() {
        let event = SessionResetEvent::scheduled_daily_reset(
            "user-123".to_string(),
            8,
            Utc::now(),
            "UTC".to_string(),
        );

        assert_eq!(event.reset_type, SessionResetEventType::ScheduledDaily);
        assert_eq!(event.previous_count, 8);
        assert_eq!(event.new_count, 0);
        assert_eq!(event.trigger_source, SessionResetTriggerSource::BackgroundService);
    }

    #[test]
    fn test_manual_reset() {
        let event = SessionResetEvent::manual_reset(
            "user-123".to_string(),
            5,
            10,
            Utc::now(),
            "UTC".to_string(),
            "device-456".to_string(),
        );

        assert_eq!(event.reset_type, SessionResetEventType::ManualReset);
        assert_eq!(event.previous_count, 5);
        assert_eq!(event.new_count, 10);
        assert_eq!(event.trigger_source, SessionResetTriggerSource::UserAction);
        assert_eq!(event.device_id, Some("device-456".to_string()));
    }

    #[test]
    fn test_timezone_change_reset() {
        let event = SessionResetEvent::timezone_change_reset(
            "user-123".to_string(),
            5,
            Utc::now(),
            "America/New_York",
            "Europe/London",
        );

        assert_eq!(event.reset_type, SessionResetEventType::TimezoneChange);
        assert_eq!(event.user_timezone, "Europe/London");
        assert!(event.context.is_some());
        assert_eq!(event.trigger_source, SessionResetTriggerSource::ConfigurationUpdate);

        let context = event.context_as_json().unwrap();
        assert_eq!(context["old_timezone"], "America/New_York");
        assert_eq!(context["new_timezone"], "Europe/London");
    }

    #[test]
    fn test_event_properties() {
        let event = SessionResetEvent::new(
            "user-123".to_string(),
            SessionResetEventType::ManualReset,
            5,
            10,
            Utc::now(),
            "UTC".to_string(),
            SessionResetTriggerSource::UserAction,
        );

        assert!(event.count_changed());
        assert_eq!(event.count_difference(), 5);
        assert!(!event.is_reset_to_zero());
        assert!(event.is_user_initiated());
        assert!(!event.is_automatic());
    }

    #[test]
    fn test_reset_to_zero_event() {
        let event = SessionResetEvent::scheduled_daily_reset(
            "user-123".to_string(),
            5,
            Utc::now(),
            "UTC".to_string(),
        );

        assert!(event.is_reset_to_zero());
        assert!(event.count_changed());
        assert_eq!(event.count_difference(), -5);
        assert!(!event.is_user_initiated());
        assert!(event.is_automatic());
    }

    #[test]
    fn test_reset_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2025, 1, 7, 12, 30, 0).single().unwrap();
        let event = SessionResetEvent::scheduled_daily_reset(
            "user-123".to_string(),
            5,
            timestamp,
            "UTC".to_string(),
        );

        assert_eq!(event.reset_timestamp(), timestamp);
        assert_eq!(event.reset_timestamp_utc, timestamp.timestamp());
    }

    #[test]
    fn test_local_time_formatting() {
        let timestamp = Utc.with_ymd_and_hms(2025, 1, 7, 12, 0, 0).single().unwrap();
        let event = SessionResetEvent::scheduled_daily_reset(
            "user-123".to_string(),
            5,
            timestamp,
            "America/New_York".to_string(),
        );

        // In January, New York is UTC-5, so 12:00 UTC = 07:00 EST
        assert!(event.local_reset_time.contains("07:00:00"));
    }

    #[test]
    fn test_event_validation() {
        let event = SessionResetEvent::new(
            "user-123".to_string(),
            SessionResetEventType::ScheduledDaily,
            5,
            0,
            Utc::now(),
            "UTC".to_string(),
            SessionResetTriggerSource::BackgroundService,
        );

        assert!(event.validate().is_ok());

        // Test invalid timezone
        let mut invalid_event = event.clone();
        invalid_event.user_timezone = "Invalid/Timezone".to_string();
        assert!(invalid_event.validate().is_err());

        // Test invalid counts
        let mut invalid_event = event.clone();
        invalid_event.previous_count = -1;
        assert!(invalid_event.validate().is_err());

        invalid_event = event.clone();
        invalid_event.new_count = -1;
        assert!(invalid_event.validate().is_err());
    }

    #[test]
    fn test_event_age() {
        let past_timestamp = Utc::now() - chrono::Duration::minutes(30);
        let event = SessionResetEvent::scheduled_daily_reset(
            "user-123".to_string(),
            5,
            past_timestamp,
            "UTC".to_string(),
        );

        assert!(event.is_recent());
        assert_eq!(event.age_description(), "30 minutes ago");
        assert_eq!(event.age_seconds(), 1800); // 30 minutes = 1800 seconds
    }

    #[test]
    fn test_create_request_to_model() {
        let request = CreateSessionResetEventRequest {
            user_configuration_id: "user-123".to_string(),
            reset_type: SessionResetEventType::ManualReset,
            previous_count: 8,
            new_count: 0,
            reset_timestamp: None,
            user_timezone: "UTC".to_string(),
            device_id: Some("device-456".to_string()),
            trigger_source: SessionResetTriggerSource::UserAction,
            context: Some(r#"{"reason": "user_request"}"#.to_string()),
        };

        let event = request.to_model();

        assert_eq!(event.user_configuration_id, "user-123");
        assert_eq!(event.reset_type, SessionResetEventType::ManualReset);
        assert_eq!(event.previous_count, 8);
        assert_eq!(event.new_count, 0);
        assert_eq!(event.device_id, Some("device-456".to_string()));
        assert_eq!(event.trigger_source, SessionResetTriggerSource::UserAction);
        assert_eq!(event.context, Some(r#"{"reason": "user_request"}"#.to_string()));
    }

    #[test]
    fn test_event_query() {
        let query = SessionResetEventQuery::new()
            .for_user("user-123".to_string())
            .with_reset_type(SessionResetEventType::ManualReset)
            .limit(50)
            .offset(10);

        assert_eq!(query.user_configuration_id, Some("user-123".to_string()));
        assert_eq!(query.reset_type, Some(SessionResetEventType::ManualReset));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(10));
    }

    #[test]
    fn test_reset_type_display() {
        assert_eq!(SessionResetEventType::ScheduledDaily.display_name(), "Scheduled Daily Reset");
        assert_eq!(SessionResetEventType::ManualReset.display_name(), "Manual Reset");
        assert_eq!(SessionResetEventType::TimezoneChange.display_name(), "Timezone Change");
        assert_eq!(SessionResetEventType::ConfigurationChange.display_name(), "Configuration Change");
        assert_eq!(SessionResetEventType::System.display_name(), "System Reset");
        assert_eq!(SessionResetEventType::Startup.display_name(), "Startup Reset");
    }

    #[test]
    fn test_trigger_source_display() {
        assert_eq!(SessionResetTriggerSource::BackgroundService.display_name(), "Background Service");
        assert_eq!(SessionResetTriggerSource::UserAction.display_name(), "User Action");
        assert_eq!(SessionResetTriggerSource::ApiCall.display_name(), "API Call");
        assert_eq!(SessionResetTriggerSource::WebSocketMessage.display_name(), "WebSocket Message");
        assert_eq!(SessionResetTriggerSource::Migration.display_name(), "Migration");
        assert_eq!(SessionResetTriggerSource::ConfigurationUpdate.display_name(), "Configuration Update");
    }
}