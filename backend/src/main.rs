//! Roma Timer backend with WebSocket support for real-time cross-device synchronization

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};

mod config;
mod database;
mod models;
mod services;
mod api;

use config::Config;
use database::DatabaseManager;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, Method, StatusCode, Uri},
    response::{Json, Response},
    routing::{get, post},
    Router,
    middleware,
};
use axum_extra::typed_header::TypedHeader;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{SinkExt, StreamExt};
use headers::{authorization::Bearer, Authorization};
use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub is_running: bool,
    pub remaining_seconds: u32,
    pub session_type: String,
    pub session_count: u32,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub last_updated: u64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRequest {
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String, // Subject (user identifier)
    pub exp: u64,    // Expiration time
    pub iat: u64,    // Issued at
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub expires_at: u64,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsRequest {
    pub work_duration: Option<u32>,
    pub short_break_duration: Option<u32>,
    pub long_break_duration: Option<u32>,
    pub long_break_frequency: Option<u32>,
}

// WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    TimerStateUpdate(TimerState),
    TimerControl(TimerRequest),
    SettingsUpdate(SettingsRequest),
    ConnectionStatus {
        connection_id: String,
        connected: bool,
        device_count: usize,
    },
    Ping,
    Pong,
}

// Connection info
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub user_agent: Option<String>,
    pub connected_at: u64,
}

// WebSocket message sender type
type WsSender = mpsc::UnboundedSender<Message>;

// WebSocket manager
pub struct WebSocketManager {
    pub connections: Arc<Mutex<HashMap<String, Connection>>>,
    pub senders: Arc<Mutex<HashMap<String, WsSender>>>,
    pub timer_state: Arc<Mutex<TimerState>>,
    pub database: Arc<DatabaseManager>,
}

impl WebSocketManager {
    pub fn new(timer_state: Arc<Mutex<TimerState>>, database: Arc<DatabaseManager>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
            timer_state,
            database,
        }
    }

    pub async fn add_connection(&self, id: String, user_agent: Option<String>, sender: WsSender) {
        let mut connections = self.connections.lock().await;
        let mut senders = self.senders.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        connections.insert(
            id.clone(),
            Connection {
                id: id.clone(),
                user_agent,
                connected_at: now,
            },
        );

        senders.insert(id.clone(), sender);

        // Broadcast connection status
        let device_count = connections.len();
        drop(connections);
        drop(senders);
        self.broadcast_message(WsMessage::ConnectionStatus {
            connection_id: id,
            connected: true,
            device_count,
        })
        .await;
    }

    pub async fn remove_connection(&self, id: String) {
        let mut connections = self.connections.lock().await;
        let mut senders = self.senders.lock().await;
        connections.remove(&id);
        senders.remove(&id);
        let device_count = connections.len();
        drop(connections);
        drop(senders);

        // Broadcast disconnection status
        self.broadcast_message(WsMessage::ConnectionStatus {
            connection_id: id,
            connected: false,
            device_count,
        })
        .await;
    }

    pub async fn update_timer_state(&self, state: TimerState) {
        // Update the shared timer state
        {
            let mut timer_state = self.timer_state.lock().await;
            *timer_state = state.clone();
        }

        // Save to database
        if let Err(e) = self.database.save_timer_state(&state).await {
            eprintln!("Failed to save timer state to database: {e}");
        }

        // Broadcast to all connected clients
        self.broadcast_message(WsMessage::TimerStateUpdate(state))
            .await;
    }

    pub async fn broadcast_message(&self, message: WsMessage) {
        let senders = self.senders.lock().await;
        let message_text = match serde_json::to_string(&message) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Failed to serialize message: {e}");
                return;
            }
        };

        let mut disconnected_senders = Vec::new();

        for (connection_id, sender) in senders.iter() {
            if sender.send(Message::Text(message_text.clone())).is_err() {
                // Connection is broken, mark for removal
                disconnected_senders.push(connection_id.clone());
            }
        }

        drop(senders);

        // Remove disconnected senders from both connections and senders maps
        if !disconnected_senders.is_empty() {
            let mut connections = self.connections.lock().await;
            let mut senders = self.senders.lock().await;
            for connection_id in disconnected_senders {
                connections.remove(&connection_id);
                senders.remove(&connection_id);
            }
        }
    }
}

type SharedState = Arc<Mutex<TimerState>>;
type SharedWsManager = Arc<WebSocketManager>;

// Webhook notification system
async fn send_webhook_notification(
    webhook_url: &str,
    session_type: &str,
    session_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let message = match session_type {
        "work" => format!("Work session #{session_count} complete! Time for a break."),
        "short_break" => "Short break over! Ready to focus?".to_string(),
        "long_break" => "Long break complete! Ready to be productive?".to_string(),
        _ => "Timer session complete!".to_string(),
    };

    let payload = serde_json::json!({
        "title": "Roma Timer",
        "message": message,
        "session_type": session_type,
        "session_count": session_count,
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Roma-Timer/1.0")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("✅ Webhook notification sent successfully to {webhook_url}");
    } else {
        println!("⚠️  Webhook notification failed: {}", response.status());
    }

    Ok(())
}

// Authentication functions
type HmacSha256 = Hmac<Sha256>;

fn get_shared_secret() -> String {
    env::var("ROMA_TIMER_SHARED_SECRET").unwrap_or_else(|_| "default-secret-change-me".to_string())
}

fn get_pepper() -> String {
    env::var("ROMA_TIMER_PEPPER")
        .unwrap_or_else(|_| "default-pepper-change-me-in-production".to_string())
}

fn generate_salt() -> String {
    let mut rng = rand::thread_rng();
    let salt: [u8; 32] = rng.gen();
    hex::encode(salt)
}

fn hash_password(
    password: &str,
    salt: &str,
    pepper: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let combined = format!("{password}{salt}{pepper}");
    let mut mac = HmacSha256::new_from_slice(combined.as_bytes())?;
    mac.update(b"roma-timer-hash");
    let hash = mac.finalize().into_bytes();
    Ok(hex::encode(hash))
}

fn verify_password(password: &str, salt: &str, pepper: &str, stored_hash: &str) -> bool {
    match hash_password(password, salt, pepper) {
        Ok(computed_hash) => computed_hash == stored_hash,
        Err(_) => false,
    }
}

fn generate_auth_token(user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret = get_shared_secret();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = AuthClaims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + 24 * 60 * 60, // 24 hours
    };

    let claims_json = serde_json::to_string(&claims)?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(claims_json.as_bytes());
    let signature = mac.finalize().into_bytes();

    let token = format!(
        "{}.{}",
        general_purpose::STANDARD.encode(claims_json.as_bytes()),
        general_purpose::STANDARD.encode(signature)
    );

    Ok(token)
}

fn verify_auth_token(token: &str) -> Result<AuthClaims, Box<dyn std::error::Error>> {
    let secret = get_shared_secret();

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return Err("Invalid token format".into());
    }

    let claims_bytes = general_purpose::STANDARD.decode(parts[0])?;
    let signature_bytes = general_purpose::STANDARD.decode(parts[1])?;

    let claims_json = String::from_utf8(claims_bytes)?;
    let claims: AuthClaims = serde_json::from_str(&claims_json)?;

    // Check expiration
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if claims.exp < now {
        return Err("Token expired".into());
    }

    // Verify signature
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(claims_json.as_bytes());
    let expected_signature = mac.finalize().into_bytes();

    use hmac::Mac;
    if signature_bytes.len() != expected_signature.len() {
        return Err("Invalid signature".into());
    }

    // Constant-time comparison
    let mut result = 0u8;
    for (a, b) in signature_bytes.iter().zip(expected_signature.iter()) {
        result |= a ^ b;
    }

    if result != 0 {
        return Err("Invalid signature".into());
    }

    Ok(claims)
}

// Service worker cache busting middleware
async fn sw_cache_middleware(
    req: axum::extract::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = req.uri().path();

    if path == "/sw.js" {
        let mut response = next.run(req).await;

        // Set cache-busting headers for service worker
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".parse().unwrap()
        );
        response.headers_mut().insert(
            header::PRAGMA,
            "no-cache".parse().unwrap()
        );
        response.headers_mut().insert(
            header::EXPIRES,
            "0".parse().unwrap()
        );

        response
    } else {
        next.run(req).await
    }
}

// Note: Authentication middleware is currently disabled
// To enable authentication, uncomment the auth_middleware function and the middleware layer in main()
/*
// Authentication middleware
async fn auth_middleware(
    req: axum::extract::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // Skip authentication only for auth endpoints and static assets
    let path = req.uri().path();
    if path == "/api/auth/login"
        || path == "/api/auth/register"
        || path == "/"
        || path.starts_with("/static")
    {
        return Ok(next.run(req).await);
    }

    // Check Authorization header
    let auth_header = req.headers().get("authorization");

    match auth_header {
        Some(header_value) => {
            let header_str = match header_value.to_str() {
                Ok(s) => s,
                Err(_) => return Err(axum::http::StatusCode::UNAUTHORIZED),
            };

            if let Some(token) = header_str.strip_prefix("Bearer ") {
                match verify_auth_token(token) {
                    Ok(_) => Ok(next.run(req).await),
                    Err(_) => Err(axum::http::StatusCode::UNAUTHORIZED),
                }
            } else {
                Err(axum::http::StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(axum::http::StatusCode::UNAUTHORIZED),
    }
}
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize logging with the configured log level
    let log_level = match config.log_level.as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    println!("🚀 Starting Roma Timer backend on {}:{}", config.host, config.port);
    println!("🗄️  Database type: {}", config.database_type);
    println!("📊 Database URL: {}", config.masked_database_url());

    // Initialize database manager
    let database_manager = Arc::new(DatabaseManager::new(&config.database_url).await?);
    database_manager.migrate().await?;
    println!("✅ Database initialized and migrated successfully");

    // Load initial state from database or use defaults
    let initial_state = match database_manager.get_current_timer_state().await? {
        Some(state) => {
            println!("📋 Loaded timer state from database");
            state
        }
        None => {
            println!("🆕 No saved state found, using defaults");
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            TimerState {
                is_running: false,
                remaining_seconds: 25 * 60, // 25 minutes
                session_type: "work".to_string(),
                session_count: 1,
                work_duration: 25 * 60,
                short_break_duration: 5 * 60,
                long_break_duration: 15 * 60,
                last_updated: now,
            }
        }
    };

    let shared_state = SharedState::new(Mutex::new(initial_state.clone()));
    let ws_manager = SharedWsManager::new(WebSocketManager::new(shared_state.clone(), database_manager.clone()));

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::UPGRADE,
            header::CONNECTION,
            header::SEC_WEBSOCKET_KEY,
            header::SEC_WEBSOCKET_VERSION,
            header::SEC_WEBSOCKET_PROTOCOL,
        ])
        .allow_origin(Any);

    // Build router
    let app = Router::new()
        // Serve frontend
        .nest_service(
            "/",
            ServeDir::new("../frontend").fallback(ServeDir::new("../frontend/index.html")),
        )
        // API routes
        .route("/api/timer", get(get_timer).post(control_timer))
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/health", get(health_check))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        // Apply service worker cache busting middleware
        .layer(middleware::from_fn(sw_cache_middleware))
        // Apply authentication middleware (temporarily disabled due to type issues)
        // .layer(middleware::from_fn(auth_middleware))
        // Apply other middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state((shared_state, ws_manager));

    // Start server
    let addr = config.bind_address();
    println!("🍅 Roma Timer server starting on http://{}", addr);
    println!("📱 Frontend will be available at http://localhost:{}/", config.port);
    println!("🔧 API available at http://localhost:{}/api/", config.port);
    println!("🌐 WebSocket available at ws://localhost:{}/ws", config.port);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_timer(
    State((state, _)): State<(SharedState, SharedWsManager)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<TimerState>, StatusCode> {
    // Check authentication
    let auth_header = headers.get("authorization");
    match auth_header {
        Some(header_value) => {
            if let Ok(header_str) = header_value.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    if verify_auth_token(token).is_err() {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    }

    let timer_state = state.lock().await.clone();
    Ok(Json(timer_state))
}

async fn control_timer(
    State((state, ws_manager)): State<(SharedState, SharedWsManager)>,
    headers: axum::http::HeaderMap,
    Json(request): Json<TimerRequest>,
) -> Result<Json<TimerState>, StatusCode> {
    // Check authentication
    let auth_header = headers.get("authorization");
    match auth_header {
        Some(header_value) => {
            if let Ok(header_str) = header_value.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    if verify_auth_token(token).is_err() {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    }
    let mut timer_state = state.lock().await;

    match request.action.as_str() {
        "start" => {
            timer_state.is_running = true;
            timer_state.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Start background timer task
            let state_clone = state.clone();
            let ws_manager_clone = ws_manager.clone();
            tokio::spawn(async move {
                tick_timer(state_clone, ws_manager_clone).await;
            });
        }
        "pause" => {
            timer_state.is_running = false;
            timer_state.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        "reset" => {
            timer_state.is_running = false;
            timer_state.remaining_seconds = match timer_state.session_type.as_str() {
                "work" => timer_state.work_duration,
                "short_break" => timer_state.short_break_duration,
                "long_break" => timer_state.long_break_duration,
                _ => timer_state.work_duration,
            };
            timer_state.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
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

            timer_state.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    let updated_state = timer_state.clone();
    drop(timer_state);

    // Broadcast state change via WebSocket
    ws_manager.update_timer_state(updated_state.clone()).await;

    Ok(Json(updated_state))
}

async fn get_settings(
    State((state, _)): State<(SharedState, SharedWsManager)>,
    headers: axum::http::HeaderMap,
) -> Result<Json<HashMap<String, u32>>, StatusCode> {
    // Check authentication
    let auth_header = headers.get("authorization");
    match auth_header {
        Some(header_value) => {
            if let Ok(header_str) = header_value.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    if verify_auth_token(token).is_err() {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    }

    let timer_state = state.lock().await;
    let mut settings = HashMap::new();
    settings.insert("work_duration".to_string(), timer_state.work_duration);
    settings.insert(
        "short_break_duration".to_string(),
        timer_state.short_break_duration,
    );
    settings.insert(
        "long_break_duration".to_string(),
        timer_state.long_break_duration,
    );
    Ok(Json(settings))
}

async fn update_settings(
    State((state, ws_manager)): State<(SharedState, SharedWsManager)>,
    headers: axum::http::HeaderMap,
    Json(request): Json<SettingsRequest>,
) -> Result<Json<TimerState>, StatusCode> {
    // Check authentication
    let auth_header = headers.get("authorization");
    match auth_header {
        Some(header_value) => {
            if let Ok(header_str) = header_value.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ") {
                    if verify_auth_token(token).is_err() {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    }
    let mut timer_state = state.lock().await;

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

    timer_state.last_updated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let updated_state = timer_state.clone();
    drop(timer_state);

    // Broadcast settings change via WebSocket
    ws_manager
        .broadcast_message(WsMessage::SettingsUpdate(request))
        .await;

    Ok(Json(updated_state))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn register_user(
    State((_, ws_manager)): State<(SharedState, SharedWsManager)>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let database = &ws_manager.database;

    // Validate input
    if request.username.len() < 3 || request.password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate salt and hash password
    let salt = generate_salt();
    let pepper = get_pepper();

    let password_hash = match hash_password(&request.password, &salt, &pepper) {
        Ok(hash) => hash,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Create user
    match database.create_user(&request.username, &password_hash, &salt).await {
        Ok(user_id) => {
            println!("✅ User registered successfully: {}", request.username);
            Ok(Json(RegisterResponse {
                message: "User registered successfully".to_string(),
                user_id,
                username: request.username.clone(),
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to register user: {e}");
            if e.to_string().contains("Username already exists") {
                return Err(StatusCode::CONFLICT);
            }
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn login_user(
    State((_, ws_manager)): State<(SharedState, SharedWsManager)>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let database = &ws_manager.database;

    // Get user by username
    match database.get_user_by_username(&request.username).await {
        Ok(Some(user)) => {
            // Verify password
            let pepper = get_pepper();
            if verify_password(&request.password, &user.salt, &pepper, &user.password_hash) {
                // Generate auth token
                let user_id = user.id.clone();
                match generate_auth_token(&user_id) {
                    Ok(token) => {
                        let claims = verify_auth_token(&token).unwrap(); // Should succeed
                        println!("✅ User logged in successfully: {}", request.username);
                        Ok(Json(AuthResponse {
                            token,
                            user_id: user_id.clone(),
                            username: user.username,
                            expires_at: claims.exp,
                        }))
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                println!("❌ Invalid password for user: {}", request.username);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Ok(None) => {
            println!("❌ User not found: {}", request.username);
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Note: get_auth_token function removed as it's no longer needed with proper authentication

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State((state, ws_manager)): State<(SharedState, SharedWsManager)>,
    auth_headers: Option<TypedHeader<Authorization<Bearer>>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    uri: Uri,
) -> Response {
    // Try to get token from Authorization header first
    let token = if let Some(auth_headers) = auth_headers {
        Some((*auth_headers).token().to_string())
    } else {
        // Fallback to query parameter for JavaScript WebSocket API
        if let Some(query) = uri.query() {
            let params: std::collections::HashMap<String, String> =
                url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();
            params.get("token").cloned()
        } else {
            None
        }
    };

    // Check if token is present
    if let Some(token) = token {
        // Verify the token and extract user info
        match verify_auth_token(&token) {
            Ok(claims) => {
                let user_id = claims.sub;
                ws.on_upgrade(move |socket| {
                    handle_websocket(
                        socket,
                        state,
                        ws_manager,
                        user_agent.map(|ua| ua.to_string()),
                        user_id,
                    )
                })
            }
            Err(_) => {
                // Return unauthorized response
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(axum::body::Body::from("Invalid or expired token"))
                    .unwrap()
            }
        }
    } else {
        // No Authorization header or token provided
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from(
                "Authorization required for WebSocket connection",
            ))
            .unwrap()
    }
}

async fn handle_websocket(
    socket: WebSocket,
    state: SharedState,
    ws_manager: SharedWsManager,
    user_agent: Option<String>,
    user_id: String,
) {
    let connection_id = Uuid::new_v4().to_string();

    println!("WebSocket connected: {connection_id} for user {user_id} (UA: {user_agent:?})");

    // Create a channel for this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Add connection to manager with the sender
    ws_manager
        .add_connection(connection_id.clone(), user_agent.clone(), tx)
        .await;

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send initial timer state
    let timer_state = state.lock().await.clone();
    let initial_msg = WsMessage::TimerStateUpdate(timer_state);
    if let Ok(msg_text) = serde_json::to_string(&initial_msg) {
        let _ = ws_sender.send(Message::Text(msg_text)).await;
    }

    // Send connection status
    let connection_msg = WsMessage::ConnectionStatus {
        connection_id: connection_id.clone(),
        connected: true,
        device_count: ws_manager.connections.lock().await.len(),
    };
    if let Ok(msg_text) = serde_json::to_string(&connection_msg) {
        let _ = ws_sender.send(Message::Text(msg_text)).await;
    }

    // Task to forward messages from the channel to the WebSocket
    let connection_id_clone = connection_id.clone();
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        println!("WebSocket forward task ended for: {connection_id_clone}");
    });

    // Task to handle incoming messages from the WebSocket
    let state_clone = state.clone();
    let ws_manager_clone = ws_manager.clone();
    let connection_id_clone2 = connection_id.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        if let Ok(ws_message) = serde_json::from_str::<WsMessage>(&text) {
                            match ws_message {
                                WsMessage::TimerControl(request) => {
                                    // Handle timer control from WebSocket
                                    let mut timer_state = state_clone.lock().await;

                                    match request.action.as_str() {
                                        "start" => {
                                            timer_state.is_running = true;
                                            timer_state.last_updated = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();

                                            let state_clone2 = state_clone.clone();
                                            let ws_manager_clone2 = ws_manager_clone.clone();
                                            tokio::spawn(async move {
                                                tick_timer(state_clone2, ws_manager_clone2).await;
                                            });
                                        }
                                        "pause" => {
                                            timer_state.is_running = false;
                                            timer_state.last_updated = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();
                                        }
                                        "reset" => {
                                            timer_state.is_running = false;
                                            timer_state.remaining_seconds = match timer_state
                                                .session_type
                                                .as_str()
                                            {
                                                "work" => timer_state.work_duration,
                                                "short_break" => timer_state.short_break_duration,
                                                "long_break" => timer_state.long_break_duration,
                                                _ => timer_state.work_duration,
                                            };
                                            timer_state.last_updated = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();
                                        }
                                        "skip" => {
                                            timer_state.is_running = false;
                                            timer_state.session_type =
                                                match timer_state.session_type.as_str() {
                                                    "work" => "short_break".to_string(),
                                                    "short_break" => "work".to_string(),
                                                    "long_break" => "work".to_string(),
                                                    _ => "work".to_string(),
                                                };

                                            if timer_state.session_type == "work" {
                                                timer_state.session_count += 1;
                                            }

                                            timer_state.remaining_seconds = match timer_state
                                                .session_type
                                                .as_str()
                                            {
                                                "work" => timer_state.work_duration,
                                                "short_break" => timer_state.short_break_duration,
                                                "long_break" => timer_state.long_break_duration,
                                                _ => timer_state.work_duration,
                                            };

                                            timer_state.last_updated = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();
                                        }
                                        _ => {}
                                    }

                                    let updated_state = timer_state.clone();
                                    drop(timer_state);

                                    // Broadcast state change
                                    ws_manager_clone.update_timer_state(updated_state).await;
                                }
                                WsMessage::SettingsUpdate(request) => {
                                    // Handle settings update from WebSocket
                                    let mut timer_state = state_clone.lock().await;

                                    if let Some(work_duration) = request.work_duration {
                                        timer_state.work_duration = work_duration;
                                        if timer_state.session_type == "work"
                                            && !timer_state.is_running
                                        {
                                            timer_state.remaining_seconds = work_duration;
                                        }
                                    }

                                    if let Some(short_break_duration) = request.short_break_duration
                                    {
                                        timer_state.short_break_duration = short_break_duration;
                                        if timer_state.session_type == "short_break"
                                            && !timer_state.is_running
                                        {
                                            timer_state.remaining_seconds = short_break_duration;
                                        }
                                    }

                                    if let Some(long_break_duration) = request.long_break_duration {
                                        timer_state.long_break_duration = long_break_duration;
                                        if timer_state.session_type == "long_break"
                                            && !timer_state.is_running
                                        {
                                            timer_state.remaining_seconds = long_break_duration;
                                        }
                                    }

                                    timer_state.last_updated = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs();

                                    drop(timer_state);

                                    // Broadcast settings change
                                    ws_manager_clone
                                        .broadcast_message(WsMessage::SettingsUpdate(request))
                                        .await;
                                }
                                WsMessage::Ping => {
                                    // Respond with pong directly to this client
                                    if let Ok(pong_msg) = serde_json::to_string(&WsMessage::Pong) {
                                        if let Some(sender) = ws_manager_clone
                                            .senders
                                            .lock()
                                            .await
                                            .get(&connection_id_clone2)
                                        {
                                            let _ = sender.send(Message::Text(pong_msg));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = forward_task => {},
        _ = receive_task => {},
    }

    // Remove connection when disconnected
    let connection_id_clone = connection_id.clone();
    ws_manager.remove_connection(connection_id).await;
    println!("WebSocket disconnected: {connection_id_clone}");
}

async fn tick_timer(state: SharedState, ws_manager: SharedWsManager) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let mut timer_state = state.lock().await;

        if timer_state.is_running && timer_state.remaining_seconds > 0 {
            timer_state.remaining_seconds -= 1;
            timer_state.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // If timer reaches zero, stop it and switch session type
            if timer_state.remaining_seconds == 0 {
                timer_state.is_running = false;

                // Store the old session type for notifications
                let completed_session_type = timer_state.session_type.clone();
                let completed_session_count = timer_state.session_count;

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

                // Send webhook notification for completed session
                // Note: This is a simple implementation - in production you'd want to get webhook_url from database
                if let Ok(webhook_url) = std::env::var("ROMA_TIMER_WEBHOOK_URL") {
                    let webhook_url_clone = webhook_url.clone();
                    let session_type_clone = completed_session_type.clone();
                    let session_count_clone = completed_session_count;

                    tokio::spawn(async move {
                        if let Err(e) = send_webhook_notification(
                            &webhook_url_clone,
                            &session_type_clone,
                            session_count_clone,
                        )
                        .await
                        {
                            eprintln!("Failed to send webhook notification: {e}");
                        }
                    });
                }
            }

            let updated_state = timer_state.clone();
            drop(timer_state);

            // Broadcast state change
            ws_manager.update_timer_state(updated_state).await;
        } else if !timer_state.is_running {
            break; // Exit the task if timer is paused
        }
    }
}
