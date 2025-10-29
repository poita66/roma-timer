//! Simple Roma Timer backend that serves the frontend and provides basic API

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::{Path as AxumPath, State},
    http::{header, Method, StatusCode},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub is_running: bool,
    pub remaining_seconds: u32,
    pub session_type: String,
    pub session_count: u32,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRequest {
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsRequest {
    pub work_duration: Option<u32>,
    pub short_break_duration: Option<u32>,
    pub long_break_duration: Option<u32>,
    pub long_break_frequency: Option<u32>,
}

type SharedState = Arc<Mutex<TimerState>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get port from environment or use default
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Initialize timer state
    let initial_state = TimerState {
        is_running: false,
        remaining_seconds: 25 * 60, // 25 minutes
        session_type: "work".to_string(),
        session_count: 1,
        work_duration: 25 * 60,
        short_break_duration: 5 * 60,
        long_break_duration: 15 * 60,
    };

    let shared_state = SharedState::new(Mutex::new(initial_state));

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_origin(Any);

    // Build router
    let app = Router::new()
        // Serve frontend
        .nest_service("/", ServeDir::new("../frontend").fallback(ServeDir::new("../frontend/index.html")))

        // API routes
        .route("/api/timer", get(get_timer).post(control_timer))
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/health", get(health_check))

        // Apply middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(shared_state);

    // Start server
    println!("🍅 Roma Timer server starting on http://{}", addr);
    println!("📱 Frontend will be available at http://localhost:{}/", port);
    println!("🔧 API available at http://localhost:{}/api/", port);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_timer(State(state): State<SharedState>) -> Json<TimerState> {
    let timer_state = state.lock().unwrap().clone();
    Json(timer_state)
}

async fn control_timer(
    State(state): State<SharedState>,
    Json(request): Json<TimerRequest>,
) -> Result<Json<TimerState>, StatusCode> {
    let mut timer_state = state.lock().unwrap();

    match request.action.as_str() {
        "start" => {
            timer_state.is_running = true;
            // Start background timer task
            let state_clone = state.clone();
            tokio::spawn(async move {
                tick_timer(state_clone).await;
            });
        }
        "pause" => {
            timer_state.is_running = false;
        }
        "reset" => {
            timer_state.is_running = false;
            timer_state.remaining_seconds = match timer_state.session_type.as_str() {
                "work" => timer_state.work_duration,
                "short_break" => timer_state.short_break_duration,
                "long_break" => timer_state.long_break_duration,
                _ => timer_state.work_duration,
            };
        }
        "skip" => {
            timer_state.is_running = false;
            // Switch to next session type
            timer_state.session_type = match timer_state.session_type.as_str() {
                "work" => "short_break".to_string(),
                "short_break" => "work".to_string(),
                "long_break" => "work".to_string(),
                _ => "work".to_string(),
            };

            // Update session count
            if timer_state.session_type == "work" {
                timer_state.session_count += 1;
            }

            // Set duration for new session type
            timer_state.remaining_seconds = match timer_state.session_type.as_str() {
                "work" => timer_state.work_duration,
                "short_break" => timer_state.short_break_duration,
                "long_break" => timer_state.long_break_duration,
                _ => timer_state.work_duration,
            };
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    Ok(Json(timer_state.clone()))
}

async fn get_settings(State(state): State<SharedState>) -> Json<HashMap<String, u32>> {
    let timer_state = state.lock().unwrap();
    let mut settings = HashMap::new();
    settings.insert("work_duration".to_string(), timer_state.work_duration);
    settings.insert("short_break_duration".to_string(), timer_state.short_break_duration);
    settings.insert("long_break_duration".to_string(), timer_state.long_break_duration);
    Json(settings)
}

async fn update_settings(
    State(state): State<SharedState>,
    Json(request): Json<SettingsRequest>,
) -> Result<Json<TimerState>, StatusCode> {
    let mut timer_state = state.lock().unwrap();

    if let Some(work_duration) = request.work_duration {
        timer_state.work_duration = work_duration;
        if timer_state.session_type == "work" && !timer_state.is_running {
            timer_state.remaining_seconds = work_duration;
        }
    }

    if let Some(short_break_duration) = request.short_break_duration {
        timer_state.short_break_duration = short_break_duration;
        if timer_state.session_type == "short_break" && !timer_state.is_running {
            timer_state.remaining_seconds = short_break_duration;
        }
    }

    if let Some(long_break_duration) = request.long_break_duration {
        timer_state.long_break_duration = long_break_duration;
        if timer_state.session_type == "long_break" && !timer_state.is_running {
            timer_state.remaining_seconds = long_break_duration;
        }
    }

    Ok(Json(timer_state.clone()))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn tick_timer(state: SharedState) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let mut timer_state = state.lock().unwrap();

        if timer_state.is_running && timer_state.remaining_seconds > 0 {
            timer_state.remaining_seconds -= 1;

            // If timer reaches zero, stop it and switch session type
            if timer_state.remaining_seconds == 0 {
                timer_state.is_running = false;

                // Switch to next session type
                timer_state.session_type = match timer_state.session_type.as_str() {
                    "work" => "short_break".to_string(),
                    "short_break" => "work".to_string(),
                    "long_break" => "work".to_string(),
                    _ => "work".to_string(),
                };

                // Update session count
                if timer_state.session_type == "work" {
                    timer_state.session_count += 1;
                }

                // Set duration for new session type
                timer_state.remaining_seconds = match timer_state.session_type.as_str() {
                    "work" => timer_state.work_duration,
                    "short_break" => timer_state.short_break_duration,
                    "long_break" => timer_state.long_break_duration,
                    _ => timer_state.work_duration,
                };
            }
        } else if !timer_state.is_running {
            break; // Exit the task if timer is paused
        }
    }
}