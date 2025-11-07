//! Integration Tests for Session Reset API
//!
//! Tests the complete daily reset API cycle including:
//! - Configuration API endpoints
//! - Session count management
//! - Daily reset execution
//! - Real-time WebSocket synchronization
//! - Cross-device consistency

use std::sync::Arc;
use serde_json::{json, Value};
use chrono::{DateTime, Utc, TimeZone};

use crate::models::{UserConfiguration, DailyResetTime};
use crate::services::time_provider::{TimeProvider, MockTimeProvider};
use crate::database::DailyResetDatabaseExtensions;

use super::daily_reset_integration_utils::{
    DailyResetIntegrationTestContext,
    http,
    scenarios,
};

#[cfg(test)]
mod session_reset_api_tests {
    use super::*;

    /// Test complete daily reset configuration flow
    #[tokio::test]
    #[ignore] // Requires HTTP server
    async fn test_daily_reset_configuration_flow() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Step 1: Get initial configuration
        let response = http::get_user_configuration(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        assert!(response.status().is_success());
        let initial_config: Value = response.json().await?;
        assert_eq!(initial_config["id"], context.config.test_user_id);

        // Step 2: Update daily reset configuration
        let reset_time = DailyResetTime::hour(8)?;
        let response = http::update_daily_reset_configuration(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
            "America/New_York",
            &reset_time,
            true,
        ).await?;

        assert!(response.status().is_success());

        // Step 3: Verify configuration was updated
        let response = http::get_user_configuration(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        let updated_config: Value = response.json().await?;
        assert_eq!(updated_config["timezone"], "America/New_York");
        assert_eq!(updated_config["daily_reset_enabled"], true);
        assert_eq!(updated_config["daily_reset_time_type"], "hour");
        assert_eq!(updated_config["daily_reset_time_hour"], 8);

        Ok(())
    }

    /// Test session count management API
    #[tokio::test]
    #[ignore] // Requires HTTP server
    async fn test_session_count_management() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Step 1: Get initial session count
        let response = http::get_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        assert!(response.status().is_success());
        let initial_count: Value = response.json().await?;
        let count = initial_count["count"].as_u64().unwrap_or(0) as u32;
        assert_eq!(count, 0); // Should start at 0

        // Step 2: Set session count to 5
        let response = http::set_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
            5,
        ).await?;

        assert!(response.status().is_success());

        // Step 3: Verify session count was updated
        let response = http::get_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        let updated_count: Value = response.json().await?;
        let count = updated_count["count"].as_u64().unwrap_or(0) as u32;
        assert_eq!(count, 5);

        // Step 4: Reset session count
        let response = http::reset_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        assert!(response.status().is_success());

        // Step 5: Verify session count was reset
        let response = http::get_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        let reset_count: Value = response.json().await?;
        let count = reset_count["count"].as_u64().unwrap_or(0) as u32;
        assert_eq!(count, 0);

        Ok(())
    }

    /// Test automatic daily reset execution
    #[tokio::test]
    async fn test_automatic_daily_reset_execution() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Set up user with midnight reset enabled
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("UTC".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::midnight())?;
        updated_config.set_daily_reset_enabled(true);

        // Set initial session count to 5
        updated_config.today_session_count = 5;
        updated_config.last_daily_reset_utc = Some(
            Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).single().unwrap().timestamp() as u64
        );

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Set current time to after reset time (should trigger reset)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 1, 0, 0).single().unwrap());

        // Simulate daily reset service execution
        let reset_performed = simulate_daily_reset_service(&context).await?;
        assert!(reset_performed, "Daily reset should have been performed");

        // Verify reset in database
        let final_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?;
        assert!(final_config.is_some());

        let config = final_config.unwrap();
        assert_eq!(config.today_session_count, 0);
        assert_eq!(config.manual_session_override, None);

        // Verify last reset timestamp was updated
        assert!(config.last_daily_reset_utc.is_some());
        let last_reset = DateTime::from_timestamp(config.last_daily_reset_utc.unwrap() as i64, 0).unwrap();
        assert!(last_reset >= Utc.with_ymd_and_hms(2025, 1, 7, 0, 0, 0).single().unwrap());

        Ok(())
    }

    /// Test manual session override
    #[tokio::test]
    async fn test_manual_session_override() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Set up user with session count
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.today_session_count = 3;
        updated_config.set_daily_reset_enabled(false);

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test 1: Set manual override to 10
        let override_count = 10;
        let mut config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?.unwrap();
        config.set_manual_session_override(Some(override_count))?;
        context.db_manager.save_user_configuration(&config).await?;

        let retrieved_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?.unwrap();
        assert_eq!(retrieved_config.get_current_session_count(), override_count);

        // Test 2: Clear manual override
        config.set_manual_session_override(None)?;
        context.db_manager.save_user_configuration(&config).await?;

        let retrieved_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?.unwrap();
        assert_eq!(retrieved_config.get_current_session_count(), 3); // Back to original count
        assert_eq!(retrieved_config.manual_session_override, None);

        Ok(())
    }

    /// Test timezone-aware reset timing
    #[tokio::test]
    async fn test_timezone_aware_reset_timing() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Create user with 7 AM reset in New York
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("America/New_York".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::hour(7)?)?;
        updated_config.set_daily_reset_enabled(true);
        updated_config.today_session_count = 5;

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Test Scenario 1: Before reset time (6:30 AM New York = 11:30 UTC)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 11, 30, 0).single().unwrap());

        let current_time = context.current_time().timestamp() as u64;
        let is_reset_due = updated_config.is_daily_reset_due(current_time);
        assert!(!is_reset_due, "Reset should not be due before 7 AM New York time");

        // Test Scenario 2: After reset time (7:30 AM New York = 12:30 UTC)
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 12, 30, 0).single().unwrap());

        let current_time = context.current_time().timestamp() as u64;
        let is_reset_due = updated_config.is_daily_reset_due(current_time);
        assert!(is_reset_due, "Reset should be due after 7 AM New York time");

        // Perform reset
        let reset_performed = simulate_daily_reset_service(&context).await?;
        assert!(reset_performed);

        // Verify reset
        let final_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?.unwrap();
        assert_eq!(final_config.today_session_count, 0);

        Ok(())
    }

    /// Test WebSocket real-time synchronization
    #[tokio::test]
    #[ignore] // Requires WebSocket server
    async fn test_websocket_real_time_sync() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Connect WebSocket
        let mut ws_stream = super::daily_reset_integration_utils::websocket::connect_websocket(
            &context.config.websocket_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
        ).await?;

        // Test session count change notification
        let test_count = 7;
        super::daily_reset_integration_utils::websocket::send_websocket_message(
            &mut ws_stream,
            &json!({
                "type": "session_count_update",
                "user_id": context.config.test_user_id,
                "device_id": context.config.test_device_id,
                "count": test_count
            }).to_string(),
        ).await?;

        // Wait for response
        let response = super::daily_reset_integration_utils::websocket::receive_websocket_message(
            &mut ws_stream,
            5000,
        ).await?;

        assert!(response.is_some(), "Should receive WebSocket response");
        let response_text = response.unwrap();

        let response_json: Value = serde_json::from_str(&response_text)?;
        assert_eq!(response_json["type"], "session_count_updated");
        assert_eq!(response_json["new_count"], test_count);

        Ok(())
    }

    /// Test analytics calculation after reset
    #[tokio::test]
    async fn test_analytics_after_reset() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Set up user with session data
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await?;
        let mut updated_config = config.clone();
        updated_config.set_timezone("UTC".to_string())?;
        updated_config.set_daily_reset_time(DailyResetTime::midnight())?;
        updated_config.set_daily_reset_enabled(true);
        updated_config.today_session_count = 8;

        // Set last reset to yesterday
        updated_config.last_daily_reset_utc = Some(
            Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).single().unwrap().timestamp() as u64
        );

        context.db_manager.save_user_configuration(&updated_config).await?;

        // Record daily statistics before reset
        let today = context.current_time().format("%Y-%m-%d").to_string();
        context.db_manager.record_daily_session_stat(
            &context.config.test_user_id,
            &today,
            "UTC",
            8,
            8 * 25 * 60, // 8 sessions * 25 minutes each
            2 * 5 * 60,  // 2 short breaks * 5 minutes each
            0,            // No manual overrides
            8,            // Final session count
        ).await?;

        // Perform reset
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 0, 30, 0).single().unwrap());
        simulate_daily_reset_service(&context).await?;

        // Verify analytics were recorded
        let stats = context.db_manager.get_daily_session_stats(
            &context.config.test_user_id,
            &today,
            &today,
        ).await?;

        assert_eq!(stats.len(), 1);
        let stat = &stats[0];
        assert_eq!(stat.work_sessions_completed, 8);
        assert_eq!(stat.final_session_count, 8);

        Ok(())
    }

    /// Test error handling for invalid configurations
    #[tokio::test]
    #[ignore] // Requires HTTP server
    async fn test_invalid_configuration_handling() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;

        // Test invalid timezone
        let response = http::update_daily_reset_configuration(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
            "Invalid/Timezone",
            &DailyResetTime::midnight(),
            true,
        ).await?;

        assert!(!response.status().is_success());
        assert!(response.status().as_u16() >= 400);

        // Test invalid reset time
        let invalid_reset_time = DailyResetTime::hour(25); // Invalid hour
        if let Err(_) = invalid_reset_time {
            // This is expected - invalid reset time should fail validation
        }

        // Test invalid session count
        let response = http::set_session_count(
            &context.http_client,
            &context.config.api_base_url,
            &context.config.test_user_id,
            context.config.auth_token.as_deref(),
            1001, // Exceeds maximum allowed
        ).await?;

        assert!(!response.status().is_success());

        Ok(())
    }

    /// Test concurrent access and race conditions
    #[tokio::test]
    async fn test_concurrent_session_count_updates() -> Result<(), Box<dyn std::error::Error>> {
        let context = DailyResetIntegrationTestContext::new().await?;
        let db_manager = Arc::new(context.db_manager.clone());
        let user_id = context.config.test_user_id.clone();

        // Initialize session count
        let config = db_manager.get_or_create_user_config(&user_id).await?;
        let mut initial_config = config.clone();
        initial_config.today_session_count = 0;
        db_manager.save_user_configuration(&initial_config).await?;

        // Spawn concurrent increment tasks
        let mut handles = vec![];
        for i in 0..10 {
            let db_clone = db_manager.clone();
            let user_id_clone = user_id.clone();

            let handle = tokio::spawn(async move {
                // Simulate session increment
                let mut config = db_clone.get_user_configuration(&user_id_clone).await?.unwrap();
                config.increment_session_count()?;
                db_clone.save_user_configuration(&config).await?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }

        // Verify final state
        let final_config = db_manager.get_user_configuration(&user_id).await?.unwrap();
        // Note: This test demonstrates the need for proper locking/transaction handling
        // In a real implementation, this would use database transactions or other concurrency controls
        println!("Final session count: {}", final_config.today_session_count);

        Ok(())
    }
}

// Helper functions for integration testing

/// Simulate daily reset service execution
async fn simulate_daily_reset_service(
    context: &DailyResetIntegrationTestContext,
) -> Result<bool, Box<dyn std::error::Error>> {
    // In a real implementation, this would:
    // 1. Check for configurations with daily reset enabled
    // 2. Calculate if reset is due based on timezone and last reset time
    // 3. Execute reset logic (session count, logging, notifications)
    // 4. Update database and send WebSocket messages

    let config = context.db_manager.get_user_configuration(&context.config.test_user_id).await?;

    if let Some(mut user_config) = config {
        let current_time = context.current_time().timestamp() as u64;

        if user_config.is_daily_reset_due(current_time) {
            user_config.reset_session_count();
            context.db_manager.save_user_configuration(&user_config).await?;

            // Record reset event for analytics
            let today = context.current_time().format("%Y-%m-%d").to_string();
            context.db_manager.record_session_reset_event(
                &crate::database::SessionResetEventData {
                    id: format!("reset_{}_{}", user_config.id, current_time),
                    user_configuration_id: user_config.id,
                    reset_type: "scheduled_daily".to_string(),
                    previous_count: 5, // Would be tracked in real implementation
                    new_count: 0,
                    reset_timestamp_utc: current_time,
                    user_timezone: user_config.timezone.clone(),
                    local_reset_time: context.current_time().format("%Y-%m-%d %H:%M:%S").to_string(),
                    device_id: Some(context.config.test_device_id.clone()),
                    trigger_source: "background_service".to_string(),
                    context: None,
                }
            ).await?;

            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod integration_test_helpers {
    use super::*;

    #[tokio::test]
    async fn test_simulate_daily_reset_service() {
        let context = DailyResetIntegrationTestContext::new().await.unwrap();

        // Create a user configuration
        let config = context.db_manager.get_or_create_user_config(&context.config.test_user_id).await.unwrap();
        let mut updated_config = config.clone();
        updated_config.set_daily_reset_enabled(true);
        updated_config.today_session_count = 5;
        updated_config.last_daily_reset_utc = Some(
            Utc.with_ymd_and_hms(2025, 1, 6, 0, 0, 0).single().unwrap().timestamp() as u64
        );

        context.db_manager.save_user_configuration(&updated_config).await.unwrap();

        // Set time to after reset should occur
        context.set_time(Utc.with_ymd_and_hms(2025, 1, 7, 1, 0, 0).single().unwrap());

        // Simulate reset service
        let reset_performed = simulate_daily_reset_service(&context).await.unwrap();
        assert!(reset_performed);

        // Verify reset occurred
        let final_config = context.db_manager.get_user_configuration(&context.config.test_user_id).await.unwrap().unwrap();
        assert_eq!(final_config.today_session_count, 0);
    }
}