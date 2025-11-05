//! Configuration API endpoints
//!
//! REST API endpoints for managing user configuration settings.

use crate::models::user_configuration::{UserConfiguration, UserConfigurationError};
use crate::services::configuration_service::{ConfigurationService, ConfigurationServiceError, ConfigurationUpdate};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use tracing::{debug, error, info, warn};

/// API error response structure
#[derive(Debug, serde::Serialize)]
struct ApiError {
    error: String,
    message: String,
    timestamp: u64,
}

impl ApiError {
    fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Validation error response with field details
#[derive(Debug, serde::Serialize)]
struct ValidationError {
    error: String,
    message: String,
    timestamp: u64,
    details: Vec<ValidationDetail>,
}

impl ValidationError {
    fn new(error: &str, message: &str, details: Vec<ValidationDetail>) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            details,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct ValidationDetail {
    field: String,
    message: String,
}

/// Get current user configuration
///
/// Returns the current user configuration settings.
pub async fn get_configuration(
    State(configuration_service): State<Arc<ConfigurationService>>,
) -> Result<Json<UserConfiguration>, (StatusCode, Json<ApiError>)> {
    debug!("GET /api/configuration - Getting current configuration");

    match configuration_service.get_configuration().await {
        Ok(config) => {
            info!("Configuration retrieved successfully");
            Ok(Json(config))
        }
        Err(e) => {
            error!("Failed to get configuration: {}", e);
            let api_error = ApiError::new("ConfigurationNotFound", &format!("Failed to retrieve configuration: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(api_error)))
        }
    }
}

/// Update user configuration
///
/// Updates user configuration settings with validation.
pub async fn update_configuration(
    State(configuration_service): State<Arc<ConfigurationService>>,
    Json(update): Json<ConfigurationUpdate>,
) -> Result<Json<UserConfiguration>, (StatusCode, Json<Value>)> {
    debug!("PUT /api/configuration - Updating configuration: {:?}", update);

    match configuration_service.update_configuration(update).await {
        Ok(config) => {
            info!("Configuration updated successfully");
            Ok(Json(config))
        }
        Err(ConfigurationServiceError::Validation(e)) => {
            warn!("Configuration validation failed: {}", e);
            let validation_error = ValidationError::new(
                "ValidationError",
                "Configuration validation failed",
                vec![ValidationDetail {
                    field: "configuration".to_string(),
                    message: e.to_string(),
                }],
            );
            Err((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::to_value(validation_error).unwrap())))
        }
        Err(ConfigurationServiceError::InvalidTheme(theme)) => {
            warn!("Invalid theme provided: {}", theme);
            let validation_error = ValidationError::new(
                "ValidationError",
                "Invalid theme provided",
                vec![ValidationDetail {
                    field: "theme".to_string(),
                    message: format!("Theme '{}' is not valid. Must be 'Light' or 'Dark'", theme),
                }],
            );
            Err((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::to_value(validation_error).unwrap())))
        }
        Err(e) => {
            error!("Failed to update configuration: {}", e);
            let api_error = ApiError::new("ConfigurationUpdateError", &format!("Failed to update configuration: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(api_error).unwrap())))
        }
    }
}

/// Reset configuration to defaults
///
/// Resets all configuration settings to their default values.
pub async fn reset_configuration(
    State(configuration_service): State<Arc<ConfigurationService>>,
) -> Result<Json<UserConfiguration>, (StatusCode, Json<ApiError>)> {
    debug!("POST /api/configuration/reset - Resetting configuration to defaults");

    match configuration_service.reset_to_defaults().await {
        Ok(config) => {
            info!("Configuration reset to defaults successfully");
            Ok(Json(config))
        }
        Err(e) => {
            error!("Failed to reset configuration: {}", e);
            let api_error = ApiError::new("ConfigurationResetError", &format!("Failed to reset configuration: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(api_error)))
        }
    }
}

/// Create configuration API router
pub fn create_router() -> Router<Arc<ConfigurationService>> {
    Router::new()
        .route("/api/configuration", get(get_configuration).put(update_configuration))
        .route("/api/configuration/reset", axum::routing::post(reset_configuration))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::configuration_service::ConfigurationService;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, put},
        Router,
    };
    use serde_json::json;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let websocket_service = crate::services::websocket_service::WebSocketService::new(pool.clone());
        let configuration_service = ConfigurationService::new(pool, websocket_service)
            .await
            .unwrap();

        create_router().with_state(Arc::new(configuration_service))
    }

    #[tokio::test]
    async fn test_get_configuration() {
        let app = create_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/api/configuration")
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let config: UserConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(config.work_duration, 1500); // 25 minutes
        assert_eq!(config.short_break_duration, 300); // 5 minutes
        assert!(config.notifications_enabled);
    }

    #[tokio::test]
    async fn test_update_configuration() {
        let app = create_test_app().await;

        let update = json!({
            "workDuration": 1800,
            "notificationsEnabled": false,
            "theme": "Dark"
        });

        let request = Request::builder()
            .method("PUT")
            .uri("/api/configuration")
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let config: UserConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(config.work_duration, 1800);
        assert!(!config.notifications_enabled);
        assert_eq!(format!("{:?}", config.theme), "Dark");
    }

    #[tokio::test]
    async fn test_invalid_configuration_update() {
        let app = create_test_app().await;

        let update = json!({
            "workDuration": 100, // Too short (less than 5 minutes)
        });

        let request = Request::builder()
            .method("PUT")
            .uri("/api/configuration")
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_invalid_theme() {
        let app = create_test_app().await;

        let update = json!({
            "theme": "InvalidTheme"
        });

        let request = Request::builder()
            .method("PUT")
            .uri("/api/configuration")
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_reset_configuration() {
        let app = create_test_app().await;

        // First make some changes
        let update = json!({
            "workDuration": 1800,
            "theme": "Dark"
        });

        let request = Request::builder()
            .method("PUT")
            .uri("/api/configuration")
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Then reset
        let request = Request::builder()
            .method("POST")
            .uri("/api/configuration/reset")
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let config: UserConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(config.work_duration, 1500); // Back to default
        assert_eq!(format!("{:?}", config.theme), "Light"); // Back to default
    }
}