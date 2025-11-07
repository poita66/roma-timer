//! Daily Reset Task Handler
//!
//! Implements the TaskHandler trait for daily session reset operations.
//! This handler is registered with the SchedulingService and executed
//! according to the cron schedule.

use crate::models::scheduled_task::{ScheduledTask, ScheduledTaskType};
use crate::services::daily_reset_service::DailyResetService;
use crate::services::scheduling_service::{TaskHandler, TaskContext, SchedulingError};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, error};

/// Task handler for daily session reset operations
pub struct DailyResetTaskHandler {
    daily_reset_service: Arc<DailyResetService>,
}

impl DailyResetTaskHandler {
    /// Create a new daily reset task handler
    pub fn new(daily_reset_service: Arc<DailyResetService>) -> Self {
        Self {
            daily_reset_service,
        }
    }
}

#[async_trait]
impl TaskHandler for DailyResetTaskHandler {
    /// Execute the daily reset task
    ///
    /// This method processes all users who have daily reset enabled and
    /// performs the reset operation for those who need it.
    async fn execute(&self, task: &ScheduledTask, context: &TaskContext) -> Result<(), SchedulingError> {
        info!("Executing daily reset task at scheduled time: {:?}", context.scheduled_time);

        // Process all pending daily resets
        match self.daily_reset_service.process_pending_daily_resets().await {
            Ok(reset_events) => {
                info!("Daily reset task completed successfully. Processed {} users.", reset_events.len());

                // Log details of each reset event
                for event in reset_events {
                    info!(
                        "Reset completed for user {} - previous sessions: {}, type: {}, timezone: {}",
                        event.user_configuration_id,
                        event.previous_count,
                        event.reset_type.display_name(),
                        event.user_timezone
                    );
                }

                Ok(())
            }
            Err(e) => {
                error!("Daily reset task failed: {}", e);
                Err(SchedulingError::TaskExecutionFailed { message: format!("Daily reset failed: {}", e) })
            }
        }
    }
}

/// Factory function to create and configure the daily reset task handler
/// This is typically called during application startup.
pub async fn create_daily_reset_task_handler(
    daily_reset_service: Arc<DailyResetService>,
) -> Arc<dyn TaskHandler> {
    let handler = DailyResetTaskHandler::new(daily_reset_service);
    Arc::new(handler)
}

/// Configuration for daily reset scheduling
pub struct DailyResetTaskConfig {
    /// Cron expression for when to run the daily reset
    pub cron_expression: String,
    /// Task identifier
    pub task_id: String,
    /// User configuration ID this task is associated with (if applicable)
    pub user_configuration_id: Option<String>,
    /// Timezone for the cron schedule
    pub timezone: String,
}

impl Default for DailyResetTaskConfig {
    fn default() -> Self {
        Self {
            // Run daily at 2:00 AM UTC by default
            cron_expression: "0 2 * * *".to_string(),
            task_id: "daily-reset-global".to_string(),
            user_configuration_id: None,
            timezone: "UTC".to_string(),
        }
    }
}

impl DailyResetTaskConfig {
    /// Create a new configuration
    pub fn new(
        cron_expression: String,
        task_id: String,
        timezone: String,
        user_configuration_id: Option<String>,
    ) -> Self {
        Self {
            cron_expression,
            task_id,
            timezone,
            user_configuration_id,
        }
    }

    /// Create a user-specific daily reset configuration
    pub fn for_user(
        user_id: String,
        cron_expression: String,
        timezone: String,
    ) -> Self {
        Self {
            cron_expression,
            task_id: format!("daily-reset-{}", user_id),
            timezone,
            user_configuration_id: Some(user_id),
        }
    }

    /// Create a global daily reset configuration that processes all users
    pub fn global() -> Self {
        Self::default()
    }

    /// Convert to a ScheduledTask for the scheduling service
    pub fn to_scheduled_task(&self) -> ScheduledTask {
        ScheduledTask {
            id: self.task_id.clone(),
            task_type: ScheduledTaskType::DailyReset,
            user_configuration_id: self.user_configuration_id.clone(),
            cron_expression: self.cron_expression.clone(),
            timezone: self.timezone.clone(),
            next_run_utc: chrono::Utc::now().timestamp(),
            last_run_utc: None,
            is_active: true,
            run_count: 0,
            failure_count: 0,
            task_data: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::time_provider::MockTimeProvider;
    use crate::database::DatabaseManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_daily_reset_task_config_default() {
        let config = DailyResetTaskConfig::default();
        assert_eq!(config.cron_expression, "0 2 * * *");
        assert_eq!(config.task_id, "daily-reset-global");
        assert_eq!(config.timezone, "UTC");
        assert!(config.user_configuration_id.is_none());
    }

    #[tokio::test]
    async fn test_daily_reset_task_config_for_user() {
        let config = DailyResetTaskConfig::for_user(
            "user123".to_string(),
            "0 3 * * *".to_string(),
            "America/New_York".to_string(),
        );

        assert_eq!(config.cron_expression, "0 3 * * *");
        assert_eq!(config.task_id, "daily-reset-user123");
        assert_eq!(config.timezone, "America/New_York");
        assert_eq!(config.user_configuration_id, Some("user123".to_string()));
    }

    #[tokio::test]
    async fn test_daily_reset_task_config_to_scheduled_task() {
        let config = DailyResetTaskConfig::default();
        let task = config.to_scheduled_task();

        assert_eq!(task.id, "daily-reset-global");
        assert_eq!(task.task_type, ScheduledTaskType::DailyReset);
        assert_eq!(task.cron_expression, "0 2 * * *");
        assert_eq!(task.timezone, "UTC");
        assert!(task.user_configuration_id.is_none());
        assert!(task.is_active);
    }

    // Integration test would require a real database
    // This is a placeholder for the structure of integration tests
    // #[tokio::test]
    // async fn test_daily_reset_task_handler_integration() {
    //     // Create a test database manager and daily reset service
    //     let time_provider = Arc::new(MockTimeProvider::new_from_now());
    //     let database_manager = Arc::new(create_test_database().await);
    //     let daily_reset_service = Arc::new(
    //         DailyResetService::new(time_provider, database_manager)
    //     );
    //
    //     // Create the task handler
    //     let handler = create_daily_reset_task_handler(daily_reset_service).await;
    //
    //     // Create a mock scheduled task and context
    //     let task = create_test_scheduled_task();
    //     let context = create_test_task_context();
    //
    //     // Execute the handler
    //     let result = handler.execute(&task, &context).await;
    //
    //     // Verify the result
    //     assert!(result.is_ok());
    // }
}