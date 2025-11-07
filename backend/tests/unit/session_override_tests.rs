//! Unit Tests for Session Override Behavior During Reset
//!
//! Test suite for ensuring manual session count overrides behave correctly
//! during daily resets and maintain proper priority for User Story 2.

use backend::services::daily_reset_service::{DailyResetService, SessionCountValidationError, DailyResetStatus};
use backend::models::user_configuration::{UserConfiguration, DailyResetTimeType};
use backend::models::session_reset_event::{SessionResetEvent, SessionResetTriggerSource};
use backend::services::time_provider::{MockTimeProvider, TimeProvider};
use backend::services::timezone_service::{TimezoneService, MockTimezoneService};
use backend::database::manager::DatabaseManager;
use chrono::{DateTime, Utc, TimeZone, NaiveDate};
use std::sync::Arc;
use tempfile::TempDir;

/// Test that manual overrides are preserved until next automated reset
#[tokio::test]
async fn test_manual_override_persistence_until_reset() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_override_persistence.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    // Set up mock time - 2 hours before reset
    let base_time = DateTime::parse_from_rfc3339("2025-01-15T22:00:00Z")
        .expect("Failed to parse base time")
        .with_timezone(&Utc);
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_override_persistence";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Start with base session count
    let initial_session_count = 5;
    daily_reset_service.set_session_count(user_id, initial_session_count, false).await
        .expect("Failed to set initial session count");

    // Set manual override
    let manual_override_count = 12;
    daily_reset_service.set_session_count(user_id, manual_override_count, true).await
        .expect("Failed to set manual override");

    // Verify manual override is active
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should be active");
    assert_eq!(status.current_session_count, manual_override_count,
               "Current session count should reflect manual override");

    // Move time forward but not to reset yet (30 minutes later)
    let later_time = base_time + chrono::Duration::minutes(30);
    mock_time.set_time(later_time);

    // Verify manual override is still active before reset
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should persist before automated reset");
    assert_eq!(status.current_session_count, manual_override_count,
               "Current session count should still reflect manual override");
}

/// Test that manual overrides are cleared during automated reset
#[tokio::test]
async fn test_manual_override_cleared_during_automated_reset() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_override_clear.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    // Set up mock time - just before midnight reset
    let base_time = DateTime::parse_from_rfc3339("2025-01-15T23:59:30Z")
        .expect("Failed to parse base time")
        .with_timezone(&Utc);
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_override_clear";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set manual override
    let manual_override_count = 8;
    daily_reset_service.set_session_count(user_id, manual_override_count, true).await
        .expect("Failed to set manual override");

    // Verify manual override is active
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should be active before reset");

    // Move time past reset time (next day)
    let reset_time = DateTime::parse_from_rfc3339("2025-01-16T00:00:30Z")
        .expect("Failed to parse reset time")
        .with_timezone(&Utc);
    mock_time.set_time(reset_time);

    // Trigger daily reset processing
    daily_reset_service.process_daily_resets().await
        .expect("Failed to process daily resets");

    // Verify manual override is cleared after automated reset
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status after reset");
    assert_eq!(status.manual_session_override, None,
               "Manual override should be cleared after automated reset");
    assert_eq!(status.current_session_count, 0,
               "Session count should be reset to 0");
}

/// Test that manual overrides have priority over automated counting
#[tokio::test]
async fn test_manual_override_priority_over_automated_counting() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_override_priority.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_override_priority";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Hour, // Reset at 7 AM
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set initial session count through normal means
    let initial_count = 3;
    daily_reset_service.set_session_count(user_id, initial_count, false).await
        .expect("Failed to set initial session count");

    // Set manual override higher than initial count
    let manual_override_count = 15;
    daily_reset_service.set_session_count(user_id, manual_override_count, true).await
        .expect("Failed to set manual override");

    // Verify manual override takes priority
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should be active");
    assert_eq!(status.current_session_count, manual_override_count,
               "Current session count should reflect manual override, not initial count");

    // Simulate additional automated session counting (would normally increment)
    // This should be ignored when manual override is active
    let automated_increment_result = daily_reset_service.increment_session_count(user_id).await;

    // The increment should be ignored due to manual override
    match automated_increment_result {
        Ok(_) => panic!("Increment should fail when manual override is active"),
        Err(_) => {
            // Expected - manual override should block automated increments
        }
    }

    // Verify manual override is still active and unchanged
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status after increment attempt");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should remain active after increment attempt");
    assert_eq!(status.current_session_count, manual_override_count,
               "Session count should remain unchanged by manual override");
}

/// Test that manual override can be manually cleared
#[tokio::test]
async fn test_manual_override_can_be_manually_cleared() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_override_manual_clear.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_override_manual_clear";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set manual override
    let manual_override_count = 20;
    daily_reset_service.set_session_count(user_id, manual_override_count, true).await
        .expect("Failed to set manual override");

    // Verify manual override is active
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should be active");

    // Clear manual override by setting session count without override flag
    let new_normal_count = 7;
    daily_reset_service.set_session_count(user_id, new_normal_count, false).await
        .expect("Failed to clear manual override");

    // Verify manual override is cleared
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status after clearing override");
    assert_eq!(status.manual_session_override, None,
               "Manual override should be cleared");
    assert_eq!(status.current_session_count, new_normal_count,
               "Session count should reflect new normal count");
}

/// Test manual override behavior with timezone changes
#[tokio::test]
async fn test_manual_override_with_timezone_changes() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_override_timezone.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = DateTime::parse_from_rfc3339("2025-01-15T20:00:00Z")
        .expect("Failed to parse base time")
        .with_timezone(&Utc);
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_override_timezone";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Midnight,
        timezone: "America/New_York".to_string(), // EST (UTC-5)
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set manual override
    let manual_override_count = 11;
    daily_reset_service.set_session_count(user_id, manual_override_count, true).await
        .expect("Failed to set manual override");

    // Verify manual override is active
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should be active");

    // Change timezone to Pacific (UTC-8)
    user_config.timezone = "America/Los_Angeles".to_string();
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to update user timezone");

    // Verify manual override persists through timezone change
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status after timezone change");
    assert_eq!(status.manual_session_override, Some(manual_override_count),
               "Manual override should persist through timezone change");

    // Move time to reset in New York but not in Los Angeles
    let ny_reset_time = DateTime::parse_from_rfc3339("2025-01-16T05:00:00Z") // Midnight EST = 5 AM UTC
        .expect("Failed to parse NY reset time")
        .with_timezone(&Utc);
    mock_time.set_time(ny_reset_time);

    // Trigger daily reset processing
    daily_reset_service.process_daily_resets().await
        .expect("Failed to process daily resets");

    // Verify manual override is cleared based on user's timezone
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status after timezone-aware reset");
    assert_eq!(status.manual_session_override, None,
               "Manual override should be cleared after timezone-aware reset");
    assert_eq!(status.current_session_count, 0,
               "Session count should be reset to 0");
}

/// Test multiple manual overrides in sequence
#[tokio::test]
async fn test_multiple_manual_overrides_sequence() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_multiple_overrides.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_multiple_overrides";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTimeType::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set initial session count
    daily_reset_service.set_session_count(user_id, 3, false).await
        .expect("Failed to set initial session count");

    // Apply multiple manual overrides in sequence
    let override_sequence = vec![5, 12, 8, 15, 1];

    for (index, override_count) in override_sequence.iter().enumerate() {
        // Set manual override
        daily_reset_service.set_session_count(user_id, *override_count, true).await
            .expect("Failed to set manual override");

        // Verify override is active
        let status = daily_reset_service.get_daily_reset_status(user_id).await
            .expect("Failed to get daily reset status");
        assert_eq!(status.manual_session_override, Some(*override_count),
                   "Manual override {} should be active at step {}", override_count, index + 1);
        assert_eq!(status.current_session_count, *override_count,
                   "Session count should reflect override {} at step {}", override_count, index + 1);
    }

    // Final override should be the active one
    let final_status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get final daily reset status");
    assert_eq!(final_status.manual_session_override, Some(1),
               "Final manual override should be 1");
    assert_eq!(final_status.current_session_count, 1,
               "Final session count should be 1");
}