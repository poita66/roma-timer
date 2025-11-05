//! Configuration API Integration Tests
//!
//! Tests for configuration API endpoints including validation and error handling

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, put},
    Router,
};
use axum_test::TestServer;
use roma_timer::api::configuration;
use roma_timer::models::user_configuration::{Theme, UserConfiguration};
use roma_timer::services::{configuration_service::ConfigurationService, websocket_service::WebSocketService};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

async fn create_test_app() -> Router {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Initialize database schema
    sqlx::query(
        r#"
        CREATE TABLE user_configurations (
            id TEXT PRIMARY KEY,
            work_duration INTEGER NOT NULL DEFAULT 1500,
            short_break_duration INTEGER NOT NULL DEFAULT 300,
            long_break_duration INTEGER NOT NULL DEFAULT 900,
            long_break_frequency INTEGER NOT NULL DEFAULT 4,
            notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            webhook_url TEXT,
            wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
            theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let websocket_service = WebSocketService::new(pool.clone());
    let configuration_service = Arc::new(
        ConfigurationService::new(pool, websocket_service)
            .await
            .unwrap()
    );

    let app = Router::new()
        .merge(configuration::create_router())
        .with_state(configuration_service);

    app
}

fn create_auth_headers() -> Vec<(&'static str, &'static str)> {
    vec![("X-Auth-Token", "test-token")]
}

#[tokio::test]
async fn test_get_configuration() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/configuration")
        .add_headers(create_auth_headers())
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.workDuration, 1500); // 25 minutes
    assert_eq!(config.shortBreakDuration, 300); // 5 minutes
    assert_eq!(config.longBreakDuration, 900); // 15 minutes
    assert_eq!(config.longBreakFrequency, 4);
    assert!(config.notificationsEnabled);
    assert!(!config.waitForInteraction);
    assert_eq!(config.theme, "Light");
}

#[tokio::test]
async fn test_update_configuration_valid() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "workDuration": 1800,  // 30 minutes
        "notificationsEnabled": false,
        "theme": "Dark"
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.workDuration, 1800);
    assert!(!config.notificationsEnabled);
    assert_eq!(config.theme, "Dark");

    // Verify other fields remain unchanged
    assert_eq!(config.shortBreakDuration, 300);
    assert_eq!(config.longBreakDuration, 900);
    assert_eq!(config.longBreakFrequency, 4);
}

#[tokio::test]
async fn test_update_configuration_partial() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Update only one field
    let update = json!({
        "theme": "Dark"
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.theme, "Dark");

    // Other fields should remain at defaults
    assert_eq!(config.workDuration, 1500);
    assert_eq!(config.shortBreakDuration, 300);
    assert!(config.notificationsEnabled);
}

#[tokio::test]
async fn test_update_configuration_invalid_work_duration() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "workDuration": 100  // Too short (less than 5 minutes)
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
    assert!(error["details"].as_array().unwrap()[0]["message"]
        .as_str()
        .unwrap()
        .contains("5-60 minutes"));
}

#[tokio::test]
async fn test_update_configuration_invalid_short_break() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "shortBreakDuration": 30  // Too short (less than 1 minute)
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
}

#[tokio::test]
async fn test_update_configuration_invalid_long_break() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "longBreakDuration": 2400  // Too long (more than 30 minutes)
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
}

#[tokio::test]
async fn test_update_configuration_invalid_frequency() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "longBreakFrequency": 15  // Too high (more than 10)
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
}

#[tokio::test]
async fn test_update_configuration_invalid_webhook() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "webhookUrl": "not-a-valid-url"
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
}

#[tokio::test]
async fn test_update_configuration_invalid_theme() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "theme": "InvalidTheme"
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: Value = response.json();
    assert_eq!(error["error"], "ValidationError");
    assert!(error["details"].as_array().unwrap()[0]["message"]
        .as_str()
        .unwrap()
        .contains("Must be 'Light' or 'Dark'"));
}

#[tokio::test]
async fn test_update_configuration_valid_webhook() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let update = json!({
        "webhookUrl": "https://example.com/webhook"
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.webhookUrl, Some("https://example.com/webhook".to_string()));
}

#[tokio::test]
async fn test_update_configuration_clear_webhook() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // First set a webhook URL
    let update1 = json!({
        "webhookUrl": "https://example.com/webhook"
    });

    let response1 = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update1)
        .await;

    assert_eq!(response1.status_code(), StatusCode::OK);

    // Then clear it
    let update2 = json!({
        "webhookUrl": null
    });

    let response2 = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update2)
        .await;

    assert_eq!(response2.status_code(), StatusCode::OK);

    let config: UserConfiguration = response2.json();
    assert_eq!(config.webhookUrl, None);
}

#[tokio::test]
async fn test_reset_configuration() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // First make some changes
    let update = json!({
        "workDuration": 1800,
        "theme": "Dark",
        "notificationsEnabled": false
    });

    let response1 = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response1.status_code(), StatusCode::OK);

    // Then reset to defaults
    let response2 = server
        .post("/api/configuration/reset")
        .add_headers(create_auth_headers())
        .await;

    assert_eq!(response2.status_code(), StatusCode::OK);

    let config: UserConfiguration = response2.json();
    assert_eq!(config.workDuration, 1500); // Back to default
    assert_eq!(config.theme, "Light"); // Back to default
    assert!(config.notificationsEnabled); // Back to default
}

#[tokio::test]
async fn test_get_configuration_not_found() {
    // This test verifies behavior when no configuration exists yet
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/configuration")
        .add_headers(create_auth_headers())
        .await;

    // Should return default configuration even if not previously saved
    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.workDuration, 1500);
    assert_eq!(config.theme, "Light");
}

#[tokio::test]
async fn test_multiple_updates() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Make first update
    let update1 = json!({
        "workDuration": 1800,
        "theme": "Dark"
    });

    let response1 = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update1)
        .await;

    assert_eq!(response1.status_code(), StatusCode::OK);

    // Make second update
    let update2 = json!({
        "shortBreakDuration": 600,
        "notificationsEnabled": false
    });

    let response2 = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update2)
        .await;

    assert_eq!(response2.status_code(), StatusCode::OK);

    let config: UserConfiguration = response2.json();

    // Verify all changes are applied
    assert_eq!(config.workDuration, 1800);
    assert_eq!(config.shortBreakDuration, 600);
    assert_eq!(config.theme, "Dark");
    assert!(!config.notificationsEnabled);

    // Verify unchanged fields remain at defaults
    assert_eq!(config.longBreakDuration, 900);
    assert_eq!(config.longBreakFrequency, 4);
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Test GET without auth token
    let response = server.get("/api/configuration").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test PUT without auth token
    let update = json!({"theme": "Dark"});
    let response = server.put("/api/configuration").json(&update).await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test POST without auth token
    let response = server.post("/api/configuration/reset").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_configuration_persistence() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Update configuration
    let update = json!({
        "workDuration": 2100,
        "theme": "Dark",
        "waitForInteraction": true
    });

    let response = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Get configuration again to verify persistence
    let response = server
        .get("/api/configuration")
        .add_headers(create_auth_headers())
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let config: UserConfiguration = response.json();
    assert_eq!(config.workDuration, 2100);
    assert_eq!(config.theme, "Dark");
    assert!(config.waitForInteraction);
}

#[tokio::test]
async fn test_boundary_values() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    // Test minimum valid values
    let update_min = json!({
        "workDuration": 300,        // 5 minutes
        "shortBreakDuration": 60,  // 1 minute
        "longBreakDuration": 300,  // 5 minutes
        "longBreakFrequency": 2    // 2 sessions
    });

    let response_min = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update_min)
        .await;

    assert_eq!(response_min.status_code(), StatusCode::OK);

    // Test maximum valid values
    let update_max = json!({
        "workDuration": 3600,        // 60 minutes
        "shortBreakDuration": 900,  // 15 minutes
        "longBreakDuration": 1800,  // 30 minutes
        "longBreakFrequency": 10    // 10 sessions
    });

    let response_max = server
        .put("/api/configuration")
        .add_headers(create_auth_headers())
        .json(&update_max)
        .await;

    assert_eq!(response_max.status_code(), StatusCode::OK);
}