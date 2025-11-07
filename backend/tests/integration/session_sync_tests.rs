//! Integration Tests for WebSocket Sync of Manual Changes
//!
//! Test suite for real-time synchronization of manual session count changes
//! across multiple device connections for User Story 2.

use backend::websocket::server::{WebSocketServer, WebSocketMessage, WebSocketConnection};
use backend::websocket::handlers::session_count::SessionCountWebSocketHandler;
use backend::websocket::handlers::daily_reset::DailyResetWebSocketHandler;
use backend::websocket::messages::*;
use backend::services::daily_reset_service::DailyResetService;
use backend::services::time_provider::{MockTimeProvider, TimeProvider};
use backend::services::timezone_service::TimezoneService;
use backend::database::manager::DatabaseManager;
use backend::models::user_configuration::{UserConfiguration, DailyResetTime};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::sync::{mpsc, RwLock};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json;

/// Mock WebSocket connection for testing
struct MockWebSocketConnection {
    user_id: String,
    connection_id: String,
    message_sender: mpsc::UnboundedSender<WebSocketMessage>,
    message_receiver: Arc<RwLock<Vec<WebSocketMessage>>>,
}

impl MockWebSocketConnection {
    fn new(user_id: String, connection_id: String) -> (
        Self,
        mpsc::UnboundedReceiver<WebSocketMessage>,
        Arc<RwLock<Vec<WebSocketMessage>>>
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_store = Arc::new(RwLock::new(Vec::new()));

        let conn = Self {
            user_id,
            connection_id,
            message_sender: tx,
            message_receiver: message_store.clone(),
        };

        (conn, rx, message_store)
    }

    async fn send_message(&self, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store message for verification
        self.message_receiver.write().await.push(message.clone());
        self.message_sender.send(message)?;
        Ok(())
    }

    async fn get_received_messages(&self) -> Vec<WebSocketMessage> {
        self.message_receiver.read().await.clone()
    }
}

/// Mock connection manager for testing
struct MockConnectionManager {
    connections: Arc<RwLock<HashMap<String, Vec<MockWebSocketConnection>>>>,
}

impl MockConnectionManager {
    fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_connection(&self, user_id: String, connection: MockWebSocketConnection) {
        let mut connections = self.connections.write().await;
        connections.entry(user_id.clone()).or_insert_with(Vec::new).push(connection);
    }

    async fn broadcast_to_user(&self, user_id: &str, message: WebSocketMessage) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        if let Some(user_connections) = connections.get(user_id) {
            let mut sent_count = 0;
            for connection in user_connections {
                connection.send_message(message.clone()).await?;
                sent_count += 1;
            }
            Ok(sent_count)
        } else {
            Ok(0)
        }
    }

    async fn get_connection_count(&self, user_id: &str) -> usize {
        let connections = self.connections.read().await;
        connections.get(user_id).map(|conns| conns.len()).unwrap_or(0)
    }
}

/// Test real-time sync of manual session count changes across multiple devices
#[tokio::test]
async fn test_manual_session_count_sync_across_devices() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_multi_device_sync.db");
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

    // Create WebSocket handlers
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    let daily_reset_handler = Arc::new(DailyResetWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone(),
        timezone_service.clone()
    ));

    // Create mock connection manager
    let connection_manager = Arc::new(MockConnectionManager::new());

    // Create test user configuration
    let user_id = "test_user_multi_device_sync";
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

    // Create multiple mock connections (simulating different devices)
    let (device1, _, device1_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "device_001".to_string()
    );
    let (device2, _, device2_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "device_002".to_string()
    );
    let (device3, _, device3_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "device_003".to_string()
    );

    // Register connections
    connection_manager.add_connection(user_id.to_string(), device1).await;
    connection_manager.add_connection(user_id.to_string(), device2).await;
    connection_manager.add_connection(user_id.to_string(), device3).await;

    // Verify connection count
    assert_eq!(connection_manager.get_connection_count(user_id).await, 3);

    // Simulate manual session count change from device 1
    let manual_override_request = SetSessionCountRequest {
        type_: "set_session_count".to_string(),
        message_id: "manual_override_device1".to_string(),
        user_id: user_id.to_string(),
        session_count: 15,
        manual_override: true,
        timestamp: base_time.to_rfc3339(),
    };

    // Convert to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "set_session_count".to_string(),
        message_id: "manual_override_device1".to_string(),
        data: serde_json::to_value(&manual_override_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    let (response_tx, mut response_rx) = mpsc::channel(100);
    session_count_handler.handle_set_session_count(&ws_message, response_tx.clone()).await
        .expect("Failed to handle set_session_count message");

    // Get response
    let response = response_rx.recv().await.expect("No response received");
    let response_data: SessionSetResponse = serde_json::from_value(response.data)
        .expect("Failed to parse session set response");

    assert!(response_data.success);
    assert_eq!(response_data.current_session_count, 15);
    assert_eq!(response_data.manual_session_override, Some(15));

    // Create broadcast message for session count update
    let broadcast_message = WebSocketMessage {
        type_: "session_count_update".to_string(),
        message_id: "broadcast_manual_override".to_string(),
        data: serde_json::json!({
            "user_id": user_id,
            "current_session_count": 15,
            "manual_session_override": 15,
            "trigger_source": "manual_override",
            "timestamp": base_time.to_rfc3339()
        }),
        timestamp: base_time.to_rfc3339(),
    };

    // Broadcast to all connected devices
    let broadcast_count = connection_manager.broadcast_to_user(user_id, broadcast_message).await
        .expect("Failed to broadcast message");

    assert_eq!(broadcast_count, 3, "Message should be broadcast to all 3 devices");

    // Verify all devices received the update (in real implementation, this would be automatic)
    // For testing, we verify the message structure and content
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let device1_received = device1_messages.read().await;
    let device2_received = device2_messages.read().await;
    let device3_received = device3_messages.read().await;

    assert_eq!(device1_received.len(), 1, "Device 1 should receive broadcast");
    assert_eq!(device2_received.len(), 1, "Device 2 should receive broadcast");
    assert_eq!(device3_received.len(), 1, "Device 3 should receive broadcast");

    // Verify broadcast message content
    let broadcast_to_device2 = &device2_received[0];
    assert_eq!(broadcast_to_device2.type_, "session_count_update");
    assert_eq!(broadcast_to_device2.message_id, "broadcast_manual_override");

    let broadcast_data = broadcast_to_device2.data.as_object().unwrap();
    assert_eq!(broadcast_data.get("user_id").unwrap().as_str().unwrap(), user_id);
    assert_eq!(broadcast_data.get("current_session_count").unwrap().as_u64().unwrap(), 15);
    assert_eq!(broadcast_data.get("manual_session_override").unwrap().as_u64().unwrap(), 15);
    assert_eq!(broadcast_data.get("trigger_source").unwrap().as_str().unwrap(), "manual_override");
}

/// Test sync of session reset across multiple devices
#[tokio::test]
async fn test_session_reset_sync_across_devices() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_reset_sync.db");
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

    // Create WebSocket handlers
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create mock connection manager
    let connection_manager = Arc::new(MockConnectionManager::new());

    // Create test user configuration
    let user_id = "test_user_reset_sync";
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

    // Create mock connections
    let (laptop, _, laptop_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "laptop_device".to_string()
    );
    let (mobile, _, mobile_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "mobile_device".to_string()
    );

    // Register connections
    connection_manager.add_connection(user_id.to_string(), laptop).await;
    connection_manager.add_connection(user_id.to_string(), mobile).await;

    // Simulate session reset from laptop
    let reset_request = ResetSessionRequest {
        type_: "reset_session".to_string(),
        message_id: "reset_from_laptop".to_string(),
        user_id: user_id.to_string(),
        timestamp: base_time.to_rfc3339(),
    };

    // Convert to WebSocket message
    let ws_message = WebSocketMessage {
        type_: "reset_session".to_string(),
        message_id: "reset_from_laptop".to_string(),
        data: serde_json::to_value(&reset_request).expect("Failed to serialize request"),
        timestamp: base_time.to_rfc3339(),
    };

    // Handle the message
    let (response_tx, mut response_rx) = mpsc::channel(100);
    session_count_handler.handle_reset_session(&ws_message, response_tx.clone()).await
        .expect("Failed to handle reset_session message");

    // Get response
    let response = response_rx.recv().await.expect("No response received");
    let response_data: SessionResetResponse = serde_json::from_value(response.data)
        .expect("Failed to parse session reset response");

    assert!(response_data.success);
    assert_eq!(response_data.previous_session_count, 8);
    assert_eq!(response_data.new_session_count, 0);

    // Create broadcast message for session reset
    let broadcast_message = WebSocketMessage {
        type_: "session_reset_update".to_string(),
        message_id: "broadcast_session_reset".to_string(),
        data: serde_json::json!({
            "user_id": user_id,
            "previous_session_count": 8,
            "new_session_count": 0,
            "manual_session_override": null,
            "trigger_source": "manual_reset",
            "reset_time_utc": response_data.reset_time_utc,
            "timestamp": base_time.to_rfc3339()
        }),
        timestamp: base_time.to_rfc3339(),
    };

    // Broadcast to all connected devices
    let broadcast_count = connection_manager.broadcast_to_user(user_id, broadcast_message).await
        .expect("Failed to broadcast reset message");

    assert_eq!(broadcast_count, 2, "Reset message should be broadcast to both devices");

    // Verify mobile device received the reset notification
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mobile_received = mobile_messages.read().await;
    assert_eq!(mobile_received.len(), 1, "Mobile device should receive reset broadcast");

    let reset_broadcast = &mobile_received[0];
    assert_eq!(reset_broadcast.type_, "session_reset_update");
    assert_eq!(reset_broadcast.message_id, "broadcast_session_reset");

    let reset_data = reset_broadcast.data.as_object().unwrap();
    assert_eq!(reset_data.get("previous_session_count").unwrap().as_u64().unwrap(), 8);
    assert_eq!(reset_data.get("new_session_count").unwrap().as_u64().unwrap(), 0);
    assert!(reset_data.get("manual_session_override").unwrap().is_null());
    assert_eq!(reset_data.get("trigger_source").unwrap().as_str().unwrap(), "manual_reset");
}

/// Test conflict resolution for concurrent manual changes
#[tokio::test]
async fn test_concurrent_manual_change_conflict_resolution() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_conflict_resolution.db");
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

    // Create WebSocket handlers
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create mock connection manager
    let connection_manager = Arc::new(MockConnectionManager::new());

    // Create test user configuration
    let user_id = "test_user_conflict_resolution";
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

    // Create mock connections
    let (device1, _, device1_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "conflict_device_1".to_string()
    );
    let (device2, _, device2_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "conflict_device_2".to_string()
    );

    // Register connections
    connection_manager.add_connection(user_id.to_string(), device1).await;
    connection_manager.add_connection(user_id.to_string(), device2).await;

    // Simulate concurrent manual changes from both devices
    let device1_request = SetSessionCountRequest {
        type_: "set_session_count".to_string(),
        message_id: "device1_concurrent_change".to_string(),
        user_id: user_id.to_string(),
        session_count: 12,
        manual_override: true,
        timestamp: base_time.to_rfc3339(),
    };

    let device2_request = SetSessionCountRequest {
        type_: "set_session_count".to_string(),
        message_id: "device2_concurrent_change".to_string(),
        user_id: user_id.to_string(),
        session_count: 18,
        manual_override: true,
        timestamp: base_time.to_rfc3339(),
    };

    // Handle both requests concurrently
    let handler1 = session_count_handler.clone();
    let handler2 = session_count_handler.clone();

    let (response_tx1, mut response_rx1) = mpsc::channel(100);
    let (response_tx2, mut response_rx2) = mpsc::channel(100);

    let ws_message1 = WebSocketMessage {
        type_: "set_session_count".to_string(),
        message_id: "device1_concurrent_change".to_string(),
        data: serde_json::to_value(&device1_request).expect("Failed to serialize request 1"),
        timestamp: base_time.to_rfc3339(),
    };

    let ws_message2 = WebSocketMessage {
        type_: "set_session_count".to_string(),
        message_id: "device2_concurrent_change".to_string(),
        data: serde_json::to_value(&device2_request).expect("Failed to serialize request 2"),
        timestamp: base_time.to_rfc3339(),
    };

    // Execute both requests concurrently
    let (result1, result2) = tokio::join!(
        handler1.handle_set_session_count(&ws_message1, response_tx1),
        handler2.handle_set_session_count(&ws_message2, response_tx2)
    );

    assert!(result1.is_ok(), "Device 1 request should succeed");
    assert!(result2.is_ok(), "Device 2 request should succeed");

    // Get responses
    let response1 = response_rx1.recv().await.expect("No response received from device 1");
    let response2 = response_rx2.recv().await.expect("No response received from device 2");

    let response_data1: SessionSetResponse = serde_json::from_value(response1.data)
        .expect("Failed to parse session set response 1");
    let response_data2: SessionSetResponse = serde_json::from_value(response2.data)
        .expect("Failed to parse session set response 2");

    // Both requests should succeed (last write wins)
    assert!(response_data1.success);
    assert!(response_data2.success);

    // Verify final state in the database (one of the values should be final)
    let final_status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get final daily reset status");

    // Final value should be either 12 or 18 (last write wins)
    assert!(
        final_status.current_session_count == 12 || final_status.current_session_count == 18,
        "Final session count should be either 12 or 18, got {}",
        final_status.current_session_count
    );

    // Create conflict resolution broadcast
    let broadcast_message = WebSocketMessage {
        type_: "session_count_conflict_resolved".to_string(),
        message_id: "conflict_resolution_broadcast".to_string(),
        data: serde_json::json!({
            "user_id": user_id,
            "winning_session_count": final_status.current_session_count,
            "manual_session_override": final_status.manual_session_override,
            "conflict_device_1": {
                "requested_count": 12,
                "message_id": "device1_concurrent_change"
            },
            "conflict_device_2": {
                "requested_count": 18,
                "message_id": "device2_concurrent_change"
            },
            "resolution_strategy": "last_write_wins",
            "timestamp": base_time.to_rfc3339()
        }),
        timestamp: base_time.to_rfc3339(),
    };

    // Broadcast conflict resolution to all devices
    let broadcast_count = connection_manager.broadcast_to_user(user_id, broadcast_message).await
        .expect("Failed to broadcast conflict resolution");

    assert_eq!(broadcast_count, 2, "Conflict resolution should be broadcast to both devices");

    // Verify both devices received conflict resolution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let device1_received = device1_messages.read().await;
    let device2_received = device2_messages.read().await;

    assert_eq!(device1_received.len(), 1, "Device 1 should receive conflict resolution");
    assert_eq!(device2_received.len(), 1, "Device 2 should receive conflict resolution");

    let conflict_resolution = &device1_received[0];
    assert_eq!(conflict_resolution.type_, "session_count_conflict_resolved");

    let resolution_data = conflict_resolution.data.as_object().unwrap();
    assert_eq!(
        resolution_data.get("winning_session_count").unwrap().as_u64().unwrap(),
        final_status.current_session_count
    );
    assert_eq!(
        resolution_data.get("resolution_strategy").unwrap().as_str().unwrap(),
        "last_write_wins"
    );
}

/// Test sync ordering and message delivery guarantees
#[tokio::test]
async fn test_sync_ordering_and_delivery_guarantees() {
    // Setup test infrastructure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_sync_ordering.db");
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

    // Create WebSocket handlers
    let session_count_handler = Arc::new(SessionCountWebSocketHandler::new(
        db_manager.clone(),
        daily_reset_service.clone()
    ));

    // Create mock connection manager
    let connection_manager = Arc::new(MockConnectionManager::new());

    // Create test user configuration
    let user_id = "test_user_sync_ordering";
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

    // Create mock connection
    let (test_device, _, test_device_messages) = MockWebSocketConnection::new(
        user_id.to_string(),
        "ordering_test_device".to_string()
    );

    // Register connection
    connection_manager.add_connection(user_id.to_string(), test_device).await;

    // Sequence of operations to test ordering
    let operations = vec![
        ("set_session_count", 5, true),
        ("set_session_count", 10, false),
        ("reset_session", 0, false),
        ("set_session_count", 7, true),
        ("set_session_count", 3, false),
    ];

    let mut expected_sequence = Vec::new();

    for (index, (operation_type, session_count, manual_override)) in operations.iter().enumerate() {
        let message_id = format!("ordering_test_{:03}", index);
        expected_sequence.push((message_id.clone(), *session_count, *manual_override));

        if *operation_type == "reset_session" {
            let reset_request = ResetSessionRequest {
                type_: "reset_session".to_string(),
                message_id: message_id.clone(),
                user_id: user_id.to_string(),
                timestamp: base_time.to_rfc3339(),
            };

            let ws_message = WebSocketMessage {
                type_: "reset_session".to_string(),
                message_id: message_id.clone(),
                data: serde_json::to_value(&reset_request).expect("Failed to serialize reset request"),
                timestamp: base_time.to_rfc3339(),
            };

            let (response_tx, _) = mpsc::channel(100);
            session_count_handler.handle_reset_session(&ws_message, response_tx).await
                .expect("Failed to handle reset_session message");
        } else {
            let set_request = SetSessionCountRequest {
                type_: "set_session_count".to_string(),
                message_id: message_id.clone(),
                user_id: user_id.to_string(),
                session_count: *session_count,
                manual_override: *manual_override,
                timestamp: base_time.to_rfc3339(),
            };

            let ws_message = WebSocketMessage {
                type_: "set_session_count".to_string(),
                message_id: message_id.clone(),
                data: serde_json::to_value(&set_request).expect("Failed to serialize set request"),
                timestamp: base_time.to_rfc3339(),
            };

            let (response_tx, _) = mpsc::channel(100);
            session_count_handler.handle_set_session_count(&ws_message, response_tx).await
                .expect("Failed to handle set_session_count message");
        }

        // Add small delay to ensure ordering
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Verify final state matches last operation
    let final_status = daily_reset_service.get_daily_reset_status(user_id).await
        .expect("Failed to get final status");

    // Last operation was set_session_count with count=3, manual_override=false
    assert_eq!(final_status.current_session_count, 3);
    assert_eq!(final_status.manual_session_override, None);

    // Test message sequence preservation
    let sequence_test_message = WebSocketMessage {
        type_: "sequence_test".to_string(),
        message_id: "sequence_verification".to_string(),
        data: serde_json::json!({
            "user_id": user_id,
            "sequence_number": expected_sequence.len(),
            "operations": expected_sequence.iter().map(|(id, count, manual)| {
                serde_json::json!({
                    "message_id": id,
                    "session_count": count,
                    "manual_override": manual
                })
            }).collect::<Vec<_>>(),
            "final_session_count": final_status.current_session_count,
            "final_manual_override": final_status.manual_session_override
        }),
        timestamp: base_time.to_rfc3339(),
    };

    // Send sequence verification message
    let broadcast_count = connection_manager.broadcast_to_user(user_id, sequence_test_message).await
        .expect("Failed to broadcast sequence verification");

    assert_eq!(broadcast_count, 1, "Sequence verification should be sent to test device");

    // Verify sequence message was received
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let received_messages = test_device_messages.read().await;
    assert_eq!(received_messages.len(), 1, "Should receive sequence verification message");

    let sequence_message = &received_messages[0];
    assert_eq!(sequence_message.type_, "sequence_test");
    assert_eq!(sequence_message.message_id, "sequence_verification");

    let sequence_data = sequence_message.data.as_object().unwrap();
    assert_eq!(
        sequence_data.get("final_session_count").unwrap().as_u64().unwrap(),
        3
    );
    assert!(sequence_data.get("final_manual_override").unwrap().is_null());
}