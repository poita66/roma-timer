#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn test_multiple_device_connection_handling() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect multiple devices simultaneously
        let mut connections = Vec::new();
        let device_ids = vec!["device-1", "device-2", "device-3"];

        for device_id in &device_ids {
            let url_with_device = format!("{}?device_id={}", ws_url, device_id);
            let (ws_stream, _) = connect_async(&url_with_device).await.unwrap();
            connections.push((device_id.to_string(), ws_stream));
        }

        // Give connections time to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start timer from first device
        let (ws1_stream, _) = &mut connections[0].1;
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        // Send start timer message
        let start_message = json!({
            "type": "StartTimer"
        });

        ws1_sender.send(Message::Text(start_message.to_string())).await.unwrap();

        // All devices should receive the timer state update
        for (_, ws_stream) in &mut connections {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            // Should receive timer state update
            let msg = ws_receiver.next().await.unwrap().unwrap();
            let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();

            assert_eq!(state["type"], "TimerStateUpdate");
            assert_eq!(state["payload"]["is_running"], true);
        }
    }

    #[tokio::test]
    async fn test_concurrent_timer_control_requests() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect two devices
        let (ws1_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        let (ws2_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws2_sender, mut ws2_receiver) = ws2_stream.split();

        // Device 1 starts timer
        let start_message = json!({
            "type": "StartTimer"
        });
        ws1_sender.send(Message::Text(start_message.to_string())).await.unwrap();

        // Device 2 tries to start timer (should be handled gracefully)
        let start_message_2 = json!({
            "type": "StartTimer"
        });
        ws2_sender.send(Message::Text(start_message_2.to_string())).await.unwrap();

        // Both devices should receive consistent state
        let msg1 = ws1_receiver.next().await.unwrap().unwrap();
        let msg2 = ws2_receiver.next().await.unwrap().unwrap();

        let state1: serde_json::Value = serde_json::from_str(&msg1.to_text().unwrap()).unwrap();
        let state2: serde_json::Value = serde_json::from_str(&msg2.to_text().unwrap()).unwrap();

        // Both should agree that timer is running
        assert_eq!(state1["payload"]["is_running"], true);
        assert_eq!(state2["payload"]["is_running"], true);

        // Timer states should be identical
        assert_eq!(state1["payload"]["id"], state2["payload"]["id"]);
    }

    #[tokio::test]
    async fn test_device_disconnection_cleanup() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect device 1
        let (ws1_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        // Connect device 2
        let (ws2_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws2_sender, mut ws2_receiver) = ws2_stream.split();

        // Start timer
        let start_message = json!({
            "type": "StartTimer"
        });
        ws1_sender.send(Message::Text(start_message.to_string())).await.unwrap();

        // Both devices should receive initial state
        let _msg1 = ws1_receiver.next().await.unwrap().unwrap();
        let _msg2 = ws2_receiver.next().await.unwrap().unwrap();

        // Disconnect device 2
        drop(ws2_sender);
        drop(ws2_receiver);

        // Give server time to cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Device 1 should still receive updates
        ws1_sender.send(Message::Text(json!({"type": "PauseTimer"}).to_string())).await.unwrap();
        let pause_msg = ws1_receiver.next().await.unwrap().unwrap();
        let pause_state: serde_json::Value = serde_json::from_str(&pause_msg.to_text().unwrap()).unwrap();
        assert_eq!(pause_state["payload"]["is_running"], false);
    }

    #[tokio::test]
    async fn test_device_connection_pooling() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect 10 devices to test connection pooling
        let mut connections = Vec::new();
        for i in 0..10 {
            let url_with_device = format!("{}?device_id=device-{}", ws_url, i);
            let (ws_stream, _) = connect_async(&url_with_device).await.unwrap();
            connections.push(ws_stream);
        }

        // Give connections time to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start timer from first device
        let (ws1_stream, _) = &mut connections[0];
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        ws1_sender.send(Message::Text(json!({"type": "StartTimer"}).to_string())).await.unwrap();

        // All devices should receive the timer state update
        let mut received_count = 0;
        for ws_stream in &mut connections {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            // Set timeout for receiving message
            let timeout = tokio::time::timeout(
                Duration::from_millis(500),
                ws_receiver.next()
            ).await;

            if let Ok(Some(msg)) = timeout {
                let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
                if state["type"] == "TimerStateUpdate" {
                    received_count += 1;
                }
            }
        }

        // Should receive updates from all connected devices
        assert_eq!(received_count, 10);
    }

    #[tokio::test]
    async fn test_device_heartbeat_monitoring() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect device with heartbeat support
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Should receive connection status
        let connect_msg = ws_receiver.next().await.unwrap().unwrap();
        let connect_state: serde_json::Value = serde_json::from_str(&connect_msg.to_text().unwrap()).unwrap();
        assert_eq!(connect_state["type"], "ConnectionStatus");

        // Send periodic pings to simulate heartbeat
        for _ in 0..3 {
            ws1_sender.send(Message::Ping(vec![])).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Connection should remain active
        let start_message = json!({
            "type": "StartTimer"
        });
        ws1_sender.send(Message::Text(start_message.to_string())).await.unwrap();

        let timer_msg = ws_receiver.next().await.unwrap().unwrap();
        let timer_state: serde_json::Value = serde_json::from_str(&timer_msg.to_text().unwrap()).unwrap();
        assert_eq!(timer_state["type"], "TimerStateUpdate");
    }

    #[tokio::test]
    async fn test_device_identification_and_tracking() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect devices with different identifiers
        let devices = vec!["mobile-device", "desktop-device", "tablet-device"];
        let mut connections = HashMap::new();

        for device_id in devices {
            let url_with_device = format!("{}?device_id={}&user_agent=test-agent", ws_url, device_id);
            let (ws_stream, _) = connect_async(&url_with_device).await.unwrap();
            connections.insert(device_id, ws_stream);
        }

        // Give connections time to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start timer and verify all devices receive updates
        let (ws_stream, _) = connections.get_mut("mobile-device").unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        ws_sender.send(Message::Text(json!({"type": "StartTimer"}).to_string())).await.unwrap();

        // All devices should receive timer state
        for (device_id, ws_stream) in &mut connections {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let msg = ws_receiver.next().await.unwrap().unwrap();
            let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();

            assert_eq!(state["type"], "TimerStateUpdate");
            // Timer state should be consistent across all devices
            assert_eq!(state["payload"]["is_running"], true);
        }
    }

    #[tokio::test]
    async fn test_concurrent_session_synchronization() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect 3 devices
        let mut connections = Vec::new();
        for _ in 0..3 {
            let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
            connections.push(ws_stream);
        }

        // Rapid timer control operations from different devices
        let operations = vec![
            ("StartTimer", 100),
            ("PauseTimer", 200),
            ("ResetTimer", 300),
            ("StartTimer", 400),
            ("SkipTimer", 500),
        ];

        for (operation, delay) in operations {
            let device_index = (delay / 100) as usize % connections.len();
            let (ws_stream, _) = &mut connections[device_index];
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let message = json!({
                "type": operation
            });

            ws_sender.send(Message::Text(message.to_string())).await.unwrap();
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        // All devices should end up with the same final state
        let mut final_states = Vec::new();
        for ws_stream in &mut connections {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            // Collect the last timer state message
            let mut last_state = None;
            for _ in 0..10 { // Wait for up to 10 messages
                if let Some(msg) = ws_receiver.next().await {
                    let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
                    if state["type"] == "TimerStateUpdate" {
                        last_state = Some(state);
                    }
                }
            }

            if let Some(state) = last_state {
                final_states.push(state);
            }
        }

        // All final states should be identical
        if final_states.len() >= 2 {
            let first_state = &final_states[0];
            for state in &final_states[1..] {
                assert_eq!(state["payload"]["id"], first_state["payload"]["id"]);
                assert_eq!(state["payload"]["timer_type"], first_state["payload"]["timer_type"]);
                assert_eq!(state["payload"]["is_running"], first_state["payload"]["is_running"]);
            }
        }
    }

    #[tokio::test]
    async fn test_multi_device_performance_scaling() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect 20 devices to test performance scaling
        let mut connections = Vec::new();
        for i in 0..20 {
            let url_with_device = format!("{}?device_id=device-{}", ws_url, i);
            let (ws_stream, _) = connect_async(&url_with_device).await.unwrap();
            connections.push(ws_stream);
        }

        // Give connections time to establish
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Measure broadcast performance
        let start_time = std::time::Instant::now();

        let (ws1_stream, _) = &mut connections[0];
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        ws1_sender.send(Message::Text(json!({"type": "StartTimer"}).to_string())).await.unwrap();

        // Count how many devices receive the update within 1 second
        let mut received_count = 0;
        for ws_stream in &mut connections {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let timeout = tokio::time::timeout(
                Duration::from_millis(1000),
                ws_receiver.next()
            ).await;

            if let Ok(Some(msg)) = timeout {
                let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
                if state["type"] == "TimerStateUpdate" {
                    received_count += 1;
                }
            }
        }

        let elapsed = start_time.elapsed();

        // Should handle 20 devices efficiently
        assert!(received_count >= 18, "Only {} of 20 devices received updates", received_count);
        assert!(elapsed.as_millis() < 1000, "Broadcast took {}ms, should be under 1000ms", elapsed.as_millis());
    }

    async fn create_test_app() -> axum::Router {
        use crate::database::Database;
        use crate::config::Config;
        use crate::services::timer_service::TimerService;
        use std::sync::Arc;

        let config = Config::for_test();
        let db = Database::new_in_memory().await.unwrap();
        let timer_service = Arc::new(TimerService::new());

        crate::main::create_app_with_services(Arc::new(config), db, timer_service)
    }
}