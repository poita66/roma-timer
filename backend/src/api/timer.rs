//! Timer API Endpoints
//!
//! REST API endpoints for timer control and state management.
//! All responses target <200ms as per performance requirements.

use crate::services::timer_service::{TimerService, TimerServiceError, TimerState};
use crate::models::timer_session::TimerSession;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

/// Create timer API routes
pub fn create_timer_routes() -> Router<Arc<TimerService>> {
    Router::new()
        .route("/", get(get_timer))
        .route("/start", post(start_timer))
        .route("/pause", post(pause_timer))
        .route("/reset", post(reset_timer))
        .route("/skip", post(skip_timer))
        .route("/:id", get(get_timer_by_id))
}

/// Get current timer state
pub async fn get_timer(
    State(timer_service): State<Arc<TimerService>>,
) -> Result<Json<TimerState>, ApiError> {
    let state = timer_service.get_timer_state().await;
    Ok(Json(state))
}

/// Start the timer
pub async fn start_timer(
    State(timer_service): State<Arc<TimerService>>,
) -> Result<Json<TimerState>, ApiError> {
    timer_service
        .start_timer()
        .await
        .map_err(|e| match e {
            TimerServiceError::AlreadyRunning => ApiError::BadRequest(e.to_string()),
            _ => ApiError::InternalError(e.to_string()),
        })?;

    let state = timer_service.get_timer_state().await;
    Ok(Json(state))
}

/// Pause the timer
pub async fn pause_timer(
    State(timer_service): State<Arc<TimerService>>,
) -> Result<Json<TimerState>, ApiError> {
    timer_service
        .pause_timer()
        .await
        .map_err(|e| match e {
            TimerServiceError::NotRunning => ApiError::BadRequest(e.to_string()),
            _ => ApiError::InternalError(e.to_string()),
        })?;

    let state = timer_service.get_timer_state().await;
    Ok(Json(state))
}

/// Reset the timer
pub async fn reset_timer(
    State(timer_service): State<Arc<TimerService>>,
) -> Result<Json<TimerState>, ApiError> {
    timer_service
        .reset_timer()
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let state = timer_service.get_timer_state().await;
    Ok(Json(state))
}

/// Skip to next session
pub async fn skip_timer(
    State(timer_service): State<Arc<TimerService>>,
) -> Result<Json<TimerState>, ApiError> {
    timer_service
        .skip_timer()
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let state = timer_service.get_timer_state().await;
    Ok(Json(state))
}

/// Get timer by ID (future extension)
pub async fn get_timer_by_id(
    State(timer_service): State<Arc<TimerService>>,
    Path(id): Path<String>,
) -> Result<Json<TimerSession>, ApiError> {
    // For now, return current timer state
    // Future implementation could store multiple timer sessions
    let current_state = timer_service.get_timer_state().await;

    if current_state.id != id {
        return Err(ApiError::NotFound("Timer session not found".to_string()));
    }

    let session = TimerSession {
        id: current_state.id,
        duration: current_state.duration,
        elapsed: current_state.elapsed,
        timer_type: serde_json::from_str(&format!("\"{}\"", current_state.timer_type))
            .map_err(|_| ApiError::InternalError("Invalid timer type".to_string()))?,
        is_running: current_state.is_running,
        created_at: current_state.created_at,
        updated_at: current_state.updated_at,
    };

    Ok(Json(session))
}

/// API Error types
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalError(String),
    Conflict(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": status.as_str(),
            "message": error_message,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use tower::ServiceBuilder;

    async fn create_test_app() -> axum::Router<Arc<TimerService>> {
        let timer_service = Arc::new(TimerService::new());
        create_timer_routes().with_state(timer_service)
    }

    #[tokio::test]
    async fn test_get_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
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

        let response = server.post("/start").await;
        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["is_running"], true);
    }

    #[tokio::test]
    async fn test_pause_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Start timer first
        let _ = server.post("/start").await;

        // Pause timer
        let response = server.post("/pause").await;
        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["is_running"], false);
    }

    #[tokio::test]
    async fn test_reset_timer_endpoint() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Start timer first
        let _ = server.post("/start").await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Reset timer
        let response = server.post("/reset").await;
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
        let initial_response = server.get("/").await;
        let initial_state: serde_json::Value = initial_response.json();
        let initial_type = initial_state["timer_type"].as_str().unwrap();

        // Skip timer
        let response = server.post("/skip").await;
        assert_eq!(response.status_code(), 200);

        let json_response: serde_json::Value = response.json();
        assert_eq!(json_response["elapsed"], 0);
        assert_eq!(json_response["is_running"], false);

        // Timer type should have changed
        let new_type = json_response["timer_type"].as_str().unwrap();
        assert_ne!(new_type, initial_type);
    }

    #[tokio::test]
    async fn test_duplicate_start_returns_error() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Start timer
        let start_response = server.post("/start").await;
        assert_eq!(start_response.status_code(), 200);

        // Try to start again
        let duplicate_start = server.post("/start").await;
        assert_eq!(duplicate_start.status_code(), 400);
    }

    #[tokio::test]
    async fn test_pause_when_not_running_returns_error() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Try to pause without starting
        let pause_response = server.post("/pause").await;
        assert_eq!(pause_response.status_code(), 400);
    }

    #[tokio::test]
    async fn test_api_response_time() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        let start = std::time::Instant::now();
        let _ = server.get("/").await;
        let elapsed = start.elapsed();

        // Should respond in under 200ms
        assert!(elapsed.as_millis() < 200, "API response took {}ms", elapsed.as_millis());
    }
}