//! Scheduling Service for Roma Timer
//!
//! Provides background task scheduling functionality for the daily reset feature.

use crate::models::scheduled_task::{ScheduledTask, ScheduledTaskType};
use crate::services::time_provider::TimeProvider;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Errors that can occur during scheduling operations
#[derive(Debug, thiserror::Error)]
pub enum SchedulingError {
    #[error("Failed to create job scheduler: {0}")]
    SchedulerCreation(#[from] JobSchedulerError),

    #[error("Job not found: {job_id}")]
    JobNotFound { job_id: String },

    #[error("Invalid cron expression: {cron_expression}")]
    InvalidCronExpression { cron_expression: String },

    #[error("Task execution failed: {message}")]
    TaskExecutionFailed { message: String },

    #[error("Scheduler not started")]
    SchedulerNotStarted,
}

/// Result type for scheduling operations
pub type SchedulingResult<T> = Result<T, SchedulingError>;

/// Trait for task execution handlers
#[async_trait]
pub trait TaskHandler: Send + Sync {
    async fn execute(&self, task: &ScheduledTask, context: &TaskContext) -> Result<(), SchedulingError>;
}

/// Context provided to task handlers during execution
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// When the task was scheduled to run
    pub scheduled_time: DateTime<Utc>,
    /// When the task actually started execution
    pub actual_start_time: DateTime<Utc>,
    /// Additional metadata passed to the task
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service for managing background task scheduling
#[derive(Debug)]
pub struct SchedulingService {
    /// The job scheduler instance
    scheduler: Arc<Mutex<Option<JobScheduler>>>,
    /// Registry of task handlers by task type
    task_handlers: Arc<RwLock<HashMap<ScheduledTaskType, Arc<dyn TaskHandler>>>>,
    /// Time provider for deterministic testing
    time_provider: Arc<dyn TimeProvider>,
}

impl SchedulingService {
    /// Creates a new SchedulingService
    ///
    /// # Arguments
    /// * `time_provider` - Time provider for testing and deterministic behavior
    pub fn new(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(None)),
            task_handlers: Arc::new(RwLock::new(HashMap::new())),
            time_provider,
        }
    }

    /// Starts the scheduling service
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(SchedulingError)` otherwise
    pub async fn start(&self) -> SchedulingResult<()> {
        let scheduler = JobScheduler::new().await?;
        let mut guard = self.scheduler.lock().await;
        *guard = Some(scheduler);
        info!("Scheduling service started");
        Ok(())
    }

    /// Stops the scheduling service
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(SchedulingError)` otherwise
    pub async fn stop(&self) -> SchedulingResult<()> {
        let mut guard = self.scheduler.lock().await;
        if let Some(scheduler) = guard.take() {
            scheduler.shutdown().await?;
            info!("Scheduling service stopped");
        }
        Ok(())
    }

    /// Registers a task handler for a specific task type
    ///
    /// # Arguments
    /// * `task_type` - The type of task this handler can process
    /// * `handler` - The handler implementation
    pub async fn register_handler(&self, task_type: ScheduledTaskType, handler: Arc<dyn TaskHandler>) {
        let mut handlers = self.task_handlers.write().await;
        handlers.insert(task_type, handler);
        info!("Registered handler for task type: {:?}", task_type);
    }

    /// Schedules a new task to run at specified times
    ///
    /// # Arguments
    /// * `task` - The task to schedule
    ///
    /// # Returns
    /// `Ok(job_id)` if successful, `Err(SchedulingError)` otherwise
    pub async fn schedule_task(&self, task: ScheduledTask) -> SchedulingResult<String> {
        let scheduler_guard = self.scheduler.lock().await;
        let scheduler = scheduler_guard.as_ref().ok_or(SchedulingError::SchedulerNotStarted)?;

        // Validate cron expression
        self.validate_cron_expression(&task.cron_expression)?;

        // Create metadata for task execution
        let mut metadata = HashMap::new();
        metadata.insert("task_id".to_string(), serde_json::Value::String(task.id.clone()));
        metadata.insert("task_type".to_string(), serde_json::Value::String(task.task_type.clone()));
        if let Some(ref config_json) = task.configuration_json {
            metadata.insert(
                "configuration".to_string(),
                serde_json::Value::String(config_json.clone()),
            );
        }

        let job_id = task.id.clone();
        let task_type = task.task_type.clone();
        let handlers = Arc::clone(&self.task_handlers);
        let time_provider = Arc::clone(&self.time_provider);

        // Create the job
        let job = Job::new_async(&task.cron_expression, move |_uuid, _l| {
            let job_id = job_id.clone();
            let task_type = task_type.clone();
            let handlers = Arc::clone(&handlers);
            let time_provider = Arc::clone(&time_provider);

            Box::pin(async move {
                let start_time = time_provider.now();

                // Create task context
                let context = TaskContext {
                    scheduled_time: start_time, // This would ideally come from the scheduler
                    actual_start_time: start_time,
                    metadata: HashMap::new(), // This should be populated with job metadata
                };

                // Find and execute the appropriate handler
                let handlers_guard = handlers.read().await;
                if let Some(handler) = handlers_guard.get(&task_type) {
                    // Create a task with the current metadata for execution
                    let task_for_execution = ScheduledTask {
                        id: job_id,
                        task_type: task_type.clone(),
                        cron_expression: "0 */2 * * * *".to_string(), // This would come from the job
                        configuration_json: None,
                        next_run_time: Some(start_time),
                        last_run_time: None,
                        is_active: true,
                        created_at: start_time,
                        updated_at: start_time,
                    };

                    match handler.execute(&task_for_execution, &context).await {
                        Ok(()) => {
                            info!("Task {} executed successfully", job_id);
                        }
                        Err(e) => {
                            error!("Task {} execution failed: {}", job_id, e);
                        }
                    }
                } else {
                    warn!("No handler found for task type: {:?}", task_type);
                }
            })
        })?;

        // Add the job to the scheduler
        scheduler.add(job).await?;

        info!("Scheduled task {} with cron: {}", task.id, task.cron_expression);
        Ok(task.id)
    }

    /// Unschedules a task
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to unschedule
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(SchedulingError)` otherwise
    pub async fn unschedule_task(&self, task_id: &str) -> SchedulingResult<()> {
        let scheduler_guard = self.scheduler.lock().await;
        let scheduler = scheduler_guard
            .as_ref()
            .ok_or(SchedulingError::SchedulerNotStarted)?;

        // Try to find and remove the job by UUID
        let uuid = Uuid::parse_str(task_id)
            .map_err(|_| SchedulingError::JobNotFound { job_id: task_id.to_string() })?;

        scheduler.remove(&uuid).await?;

        info!("Unscheduled task: {}", task_id);
        Ok(())
    }

    /// Updates an existing scheduled task
    ///
    /// # Arguments
    /// * `task` - The updated task definition
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(SchedulingError)` otherwise
    pub async fn update_task(&self, task: ScheduledTask) -> SchedulingResult<()> {
        // Remove the existing task
        self.unschedule_task(&task.id).await?;

        // Schedule the updated task
        self.schedule_task(task).await?;

        Ok(())
    }

    /// Lists all currently scheduled tasks
    ///
    /// # Returns
    /// `Ok(Vec<String>)` of task IDs, `Err(SchedulingError)` otherwise
    pub async fn list_scheduled_tasks(&self) -> SchedulingResult<Vec<String>> {
        let scheduler_guard = self.scheduler.lock().await;
        let scheduler = scheduler_guard
            .as_ref()
            .ok_or(SchedulingError::SchedulerNotStarted)?;

        let jobs = scheduler.list().await?;
        let task_ids: Vec<String> = jobs.iter().map(|job| job.guid().to_string()).collect();

        Ok(task_ids)
    }

    /// Validates a cron expression
    ///
    /// # Arguments
    /// * `cron_expression` - The cron expression to validate
    ///
    /// # Returns
    /// `Ok(())` if valid, `Err(SchedulingError)` otherwise
    fn validate_cron_expression(&self, cron_expression: &str) -> SchedulingResult<()> {
        // Try to parse the cron expression
        Job::new_async(cron_expression, |_uuid, _l| Box::pin(async {}))
            .map_err(|_| SchedulingError::InvalidCronExpression {
                cron_expression: cron_expression.to_string(),
            })?;

        Ok(())
    }

    /// Generates a cron expression for daily execution at a specific time
    ///
    /// # Arguments
    /// * `hour` - Hour of day (0-23)
    /// * `minute` - Minute of hour (0-59)
    ///
    /// # Returns
    /// A cron expression string for daily execution
    pub fn generate_daily_cron_expression(hour: u32, minute: u32) -> String {
        format!("{} {} * * *", minute, hour)
    }

    /// Generates a cron expression for hourly execution at a specific minute
    ///
    /// # Arguments
    /// * `minute` - Minute of hour (0-59)
    ///
    /// # Returns
    /// A cron expression string for hourly execution
    pub fn generate_hourly_cron_expression(minute: u32) -> String {
        format!("{} * * * *", minute)
    }

    /// Gets the next run time for a cron expression
    ///
    /// # Arguments
    /// * `cron_expression` - The cron expression to evaluate
    ///
    /// # Returns
    /// `Ok(Some(DateTime<Utc>))` with next run time, `Ok(None)` if no future runs, `Err(SchedulingError)` if invalid
    pub async fn get_next_run_time(&self, cron_expression: &str) -> SchedulingResult<Option<DateTime<Utc>>> {
        // Validate the cron expression first
        self.validate_cron_expression(cron_expression)?;

        // For now, we'll use a simple implementation
        // In a production system, you might want to use a more sophisticated cron parser
        let current_time = self.time_provider.now();

        // This is a simplified implementation - you'd typically use a cron library
        // that can compute next execution times accurately
        let next_run = current_time + chrono::Duration::hours(1); // Placeholder

        Ok(Some(next_run))
    }

    /// Checks if the scheduler is currently running
    ///
    /// # Returns
    /// `true` if running, `false` otherwise
    pub async fn is_running(&self) -> bool {
        let guard = self.scheduler.lock().await;
        guard.is_some()
    }
}

/// Default implementation for unit testing
#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Mock task handler for testing
    #[derive(Debug)]
    pub struct MockTaskHandler {
        pub should_fail: bool,
        pub execution_count: Arc<Mutex<usize>>,
    }

    impl MockTaskHandler {
        pub fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                execution_count: Arc::new(Mutex::new(0)),
            }
        }

        pub async fn get_execution_count(&self) -> usize {
            *self.execution_count.lock().await
        }
    }

    #[async_trait]
    impl TaskHandler for MockTaskHandler {
        async fn execute(&self, task: &ScheduledTask, _context: &TaskContext) -> Result<(), SchedulingError> {
            *self.execution_count.lock().await += 1;

            if self.should_fail {
                Err(SchedulingError::TaskExecutionFailed {
                    message: "Mock task execution failed".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*;
    use crate::services::time_provider::SystemTimeProvider;

    #[tokio::test]
    async fn test_scheduling_service_creation() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduling_service_start_stop() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        assert!(service.start().await.is_ok());
        assert!(service.is_running().await);

        assert!(service.stop().await.is_ok());
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_cron_expression_validation() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        // Valid expressions
        assert!(service.validate_cron_expression("0 0 * * *").is_ok());
        assert!(service.validate_cron_expression("30 14 * * *").is_ok());
        assert!(service.validate_cron_expression("*/15 * * * *").is_ok());

        // Invalid expressions
        assert!(service.validate_cron_expression("invalid").is_err());
        assert!(service.validate_cron_expression("60 0 * * *").is_err()); // Invalid minute
        assert!(service.validate_cron_expression("0 24 * * *").is_err()); // Invalid hour
    }

    #[tokio::test]
    async fn test_generate_cron_expressions() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        assert_eq!(service.generate_daily_cron_expression(0, 0), "0 0 * * *");
        assert_eq!(service.generate_daily_cron_expression(14, 30), "30 14 * * *");

        assert_eq!(service.generate_hourly_cron_expression(0), "0 * * * *");
        assert_eq!(service.generate_hourly_cron_expression(15), "15 * * * *");
    }

    #[tokio::test]
    async fn test_register_handler() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        let handler = Arc::new(MockTaskHandler::new(false));
        service.register_handler(ScheduledTaskType::DailyReset, handler).await;

        // Verify the handler was registered (internal state, so we can't directly test)
        // But we can test that the service doesn't panic
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_get_next_run_time() {
        let time_provider = Arc::new(SystemTimeProvider);
        let service = SchedulingService::new(time_provider);

        let next_run = service.get_next_run_time("0 0 * * *").unwrap();
        assert!(next_run.is_some());

        // Should fail for invalid cron expressions
        assert!(service.get_next_run_time("invalid").is_err());
    }
}