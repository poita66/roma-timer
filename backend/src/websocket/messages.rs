//! WebSocket Message Extensions
//!
//! Provides request/response message patterns for daily reset WebSocket operations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Request-response message wrapper for WebSocket operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DailyResetRequestMessage {
    /// Configure daily reset settings
    #[serde(rename = "configure_daily_reset")]
    ConfigureDailyReset(ConfigureDailyResetRequest),

    /// Get daily reset status
    #[serde(rename = "get_daily_reset_status")]
    GetDailyResetStatus(GetDailyResetStatusRequest),

    /// Get session count
    #[serde(rename = "get_session_count")]
    GetSessionCount(GetSessionCountRequest),

    /// Set session count
    #[serde(rename = "set_session_count")]
    SetSessionCount(SetSessionCountRequest),

    /// Reset session
    #[serde(rename = "reset_session")]
    ResetSession(ResetSessionRequest),

    /// Get daily stats
    #[serde(rename = "get_daily_stats")]
    GetDailyStats(GetDailyStatsRequest),

    /// Get reset events
    #[serde(rename = "get_reset_events")]
    GetResetEvents(GetResetEventsRequest),

    /// Get session summary
    #[serde(rename = "get_session_summary")]
    GetSessionSummary(GetSessionSummaryRequest),
}

/// Response message wrapper for WebSocket operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DailyResetResponseMessage {
    /// Configure daily reset response
    #[serde(rename = "configure_daily_reset_response")]
    ConfigureDailyResetResponse(ConfigureDailyResetResponse),

    /// Daily reset status response
    #[serde(rename = "daily_reset_status_response")]
    DailyResetStatusResponse(DailyResetStatusResponse),

    /// Session count response
    #[serde(rename = "session_count_response")]
    SessionCountResponse(SessionCountResponse),

    /// Session set response
    #[serde(rename = "session_set_response")]
    SessionSetResponse(SessionSetResponse),

    /// Session reset response
    #[serde(rename = "session_reset_response")]
    SessionResetResponse(SessionResetResponse),

    /// Daily stats response
    #[serde(rename = "daily_stats_response")]
    DailyStatsResponse(DailyStatsResponse),

    /// Reset events response
    #[serde(rename = "reset_events_response")]
    ResetEventsResponse(ResetEventsResponse),

    /// Session summary response
    #[serde(rename = "session_summary_response")]
    SessionSummaryResponse(SessionSummaryResponse),

    /// Error response
    #[serde(rename = "error")]
    Error(ErrorResponse),
}

// Request message types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureDailyResetRequest {
    pub message_id: String,
    pub user_id: String,
    pub enabled: bool,
    pub reset_time_type: String, // "midnight", "hour", "custom"
    pub reset_hour: Option<u8>,
    pub custom_time: Option<String>,
    pub timezone: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyResetStatusRequest {
    pub message_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionCountRequest {
    pub message_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSessionCountRequest {
    pub message_id: String,
    pub user_id: String,
    pub session_count: u32,
    pub manual_override: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetSessionRequest {
    pub message_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyStatsRequest {
    pub message_id: String,
    pub user_id: String,
    pub date: Option<String>,
    pub days: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResetEventsRequest {
    pub message_id: String,
    pub user_id: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionSummaryRequest {
    pub message_id: String,
    pub user_id: String,
    pub period: String, // "week", "month", "year"
    pub count: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

// Response message types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureDailyResetResponse {
    pub message_id: String,
    pub success: bool,
    pub configuration: Option<crate::models::user_configuration::UserConfiguration>,
    pub next_reset_time_utc: Option<i64>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyResetStatusResponse {
    pub message_id: String,
    pub success: bool,
    pub configuration: Option<crate::models::user_configuration::UserConfiguration>,
    pub next_reset_time_utc: Option<i64>,
    pub reset_due_today: Option<bool>,
    pub current_session_count: Option<u32>,
    pub manual_session_override: Option<u32>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCountResponse {
    pub message_id: String,
    pub success: bool,
    pub current_session_count: u32,
    pub manual_session_override: Option<u32>,
    pub last_reset_utc: Option<i64>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSetResponse {
    pub message_id: String,
    pub success: bool,
    pub current_session_count: u32,
    pub manual_session_override: Option<u32>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResetResponse {
    pub message_id: String,
    pub success: bool,
    pub previous_session_count: u32,
    pub new_session_count: u32,
    pub reset_time_utc: i64,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatsResponse {
    pub message_id: String,
    pub success: bool,
    pub stats: Vec<crate::models::daily_session_stats::DailySessionStats>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetEventsResponse {
    pub message_id: String,
    pub success: bool,
    pub events: Vec<crate::models::session_reset_event::SessionResetEvent>,
    pub total_count: Option<u32>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryResponse {
    pub message_id: String,
    pub success: bool,
    pub summary: Vec<SessionSummaryData>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryData {
    pub period_label: String,
    pub total_work_sessions: u32,
    pub total_work_minutes: u32,
    pub avg_sessions_per_day: f64,
    pub productivity_score: u32,
    pub manual_overrides: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message_id: String,
    pub error_code: String,
    pub error_message: String,
    pub timestamp: DateTime<Utc>,
}

/// Message validation error
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl DailyResetRequestMessage {
    /// Generate a new message ID
    pub fn generate_message_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get the message ID
    pub fn get_message_id(&self) -> Option<&str> {
        match self {
            DailyResetRequestMessage::ConfigureDailyReset(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::GetDailyResetStatus(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::GetSessionCount(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::SetSessionCount(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::ResetSession(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::GetDailyStats(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::GetResetEvents(msg) => Some(&msg.message_id),
            DailyResetRequestMessage::GetSessionSummary(msg) => Some(&msg.message_id),
        }
    }

    /// Get the user ID
    pub fn get_user_id(&self) -> &str {
        match self {
            DailyResetRequestMessage::ConfigureDailyReset(msg) => &msg.user_id,
            DailyResetRequestMessage::GetDailyResetStatus(msg) => &msg.user_id,
            DailyResetRequestMessage::GetSessionCount(msg) => &msg.user_id,
            DailyResetRequestMessage::SetSessionCount(msg) => &msg.user_id,
            DailyResetRequestMessage::ResetSession(msg) => &msg.user_id,
            DailyResetRequestMessage::GetDailyStats(msg) => &msg.user_id,
            DailyResetRequestMessage::GetResetEvents(msg) => &msg.user_id,
            DailyResetRequestMessage::GetSessionSummary(msg) => &msg.user_id,
        }
    }

    /// Validate the message structure
    pub fn validate(&self) -> Result<(), ValidationError> {
        let message_id = self.get_message_id().ok_or_else(|| {
            ValidationError::MissingField("message_id".to_string())
        })?;

        if message_id.is_empty() {
            return Err(ValidationError::InvalidValue("message_id cannot be empty".to_string()));
        }

        let user_id = self.get_user_id();
        if user_id.is_empty() {
            return Err(ValidationError::InvalidValue("user_id cannot be empty".to_string()));
        }

        // Message-specific validation
        match self {
            DailyResetRequestMessage::SetSessionCount(msg) => {
                if msg.session_count > 100 {
                    return Err(ValidationError::InvalidValue("session_count cannot exceed 100".to_string()));
                }
            }
            DailyResetRequestMessage::GetSessionSummary(msg) => {
                if !["week", "month", "year"].contains(&msg.period.as_str()) {
                    return Err(ValidationError::InvalidValue("period must be 'week', 'month', or 'year'".to_string()));
                }
            }
            _ => {} // No additional validation needed for other message types
        }

        Ok(())
    }

    /// Serialize message to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize message from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl DailyResetResponseMessage {
    /// Get the message ID
    pub fn get_message_id(&self) -> &str {
        match self {
            DailyResetResponseMessage::ConfigureDailyResetResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::DailyResetStatusResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::SessionCountResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::SessionSetResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::SessionResetResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::DailyStatsResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::ResetEventsResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::SessionSummaryResponse(msg) => &msg.message_id,
            DailyResetResponseMessage::Error(msg) => &msg.message_id,
        }
    }

    /// Check if the response was successful
    pub fn is_success(&self) -> bool {
        match self {
            DailyResetResponseMessage::ConfigureDailyResetResponse(msg) => msg.success,
            DailyResetResponseMessage::DailyResetStatusResponse(msg) => msg.success,
            DailyResetResponseMessage::SessionCountResponse(msg) => msg.success,
            DailyResetResponseMessage::SessionSetResponse(msg) => msg.success,
            DailyResetResponseMessage::SessionResetResponse(msg) => msg.success,
            DailyResetResponseMessage::DailyStatsResponse(msg) => msg.success,
            DailyResetResponseMessage::ResetEventsResponse(msg) => msg.success,
            DailyResetResponseMessage::SessionSummaryResponse(msg) => msg.success,
            DailyResetResponseMessage::Error(_) => false,
        }
    }

    /// Get error message if any
    pub fn get_error(&self) -> Option<&str> {
        match self {
            DailyResetResponseMessage::ConfigureDailyResetResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::DailyResetStatusResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::SessionCountResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::SessionSetResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::SessionResetResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::DailyStatsResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::ResetEventsResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::SessionSummaryResponse(msg) => msg.error.as_deref(),
            DailyResetResponseMessage::Error(msg) => Some(&msg.error_message),
        }
    }

    /// Serialize response to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize response from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}