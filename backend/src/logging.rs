//! Logging configuration for Roma Timer
//!
//! Structured logging setup with appropriate levels and formatting.

use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, Registry,
};

/// Initialize the application logging system
pub fn init_logging() {
    let default_filter = "roma_timer=info,tower_http=info,axum::rejection=trace".to_string();

    // Read log level from environment variable or use default
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| default_filter);

    // Parse filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    // Create the subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(json_layer())
        .with(console_layer());

    // Set as global subscriber
    subscriber.init();

    tracing::info!("Logging system initialized");
}

/// JSON logging layer for production
fn json_layer() -> impl Layer<Registry> {
    fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
}

/// Console logging layer for development
fn console_layer() -> impl Layer<Registry> {
    fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .with_ansi(true)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
}

/// Create a span for request logging
#[macro_export]
macro_rules! request_span {
    ($method:expr, $path:expr) => {
        tracing::info_span!(
            "http_request",
            method = %$method,
            path = %$path,
            status_code = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
        )
    };
}

/// Create a span for WebSocket connection logging
#[macro_export]
macro_rules! websocket_span {
    ($connection_id:expr) => {
        tracing::info_span!(
            "websocket_connection",
            connection_id = %$connection_id,
            user_agent = tracing::field::Empty,
            connected_at = tracing::field::Empty,
            messages_sent = tracing::field::Empty,
            messages_received = tracing::field::Empty,
        )
    };
}

/// Create a span for database operations
#[macro_export]
macro_rules! db_span {
    ($operation:expr, $table:expr) => {
        tracing::debug_span!(
            "database_operation",
            operation = %$operation,
            table = %$table,
            duration_ms = tracing::field::Empty,
            rows_affected = tracing::field::Empty,
        )
    };
}

/// Create a span for timer operations
#[macro_export]
macro_rules! timer_span {
    ($operation:expr, $session_id:expr) => {
        tracing::info_span!(
            "timer_operation",
            operation = %$operation,
            session_id = %$session_id,
            timer_type = tracing::field::Empty,
            duration = tracing::field::Empty,
            elapsed = tracing::field::Empty,
        )
    };
}

/// Log application startup
pub fn log_startup() {
    tracing::info!(
        "Roma Timer starting up",
        version = env!("CARGO_PKG_VERSION"),
        git_commit = option_env!("GIT_COMMIT").unwrap_or("unknown"),
        build_time = option_env!("BUILD_TIME").unwrap_or("unknown"),
    );
}

/// Log WebSocket connection established
pub fn log_websocket_connected(connection_id: &str, user_agent: Option<&str>) {
    tracing::info!(
        "WebSocket connection established",
        connection_id = %connection_id,
        user_agent = ?user_agent,
    );
}

/// Log WebSocket connection closed
pub fn log_websocket_disconnected(connection_id: &str, reason: &str) {
    tracing::info!(
        "WebSocket connection closed",
        connection_id = %connection_id,
        reason = %reason,
    );
}

/// Log WebSocket message received
pub fn log_websocket_message_received(connection_id: &str, message_type: &str) {
    tracing::debug!(
        "WebSocket message received",
        connection_id = %connection_id,
        message_type = %message_type,
    );
}

/// Log WebSocket message sent
pub fn log_websocket_message_sent(connection_id: &str, message_type: &str) {
    tracing::debug!(
        "WebSocket message sent",
        connection_id = %connection_id,
        message_type = %message_type,
    );
}

/// Log timer state change
pub fn log_timer_state_change(
    session_id: &str,
    operation: &str,
    timer_type: &str,
    elapsed: u32,
    duration: u32,
) {
    tracing::info!(
        "Timer state changed",
        session_id = %session_id,
        operation = %operation,
        timer_type = %timer_type,
        elapsed = %elapsed,
        duration = %duration,
        remaining = duration.saturating_sub(elapsed),
    );
}

/// Log timer session completion
pub fn log_timer_session_completed(session_id: &str, timer_type: &str, work_sessions_completed: u32) {
    tracing::info!(
        "Timer session completed",
        session_id = %session_id,
        timer_type = %timer_type,
        work_sessions_completed = %work_sessions_completed,
    );
}

/// Log configuration update
pub fn log_configuration_update(user_id: &str, updated_fields: &[&str]) {
    tracing::info!(
        "Configuration updated",
        user_id = %user_id,
        updated_fields = ?updated_fields,
    );
}

/// Log notification delivery
pub fn log_notification_delivery(
    session_id: &str,
    notification_type: &str,
    delivery_method: &str,
    success: bool,
) {
    if success {
        tracing::info!(
            "Notification delivered successfully",
            session_id = %session_id,
            notification_type = %notification_type,
            delivery_method = %delivery_method,
        );
    } else {
        tracing::warn!(
            "Notification delivery failed",
            session_id = %session_id,
            notification_type = %notification_type,
            delivery_method = %delivery_method,
        );
    }
}

/// Log database operation
pub fn log_database_operation(operation: &str, table: &str, duration_ms: u64, rows_affected: Option<u64>) {
    tracing::debug!(
        "Database operation completed",
        operation = %operation,
        table = %table,
        duration_ms = %duration_ms,
        rows_affected = ?rows_affected,
    );
}

/// Log authentication event
pub fn log_authentication_event(event: &str, user_id: Option<&str>, success: bool) {
    if success {
        tracing::info!(
            "Authentication successful",
            event = %event,
            user_id = ?user_id,
        );
    } else {
        tracing::warn!(
            "Authentication failed",
            event = %event,
            user_id = ?user_id,
        );
    }
}

/// Log error with context
pub fn log_error(error: &str, context: &str, session_id: Option<&str>) {
    tracing::error!(
        error = %error,
        context = %context,
        session_id = ?session_id,
        "Application error occurred"
    );
}

/// Log warning with context
pub fn log_warning(warning: &str, context: &str, session_id: Option<&str>) {
    tracing::warn!(
        warning = %warning,
        context = %context,
        session_id = ?session_id,
        "Application warning"
    );
}

/// Log performance metrics
pub fn log_performance_metrics(
    operation: &str,
    duration_ms: u64,
    success: bool,
    additional_metrics: &[(&str, u64)],
) {
    let mut fields = Vec::new();
    for (key, value) in additional_metrics {
        fields.push(format!("{}={}", key, value));
    }

    if success {
        tracing::info!(
            "Performance metrics",
            operation = %operation,
            duration_ms = %duration_ms,
            additional = ?fields.join(", "),
        );
    } else {
        tracing::warn!(
            "Performance metrics (failed)",
            operation = %operation,
            duration_ms = %duration_ms,
            additional = ?fields.join(", "),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        // This test just verifies that the logging setup doesn't panic
        // In a real test environment, you might want to capture logs
        init_logging();
    }

    #[test]
    fn test_log_macros_compilation() {
        // Test that log macros compile correctly
        let _span = request_span!("GET", "/api/health");
        let _span = websocket_span!("test-connection");
        let _span = db_span!("SELECT", "timer_sessions");
        let _span = timer_span!("START", "test-session");
    }
}