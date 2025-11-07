//! Integration Test Utilities for Daily Session Reset
//!
//! Provides integration test utilities including:
//! - Full application setup for testing
//! - HTTP client testing utilities
//! - WebSocket testing framework
//! - End-to-end test scenarios

use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::database::DatabaseManager;
use crate::services::time_provider::{TimeProvider, MockTimeProvider};
use chrono::{DateTime, Utc};

/// Integration test context with full application setup
#[derive(Debug)]
pub struct DailyResetIntegrationTestContext {
    /// Temporary database directory
    #[allow(dead_code)]
    temp_dir: Arc<TempDir>,

    /// Database manager
    pub db_manager: Arc<DatabaseManager>,

    /// Mock time provider
    pub time_provider: Arc<MockTimeProvider>,

    /// Test configuration
    pub config: TestConfig,

    /// HTTP client for API testing
    #[cfg(feature = "integration-tests")]
    pub http_client: reqwest::Client,

    /// WebSocket connections (for real-time testing)
    #[cfg(feature = "integration-tests")]
    pub websocket_connections: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Base URL for HTTP API
    pub api_base_url: String,

    /// WebSocket URL
    pub websocket_url: String,

    /// Test user ID
    pub test_user_id: String,

    /// Test device ID
    pub test_device_id: String,

    /// Test authentication token (if required)
    pub auth_token: Option<String>,

    /// Server port
    pub server_port: u16,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:3001".to_string(),
            websocket_url: "ws://localhost:3001/ws".to_string(),
            test_user_id: Uuid::new_v4().to_string(),
            test_device_id: Uuid::new_v4().to_string(),
            auth_token: None,
            server_port: 3001,
        }
    }
}

impl DailyResetIntegrationTestContext {
    /// Create a new integration test context
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_config(TestConfig::default()).await
    }

    /// Create a new integration test context with custom configuration
    pub async fn new_with_config(config: TestConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create temporary database
        let temp_dir = Arc::new(TempDir::new()?);
        let db_path = temp_dir.path().join("integration_test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Initialize database
        let db_manager = Arc::new(DatabaseManager::new(&db_url).await?);
        db_manager.migrate().await?;

        // Initialize mock time provider
        let start_time = Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap();
        let time_provider = Arc::new(MockTimeProvider::new(start_time));

        #[cfg(feature = "integration-tests")]
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let context = Self {
            temp_dir,
            db_manager,
            time_provider,
            config,
            #[cfg(feature = "integration-tests")]
            http_client,
            #[cfg(feature = "integration-tests")]
            websocket_connections: Arc::new(Mutex::new(Vec::new())),
        };

        // Wait for server to be ready (if starting server)
        #[cfg(feature = "integration-tests")]
        context.wait_for_server_ready().await?;

        Ok(context)
    }

    /// Wait for the server to be ready for connections
    #[cfg(feature = "integration-tests")]
    async fn wait_for_server_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut attempts = 0;
        let max_attempts = 30; // 30 seconds timeout

        while attempts < max_attempts {
            match self.http_client
                .get(&format!("{}/health", self.config.api_base_url))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                Ok(_) | Err(_) => {
                    // Server not ready, wait and retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    attempts += 1;
                }
            }
        }

        Err("Server failed to become ready within timeout".into())
    }

    /// Advance mock time
    pub fn advance_time(&self, hours: i64, minutes: i64, seconds: i64) {
        let duration = chrono::Duration::hours(hours) +
            chrono::Duration::minutes(minutes) +
            chrono::Duration::seconds(seconds);
        self.time_provider.advance(duration);
    }

    /// Set mock time to specific datetime
    pub fn set_time(&self, datetime: DateTime<Utc>) {
        self.time_provider.set_time(datetime);
    }

    /// Get current mock time
    pub fn current_time(&self) -> DateTime<Utc> {
        self.time_provider.now_utc()
    }
}

/// HTTP API testing utilities
#[cfg(feature = "integration-tests")]
pub mod http {
    use super::*;
    use serde_json::Value;

    /// Make authenticated API request
    pub async fn authenticated_request(
        client: &reqwest::Client,
        method: reqwest::Method,
        url: &str,
        auth_token: Option<&str>,
        body: Option<Value>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = client.request(method, url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        if let Some(body_value) = body {
            request = request.json(&body_value);
        }

        request.header("Content-Type", "application/json")
            .send()
            .await
    }

    /// Get user configuration via API
    pub async fn get_user_configuration(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/api/configuration/{}", base_url, user_id);
        authenticated_request(
            client,
            reqwest::Method::GET,
            &url,
            auth_token,
            None,
        ).await
    }

    /// Update daily reset configuration via API
    pub async fn update_daily_reset_configuration(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
        timezone: &str,
        reset_time: &crate::models::DailyResetTime,
        enabled: bool,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/api/configuration/{}/daily-reset", base_url, user_id);
        let body = serde_json::json!({
            "timezone": timezone,
            "daily_reset_time": reset_time,
            "daily_reset_enabled": enabled
        });

        authenticated_request(
            client,
            reqwest::Method::PUT,
            &url,
            auth_token,
            Some(body),
        ).await
    }

    /// Get session count via API
    pub async fn get_session_count(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/api/session/count", base_url);
        let mut request = client.get(&url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("X-User-ID", user_id)
            .send()
            .await
    }

    /// Set session count via API
    pub async fn set_session_count(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
        count: u32,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/api/session/count", base_url);
        let body = serde_json::json!({
            "count": count
        });

        let mut request = client.put(&url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("X-User-ID", user_id)
            .json(&body)
            .send()
            .await
    }

    /// Reset session count via API
    pub async fn reset_session_count(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/api/session/reset", base_url);
        let mut request = client.post(&url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("X-User-ID", user_id)
            .send()
            .await
    }

    /// Get daily analytics via API
    pub async fn get_daily_analytics(
        client: &reqwest::Client,
        base_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
        start_date: &str,
        end_date: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "{}/api/analytics/daily-stats?start_date={}&end_date={}",
            base_url, start_date, end_date
        );
        let mut request = client.get(&url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request.header("X-User-ID", user_id)
            .send()
            .await
    }
}

/// WebSocket testing utilities
#[cfg(feature = "integration-tests")]
pub mod websocket {
    use super::*;
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};

    /// Connect to WebSocket with authentication
    pub async fn connect_websocket(
        url: &str,
        user_id: &str,
        auth_token: Option<&str>,
    ) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Box<dyn std::error::Error>> {
        let mut ws_url = url.to_string();
        if let Some(token) = auth_token {
            ws_url.push_str(&format!("?token={}", token));
        }

        let (ws_stream, _) = connect_async(&ws_url).await?;
        Ok(ws_stream)
    }

    /// Send WebSocket message
    pub async fn send_websocket_message(
        ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        ws_stream.send(Message::Text(message.to_string())).await?;
        Ok(())
    }

    /// Receive WebSocket message with timeout
    pub async fn receive_websocket_message(
        ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        timeout_ms: u64,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        use tokio::time::timeout;

        match timeout(tokio::time::Duration::from_millis(timeout_ms), ws_stream.next()).await {
            Ok(Some(Ok(msg))) => {
                match msg {
                    Message::Text(text) => Ok(Some(text)),
                    Message::Binary(data) => Ok(Some(String::from_utf8(data)?)),
                    Message::Close(_) => Ok(None),
                    _ => Ok(None),
                }
            }
            Ok(Some(Err(e))) => Err(e.into()),
            Ok(None) => Ok(None),
            Err(_) => Err("WebSocket receive timeout".into()),
        }
    }

    /// Test WebSocket message roundtrip
    pub async fn test_websocket_roundtrip(
        ws_url: &str,
        user_id: &str,
        auth_token: Option<&str>,
        send_message: &str,
        expected_response: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut ws_stream = connect_websocket(ws_url, user_id, auth_token).await?;
        send_websocket_message(&mut ws_stream, send_message).await?;

        let response = receive_websocket_message(&mut ws_stream, 5000).await?;

        match (expected_response, response) {
            (Some(expected), Some(received)) => Ok(received.contains(expected)),
            (None, _) => Ok(true), // No specific response expected
            (Some(_), None) => Ok(false), // Expected response but got nothing
        }
    }
}

/// Test scenarios for integration testing
pub mod scenarios {
    use super::*;

    /// Test scenario for daily reset configuration
    #[derive(Debug, Clone)]
    pub struct DailyResetConfigurationScenario {
        pub name: &'static str,
        pub timezone: &'static str,
        pub reset_time: crate::models::DailyResetTime,
        pub enabled: bool,
        pub expected_cron: &'static str,
        pub description: &'static str,
    }

    /// Get common daily reset test scenarios
    pub fn daily_reset_configuration_scenarios() -> Vec<DailyResetConfigurationScenario> {
        vec![
            DailyResetConfigurationScenario {
                name: "midnight_utc_enabled",
                timezone: "UTC",
                reset_time: crate::models::DailyResetTime::midnight(),
                enabled: true,
                expected_cron: "0 0 * * *",
                description: "Midnight reset in UTC, enabled",
            },
            DailyResetConfigurationScenario {
                name: "morning_ny_enabled",
                timezone: "America/New_York",
                reset_time: crate::models::DailyResetTime::hour(7).unwrap(),
                enabled: true,
                expected_cron: "0 7 * * *",
                description: "7 AM reset in New York, enabled",
            },
            DailyResetConfigurationScenario {
                name: "custom_time_london_enabled",
                timezone: "Europe/London",
                reset_time: crate::models::DailyResetTime::custom("09:30".to_string()).unwrap(),
                enabled: true,
                expected_cron: "0 30 9 * * *",
                description: "9:30 AM custom reset in London, enabled",
            },
            DailyResetConfigurationScenario {
                name: "midnight_utc_disabled",
                timezone: "UTC",
                reset_time: crate::models::DailyResetTime::midnight(),
                enabled: false,
                expected_cron: "0 0 * * *",
                description: "Midnight reset in UTC, disabled",
            },
        ]
    }

    /// Test scenario for session count operations
    #[derive(Debug, Clone)]
    pub struct SessionCountOperationScenario {
        pub name: &'static str,
        pub initial_count: u32,
        pub operation: SessionCountOperation,
        pub expected_count: u32,
        pub description: &'static str,
    }

    /// Types of session count operations
    #[derive(Debug, Clone)]
    pub enum SessionCountOperation {
        Increment,
        Reset,
        Set(u32),
    }

    /// Get session count test scenarios
    pub fn session_count_operation_scenarios() -> Vec<SessionCountOperationScenario> {
        vec![
            SessionCountOperationScenario {
                name: "increment_from_zero",
                initial_count: 0,
                operation: SessionCountOperation::Increment,
                expected_count: 1,
                description: "Increment from 0 should result in 1",
            },
            SessionCountOperationScenario {
                name: "increment_from_five",
                initial_count: 5,
                operation: SessionCountOperation::Increment,
                expected_count: 6,
                description: "Increment from 5 should result in 6",
            },
            SessionCountOperationScenario {
                name: "reset_from_ten",
                initial_count: 10,
                operation: SessionCountOperation::Reset,
                expected_count: 0,
                description: "Reset from 10 should result in 0",
            },
            SessionCountOperationScenario {
                name: "set_to_fifteen",
                initial_count: 3,
                operation: SessionCountOperation::Set(15),
                expected_count: 15,
                description: "Set to 15 should result in 15",
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_context_creation() {
        let context = DailyResetIntegrationTestContext::new().await.unwrap();

        assert!(!context.config.test_user_id.is_empty());
        assert!(!context.config.test_device_id.is_empty());
        assert_eq!(context.current_time(),
            Utc.with_ymd_and_hms(2025, 1, 7, 10, 0, 0).single().unwrap());
    }

    #[test]
    fn test_scenarios() {
        let config_scenarios = scenarios::daily_reset_configuration_scenarios();
        assert!(!config_scenarios.is_empty());
        assert_eq!(config_scenarios[0].name, "midnight_utc_enabled");

        let count_scenarios = scenarios::session_count_operation_scenarios();
        assert!(!count_scenarios.is_empty());
        assert_eq!(count_scenarios[0].name, "increment_from_zero");
    }

    #[test]
    fn test_time_manipulation() {
        let context = DailyResetIntegrationTestContext::new().await.unwrap();

        let initial_time = context.current_time();
        context.advance_time(1, 30, 0);
        let new_time = context.current_time();

        assert!(new_time > initial_time);
        assert_eq!(new_time - initial_time, chrono::Duration::minutes(90));
    }
}