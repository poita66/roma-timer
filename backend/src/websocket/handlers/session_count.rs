//! Session Count WebSocket Message Handlers
//!
//! Handles WebSocket messages for manual session count management.

use crate::services::daily_reset_service::{DailyResetService, DailyResetStatus};
use crate::database::DatabaseManager;
use crate::websocket::messages::*;
use crate::error::AppError;
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, instrument};

/// Session Count WebSocket Handler
#[derive(Debug, Clone)]
pub struct SessionCountWebSocketHandler {
    database_manager: Arc<DatabaseManager>,
    daily_reset_service: Arc<DailyResetService>,
}

impl SessionCountWebSocketHandler {
    /// Create a new session count WebSocket handler
    pub fn new(
        database_manager: Arc<DatabaseManager>,
        daily_reset_service: Arc<DailyResetService>,
    ) -> Self {
        Self {
            database_manager,
            daily_reset_service,
        }
    }

    /// Handle get session count message
    #[instrument(skip(self, message, response_tx))]
    pub async fn handle_get_session_count(
        &self,
        message: &crate::services::websocket_service::WebSocketMessage,
        response_tx: mpsc::UnboundedSender<crate::services::websocket_service::WebSocketMessage>,
    ) -> Result<(), AppError> {
        let request: GetSessionCountRequest = serde_json::from_value(message.data.clone())
            .map_err(|e| AppError::Serialization(e))?;

        info!("Handling get session count request for user {}", request.user_id);

        // Get daily reset status
        let status = match self.daily_reset_service.get_daily_reset_status(&request.user_id).await {
            Ok(status) => status,
            Err(e) => {
                error!("Failed to get daily reset status for user {}: {}", request.user_id, e);

                let response = SessionCountResponse {
                    message_id: request.message_id,
                    success: false,
                    current_session_count: 0,
                    manual_session_override: None,
                    last_reset_utc: None,
                    error: Some(format!("Failed to get session count: {}", e)),
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_count_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;
                return Ok(());
            }
        };

        // Create successful response
        let response = SessionCountResponse {
            message_id: request.message_id,
            success: true,
            current_session_count: status.current_session_count,
            manual_session_override: status.manual_session_override,
            last_reset_utc: status.last_reset_utc,
            error: None,
            timestamp: chrono::Utc::now(),
        };

        let response_data = serde_json::json!({
            "type": "session_count_response",
            "message_id": request.message_id,
            "data": response,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
            payload: response_data,
        };

        response_tx.send(response_message)
            .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;

        info!("Successfully retrieved session count for user {}: {}", request.user_id, status.current_session_count);

        Ok(())
    }

    /// Handle set session count message
    #[instrument(skip(self, message, response_tx))]
    pub async fn handle_set_session_count(
        &self,
        message: &crate::services::websocket_service::WebSocketMessage,
        response_tx: mpsc::UnboundedSender<crate::services::websocket_service::WebSocketMessage>,
    ) -> Result<(), AppError> {
        let request: SetSessionCountRequest = serde_json::from_value(message.data.clone())
            .map_err(|e| AppError::Serialization(e))?;

        info!("Handling set session count request for user {}: {} (override: {})",
              request.user_id, request.session_count, request.manual_override);

        // Set session count via daily reset service
        match self.daily_reset_service.set_session_count(
            &request.user_id,
            request.session_count,
            request.manual_override,
        ).await {
            Ok(_) => {
                // Get updated status
                let status = match self.daily_reset_service.get_daily_reset_status(&request.user_id).await {
                    Ok(status) => status,
                    Err(e) => {
                        error!("Failed to get updated status for user {}: {}", request.user_id, e);

                        let response = SessionSetResponse {
                            message_id: request.message_id,
                            success: true, // The set succeeded even if we can't get status
                            current_session_count: request.session_count,
                            manual_session_override: if request.manual_override { Some(request.session_count) } else { None },
                            error: Some(format!("Session count set but failed to get updated status: {}", e)),
                            timestamp: chrono::Utc::now(),
                        };

                        let response_data = serde_json::json!({
                            "type": "session_set_response",
                            "message_id": request.message_id,
                            "data": response,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                            payload: response_data,
                        };

                        response_tx.send(response_message)
                            .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;
                        return Ok(());
                    }
                };

                // Create successful response
                let response = SessionSetResponse {
                    message_id: request.message_id,
                    success: true,
                    current_session_count: status.current_session_count,
                    manual_session_override: status.manual_session_override,
                    error: None,
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_set_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;

                info!("Successfully set session count for user {} to {} (override: {})",
                      request.user_id, request.session_count, request.manual_override);
            }
            Err(e) => {
                error!("Failed to set session count for user {}: {}", request.user_id, e);

                let response = SessionSetResponse {
                    message_id: request.message_id,
                    success: false,
                    current_session_count: 0,
                    manual_session_override: None,
                    error: Some(format!("Failed to set session count: {}", e)),
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_set_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;
            }
        }

        Ok(())
    }

    /// Handle reset session message
    #[instrument(skip(self, message, response_tx))]
    pub async fn handle_reset_session(
        &self,
        message: &crate::services::websocket_service::WebSocketMessage,
        response_tx: mpsc::UnboundedSender<crate::services::websocket_service::WebSocketMessage>,
    ) -> Result<(), AppError> {
        let request: ResetSessionRequest = serde_json::from_value(message.data.clone())
            .map_err(|e| AppError::Serialization(e))?;

        info!("Handling reset session request for user {}", request.user_id);

        // Get current status before reset
        let previous_status = match self.daily_reset_service.get_daily_reset_status(&request.user_id).await {
            Ok(status) => status,
            Err(e) => {
                error!("Failed to get current status for user {}: {}", request.user_id, e);

                let response = SessionResetResponse {
                    message_id: request.message_id,
                    success: false,
                    previous_session_count: 0,
                    new_session_count: 0,
                    reset_time_utc: chrono::Utc::now().timestamp(),
                    error: Some(format!("Failed to get current session count: {}", e)),
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_reset_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;
                return Ok(());
            }
        };

        let previous_count = previous_status.current_session_count;

        // Reset session count to 0 and clear manual override
        match self.daily_reset_service.set_session_count(&request.user_id, 0, false).await {
            Ok(_) => {
                // Create successful response
                let response = SessionResetResponse {
                    message_id: request.message_id,
                    success: true,
                    previous_session_count: previous_count,
                    new_session_count: 0,
                    reset_time_utc: chrono::Utc::now().timestamp(),
                    error: None,
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_reset_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;

                info!("Successfully reset session count for user {} from {} to 0", request.user_id, previous_count);
            }
            Err(e) => {
                error!("Failed to reset session count for user {}: {}", request.user_id, e);

                let response = SessionResetResponse {
                    message_id: request.message_id,
                    success: false,
                    previous_session_count: previous_count,
                    new_session_count: previous_count,
                    reset_time_utc: chrono::Utc::now().timestamp(),
                    error: Some(format!("Failed to reset session count: {}", e)),
                    timestamp: chrono::Utc::now(),
                };

                let response_data = serde_json::json!({
                    "type": "session_reset_response",
                    "message_id": request.message_id,
                    "data": response,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let response_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
                    payload: response_data,
                };

                response_tx.send(response_message)
                    .map_err(|_| AppError::WebSocket("Failed to send response".to_string()))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::time_provider::MockTimeProvider;
    use tempfile::TempDir;
    use std::time::SystemTime;

    async fn create_test_handler() -> (SessionCountWebSocketHandler, Arc<MockTimeProvider>) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test_session_count.db");
        let db_manager = Arc::new(DatabaseManager::new(&db_path.to_string_lossy()).await
            .expect("Failed to create DatabaseManager"));

        let mock_time = Arc::new(MockTimeProvider::new());
        mock_time.set_time(chrono::Utc::now());

        let timezone_service = Arc::new(crate::services::timezone_service::TimezoneService::new(mock_time.clone()));
        let daily_reset_service = Arc::new(DailyResetService::new(mock_time.clone(), db_manager.clone()));

        let handler = SessionCountWebSocketHandler::new(db_manager, daily_reset_service);

        (handler, mock_time)
    }

    #[tokio::test]
    async fn test_handle_get_session_count() -> Result<(), Box<dyn std::error::Error>> {
        let (handler, _) = create_test_handler().await;

        let request = GetSessionCountRequest {
            message_id: "test_001".to_string(),
            user_id: "test_user".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let (response_tx, mut response_rx) = mpsc::unbounded_channel();

        let ws_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
            payload: serde_json::json!({
                "type": "get_session_count",
                "message_id": "test_001",
                "data": serde_json::to_value(&request)?,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        };

        handler.handle_get_session_count(&ws_message, response_tx).await?;

        let response = response_rx.recv().await?;
        assert_eq!(response.type_, "session_count_response");
        assert_eq!(response.message_id, "test_001");

        let response_data: SessionCountResponse = serde_json::from_value(response.data)?;
        assert_eq!(response_data.message_id, "test_001");
        // Note: Since user doesn't exist, it should return an error response
        assert!(!response_data.success);
        assert!(response_data.error.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_set_session_count() -> Result<(), Box<dyn std::error::Error>> {
        let (handler, _) = create_test_handler().await;

        let request = SetSessionCountRequest {
            message_id: "test_002".to_string(),
            user_id: "test_user".to_string(),
            session_count: 5,
            manual_override: true,
            timestamp: chrono::Utc::now(),
        };

        let (response_tx, mut response_rx) = mpsc::unbounded_channel();

        let ws_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
            payload: serde_json::json!({
                "type": "set_session_count",
                "message_id": "test_002",
                "data": serde_json::to_value(&request)?,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        };

        handler.handle_set_session_count(&ws_message, response_tx).await?;

        let response = response_rx.recv().await?;
        assert_eq!(response.type_, "session_set_response");
        assert_eq!(response.message_id, "test_002");

        let response_data: SessionSetResponse = serde_json::from_value(response.data)?;
        assert_eq!(response_data.message_id, "test_002");
        // Note: Since user doesn't exist, it should return an error response
        assert!(!response_data.success);
        assert!(response_data.error.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_reset_session() -> Result<(), Box<dyn std::error::Error>> {
        let (handler, _) = create_test_handler().await;

        let request = ResetSessionRequest {
            message_id: "test_003".to_string(),
            user_id: "test_user".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let (response_tx, mut response_rx) = mpsc::unbounded_channel();

        let ws_message = crate::services::websocket_service::WebSocketMessage::ConfigurationUpdate {
            payload: serde_json::json!({
                "type": "reset_session",
                "message_id": "test_003",
                "data": serde_json::to_value(&request)?,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        };

        handler.handle_reset_session(&ws_message, response_tx).await?;

        let response = response_rx.recv().await?;
        assert_eq!(response.type_, "session_reset_response");
        assert_eq!(response.message_id, "test_003");

        let response_data: SessionResetResponse = serde_json::from_value(response.data)?;
        assert_eq!(response_data.message_id, "test_003");
        // Note: Since user doesn't exist, it should return an error response
        assert!(!response_data.success);
        assert!(response_data.error.is_some());

        Ok(())
    }
}