//! Unit Tests for Session Count Validation
//!
//! Test suite for validating manual session count input and ensuring
//! proper bounds checking and error handling for User Story 2.

use backend::services::daily_reset_service::{DailyResetService, SessionCountValidationError, DailyResetStatus};
use backend::models::user_configuration::{UserConfiguration, DailyResetTimeType};
use backend::services::time_provider::{MockTimeProvider, TimeProvider};
use backend::services::timezone_service::{TimezoneService, MockTimezoneService};
use backend::database::manager::DatabaseManager;
use chrono::{DateTime, Utc, TimeZone};
use std::sync::Arc;
use tempfile::TempDir;

/// Test session count validation within acceptable bounds
#[tokio::test]
async fn test_valid_session_count_ranges() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_session_count.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Test valid session count values (0-100)
    let valid_counts = vec![0, 1, 5, 10, 25, 50, 75, 99, 100];

    for count in valid_counts {
        let result = daily_reset_service.validate_session_count(count).await;
        assert!(
            result.is_ok(),
            "Session count {} should be valid, but got error: {:?}",
            count,
            result.err()
        );
    }
}

/// Test session count validation outside acceptable bounds
#[tokio::test]
async fn test_invalid_session_count_ranges() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_session_count_validation.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Test invalid session count values
    let invalid_test_cases = vec![
        (-1, "Session count cannot be negative"),
        (-10, "Session count cannot be negative"),
        (101, "Session count exceeds maximum allowed"),
        (150, "Session count exceeds maximum allowed"),
        (1000, "Session count exceeds maximum allowed"),
    ];

    for (invalid_count, expected_error_pattern) in invalid_test_cases {
        let result = daily_reset_service.validate_session_count(invalid_count).await;
        assert!(
            result.is_err(),
            "Session count {} should be invalid",
            invalid_count
        );

        let error = result.err().unwrap();
        match error {
            SessionCountValidationError::OutOfRange { min, max, value } => {
                assert_eq!(min, 0, "Minimum bound should be 0");
                assert_eq!(max, 100, "Maximum bound should be 100");
                assert_eq!(value, invalid_count, "Error should contain the invalid value");
            }
            _ => panic!("Expected OutOfRange error for count {}, got: {:?}", invalid_count, error),
        }

        // Check that error message contains expected pattern
        let error_message = format!("{}", error);
        assert!(
            error_message.contains(expected_error_pattern),
            "Error message '{}' should contain '{}'",
            error_message,
            expected_error_pattern
        );
    }
}

/// Test session count validation with edge cases
#[tokio::test]
async fn test_session_count_validation_edge_cases() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_edge_cases.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Test boundary values
    let boundary_test_cases = vec![
        (0, true),   // Minimum valid value
        (1, true),   // Just above minimum
        (99, true),  // Just below maximum
        (100, true), // Maximum valid value
        (-1, false), // Just below minimum
        (101, false),// Just above maximum
    ];

    for (count, should_be_valid) in boundary_test_cases {
        let result = daily_reset_service.validate_session_count(count).await;

        if should_be_valid {
            assert!(
                result.is_ok(),
                "Session count {} should be valid at boundary",
                count
            );
        } else {
            assert!(
                result.is_err(),
                "Session count {} should be invalid at boundary",
                count
            );
        }
    }
}

/// Test session count validation with special numeric values
#[tokio::test]
async fn test_session_count_validation_special_values() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_special_values.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Note: These tests would require the service to handle potential overflow cases
    // For now, we test reasonable large numbers that might cause issues

    // Test very large numbers that could cause overflow
    let large_numbers = vec![
        u32::MAX as i64,
        i64::MAX,
    ];

    for large_number in large_numbers {
        let result = daily_reset_service.validate_session_count(large_number).await;
        assert!(
            result.is_err(),
            "Very large session count {} should be invalid",
            large_number
        );

        let error = result.err().unwrap();
        match error {
            SessionCountValidationError::OutOfRange { min, max, .. } => {
                assert_eq!(min, 0);
                assert_eq!(max, 100);
            }
            _ => panic!("Expected OutOfRange error for large number, got: {:?}", error),
        }
    }
}

/// Test session count validation error message formatting
#[tokio::test]
async fn test_session_count_validation_error_formatting() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_error_formatting.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Test that error messages are user-friendly and contain relevant information
    let invalid_count = -5;
    let result = daily_reset_service.validate_session_count(invalid_count).await;
    assert!(result.is_err());

    let error = result.err().unwrap();
    let error_message = format!("{}", error);

    // Verify error message contains key information
    assert!(error_message.contains("session count"), "Error should mention 'session count'");
    assert!(error_message.contains("invalid"), "Error should indicate the value is invalid");
    assert!(error_message.contains(&invalid_count.to_string()), "Error should include the invalid value");
    assert!(error_message.contains("0"), "Error should mention the minimum valid value");
    assert!(error_message.contains("100"), "Error should mention the maximum valid value");

    // Test positive invalid count
    let invalid_positive_count = 150;
    let result = daily_reset_service.validate_session_count(invalid_positive_count).await;
    assert!(result.is_err());

    let error = result.err().unwrap();
    let error_message = format!("{}", error);

    assert!(error_message.contains(&invalid_positive_count.to_string()),
             "Error should include the invalid positive value");
}

/// Test concurrent session count validation
#[tokio::test]
async fn test_concurrent_session_count_validation() {
    // Setup test dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_concurrent_validation.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let mock_time = Arc::new(MockTimeProvider::new());
    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        mock_time.clone(),
        db_manager.clone()
    ));

    // Test multiple concurrent validation requests
    let validation_tasks = vec![
        daily_reset_service.validate_session_count(5),
        daily_reset_service.validate_session_count(10),
        daily_reset_service.validate_session_count(50),
        daily_reset_service.validate_session_count(-1),  // Invalid
        daily_reset_service.validate_session_count(101), // Invalid
        daily_reset_service.validate_session_count(0),
        daily_reset_service.validate_session_count(100),
    ];

    let results = futures::future::join_all(validation_tasks).await;

    // Check results
    let expected_validity = vec![true, true, true, false, false, true, true];

    for (i, result) in results.iter().enumerate() {
        if expected_validity[i] {
            assert!(
                result.is_ok(),
                "Validation at index {} should succeed",
                i
            );
        } else {
            assert!(
                result.is_err(),
                "Validation at index {} should fail",
                i
            );
        }
    }
}