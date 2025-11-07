//! Session Count WebSocket Message Handlers
//!
//! Handles WebSocket messages for manual session count management.

use crate::models::user_configuration::UserConfiguration;
use crate::services::daily_reset_service::DailyResetService;
use crate::database::DatabaseManager;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error, instrument};

/// Get Session Count Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionCountRequest {
    /// User configuration ID
    pub user_id: String,
}

/// Get Session Count Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionCountResponse {
    /// Success status
    pub success: bool,
    /// Current session count
    pub current_session_count: u32,
    /// Manual session override if set
    pub manual_session_override: Option<u32>,
    /// Last reset time (UTC)
    pub last_reset_utc: Option<i64>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Set Session Count Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSessionCountRequest {
    /// User configuration ID
    pub user_id: String,
    /// New session count
    pub session_count: u32,
    /// Whether this is a manual override (true) or update (false)
    pub manual_override: bool,
}

/// Set Session Count Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSessionCountResponse {
    /// Success status
    pub success: bool,
    /// Updated session count
    pub current_session_count: u32,
    /// Manual override status
    pub manual_session_override: Option<u32>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Reset Session Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetSessionRequest {
    /// User configuration ID
    pub user_id: String,
}

/// Reset Session Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetSessionResponse {
    /// Success status
    pub success: bool,
    /// Previous session count
    pub previous_session_count: u32,
    /// New session count (should be 0)
    pub new_session_count: u32,
    /// Reset time (UTC)
    pub reset_time_utc: i64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Session Count WebSocket Handler
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
    #[instrument(skip(self))]
    pub async fn handle_get_session_count(
        &self,
        request: GetSessionCountRequest,
    ) -> GetSessionCountResponse {
        info!("Handling get session count request for user {}", request.user_id);

        // Load configuration
        let config = match self.load_user_configuration(&request.user_id).await {
            Ok(config) => config,
            Err(e) => {
                return GetSessionCountResponse {
                    success: false,
                    current_session_count: 0,
                    manual_session_override: None,
                    last_reset_utc: None,
                    error: Some(format!("Failed to load configuration: {}", e)),
                };
            }
        };

        let current_count = config.get_current_session_count();

        GetSessionCountResponse {
            success: true,
            current_session_count: current_count,
            manual_session_override: config.manual_session_override,
            last_reset_utc: config.last_daily_reset_utc,
            error: None,
        }
    }

    /// Handle set session count message
    #[instrument(skip(self))]
    pub async fn handle_set_session_count(
        &self,
        request: SetSessionCountRequest,
    ) -> SetSessionCountResponse {
        info!("Handling set session count request for user {}: {} (override: {})",
              request.user_id, request.session_count, request.manual_override);

        // Validate session count
        if request.session_count > 100 {
            return SetSessionCountResponse {
                success: false,
                current_session_count: 0,
                manual_session_override: None,
                error: Some("Session count cannot exceed 100".to_string()),
            };
        }

        // Load configuration
        let mut config = match self.load_user_configuration(&request.user_id).await {
            Ok(config) => config,
            Err(e) => {
                return SetSessionCountResponse {
                    success: false,
                    current_session_count: 0,
                    manual_session_override: None,
                    error: Some(format!("Failed to load configuration: {}", e)),
                };
            }
        };

        // Update session count
        if request.manual_override {
            match config.set_manual_session_override(Some(request.session_count)) {
                Ok(_) => {},
                Err(e) => {
                    warn!("Failed to set manual session override: {}", e);
                    return SetSessionCountResponse {
                        success: false,
                        current_session_count: 0,
                        manual_session_override: None,
                        error: Some(format!("Failed to set manual session override: {}", e)),
                    };
                }
            }
        } else {
            config.today_session_count = request.session_count;
            config.manual_session_override = None; // Clear manual override
        }

        // Save configuration
        if let Err(e) = self.save_user_configuration(&config).await {
            return SetSessionCountResponse {
                success: false,
                current_session_count: 0,
                manual_session_override: None,
                error: Some(format!("Failed to save configuration: {}", e)),
            };
        }

        let final_count = config.get_current_session_count();

        SetSessionCountResponse {
            success: true,
            current_session_count: final_count,
            manual_session_override: config.manual_session_override,
            error: None,
        }
    }

    /// Handle reset session message
    #[instrument(skip(self))]
    pub async fn handle_reset_session(
        &self,
        request: ResetSessionRequest,
    ) -> ResetSessionResponse {
        info!("Handling reset session request for user {}", request.user_id);

        // Load configuration
        let mut config = match self.load_user_configuration(&request.user_id).await {
            Ok(config) => config,
            Err(e) => {
                return ResetSessionResponse {
                    success: false,
                    previous_session_count: 0,
                    new_session_count: 0,
                    reset_time_utc: 0,
                    error: Some(format!("Failed to load configuration: {}", e)),
                };
            }
        };

        let previous_count = config.get_current_session_count();

        // Reset session count
        config.reset_session_count();

        // Save configuration
        if let Err(e) = self.save_user_configuration(&config).await {
            return ResetSessionResponse {
                success: false,
                previous_session_count: previous_count,
                new_session_count: 0,
                reset_time_utc: 0,
                error: Some(format!("Failed to save configuration: {}", e)),
            };
        }

        let reset_time = chrono::Utc::now().timestamp();

        ResetSessionResponse {
            success: true,
            previous_session_count: previous_count,
            new_session_count: 0,
            reset_time_utc: reset_time,
            error: None,
        }
    }

    /// Load user configuration from database
    async fn load_user_configuration(&self, user_id: &str) -> Result<UserConfiguration, AppError> {
        // This would typically load from the configuration service
        // For now, return a default configuration
        Ok(UserConfiguration::with_id(user_id.to_string()))
    }

    /// Save user configuration to database
    async fn save_user_configuration(&self, config: &UserConfiguration) -> Result<(), AppError> {
        // This would typically save via the configuration service
        // For now, just touch the configuration to update timestamps
        info!("Saving configuration for user {}", config.id);
        Ok(())
    }
}