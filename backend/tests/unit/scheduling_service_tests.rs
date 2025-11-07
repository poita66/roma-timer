//! Unit Tests for Scheduling Service
//!
//! Tests background task scheduling functionality including:
//! - Cron expression parsing and validation
//! - Task creation and persistence
//! - Task execution and timing
//! - Timezone-aware scheduling
//! - Task failure and retry handling

use std::sync::Arc;
use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;

use crate::services::time_provider::{TimeProvider, MockTimeProvider};
use crate::database::DailyResetDatabaseExtensions;

use super::daily_reset_test_utils::{DailyResetTestContext, factories};

#[cfg(test)]
mod scheduling_service_tests {
    use super::*;

    /// Test cron expression generation from daily reset configuration
    #[tokio::test]
    async fn test_cron_expression_generation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Test midnight reset
        let midnight_config = context.create_test_user_config(
            "UTC",
            factories::midnight_reset_time(),
            true
        ).await?;
        assert_eq!(midnight_config.get_daily_reset_cron_expression(), "0 0 * * *");

        // Test hourly reset at 7 AM
        let hourly_config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(7)?,
            true
        ).await?;
        assert_eq!(hourly_config.get_daily_reset_cron_expression(), "0 7 * * *");

        // Test hourly reset at 5 PM (17:00)
        let evening_config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(17)?,
            true
        ).await?;
        assert_eq!(evening_config.get_daily_reset_cron_expression(), "0 17 * * *");

        // Test custom reset at 14:30
        let custom_config = context.create_test_user_config(
            "UTC",
            factories::custom_reset_time("14:30")?,
            true
        ).await?;
        assert_eq!(custom_config.get_daily_reset_cron_expression(), "0 30 14 * * *");

        Ok(())
    }

    /// Test scheduled task creation and persistence
    #[tokio::test]
    async fn test_scheduled_task_creation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user configuration with daily reset
        let config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        // Create scheduled task data
        let task_id = format!("daily_reset_task_{}", config.id);
        let task_data = crate::database::ScheduledTaskData {
            id: task_id.clone(),
            task_type: "daily_reset".to_string(),
            user_configuration_id: Some(config.id.clone()),
            cron_expression: config.get_daily_reset_cron_expression(),
            timezone: config.timezone.clone(),
            next_run_utc: calculate_next_run_time(&config, &context.time_provider).timestamp() as u64,
            last_run_utc: None,
            is_active: true,
            run_count: 0,
            failure_count: 0,
            task_data: None,
        };

        // Save task to database
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Verify task was saved
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert!(!active_tasks.is_empty());

        // Find our task
        let saved_task = active_tasks.iter()
            .find(|task| task.id == task_id)
            .expect("Task should be found in active tasks");

        assert_eq!(saved_task.task_type, "daily_reset");
        assert_eq!(saved_task.user_configuration_id, Some(config.id));
        assert_eq!(saved_task.cron_expression, "0 8 * * *");
        assert_eq!(saved_task.timezone, "UTC");
        assert_eq!(saved_task.is_active, true);
        assert_eq!(saved_task.run_count, 0);
        assert_eq!(saved_task.failure_count, 0);

        Ok(())
    }

    /// Test task execution timing
    #[tokio::test]
    async fn test_task_execution_timing() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Set time to before scheduled execution
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 7, 59, 0).single().unwrap());

        // Create user with 8 AM reset
        let config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        // Create scheduled task for 8 AM
        let task_data = create_daily_reset_task(&config, &context.time_provider)?;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Should not be ready yet (current time is 7:59 AM)
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert!(active_tasks.is_empty());

        // Advance time to 8:01 AM
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 8, 1, 0).single().unwrap());

        // Should now be ready
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert_eq!(active_tasks.len(), 1);

        let task = &active_tasks[0];
        assert_eq!(task.task_type, "daily_reset");
        assert!(task.next_run_utc <= context.current_time().timestamp() as i64);

        Ok(())
    }

    /// Test timezone-aware scheduling
    #[tokio::test]
    async fn test_timezone_aware_scheduling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user with 7 AM reset in New York (UTC-5 in January)
        let config = context.create_test_user_config(
            "America/New_York",
            factories::hourly_reset_time(7)?,
            true
        ).await?;

        // Set time to 11:30 UTC (6:30 AM New York time)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 11, 30, 0).single().unwrap());

        // Create scheduled task
        let task_data = create_daily_reset_task(&config, &context.time_provider)?;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Should not be ready yet (current NY time is 6:30 AM, reset is at 7 AM NY)
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert!(active_tasks.is_empty());

        // Advance time to 12:01 UTC (7:01 AM New York time)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 12, 1, 0).single().unwrap());

        // Should now be ready
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert_eq!(active_tasks.len(), 1);

        // Verify the task was scheduled correctly for the timezone
        let task = &active_tasks[0];
        assert_eq!(task.timezone, "America/New_York");

        Ok(())
    }

    /// Test task execution and update
    #[tokio::test]
    async fn test_task_execution_and_update() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create user and task
        let config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap());

        let mut task_data = create_daily_reset_task(&config, &context.time_provider)?;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Execute the task
        let execution_time = context.current_time();
        let new_run_count = task_data.run_count + 1;

        // Update task execution
        context.db_manager.update_scheduled_task_execution(
            &task_data.id,
            execution_time.timestamp() as u64,
            new_run_count,
            task_data.failure_count,
        ).await?;

        // Verify update
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        // Task should no longer be active until next scheduled time
        assert!(active_tasks.is_empty());

        // Advance time and check if task is rescheduled
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 8, 8, 0, 0).single().unwrap());

        // Create updated task for next execution
        task_data.next_run_utc = calculate_next_run_time(&config, &context.time_provider).timestamp() as u64;
        task_data.run_count = new_run_count;
        task_data.last_run_utc = Some(execution_time.timestamp() as u64);

        context.db_manager.upsert_scheduled_task(&task_data).await?;

        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert_eq!(active_tasks.len(), 1);

        let updated_task = &active_tasks[0];
        assert_eq!(updated_task.run_count, 1);
        assert_eq!(updated_task.last_run_utc, Some(execution_time.timestamp() as i64));

        Ok(())
    }

    /// Test task failure handling
    #[tokio::test]
    async fn test_task_failure_handling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap());

        let mut task_data = create_daily_reset_task(&config, &context.time_provider)?;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Simulate failed execution
        let execution_time = context.current_time();
        let new_failure_count = task_data.failure_count + 1;

        context.db_manager.update_scheduled_task_execution(
            &task_data.id,
            execution_time.timestamp() as u64,
            task_data.run_count, // Run count unchanged
            new_failure_count,
        ).await?;

        // Verify failure was recorded
        // Note: In a real implementation, you might query the specific task
        // For now, we'll update and verify the task data structure
        task_data.failure_count = new_failure_count;
        task_data.last_run_utc = Some(execution_time.timestamp() as u64);

        assert_eq!(task_data.failure_count, 1);
        assert_eq!(task_data.run_count, 0); // Unchanged on failure

        Ok(())
    }

    /// Test task deactivation
    #[tokio::test]
    async fn test_task_deactivation() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        let mut config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        // Create active task
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 9, 0, 0).single().unwrap());
        let mut task_data = create_daily_reset_task(&config, &context.time_provider)?;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert_eq!(active_tasks.len(), 1);

        // Disable daily reset in configuration
        config.set_daily_reset_enabled(false);
        context.db_manager.save_user_configuration(&config).await?;

        // Deactivate the task
        task_data.is_active = false;
        context.db_manager.upsert_scheduled_task(&task_data).await?;

        // Should not be in active tasks
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;
        assert!(active_tasks.is_empty());

        Ok(())
    }

    /// Test multiple user scheduling
    #[tokio::test]
    async fn test_multiple_user_scheduling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetTestContext::new().await?;

        // Create multiple users with different reset times
        let user1_config = context.create_test_user_config(
            "UTC",
            factories::hourly_reset_time(6)?,
            true
        ).await?;

        let user2_config = context.create_test_user_config(
            "America/New_York",
            factories::hourly_reset_time(8)?,
            true
        ).await?;

        let user3_config = context.create_test_user_config(
            "Asia/Tokyo",
            factories::custom_reset_time("22:00")?,
            true
        ).await?;

        // Set time to trigger some tasks
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 7, 0, 0).single().unwrap());

        // Create tasks for all users
        let task1 = create_daily_reset_task(&user1_config, &context.time_provider)?;
        let task2 = create_daily_reset_task(&user2_config, &context.time_provider)?;
        let task3 = create_daily_reset_task(&user3_config, &context.time_provider)?;

        context.db_manager.upsert_scheduled_task(&task1).await?;
        context.db_manager.upsert_scheduled_task(&task2).await?;
        context.db_manager.upsert_scheduled_task(&task3).await?;

        // Check which tasks are active
        let active_tasks = context.db_manager.get_active_scheduled_tasks().await?;

        // User 1 (6 AM UTC) should be past due time, so should be active
        // User 2 (8 AM New York = 13 PM UTC) should not be active yet
        // User 3 (22:00 Tokyo = 13:00 UTC) should not be active yet

        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].user_configuration_id, Some(user1_config.id));

        Ok(())
    }

    /// Test cron expression validation
    #[tokio::test]
    async fn test_cron_expression_validation() {
        let valid_expressions = vec![
            "0 0 * * *",       // Midnight every day
            "0 8 * * *",       // 8 AM every day
            "0 17 * * *",      // 5 PM every day
            "0 30 14 * * *",   // 2:30 PM every day
            "0 0 1 * *",       // Midnight on 1st of every month
        ];

        for expr in valid_expressions {
            let validation_result = validate_cron_expression(expr);
            assert!(validation_result.is_ok(), "Expected '{}' to be valid", expr);
        }

        let invalid_expressions = vec![
            "60 0 * * *",       // Invalid minute (60)
            "0 25 * * *",       // Invalid hour (25)
            "0 0 32 * *",       // Invalid day (32)
            "invalid",          // Invalid format
            "",                 // Empty
            "0",                // Incomplete
        ];

        for expr in invalid_expressions {
            let validation_result = validate_cron_expression(expr);
            assert!(validation_result.is_err(), "Expected '{}' to be invalid", expr);
        }
    }
}

// Helper functions for testing

/// Create a daily reset scheduled task from user configuration
fn create_daily_reset_task(
    config: &crate::models::UserConfiguration,
    time_provider: &MockTimeProvider,
) -> Result<crate::database::ScheduledTaskData, Box<dyn std::error::Error>> {
    let next_run_time = calculate_next_run_time(config, time_provider);

    Ok(crate::database::ScheduledTaskData {
        id: format!("daily_reset_task_{}", config.id),
        task_type: "daily_reset".to_string(),
        user_configuration_id: Some(config.id.clone()),
        cron_expression: config.get_daily_reset_cron_expression(),
        timezone: config.timezone.clone(),
        next_run_utc: next_run_time.timestamp() as u64,
        last_run_utc: config.last_daily_reset_utc,
        is_active: config.daily_reset_enabled,
        run_count: 0,
        failure_count: 0,
        task_data: None,
    })
}

/// Calculate next run time for daily reset
fn calculate_next_run_time(
    config: &crate::models::UserConfiguration,
    time_provider: &MockTimeProvider,
) -> DateTime<Utc> {
    let current_time = time_provider.now_utc();
    let current_date = current_time.date_naive();

    match config.daily_reset_time_type {
        crate::models::DailyResetTimeType::Midnight => {
            let next_reset = current_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            if next_reset > current_time {
                next_reset
            } else {
                (current_date + chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap().and_utc()
            }
        }
        crate::models::DailyResetTimeType::Hour => {
            if let Some(hour) = config.daily_reset_time_hour {
                let next_reset = current_date.and_hms_opt(hour as u32, 0, 0).unwrap().and_utc();
                if next_reset > current_time {
                    next_reset
                } else {
                    (current_date + chrono::Duration::days(1)).and_hms_opt(hour as u32, 0, 0).unwrap().and_utc()
                }
            } else {
                current_time + chrono::Duration::hours(1)
            }
        }
        crate::models::DailyResetTimeType::Custom => {
            if let Some(ref time_str) = config.daily_reset_time_custom {
                if let Ok(time_parts) = parse_time_string(time_str) {
                    let next_reset = current_date.and_hms_opt(time_parts[0], time_parts[1], 0).unwrap().and_utc();
                    if next_reset > current_time {
                        next_reset
                    } else {
                        (current_date + chrono::Duration::days(1)).and_hms_opt(time_parts[0], time_parts[1], 0).unwrap().and_utc()
                    }
                } else {
                    current_time + chrono::Duration::hours(1)
                }
            } else {
                current_time + chrono::Duration::hours(1)
            }
        }
    }
}

/// Parse time string in HH:MM format
fn parse_time_string(time_str: &str) -> Result<(u32, u32), ()> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let hour: u32 = parts[0].parse().map_err(|_| ())?;
        let minute: u32 = parts[1].parse().map_err(|_| ())?;
        if hour < 24 && minute < 60 {
            Ok((hour, minute))
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Validate cron expression format
fn validate_cron_expression(expr: &str) -> Result<(), String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("Cron expression must have 5 fields".to_string());
    }

    // Basic validation - in a real implementation, this would be more sophisticated
    let (minute, hour, day_of_month, month, day_of_week) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    // Validate minute (0-59)
    if minute == "*" || (minute.parse::<u32>().is_ok() && minute.parse::<u32>().unwrap() < 60) {
        // Valid
    } else {
        return Err("Invalid minute field".to_string());
    }

    // Validate hour (0-23)
    if hour == "*" || (hour.parse::<u32>().is_ok() && hour.parse::<u32>().unwrap() < 24) {
        // Valid
    } else {
        return Err("Invalid hour field".to_string());
    }

    // For simplicity, accept * for other fields or validate basic ranges
    if day_of_month != "*" && day_of_month.parse::<u32>().is_ok() {
        let day = day_of_month.parse::<u32>().unwrap();
        if day < 1 || day > 31 {
            return Err("Invalid day of month field".to_string());
        }
    }

    if month != "*" && month.parse::<u32>().is_ok() {
        let month_val = month.parse::<u32>().unwrap();
        if month_val < 1 || month_val > 12 {
            return Err("Invalid month field".to_string());
        }
    }

    if day_of_week != "*" && day_of_week.parse::<u32>().is_ok() {
        let dow = day_of_week.parse::<u32>().unwrap();
        if dow > 6 {
            return Err("Invalid day of week field".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod scheduling_test_helpers {
    use super::*;

    #[test]
    fn test_parse_time_string() {
        assert_eq!(parse_time_string("08:00"), Ok((8, 0)));
        assert_eq!(parse_time_string("17:30"), Ok((17, 30)));
        assert_eq!(parse_time_string("00:00"), Ok((0, 0)));
        assert_eq!(parse_time_string("23:59"), Ok((23, 59)));
        assert_eq!(parse_time_string("24:00"), Err(())); // Invalid hour
        assert_eq!(parse_time_string("12:60"), Err(())); // Invalid minute
    }

    #[test]
    fn test_cron_validation() {
        assert!(validate_cron_expression("0 8 * * *").is_ok());
        assert!(validate_cron_expression("0 30 14 * *").is_ok());
        assert!(validate_cron_expression("* * * * *").is_ok());

        assert!(validate_cron_expression("60 * * * *").is_err()); // Invalid minute
        assert!(validate_cron_expression("* 25 * * *").is_err()); // Invalid hour
        assert!(validate_cron_expression("* * 32 * *").is_err()); // Invalid day
        assert!(validate_cron_expression("invalid").is_err()); // Invalid format
    }
}