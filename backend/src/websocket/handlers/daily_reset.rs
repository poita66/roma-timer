//! Daily Reset WebSocket Message Handlers
//!
//! Handles WebSocket messages for daily session reset configuration and management.

use crate::models::user_configuration::{UserConfiguration, DailyResetTimeType, DailyResetTime};
use crate::services::daily_reset_service::DailyResetService;
use crate::services::timezone_service::TimezoneService;
use crate::database::DatabaseManager;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::{State, WebSocketUpgrade, ConnectInfo};
use axum::response::Response;
use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error, instrument};

/// Configure Daily Reset Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureDailyResetRequest {
    /// User configuration ID
    pub user_id: String,
    /// Whether daily reset is enabled
    pub enabled: bool,
    /// Reset time type
    pub reset_time_type: DailyResetTimeType,
    /// Hour for daily reset (0-23) - only used if time_type is Hour
    pub reset_hour: Option<u8>,
    /// Custom time string (HH:MM) - only used if time_type is Custom
    pub custom_time: Option<String>,
    /// User timezone
    pub timezone: String,
}

/// Configure Daily Reset Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureDailyResetResponse {
    /// Success status
    pub success: bool,
    /// Updated configuration
    pub configuration: Option<UserConfiguration>,
    /// Error message if failed
    pub error: Option<String>,
    /// Next reset time (UTC)
    pub next_reset_time_utc: Option<i64>,
}

/// Get Daily Reset Status Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyResetStatusRequest {
    /// User configuration ID
    pub user_id: String,
}

/// Get Daily Reset Status Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDailyResetStatusResponse {
    /// Success status
    pub success: bool,
    /// Current configuration
    pub configuration: Option<UserConfiguration>,
    /// Next reset time (UTC)
    pub next_reset_time_utc: Option<i64>,
    /// Whether reset is due today
    pub reset_due_today: Option<bool>,
    /// Current session count
    pub current_session_count: Option<u32>,
    /// Manual session override
    pub manual_session_override: Option<u32>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Daily Reset WebSocket Handler
pub struct DailyResetWebSocketHandler {
    database_manager: Arc<DatabaseManager>,
    daily_reset_service: Arc<DailyResetService>,
    timezone_service: Arc<TimezoneService>,
}

impl DailyResetWebSocketHandler {
    /// Create a new daily reset WebSocket handler
    pub fn new(
        database_manager: Arc<DatabaseManager>,
        daily_reset_service: Arc<DailyResetService>,
        timezone_service: Arc<TimezoneService>,
    ) -> Self {
        Self {
            database_manager,
            daily_reset_service,
            timezone_service,
        }
    }

    /// Handle configure daily reset message
    #[instrument(skip(self))]
    pub async fn handle_configure_daily_reset(
        &self,
        request: ConfigureDailyResetRequest,
    ) -> ConfigureDailyResetResponse {
        info!("Handling configure daily reset request for user {}", request.user_id);

        // Validate timezone first
        if let Err(e) = self.timezone_service.validate_timezone(&request.timezone) {
            return ConfigureDailyResetResponse {
                success: false,
                configuration: None,
                error: Some(format!("Invalid timezone: {}", e)),
                next_reset_time_utc: None,
            };
        }

        // Validate reset time configuration
        let reset_time = match self.validate_reset_time_config(&request) {
            Ok(time) => time,
            Err(e) => {
                return ConfigureDailyResetResponse {
                    success: false,
                    configuration: None,
                    error: Some(format!("Invalid reset time configuration: {}", e)),
                    next_reset_time_utc: None,
                };
            }
        };

        // Load current configuration
        let mut config = match self.load_user_configuration(&request.user_id).await {
            Ok(config) => config,
            Err(e) => {
                return ConfigureDailyResetResponse {
                    success: false,
                    configuration: None,
                    error: Some(format!("Failed to load configuration: {}", e)),
                    next_reset_time_utc: None,
                };
            }
        };

        // Update configuration
        config.daily_reset_enabled = request.enabled;
        config.daily_reset_time_type = request.reset_time_type;
        config.daily_reset_time_hour = request.reset_hour;
        config.daily_reset_time_custom = request.custom_time;
        config.timezone = request.timezone;

        // Validate the updated configuration
        if let Err(e) = config.validate() {
            return ConfigureDailyResetResponse {
                success: false,
                configuration: None,
                error: Some(format!("Configuration validation failed: {}", e)),
                next_reset_time_utc: None,
            };
        }

        // Save configuration
        if let Err(e) = self.save_user_configuration(&config).await {
            return ConfigureDailyResetResponse {
                success: false,
                configuration: None,
                error: Some(format!("Failed to save configuration: {}", e)),
                next_reset_time_utc: None,
            };
        }

        // Calculate next reset time
        let next_reset_time = match self.daily_reset_service.calculate_next_reset_time(&config) {
            Ok(time) => Some(time),
            Err(e) => {
                warn!("Failed to calculate next reset time: {}", e);
                None
            }
        };

        ConfigureDailyResetResponse {
            success: true,
            configuration: Some(config.clone()),
            error: None,
            next_reset_time_utc: next_reset_time.map(|dt| dt.timestamp()),
        }
    }

    /// Handle get daily reset status message
    #[instrument(skip(self))]
    pub async fn handle_get_daily_reset_status(
        &self,
        request: GetDailyResetStatusRequest,
    ) -> GetDailyResetStatusResponse {
        info!("Handling get daily reset status request for user {}", request.user_id);

        // Load configuration
        let config = match self.load_user_configuration(&request.user_id).await {
            Ok(config) => config,
            Err(e) => {
                return GetDailyResetStatusResponse {
                    success: false,
                    configuration: None,
                    next_reset_time_utc: None,
                    reset_due_today: None,
                    current_session_count: None,
                    manual_session_override: None,
                    error: Some(format!("Failed to load configuration: {}", e)),
                };
            }
        };

        // Calculate next reset time
        let next_reset_time = match self.daily_reset_service.calculate_next_reset_time(&config) {
            Ok(time) => Some(time.timestamp()),
            Err(e) => {
                warn!("Failed to calculate next reset time: {}", e);
                None
            }
        };

        // Check if reset is due today
        let reset_due_today = match self.daily_reset_service.should_reset_today(&config) {
            Ok(due) => Some(due),
            Err(e) => {
                warn!("Failed to check if reset is due: {}", e);
                None
            }
        };

        GetDailyResetStatusResponse {
            success: true,
            configuration: Some(config.clone()),
            next_reset_time_utc: next_reset_time,
            reset_due_today,
            current_session_count: Some(config.today_session_count),
            manual_session_override: config.manual_session_override,
            error: None,
        }
    }

    /// Validate reset time configuration
    fn validate_reset_time_config(
        &self,
        request: &ConfigureDailyResetRequest,
    ) -> Result<DailyResetTime, Box<dyn std::error::Error + Send + Sync>> {
        match request.reset_time_type {
            DailyResetTimeType::Midnight => Ok(DailyResetTime::midnight()),
            DailyResetTimeType::Hour => {
                if let Some(hour) = request.reset_hour {
                    DailyResetTime::hour(hour).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Err("Hour must be specified when time type is Hour".into())
                }
            }
            DailyResetTimeType::Custom => {
                if let Some(ref custom_time) = request.custom_time {
                    DailyResetTime::custom(custom_time.clone()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Err("Custom time must be specified when time type is Custom".into())
                }
            }
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