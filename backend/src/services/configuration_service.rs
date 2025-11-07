//! Configuration Service
//!
//! Handles user configuration management, persistence, and real-time synchronization.

use crate::models::user_configuration::{UserConfiguration, UserConfigurationError};
use crate::services::websocket_service::{WebSocketService, WebSocketMessage};
use crate::database::{DatabaseManager, connection::DatabasePool};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Database row structure for user configurations
#[derive(Debug, sqlx::FromRow)]
struct UserConfigurationRow {
    id: Option<String>,
    work_duration: i64,
    short_break_duration: i64,
    long_break_duration: i64,
    long_break_frequency: i64,
    notifications_enabled: bool,
    webhook_url: Option<String>,
    wait_for_interaction: bool,
    theme: String,
    // Daily session reset fields
    timezone: String,
    daily_reset_time_type: String,
    daily_reset_time_hour: Option<i64>,
    daily_reset_time_custom: Option<String>,
    daily_reset_enabled: bool,
    last_daily_reset_utc: Option<i64>,
    today_session_count: i64,
    manual_session_override: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

/// Configuration service responsible for managing user settings
#[derive(Debug, Clone)]
pub struct ConfigurationService {
    /// Database manager
    database_manager: Arc<DatabaseManager>,

    /// In-memory cache of current configuration
    config_cache: Arc<RwLock<UserConfiguration>>,

    /// WebSocket service for real-time updates
    websocket_service: WebSocketService,
}

/// Configuration update request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigurationUpdate {
    /// Work session duration in seconds
    pub work_duration: Option<u32>,

    /// Short break duration in seconds
    pub short_break_duration: Option<u32>,

    /// Long break duration in seconds
    pub long_break_duration: Option<u32>,

    /// Number of work sessions before long break
    pub long_break_frequency: Option<u32>,

    /// Whether browser notifications are enabled
    pub notifications_enabled: Option<bool>,

    /// Optional webhook URL for notifications
    pub webhook_url: Option<Option<String>>,

    /// Whether to wait for user interaction before starting next session
    pub wait_for_interaction: Option<bool>,

    /// UI theme preference
    pub theme: Option<String>,
}

/// Configuration service errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationServiceError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Configuration validation error: {0}")]
    Validation(#[from] UserConfigurationError),

    #[error("Configuration not found")]
    NotFound,

    #[error("Invalid theme: {0}")]
    InvalidTheme(String),

    #[error("WebSocket broadcast error: {0}")]
    WebSocket(String),
}

impl ConfigurationService {
    /// Create a new configuration service
    pub async fn new(
        database_manager: Arc<DatabaseManager>,
        websocket_service: WebSocketService,
    ) -> Result<Self, ConfigurationServiceError> {
        let service = Self {
            database_manager: database_manager.clone(),
            config_cache: Arc::new(RwLock::new(UserConfiguration::new())),
            websocket_service,
        };

        // Load existing configuration or create default
        service.load_configuration().await?;

        info!("Configuration service initialized with {} database", database_manager.database_type);
        Ok(service)
    }

    
    /// Load configuration from database or create default
    async fn load_configuration(&self) -> Result<(), ConfigurationServiceError> {
        debug!("Loading configuration from {} database", self.database_manager.database_type);

        // Try to load configuration from database
        let query = sqlx::query_as::<_, UserConfigurationRow>(
            r#"
            SELECT id, work_duration, short_break_duration, long_break_duration,
                   long_break_frequency, notifications_enabled, webhook_url,
                   wait_for_interaction, theme, timezone, daily_reset_time_type,
                   daily_reset_time_hour, daily_reset_time_custom, daily_reset_enabled,
                   last_daily_reset_utc, today_session_count, manual_session_override,
                   created_at, updated_at
            FROM user_configurations
            ORDER BY updated_at DESC
            LIMIT 1
            "#
        );

        match query.fetch_one(match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        }).await {
            Ok(row) => {
                let config = UserConfiguration {
                    id: row.id.expect("Database row missing id"),
                    work_duration: row.work_duration as u32,
                    short_break_duration: row.short_break_duration as u32,
                    long_break_duration: row.long_break_duration as u32,
                    long_break_frequency: row.long_break_frequency as u32,
                    notifications_enabled: row.notifications_enabled,
                    webhook_url: row.webhook_url,
                    wait_for_interaction: row.wait_for_interaction,
                    theme: match row.theme.as_str() {
                        "Dark" => crate::models::user_configuration::Theme::Dark,
                        _ => crate::models::user_configuration::Theme::Light,
                    },
                    // Daily session reset fields
                    timezone: row.timezone,
                    daily_reset_time_type: match row.daily_reset_time_type.as_str() {
                        "hour" => crate::models::user_configuration::DailyResetTimeType::Hour,
                        "custom" => crate::models::user_configuration::DailyResetTimeType::Custom,
                        _ => crate::models::user_configuration::DailyResetTimeType::Midnight,
                    },
                    daily_reset_time_hour: row.daily_reset_time_hour.map(|x| x as u8),
                    daily_reset_time_custom: row.daily_reset_time_custom,
                    daily_reset_enabled: row.daily_reset_enabled,
                    last_daily_reset_utc: row.last_daily_reset_utc.map(|x| x as u64),
                    today_session_count: row.today_session_count as u32,
                    manual_session_override: row.manual_session_override.map(|x| x as u32),
                    created_at: row.created_at as u64,
                    updated_at: row.updated_at as u64,
                };

                debug!("Configuration loaded from database");
                *self.config_cache.write().await = config;
            }
            Err(_) => {
                // No configuration found, use default
                let default_config = UserConfiguration::new();
                debug!("No configuration found in database, using default");
                *self.config_cache.write().await = default_config.clone();

                // Save default configuration to database
                if let Err(e) = self.save_configuration(&default_config).await {
                    warn!("Failed to save default configuration: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get current user configuration
    pub async fn get_configuration(&self) -> Result<UserConfiguration, ConfigurationServiceError> {
        let config = self.config_cache.read().await;
        Ok(config.clone())
    }

    /// Update user configuration
    pub async fn update_configuration(
        &self,
        update: ConfigurationUpdate,
    ) -> Result<UserConfiguration, ConfigurationServiceError> {
        debug!("Updating configuration: {:?}", update);

        let mut config = self.config_cache.write().await;

        // Apply updates with validation
        if let Some(work_duration) = update.work_duration {
            config.set_work_duration(work_duration)?;
        }

        if let Some(short_break_duration) = update.short_break_duration {
            config.set_short_break_duration(short_break_duration)?;
        }

        if let Some(long_break_duration) = update.long_break_duration {
            config.set_long_break_duration(long_break_duration)?;
        }

        if let Some(long_break_frequency) = update.long_break_frequency {
            config.set_long_break_frequency(long_break_frequency)?;
        }

        if let Some(notifications_enabled) = update.notifications_enabled {
            config.notifications_enabled = notifications_enabled;
            config.touch();
        }

        if let Some(webhook_url) = update.webhook_url {
            config.set_webhook_url(webhook_url)?;
        }

        if let Some(wait_for_interaction) = update.wait_for_interaction {
            config.wait_for_interaction = wait_for_interaction;
            config.touch();
        }

        if let Some(theme_str) = update.theme {
            let theme = match theme_str.as_str() {
                "Light" => crate::models::user_configuration::Theme::Light,
                "Dark" => crate::models::user_configuration::Theme::Dark,
                _ => return Err(ConfigurationServiceError::InvalidTheme(theme_str)),
            };
            config.set_theme(theme);
        }

        // Validate complete configuration
        config.validate()?;

        // Save to database
        self.save_configuration(&config).await?;

        // Broadcast update to all connected clients
        if let Err(e) = self.broadcast_configuration_update(&config).await {
            warn!("Failed to broadcast configuration update: {}", e);
        }

        let updated_config = config.clone();
        drop(config); // Release the lock

        info!("Configuration updated successfully");
        Ok(updated_config)
    }

    /// Save configuration to database
    async fn save_configuration(&self, config: &UserConfiguration) -> Result<(), ConfigurationServiceError> {
        debug!("Saving configuration to {} database", self.database_manager.database_type);

        let theme_str = match config.theme {
            crate::models::user_configuration::Theme::Light => "Light",
            crate::models::user_configuration::Theme::Dark => "Dark",
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Use UPSERT (INSERT OR REPLACE for SQLite, ON CONFLICT for PostgreSQL)
        let query = match self.database_manager.database_type {
            crate::database::DatabaseType::Sqlite => {
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO user_configurations
                    (id, work_duration, short_break_duration, long_break_duration,
                     long_break_frequency, notifications_enabled, webhook_url,
                     wait_for_interaction, theme, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(&config.id)
                .bind(config.work_duration as i64)
                .bind(config.short_break_duration as i64)
                .bind(config.long_break_duration as i64)
                .bind(config.long_break_frequency as i64)
                .bind(config.notifications_enabled)
                .bind(&config.webhook_url)
                .bind(config.wait_for_interaction)
                .bind(theme_str)
                .bind(config.created_at as i64)
                .bind(now)
            }
            crate::database::DatabaseType::Postgres => {
                sqlx::query(
                    r#"
                    INSERT INTO user_configurations
                    (id, work_duration, short_break_duration, long_break_duration,
                     long_break_frequency, notifications_enabled, webhook_url,
                     wait_for_interaction, theme, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    ON CONFLICT (id) DO UPDATE SET
                        work_duration = EXCLUDED.work_duration,
                        short_break_duration = EXCLUDED.short_break_duration,
                        long_break_duration = EXCLUDED.long_break_duration,
                        long_break_frequency = EXCLUDED.long_break_frequency,
                        notifications_enabled = EXCLUDED.notifications_enabled,
                        webhook_url = EXCLUDED.webhook_url,
                        wait_for_interaction = EXCLUDED.wait_for_interaction,
                        theme = EXCLUDED.theme,
                        updated_at = EXCLUDED.updated_at
                    "#
                )
                .bind(&config.id)
                .bind(config.work_duration as i64)
                .bind(config.short_break_duration as i64)
                .bind(config.long_break_duration as i64)
                .bind(config.long_break_frequency as i64)
                .bind(config.notifications_enabled)
                .bind(&config.webhook_url)
                .bind(config.wait_for_interaction)
                .bind(theme_str)
                .bind(config.created_at as i64)
                .bind(now)
            }
        };

        query.execute(match &self.database_manager.pool {
            DatabasePool::Sqlite(pool) => pool,
        }).await
            .map_err(|e| anyhow::anyhow!("Failed to save configuration: {}", e))?;

        debug!("Configuration saved successfully to database");
        Ok(())
    }

    /// Broadcast configuration update to all connected clients
    async fn broadcast_configuration_update(
        &self,
        config: &UserConfiguration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = WebSocketMessage::ConfigurationUpdate {
            payload: serde_json::json!({
                "id": config.id,
                "workDuration": config.work_duration,
                "shortBreakDuration": config.short_break_duration,
                "longBreakDuration": config.long_break_duration,
                "longBreakFrequency": config.long_break_frequency,
                "notificationsEnabled": config.notifications_enabled,
                "webhookUrl": config.webhook_url,
                "waitForInteraction": config.wait_for_interaction,
                "theme": match config.theme {
                    crate::models::user_configuration::Theme::Light => "Light",
                    crate::models::user_configuration::Theme::Dark => "Dark",
                },
                "createdAt": config.created_at,
                "updatedAt": config.updated_at,
            }),
        };

        // Send broadcast message
        if let Err(e) = self.websocket_service.broadcast_message(message).await {
            warn!("Failed to broadcast configuration update: {}", e);
        }

        debug!("Configuration update broadcasted to all connected clients");
        Ok(())
    }

    /// Reset configuration to defaults
    pub async fn reset_to_defaults(&self) -> Result<UserConfiguration, ConfigurationServiceError> {
        info!("Resetting configuration to defaults");

        let default_config = UserConfiguration::new();
        self.update_configuration(ConfigurationUpdate {
            work_duration: Some(default_config.work_duration),
            short_break_duration: Some(default_config.short_break_duration),
            long_break_duration: Some(default_config.long_break_duration),
            long_break_frequency: Some(default_config.long_break_frequency),
            notifications_enabled: Some(default_config.notifications_enabled),
            webhook_url: Some(None),
            wait_for_interaction: Some(default_config.wait_for_interaction),
            theme: Some(match default_config.theme {
                crate::models::user_configuration::Theme::Light => "Light".to_string(),
                crate::models::user_configuration::Theme::Dark => "Dark".to_string(),
            }),
        })
        .await
    }

    /// Get work duration for timer sessions
    pub async fn get_work_duration(&self) -> Result<u32, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.work_duration)
    }

    /// Get short break duration for timer sessions
    pub async fn get_short_break_duration(&self) -> Result<u32, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.short_break_duration)
    }

    /// Get long break duration for timer sessions
    pub async fn get_long_break_duration(&self) -> Result<u32, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.long_break_duration)
    }

    /// Get long break frequency
    pub async fn get_long_break_frequency(&self) -> Result<u32, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.long_break_frequency)
    }

    /// Check if notifications should be sent
    pub async fn should_send_notifications(&self) -> Result<bool, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.should_send_notifications())
    }

    /// Get webhook URL if configured
    pub async fn get_webhook_url(&self) -> Result<Option<String>, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.webhook_url.clone())
    }

    /// Check if should wait for interaction
    pub async fn should_wait_for_interaction(&self) -> Result<bool, ConfigurationServiceError> {
        let config = self.get_configuration().await?;
        Ok(config.wait_for_interaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_configuration::Theme;
    use sqlx::SqlitePool;

    async fn create_test_service() -> (ConfigurationService, SqlitePool) {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create mock websocket service (for testing we can create a simple one)
        let websocket_service = WebSocketService::new(pool.clone());

        let service = ConfigurationService::new(pool.clone(), websocket_service)
            .await
            .unwrap();

        (service, pool)
    }

    #[tokio::test]
    async fn test_configuration_creation() {
        let (service, _pool) = create_test_service().await;

        let config = service.get_configuration().await.unwrap();
        assert_eq!(config.work_duration, 1500); // 25 minutes
        assert_eq!(config.short_break_duration, 300); // 5 minutes
        assert_eq!(config.long_break_duration, 900); // 15 minutes
        assert_eq!(config.long_break_frequency, 4);
        assert!(config.notifications_enabled);
        assert_eq!(config.theme, Theme::Light);
    }

    #[tokio::test]
    async fn test_configuration_update() {
        let (service, _pool) = create_test_service().await;

        let update = ConfigurationUpdate {
            work_duration: Some(1800), // 30 minutes
            notifications_enabled: Some(false),
            theme: Some("Dark".to_string()),
            ..Default::default()
        };

        let updated_config = service.update_configuration(update).await.unwrap();

        assert_eq!(updated_config.work_duration, 1800);
        assert!(!updated_config.notifications_enabled);
        assert_eq!(updated_config.theme, Theme::Dark);

        // Verify the change persists
        let retrieved_config = service.get_configuration().await.unwrap();
        assert_eq!(retrieved_config.work_duration, 1800);
        assert_eq!(retrieved_config.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_invalid_configuration_update() {
        let (service, _pool) = create_test_service().await;

        let update = ConfigurationUpdate {
            work_duration: Some(100), // Too short (less than 5 minutes)
            ..Default::default()
        };

        let result = service.update_configuration(update).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_theme() {
        let (service, _pool) = create_test_service().await;

        let update = ConfigurationUpdate {
            theme: Some("InvalidTheme".to_string()),
            ..Default::default()
        };

        let result = service.update_configuration(update).await;
        assert!(matches!(result, Err(ConfigurationServiceError::InvalidTheme(_))));
    }

    #[tokio::test]
    async fn test_reset_to_defaults() {
        let (service, _pool) = create_test_service().await;

        // First make some changes
        let update = ConfigurationUpdate {
            work_duration: Some(1800),
            theme: Some("Dark".to_string()),
            ..Default::default()
        };
        service.update_configuration(update).await.unwrap();

        // Then reset
        let reset_config = service.reset_to_defaults().await.unwrap();
        assert_eq!(reset_config.work_duration, 1500);
        assert_eq!(reset_config.theme, Theme::Light);
    }
}

impl Default for ConfigurationUpdate {
    fn default() -> Self {
        Self {
            work_duration: None,
            short_break_duration: None,
            long_break_duration: None,
            long_break_frequency: None,
            notifications_enabled: None,
            webhook_url: None,
            wait_for_interaction: None,
            theme: None,
        }
    }
}