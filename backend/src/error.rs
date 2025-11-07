//! Error handling for Roma Timer
//!
//! Centralized error types and handling for the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Timer session error: {0}")]
    TimerSession(#[from] crate::models::timer_session::TimerSessionError),

    #[error("User configuration error: {0}")]
    UserConfiguration(#[from] crate::models::user_configuration::UserConfigurationError),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UUID generation error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("HMAC error: {0}")]
    Hmac(#[from] hmac::digest::MacError),

    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Timer already running")]
    TimerAlreadyRunning,

    #[error("Timer not running")]
    TimerNotRunning,

    #[error("Invalid timer state: {0}")]
    InvalidTimerState(String),

    #[error("Configuration not found")]
    ConfigurationNotFound,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Device not connected")]
    DeviceNotConnected,

    #[error("Notification delivery failed: {0}")]
    NotificationDeliveryFailed(String),

    // Daily Session Reset specific errors
    #[error("Daily reset scheduling error: {0}")]
    DailyResetScheduling(String),

    #[error("Timezone validation error: {0}")]
    TimezoneValidation(String),

    #[error("Session reset failed: {0}")]
    SessionResetFailed(String),

    #[error("Manual session override invalid: {0}")]
    ManualSessionOverrideInvalid(String),

    #[error("Daily reset configuration invalid: {0}")]
    DailyResetConfigurationInvalid(String),

    #[error("Background task failed: {0}")]
    BackgroundTaskFailed(String),

    #[error("Analytics calculation error: {0}")]
    AnalyticsCalculation(String),

    #[error("WebSocket message validation error: {0}")]
    WebSocketMessageValidation(#[from] crate::models::websocket_messages::ValidationError),

    #[error("Cron expression invalid: {0}")]
    InvalidCronExpression(String),
}

impl AppError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) | AppError::Internal(_) | AppError::Io(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::BadRequest(_) | AppError::Validation(_) | AppError::InvalidTimerState(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) | AppError::ConfigurationNotFound | AppError::SessionNotFound => {
                StatusCode::NOT_FOUND
            }
            AppError::Conflict(_) | AppError::TimerAlreadyRunning => StatusCode::CONFLICT,
            AppError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::WebSocket(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UserConfiguration(_) => StatusCode::BAD_REQUEST,
            AppError::TimerSession(_) => StatusCode::BAD_REQUEST,
            AppError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Uuid(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpClient(_) => StatusCode::BAD_GATEWAY,
            AppError::UrlParse(_) => StatusCode::BAD_REQUEST,
            AppError::Hmac(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Base64(_) => StatusCode::BAD_REQUEST,
            AppError::TimerNotRunning => StatusCode::CONFLICT,
            AppError::DeviceNotConnected => StatusCode::SERVICE_UNAVAILABLE,
            AppError::NotificationDeliveryFailed(_) => StatusCode::BAD_GATEWAY,
            // Daily reset errors
            AppError::DailyResetScheduling(_) | AppError::BackgroundTaskFailed(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::TimezoneValidation(_)
            | AppError::SessionResetFailed(_)
            | AppError::ManualSessionOverrideInvalid(_)
            | AppError::DailyResetConfigurationInvalid(_)
            | AppError::AnalyticsCalculation(_)
            | AppError::InvalidCronExpression(_) => StatusCode::BAD_REQUEST,
            AppError::WebSocketMessageValidation(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DatabaseError",
            AppError::BadRequest(_) => "BadRequest",
            AppError::Unauthorized => "Unauthorized",
            AppError::Forbidden => "Forbidden",
            AppError::NotFound(_) => "NotFound",
            AppError::Conflict(_) => "Conflict",
            AppError::TooManyRequests => "TooManyRequests",
            AppError::ServiceUnavailable => "ServiceUnavailable",
            AppError::Authentication(_) => "AuthenticationError",
            AppError::WebSocket(_) => "WebSocketError",
            AppError::Validation(_) => "ValidationError",
            AppError::Internal(_) => "InternalError",
            AppError::UserConfiguration(_) => "ConfigurationError",
            AppError::TimerSession(_) => "TimerSessionError",
            AppError::Serialization(_) => "SerializationError",
            AppError::Uuid(_) => "UuidError",
            AppError::Io(_) => "IoError",
            AppError::HttpClient(_) => "HttpClientError",
            AppError::UrlParse(_) => "UrlParseError",
            AppError::Hmac(_) => "HmacError",
            AppError::Base64(_) => "Base64Error",
            AppError::TimerAlreadyRunning => "TimerAlreadyRunning",
            AppError::TimerNotRunning => "TimerNotRunning",
            AppError::InvalidTimerState(_) => "InvalidTimerState",
            AppError::ConfigurationNotFound => "ConfigurationNotFound",
            AppError::SessionNotFound => "SessionNotFound",
            AppError::DeviceNotConnected => "DeviceNotConnected",
            AppError::NotificationDeliveryFailed(_) => "NotificationDeliveryFailed",
            // Daily reset error codes
            AppError::DailyResetScheduling(_) => "DailyResetSchedulingError",
            AppError::TimezoneValidation(_) => "TimezoneValidationError",
            AppError::SessionResetFailed(_) => "SessionResetFailed",
            AppError::ManualSessionOverrideInvalid(_) => "ManualSessionOverrideInvalid",
            AppError::DailyResetConfigurationInvalid(_) => "DailyResetConfigurationInvalid",
            AppError::BackgroundTaskFailed(_) => "BackgroundTaskFailed",
            AppError::AnalyticsCalculation(_) => "AnalyticsCalculationError",
            AppError::WebSocketMessageValidation(_) => "WebSocketMessageValidationError",
            AppError::InvalidCronExpression(_) => "InvalidCronExpression",
        }
    }

    /// Check if this error should be logged as an error vs warning
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            AppError::Database(_)
                | AppError::Internal(_)
                | AppError::Io(_)
                | AppError::Serialization(_)
                | AppError::Uuid(_)
                | AppError::Hmac(_)
                | AppError::HttpClient(_)
                | AppError::NotificationDeliveryFailed(_)
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let body = Json(json!({
            "error": error_code,
            "message": message,
            "timestamp": timestamp
        }));

        (status, body).into_response()
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

/// Convert authentication errors
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

/// Convert timer state errors
impl AppError {
    pub fn timer_already_running() -> Self {
        AppError::TimerAlreadyRunning
    }

    pub fn timer_not_running() -> Self {
        AppError::TimerNotRunning
    }

    pub fn invalid_timer_state(state: &str) -> Self {
        AppError::InvalidTimerState(state.to_string())
    }

    pub fn configuration_not_found() -> Self {
        AppError::ConfigurationNotFound
    }

    pub fn session_not_found() -> Self {
        AppError::SessionNotFound
    }

    pub fn device_not_connected() -> Self {
        AppError::DeviceNotConnected
    }

    pub fn authentication_failed(message: &str) -> Self {
        AppError::Authentication(message.to_string())
    }

    pub fn websocket_error(message: &str) -> Self {
        AppError::WebSocket(message.to_string())
    }

    pub fn validation_error(message: &str) -> Self {
        AppError::Validation(message.to_string())
    }

    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} not found", resource))
    }

    pub fn bad_request(message: &str) -> Self {
        AppError::BadRequest(message.to_string())
    }

    pub fn conflict(message: &str) -> Self {
        AppError::Conflict(message.to_string())
    }

    pub fn internal_error(message: &str) -> Self {
        AppError::Internal(message.to_string())
    }

    pub fn notification_delivery_failed(message: &str) -> Self {
        AppError::NotificationDeliveryFailed(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(
            AppError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            AppError::Internal("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_app_error_codes() {
        assert_eq!(AppError::BadRequest("test".to_string()).error_code(), "BadRequest");
        assert_eq!(AppError::Unauthorized.error_code(), "Unauthorized");
        assert_eq!(AppError::NotFound("test".to_string()).error_code(), "NotFound");
        assert_eq!(AppError::Internal("test".to_string()).error_code(), "InternalError");
    }

    #[test]
    fn test_server_error_detection() {
        assert!(AppError::Internal("test".to_string()).is_server_error());
        assert!(AppError::Database(sqlx::Error::RowNotFound).is_server_error());
        assert!(!AppError::BadRequest("test".to_string()).is_server_error());
        assert!(!AppError::Unauthorized.is_server_error());
    }

    #[test]
    fn test_convenience_constructors() {
        let error = AppError::authentication_failed("Invalid token");
        assert!(matches!(error, AppError::Authentication(_)));

        let error = AppError::validation_error("Invalid input");
        assert!(matches!(error, AppError::Validation(_)));

        let error = AppError::not_found("User");
        assert!(matches!(error, AppError::NotFound(_)));

        let error = AppError::timer_already_running();
        assert_eq!(error, AppError::TimerAlreadyRunning);
    }

    #[test]
    fn test_error_response_format() {
        let error = AppError::BadRequest("Invalid input".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}