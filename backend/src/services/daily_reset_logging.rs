//! Structured Logging for Daily Session Reset Operations
//!
//! Provides specialized logging macros and utilities for daily reset functionality including:
//! - Session reset events with detailed context
//! - Configuration changes with audit trail
//! - Background task execution and failures
//! - Performance metrics and analytics

use tracing::{debug, error, info, warn, Span};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

/// Structured logging context for daily reset operations
#[derive(Debug, Clone)]
pub struct DailyResetLoggingContext {
    /// User configuration ID
    pub user_id: String,

    /// Device ID (if applicable)
    pub device_id: Option<String>,

    /// Request ID for tracing
    pub request_id: Option<String>,

    /// Operation type
    pub operation: DailyResetOperation,

    /// Additional context data
    pub context: Option<Value>,
}

/// Types of daily reset operations for logging
#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
pub enum DailyResetOperation {
    /// Session reset execution
    SessionReset,

    /// Manual session override
    ManualOverride,

    /// Configuration change (timezone, reset time)
    ConfigurationChange,

    /// Background task scheduling
    TaskScheduling,

    /// Task execution
    TaskExecution,

    /// Timezone validation
    TimezoneValidation,

    /// Analytics calculation
    AnalyticsCalculation,

    /// Database operation
    DatabaseOperation,

    /// WebSocket message handling
    WebSocketMessage,

    /// Error handling
    ErrorHandling,

    /// Startup/initialization
    Initialization,
}

/// Macro for logging session reset events with full context
#[macro_export]
macro_rules! log_session_reset {
    ($level:ident, $context:expr, $message:expr $(, $key:expr => $value:expr)*) => {
        {
            use tracing::$level;
            $level!(
                user_id = %$context.user_id,
                device_id = ?$context.device_id,
                request_id = ?$context.request_id,
                operation = %$context.operation,
                $(($key) = $value,)*
                $message
            );
        }
    };
}

/// Macro for logging with span creation
#[macro_export]
macro_rules! log_with_span {
    ($operation:expr, $user_id:expr, $message:expr, $block:block) => {
        {
            let span = tracing::info_span!(
                "daily_reset_operation",
                operation = %$operation,
                user_id = %$user_id,
                request_id = tracing::field::Empty,
            );

            let _enter = span.enter();
            tracing::info!($message);
            $block
        }
    };
}

/// Daily reset specific logging utilities
pub struct DailyResetLogger;

impl DailyResetLogger {
    /// Create a new logging context
    pub fn new_context(
        user_id: String,
        operation: DailyResetOperation,
    ) -> DailyResetLoggingContext {
        DailyResetLoggingContext {
            user_id,
            device_id: None,
            request_id: None,
            operation,
            context: None,
        }
    }
}

impl DailyResetLoggingContext {
    /// Create context with device ID
    pub fn with_device_id(
        mut self,
        device_id: String,
    ) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Create context with request ID
    pub fn with_request_id(
        mut self,
        request_id: String,
    ) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Create context with additional data
    pub fn with_context(
        mut self,
        context: Value,
    ) -> Self {
        self.context = Some(context);
        self
    }
}

impl DailyResetLogger {
    /// Log session reset event
    pub fn log_session_reset_event(
        &self,
        context: &DailyResetLoggingContext,
        reset_type: &str,
        previous_count: u32,
        new_count: u32,
        success: bool,
        error_message: Option<&str>,
    ) {
        let log_context = json!({
            "reset_type": reset_type,
            "previous_count": previous_count,
            "new_count": new_count,
            "success": success,
            "error_message": error_message,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if success {
            info!(
                user_id = %context.user_id,
                device_id = ?context.device_id,
                operation = %context.operation,
                context = %log_context,
                "Session reset completed successfully"
            );
        } else {
            error!(
                user_id = %context.user_id,
                device_id = ?context.device_id,
                operation = %context.operation,
                context = %log_context,
                "Session reset failed"
            );
        }
    }

    /// Log configuration change
    pub fn log_configuration_change(
        &self,
        context: &DailyResetLoggingContext,
        previous_config: Value,
        new_config: Value,
        change_source: &str,
    ) {
        let log_context = json!({
            "previous_config": previous_config,
            "new_config": new_config,
            "change_source": change_source,
            "timestamp": Utc::now().to_rfc3339(),
        });

        info!(
            user_id = %context.user_id,
            device_id = ?context.device_id,
            operation = %context.operation,
            context = %log_context,
            "Daily reset configuration changed"
        );
    }

    /// Log background task execution
    pub fn log_task_execution(
        &self,
        context: &DailyResetLoggingContext,
        task_id: &str,
        task_type: &str,
        execution_time_ms: u64,
        success: bool,
        error_message: Option<&str>,
    ) {
        let log_context = json!({
            "task_id": task_id,
            "task_type": task_type,
            "execution_time_ms": execution_time_ms,
            "success": success,
            "error_message": error_message,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if success {
            info!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Background task executed successfully"
            );
        } else {
            error!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Background task execution failed"
            );
        }
    }

    /// Log performance metrics
    pub fn log_performance_metrics(
        &self,
        context: &DailyResetLoggingContext,
        operation_name: &str,
        duration_ms: u64,
        success: bool,
        additional_metrics: Option<Value>,
    ) {
        let mut log_context = json!({
            "operation_name": operation_name,
            "duration_ms": duration_ms,
            "success": success,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Some(metrics) = additional_metrics {
            log_context["additional_metrics"] = metrics;
        }

        if duration_ms > 1000 {
            warn!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Slow operation detected"
            );
        } else {
            info!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Operation completed"
            );
        }
    }

    /// Log timezone validation
    pub fn log_timezone_validation(
        &self,
        context: &DailyResetLoggingContext,
        timezone: &str,
        is_valid: bool,
        validation_error: Option<&str>,
    ) {
        let log_context = json!({
            "timezone": timezone,
            "is_valid": is_valid,
            "validation_error": validation_error,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if is_valid {
            debug!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Timezone validation passed"
            );
        } else {
            warn!(
                user_id = %context.user_id,
                operation = %context.operation,
                context = %log_context,
                "Timezone validation failed"
            );
        }
    }

    /// Log WebSocket message
    pub fn log_websocket_message(
        &self,
        context: &DailyResetLoggingContext,
        message_type: &str,
        message_size: usize,
        processing_time_ms: u64,
        success: bool,
    ) {
        let log_context = json!({
            "message_type": message_type,
            "message_size": message_size,
            "processing_time_ms": processing_time_ms,
            "success": success,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if success {
            debug!(
                user_id = %context.user_id,
                device_id = ?context.device_id,
                operation = %context.operation,
                context = %log_context,
                "WebSocket message processed"
            );
        } else {
            error!(
                user_id = %context.user_id,
                device_id = ?context.device_id,
                operation = %context.operation,
                context = %log_context,
                "WebSocket message processing failed"
            );
        }
    }

    /// Log analytics calculation
    pub fn log_analytics_calculation(
        &self,
        context: &DailyResetLoggingContext,
        calculation_type: &str,
        date_range: &str,
        record_count: usize,
        duration_ms: u64,
    ) {
        let log_context = json!({
            "calculation_type": calculation_type,
            "date_range": date_range,
            "record_count": record_count,
            "duration_ms": duration_ms,
            "timestamp": Utc::now().to_rfc3339(),
        });

        info!(
            user_id = %context.user_id,
            operation = %context.operation,
            context = %log_context,
            "Analytics calculation completed"
        );
    }

    /// Log error with structured context
    pub fn log_error(
        &self,
        context: &DailyResetLoggingContext,
        error_type: &str,
        error_message: &str,
        recoverable: bool,
        error_details: Option<Value>,
    ) {
        let mut log_context = json!({
            "error_type": error_type,
            "error_message": error_message,
            "recoverable": recoverable,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Some(details) = error_details {
            log_context["error_details"] = details;
        }

        error!(
            user_id = %context.user_id,
            device_id = ?context.device_id,
            operation = %context.operation,
            context = %log_context,
            "Daily reset operation error"
        );
    }
}

/// Utility functions for common logging patterns
pub mod utils {
    use super::*;
    use std::time::Instant;

    /// Measure execution time and log performance
    pub async fn measure_and_log<F, T>(
        context: &DailyResetLoggingContext,
        logger: &DailyResetLogger,
        operation_name: &str,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let start_time = Instant::now();
        let result = operation.await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => {
                logger.log_performance_metrics(context, operation_name, duration_ms, true, None);
            }
            Err(e) => {
                let error_details = json!({
                    "error_type": std::any::type_name_of_val(e),
                    "error_message": e.to_string(),
                });
                logger.log_performance_metrics(context, operation_name, duration_ms, false, Some(error_details));
            }
        }

        result
    }

    /// Generate a unique request ID
    pub fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Create a span for operation tracing
    pub fn create_operation_span(
        operation: &DailyResetOperation,
        user_id: &str,
        request_id: Option<&str>,
    ) -> Span {
        let span = tracing::info_span!(
            "daily_reset_operation",
            operation = %operation,
            user_id = %user_id,
        );

        if let Some(req_id) = request_id {
            span.record("request_id", req_id);
        }

        span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_context_creation() {
        let logger = DailyResetLogger;
        let context = logger
            .new_context("test-user".to_string(), DailyResetOperation::SessionReset)
            .with_device_id("test-device".to_string())
            .with_request_id("test-request".to_string());

        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.device_id, Some("test-device".to_string()));
        assert_eq!(context.request_id, Some("test-request".to_string()));
        assert_eq!(context.operation, DailyResetOperation::SessionReset);
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = utils::generate_request_id();
        let id2 = utils::generate_request_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID string length
        assert_eq!(id2.len(), 36);
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(DailyResetOperation::SessionReset.to_string(), "SessionReset");
        assert_eq!(DailyResetOperation::ManualOverride.to_string(), "ManualOverride");
        assert_eq!(DailyResetOperation::ConfigurationChange.to_string(), "ConfigurationChange");
    }
}