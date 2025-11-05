#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use std::time::Duration;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn test_websocket_timer_state_synchronization() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Connect two WebSocket clients
        let ws_url = format!("ws://localhost/ws");

        let (ws1_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws1_sender, mut ws1_receiver) = ws1_stream.split();

        let (ws2_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws2_sender, mut ws2_receiver) = ws2_stream.split();

        // Start timer via HTTP API
        let _response = server.post("/api/timer/start").await;

        // Both clients should receive timer state update
        let msg1 = ws1_receiver.next().await.unwrap().unwrap();
        let msg2 = ws2_receiver.next().await.unwrap().unwrap();

        let state1: serde_json::Value = serde_json::from_str(&msg1.to_text().unwrap()).unwrap();
        let state2: serde_json::Value = serde_json::from_str(&msg2.to_text().unwrap()).unwrap();

        // Both should receive the same timer state
        assert_eq!(state1["type"], "TimerStateUpdate");
        assert_eq!(state2["type"], "TimerStateUpdate");
        assert_eq!(state1["payload"]["is_running"], true);
        assert_eq!(state2["payload"]["is_running"], true);

        // Timer IDs should match (same session)
        assert_eq!(state1["payload"]["id"], state2["payload"]["id"]);
    }

    #[tokio::test]
    async fn test_websocket_real_time_timer_updates() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Start timer
        let _response = server.post("/api/timer/start").await;

        // Should receive initial state
        let initial_msg = ws_receiver.next().await.unwrap().unwrap();
        let initial_state: serde_json::Value = serde_json::from_str(&initial_msg.to_text().unwrap()).unwrap();
        assert_eq!(initial_state["payload"]["is_running"], true);

        let initial_elapsed = initial_state["payload"]["elapsed"].as_u64().unwrap();

        // Wait for timer to progress
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should receive updated state
        let updated_msg = ws_receiver.next().await.unwrap().unwrap();
        let updated_state: serde_json::Value = serde_json::from_str(&updated_msg.to_text().unwrap()).unwrap();

        let updated_elapsed = updated_state["payload"]["elapsed"].as_u64().unwrap();

        // Elapsed time should have increased
        assert!(updated_elapsed > initial_elapsed);

        // Pause timer
        let _response = server.post("/api/timer/pause").await;

        // Should receive paused state
        let pause_msg = ws_receiver.next().await.unwrap().unwrap();
        let pause_state: serde_json::Value = serde_json::from_str(&pause_msg.to_text().unwrap()).unwrap();
        assert_eq!(pause_state["payload"]["is_running"], false);
    }

    #[tokio::test]
    async fn test_websocket_session_type_synchronization() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Start with Work session
        let _response = server.post("/api/timer/start").await;
        let work_msg = ws_receiver.next().await.unwrap().unwrap();
        let work_state: serde_json::Value = serde_json::from_str(&work_msg.to_text().unwrap()).unwrap();
        assert_eq!(work_state["payload"]["timer_type"], "Work");

        // Skip to next session (Short Break)
        let _response = server.post("/api/timer/skip").await;
        let break_msg = ws_receiver.next().await.unwrap().unwrap();
        let break_state: serde_json::Value = serde_json::from_str(&break_msg.to_text().unwrap()).unwrap();
        assert_eq!(break_state["payload"]["timer_type"], "ShortBreak");

        // Timer should be paused after skip
        assert_eq!(break_state["payload"]["is_running"], false);
        assert_eq!(break_state["payload"]["elapsed"], 0);
    }

    #[tokio::test]
    async fn test_websocket_timer_completion_synchronization() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Start timer with very short duration for testing
        let _response = server.post("/api/timer/start").await;

        // Wait for timer completion (simulated in test)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should receive completion notification
        let completion_msg = ws_receiver.next().await.unwrap().unwrap();
        let completion_state: serde_json::Value = serde_json::from_str(&completion_msg.to_text().unwrap()).unwrap();

        assert_eq!(completion_state["type"], "TimerStateUpdate");
        assert_eq!(completion_state["payload"]["is_running"], false);
        assert_eq!(completion_state["payload"]["timer_type"], "ShortBreak"); // Auto-transition
    }

    #[tokio::test]
    async fn test_websocket_multiple_clients_receive_same_state() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");

        // Connect 5 clients
        let mut clients = Vec::new();
        for _ in 0..5 {
            let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
            let (_, ws_receiver) = ws_stream.split();
            clients.push(ws_receiver);
        }

        // Start timer
        let _response = server.post("/api/timer/start").await;

        // All clients should receive the same timer state
        let mut received_states = Vec::new();
        for client in &mut clients {
            let msg = client.next().await.unwrap().unwrap();
            let state: serde_json::Value = serde_json::from_str(&msg.to_text().unwrap()).unwrap();
            received_states.push(state);
        }

        // All states should be identical
        let first_state = &received_states[0];
        for state in &received_states {
            assert_eq!(state["payload"]["id"], first_state["payload"]["id"]);
            assert_eq!(state["payload"]["is_running"], first_state["payload"]["is_running"]);
            assert_eq!(state["payload"]["timer_type"], first_state["payload"]["timer_type"]);
        }
    }

    #[tokio::test]
    async fn test_websocket_connection_status_messages() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Should receive connection status on connect
        let connect_msg = ws_receiver.next().await.unwrap().unwrap();
        let connect_state: serde_json::Value = serde_json::from_str(&connect_msg.to_text().unwrap()).unwrap();
        assert_eq!(connect_state["type"], "ConnectionStatus");

        // Test ping/pong for connection monitoring
        ws_sender.send(Message::Ping(vec![])).await.unwrap();

        // Should receive pong (handled by tungstenite automatically)
        // Connection should remain active
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start timer to verify connection is still working
        let _response = server.post("/api/timer/start").await;
        let timer_msg = ws_receiver.next().await.unwrap().unwrap();
        let timer_state: serde_json::Value = serde_json::from_str(&timer_msg.to_text().unwrap()).unwrap();
        assert_eq!(timer_state["type"], "TimerStateUpdate");
    }

    #[tokio::test]
    async fn test_websocket_message_format_consistency() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Reset timer first
        let _response = server.post("/api/timer/reset").await;
        let reset_msg = ws_receiver.next().await.unwrap().unwrap();
        let reset_state: serde_json::Value = serde_json::from_str(&reset_msg.to_text().unwrap()).unwrap();

        // Verify message structure
        assert!(reset_state.get("type").is_some());
        assert!(reset_state.get("payload").is_some());

        let payload = &reset_state["payload"];
        assert!(payload.get("id").is_some());
        assert!(payload.get("duration").is_some());
        assert!(payload.get("elapsed").is_some());
        assert!(payload.get("timer_type").is_some());
        assert!(payload.get("is_running").is_some());
        assert!(payload.get("created_at").is_some());
        assert!(payload.get("updated_at").is_some());

        // Verify data types
        assert!(payload["id"].is_string());
        assert!(payload["duration"].is_number());
        assert!(payload["elapsed"].is_number());
        assert!(payload["timer_type"].is_string());
        assert!(payload["is_running"].is_boolean());
    }

    #[tokio::test]
    async fn test_websocket_performance_sub_500ms_sync() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let ws_url = format!("ws://localhost/ws");
        let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Measure time from API call to WebSocket message
        let start_time = std::time::Instant::now();

        // Start timer via API
        let _response = server.post("/api/timer/start").await;

        // Wait for WebSocket message
        let _msg = ws_receiver.next().await.unwrap().unwrap();

        let elapsed = start_time.elapsed();

        // Should receive message within 500ms (requirement)
        assert!(elapsed.as_millis() < 500,
               "WebSocket sync took {}ms, should be under 500ms",
               elapsed.as_millis());
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