//! Integration Tests for Manual Session Count WebSocket Procedures
//!
//! Test suite for WebSocket-based session count management including
//! manual overrides, validation, and real-time updates for User Story 2.

use backend::websocket::server::{WebSocketServer, WebSocketMessage};
use backend::websocket::handlers::session_count::SessionCountWebSocketHandler;
use backend::websocket::messages::*;
use backend::services::daily_reset_service::DailyResetService;
use backend::services::time_provider::{MockTimeProvider, TimeProvider};
use backend::services::timezone_service::TimezoneService;
use backend::database::manager::DatabaseManager;
use backend::models::user_configuration::{UserConfiguration, DailyResetTime};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json;

/// Test get_session_count WebSocket procedure
#[tokio::test]
async fn test_get_session_count_websocket_procedure() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_get_session_count.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_get_session_count";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTime::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set initial session count
    daily_reset_service.set_session_count(user_id, 5, false).await
        .expect("Failed to set initial session count");

    // Create WebSocket message for getting session count
    let get_request = GetSessionCountRequest {
        type_: "get_session_count".to_string(),
        message_id: "test_get_session_count_001".to_string(),
        user_id: user_id.to_string(),
        timestamp: base_time.to_rfc3339(),
    };

    // Simulate WebSocket message handling
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Convert request to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "get_session_count".to_string(),
        message_id: "test_get_session_count_001".to_string(),
        data: serde_json::to_value(&get_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    session_count_handler.handle_get_session_count(&ws_message, response_tx.clone()).await
        .expect("Failed to handle get_session_count message");

    // Receive response
    if let Some(response) = response_rx.recv().await {
        // Verify response structure
        assert_eq!(response.type_, "session_count_response");
        assert_eq!(response.message_id, "test_get_session_count_001");

        // Parse response data
        let response_data: SessionCountResponse = serde_json::from_value(response.data)
            .expect("Failed to parse session count response");

        // Verify response content
        assert_eq!(response_data.type_, "session_count_response");
        assert_eq!(response_data.message_id, "test_get_session_count_001");
        assert!(response_data.success);
        assert_eq!(response_data.current_session_count, 5);
        assert_eq!(response_data.manual_session_override, None);
        assert!(response_data.error.is_none());
    } else {
        panic!("No response received for get_session_count request");
    }
}

/// Test set_session_count WebSocket procedure with manual override
#[tokio::test]
async fn test_set_session_count_websocket_procedure_with_override() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_set_session_count.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_set_session_count";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTime::Midnight,
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

    // Create WebSocket message for setting session count with manual override
    let set_request = SetSessionCountRequest {
        type_: "set_session_count".to_string(),
        message_id: "test_set_session_count_001".to_string(),
        user_id: user_id.to_string(),
        session_count: 12,
        manual_override: true,
        timestamp: base_time.to_rfc3339(),
    };

    // Simulate WebSocket message handling
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Convert request to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "set_session_count".to_string(),
        message_id: "test_set_session_count_001".to_string(),
        data: serde_json::to_value(&set_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    session_count_handler.handle_set_session_count(&ws_message, response_tx.clone()).await
        .expect("Failed to handle set_session_count message");

    // Receive response
    if let Some(response) = response_rx.recv().await {
        // Verify response structure
        assert_eq!(response.type_, "session_set_response");
        assert_eq!(response.message_id, "test_set_session_count_001");

        // Parse response data
        let response_data: SessionSetResponse = serde_json::from_value(response.data)
            .expect("Failed to parse session set response");

        // Verify response content
        assert_eq!(response_data.type_, "session_set_response");
        assert_eq!(response_data.message_id, "test_set_session_count_001");
        assert!(response_data.success);
        assert_eq!(response_data.current_session_count, 12);
        assert_eq!(response_data.manual_session_override, Some(12));
        assert!(response_data.error.is_none());
    } else {
        panic!("No response received for set_session_count request");
    }

    // Verify the session count was actually set in the service
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.current_session_count, 12);
    assert_eq!(status.manual_session_override, Some(12));
}

/// Test set_session_count WebSocket procedure with validation error
#[tokio::test]
async fn test_set_session_count_websocket_procedure_validation_error() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_set_session_count_validation.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_set_session_count_validation";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTime::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Create WebSocket message with invalid session count (negative)
    let set_request = SetSessionCountRequest {
        type_: "set_session_count".to_string(),
        message_id: "test_set_session_count_invalid".to_string(),
        user_id: user_id.to_string(),
        session_count: -5, // Invalid negative value
        manual_override: false,
        timestamp: base_time.to_rfc3339(),
    };

    // Simulate WebSocket message handling
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Convert request to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "set_session_count".to_string(),
        message_id: "test_set_session_count_invalid".to_string(),
        data: serde_json::to_value(&set_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    session_count_handler.handle_set_session_count(&ws_message, response_tx.clone()).await
        .expect("Failed to handle set_session_count message");

    // Receive response
    if let Some(response) = response_rx.recv().await {
        // Verify response structure
        assert_eq!(response.type_, "session_set_response");
        assert_eq!(response.message_id, "test_set_session_count_invalid");

        // Parse response data
        let response_data: SessionSetResponse = serde_json::from_value(response.data)
            .expect("Failed to parse session set response");

        // Verify response indicates error
        assert_eq!(response_data.type_, "session_set_response");
        assert_eq!(response_data.message_id, "test_set_session_count_invalid");
        assert!(!response_data.success);
        assert!(response_data.error.is_some());

        // Verify error message contains relevant information
        let error_message = response_data.error.unwrap();
        assert!(error_message.contains("invalid") || error_message.contains("negative"));
        assert!(error_message.contains("session count"));
    } else {
        panic!("No response received for invalid set_session_count request");
    }
}

/// Test reset_session WebSocket procedure
#[tokio::test]
async fn test_reset_session_websocket_procedure() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_reset_session.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_reset_session";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTime::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Set initial session count with manual override
    daily_reset_service.set_session_count(user_id, 8, true).await
        .expect("Failed to set initial session count");

    // Create WebSocket message for resetting session
    let reset_request = ResetSessionRequest {
        type_: "reset_session".to_string(),
        message_id: "test_reset_session_001".to_string(),
        user_id: user_id.to_string(),
        timestamp: base_time.to_rfc3339(),
    };

    // Simulate WebSocket message handling
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Convert request to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "reset_session".to_string(),
        message_id: "test_reset_session_001".to_string(),
        data: serde_json::to_value(&reset_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    session_count_handler.handle_reset_session(&ws_message, response_tx.clone()).await
        .expect("Failed to handle reset_session message");

    // Receive response
    if let Some(response) = response_rx.recv().await {
        // Verify response structure
        assert_eq!(response.type_, "session_reset_response");
        assert_eq!(response.message_id, "test_reset_session_001");

        // Parse response data
        let response_data: SessionResetResponse = serde_json::from_value(response.data)
            .expect("Failed to parse session reset response");

        // Verify response content
        assert_eq!(response_data.type_, "session_reset_response");
        assert_eq!(response_data.message_id, "test_reset_session_001");
        assert!(response_data.success);
        assert_eq!(response_data.previous_session_count, 8);
        assert_eq!(response_data.new_session_count, 0);
        assert!(response_data.reset_time_utc > 0);
        assert!(response_data.error.is_none());
    } else {
        panic!("No response received for reset_session request");
    }

    // Verify the session was actually reset and manual override cleared
    let status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get daily reset status");
    assert_eq!(status.current_session_count, 0);
    assert_eq!(status.manual_session_override, None);
}

/// Test session count WebSocket procedure with non-existent user
#[tokio::test]
async fn test_session_count_websocket_procedure_nonexistent_user() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_nonexistent_user.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Use non-existent user ID
    let non_existent_user_id = "non_existent_user";

    // Create WebSocket message for getting session count
    let get_request = GetSessionCountRequest {
        type_: "get_session_count".to_string(),
        message_id: "test_non_existent_user".to_string(),
        user_id: non_existent_user_id.to_string(),
        timestamp: base_time.to_rfc3339(),
    };

    // Simulate WebSocket message handling
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Convert request to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "get_session_count".to_string(),
        message_id: "test_non_existent_user".to_string(),
        data: serde_json::to_value(&get_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    session_count_handler.handle_get_session_count(&ws_message, response_tx.clone()).await
        .expect("Failed to handle get_session_count message");

    // Receive response
    if let Some(response) = response_rx.recv().await {
        // Verify response structure
        assert_eq!(response.type_, "session_count_response");
        assert_eq!(response.message_id, "test_non_existent_user");

        // Parse response data
        let response_data: SessionCountResponse = serde_json::from_value(response.data)
            .expect("Failed to parse session count response");

        // Verify response indicates error for non-existent user
        assert_eq!(response_data.type_, "session_count_response");
        assert_eq!(response_data.message_id, "test_non_existent_user");
        assert!(!response_data.success);
        assert!(response_data.error.is_some());

        // Verify error message contains relevant information
        let error_message = response_data.error.unwrap();
        assert!(error_message.contains("user") && (error_message.contains("not found") || error_message.contains("exist")));
        assert_eq!(response_data.current_session_count, 0); // Default value
        assert_eq!(response_data.manual_session_override, None); // Default value
    } else {
        panic!("No response received for non-existent user request");
    }
}

/// Test session count WebSocket procedure message correlation
#[tokio::test]
async fn test_session_count_websocket_message_correlation() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_message_correlation.db");
    let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
        .expect("Failed to create DatabaseManager"));

    let base_time = Utc::now();
    let mock_time = Arc::new(MockTimeProvider::new());
    mock_time.set_time(base_time);

    let timezone_service = Arc::new(TimezoneService::new(mock_time.clone()));
    let daily_reset_service = Arc::new(DailyResetService::new(
        db_manager.clone(),
        timezone_service.clone(),
        mock_time.clone()
    ));

    // Create WebSocket handler
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create test user configuration
    let user_id = "test_user_message_correlation";
    let user_config = UserConfiguration {
        user_id: user_id.to_string(),
        daily_reset_enabled: true,
        daily_reset_time_type: DailyResetTime::Midnight,
        timezone: "UTC".to_string(),
        created_at: base_time,
        updated_at: base_time,
    };

    // Save user configuration
    db_manager.save_user_configuration(&user_config).await
        .expect("Failed to save user configuration");

    // Test multiple concurrent requests with different message IDs
    let message_requests = vec![
        ("msg_001", 5, true),
        ("msg_002", 10, false),
        ("msg_003", 15, true),
        ("msg_004", 2, false),
    ];

    let mut response_channels = Vec::new();
    let mut expected_responses = Vec::new();

    for (message_id, session_count, manual_override) in message_requests {
        // Create request
        let set_request = SetSessionCountRequest {
            type_: "set_session_count".to_string(),
            message_id: message_id.to_string(),
            user_id: user_id.to_string(),
            session_count,
            manual_override,
            timestamp: base_time.to_rfc3339(),
        };

        // Create response channel
        let (response_tx, response_rx) = mpsc::channel(100);
        response_channels.push((message_id.to_string(), response_rx, session_count, manual_override));

        // Convert to WebSocket message
        let ws_message = WebSocketMessage {
            type_: "set_session_count".to_string(),
            message_id: message_id.to_string(),
            data: serde_json::to_value(&set_request).expect("Failed to serialize request"),
            timestamp: base_time.to_rfc3339(),
        };

        // Handle message asynchronously
        let handler = session_count_handler.clone();
        tokio::spawn(async move {
            handler.handle_set_session_count(&ws_message, response_tx).await
                .expect("Failed to handle set_session_count message");
        });

        expected_responses.push((message_id.to_string(), session_count, manual_override));
    }

    // Collect responses and verify correlation
    for (expected_message_id, expected_session_count, expected_manual_override) in expected_responses {
        let found_response = response_channels.iter().find(|(msg_id, _, _, _)| msg_id == &expected_message_id);

        assert!(found_response.is_some(), "No response found for message ID: {}", expected_message_id);

        let (_, response_rx, _, _) = found_response.unwrap();

        if let Some(response) = response_rx.recv().await {
            assert_eq!(response.message_id, expected_message_id, "Message ID mismatch");

            let response_data: SessionSetResponse = serde_json::from_value(response.data)
                .expect("Failed to parse session set response");

            assert!(response_data.success, "Response should be successful for message: {}", expected_message_id);
            assert_eq!(response_data.current_session_count, expected_session_count, "Session count mismatch for message: {}", expected_message_id);

            if expected_manual_override {
                assert_eq!(response_data.manual_session_override, Some(expected_session_count), "Manual override should be set for message: {}", expected_message_id);
            } else {
                assert_eq!(response_data.manual_session_override, None, "Manual override should not be set for message: {}", expected_message_id);
            }
        } else {
            panic!("No response received for message ID: {}", expected_message_id);
        }
    }
}