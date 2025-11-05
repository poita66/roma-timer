#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;
    use tower::ServiceBuilder;

    #[tokio::test]
    async fn test_get_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/timer").await;

        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert!(json_response.get("id").is_some());
        assert!(json_response.get("duration").is_some());
        assert!(json_response.get("elapsed").is_some());
        assert!(json_response.get("timer_type").is_some());
        assert!(json_response.get("is_running").is_some());
    }

    #[tokio::test]
    async fn test_start_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // First ensure timer is stopped
        let _ = server.post("/api/timer/pause").await;

        // Start timer
        let response = server.post("/api/timer/start").await;

        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["is_running"], true);
    }

    #[tokio::test]
    async fn test_pause_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Start timer first
        let _ = server.post("/api/timer/start").await;

        // Pause timer
        let response = server.post("/api/timer/pause").await;

        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["is_running"], false);
    }

    #[tokio::test]
    async fn test_reset_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Start timer and let it run
        let _ = server.post("/api/timer/start").await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Reset timer
        let response = server.post("/api/timer/reset").await;

        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["elapsed"], 0);
        assert_eq!(json_response["is_running"], false);
        assert_eq!(json_response["timer_type"], "Work");
    }

    #[tokio::test]
    async fn test_skip_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Get initial state
        let initial_response = server.get("/api/timer").await;
        let initial_state: serde_json::Value = initial_response.json();
        let initial_type = initial_state["timer_type"].as_str().unwrap();

        // Skip timer
        let response = server.post("/api/timer/skip").await;

        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["elapsed"], 0);
        assert_eq!(json_response["is_running"], false);

        // Timer type should have changed
        let new_type = json_response["timer_type"].as_str().unwrap();
        assert_ne!(new_type, initial_type);
    }

    #[tokio::test]
    async fn test_timer_workflow_sequence() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // 1. Get initial state
        let initial = server.get("/api/timer").await;
        assert_eq!(initial.status_code(), 200);

        // 2. Start timer
        let start = server.post("/api/timer/start").await;
        assert_eq!(start.status_code(), 200);
        let start_state: serde_json::Value = start.json();
        assert_eq!(start_state["is_running"], true);

        // 3. Pause timer
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let pause = server.post("/api/timer/pause").await;
        assert_eq!(pause.status_code(), 200);
        let pause_state: serde_json::Value = pause.json();
        assert_eq!(pause_state["is_running"], false);
        assert!(pause_state["elapsed"].as_u64().unwrap() > 0);

        // 4. Reset timer
        let reset = server.post("/api/timer/reset").await;
        assert_eq!(reset.status_code(), 200);
        let reset_state: serde_json::Value = reset.json();
        assert_eq!(reset_state["elapsed"], 0);
        assert_eq!(reset_state["is_running"], false);

        // 5. Skip timer
        let skip = server.post("/api/timer/skip").await;
        assert_eq!(skip.status_code(), 200);
        let skip_state: serde_json::Value = skip.json();
        assert_eq!(skip_state["elapsed"], 0);
        assert_eq!(skip_state["is_running"], false);
    }

    #[tokio::test]
    async fn test_invalid_timer_operations() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Try to pause when not running
        let pause_response = server.post("/api/timer/pause").await;
        // Should still return 200 but indicate timer is already stopped
        assert_eq!(pause_response.status_code(), 200);

        // Try to start when already running
        let _ = server.post("/api/timer/start").await;
        let start_response = server.post("/api/timer/start").await;
        // Should return 400 or 409 for conflict
        assert!(start_response.status_code() == 400 || start_response.status_code() == 409);
    }

    #[tokio::test]
    async fn test_api_response_times() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Test that API responses are under 200ms as per requirements
        let start = std::time::Instant::now();
        let _ = server.get("/api/timer").await;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 200, "API response took {}ms, should be under 200ms", elapsed.as_millis());
    }

    #[tokio::test]
    async fn test_concurrent_timer_operations() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Test concurrent operations don't cause conflicts
        let mut handles = vec![];

        for _ in 0..5 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                server_clone.get("/api/timer").await
            });
            handles.push(handle);
        }

        // All concurrent requests should succeed
        for handle in handles {
            let response = handle.await.unwrap();
            assert_eq!(response.status_code(), 200);
        }
    }

    async fn create_test_app() -> axum::Router {
        // Create a test app with in-memory database and test configuration
        use crate::database::Database;
        use crate::config::Config;
        use std::sync::Arc;

        let config = Config::for_test();
        let db = Database::new_in_memory().await.unwrap();

        crate::main::create_app(Arc::new(config), db)
    }
}