//! Configuration management for Roma Timer
//!
//! Handles environment variables and application settings.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tracing::{info, warn};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server bind address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Database URL
    pub database_url: String,

    /// Shared secret for authentication
    pub shared_secret: String,

    /// Environment (development, production)
    pub environment: String,

    /// Log level
    pub log_level: String,

    /// Frontend directory for PWA serving
    pub frontend_dir: PathBuf,

    /// Data directory for SQLite database
    pub data_dir: PathBuf,

    /// CORS origins (empty means allow all)
    pub cors_origins: Vec<String>,

    /// WebSocket heartbeat interval in seconds
    pub websocket_heartbeat_interval: u64,

    /// WebSocket connection timeout in seconds
    pub websocket_timeout: u64,

    /// Maximum concurrent WebSocket connections
    pub max_websocket_connections: usize,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Enable request logging
    pub enable_request_logging: bool,

    /// Enable performance metrics
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            database_url: "sqlite:roma-timer.db".to_string(),
            shared_secret: "change-me-in-production".to_string(),
            environment: "development".to_string(),
            log_level: "info".to_string(),
            frontend_dir: PathBuf::from("../frontend"),
            data_dir: PathBuf::from("./data"),
            cors_origins: vec![],
            websocket_heartbeat_interval: 30,
            websocket_timeout: 300,
            max_websocket_connections: 100,
            request_timeout: 30,
            enable_request_logging: true,
            enable_metrics: true,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Server configuration
        if let Ok(host) = env::var("ROMA_TIMER_HOST") {
            config.host = host;
        }

        if let Ok(port) = env::var("ROMA_TIMER_PORT") {
            config.port = port.parse()
                .map_err(|_| ConfigError::InvalidPort(port))?;
        }

        // Database configuration
        if let Ok(database_url) = env::var("ROMA_TIMER_DATABASE_URL") {
            config.database_url = database_url;
        }

        // Data directory
        if let Ok(data_dir) = env::var("ROMA_TIMER_DATA_DIR") {
            config.data_dir = PathBuf::from(data_dir);
        }

        // Authentication
        if let Ok(shared_secret) = env::var("ROMA_TIMER_SECRET") {
            config.shared_secret = shared_secret;
        }

        // Environment
        if let Ok(environment) = env::var("ROMA_TIMER_ENVIRONMENT") {
            config.environment = environment;
        }

        // Logging
        if let Ok(log_level) = env::var("ROMA_TIMER_LOG_LEVEL") {
            config.log_level = log_level;
        }

        // Frontend directory
        if let Ok(frontend_dir) = env::var("ROMA_TIMER_FRONTEND_DIR") {
            config.frontend_dir = PathBuf::from(frontend_dir);
        }

        // CORS origins
        if let Ok(cors_origins) = env::var("ROMA_TIMER_CORS_ORIGINS") {
            config.cors_origins = cors_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // WebSocket settings
        if let Ok(heartbeat_interval) = env::var("ROMA_TIMER_WEBSOCKET_HEARTBEAT_INTERVAL") {
            config.websocket_heartbeat_interval = heartbeat_interval.parse()
                .map_err(|_| ConfigError::InvalidWebSocketHeartbeat(heartbeat_interval))?;
        }

        if let Ok(timeout) = env::var("ROMA_TIMER_WEBSOCKET_TIMEOUT") {
            config.websocket_timeout = timeout.parse()
                .map_err(|_| ConfigError::InvalidWebSocketTimeout(timeout))?;
        }

        if let Ok(max_connections) = env::var("ROMA_TIMER_MAX_WEBSOCKET_CONNECTIONS") {
            config.max_websocket_connections = max_connections.parse()
                .map_err(|_| ConfigError::InvalidMaxConnections(max_connections))?;
        }

        // Request timeout
        if let Ok(timeout) = env::var("ROMA_TIMER_REQUEST_TIMEOUT") {
            config.request_timeout = timeout.parse()
                .map_err(|_| ConfigError::InvalidRequestTimeout(timeout))?;
        }

        // Feature flags
        if let Ok(enable_logging) = env::var("ROMA_TIMER_ENABLE_REQUEST_LOGGING") {
            config.enable_request_logging = enable_logging.parse()
                .map_err(|_| ConfigError::InvalidBool(enable_logging))?;
        }

        if let Ok(enable_metrics) = env::var("ROMA_TIMER_ENABLE_METRICS") {
            config.enable_metrics = enable_metrics.parse()
                .map_err(|_| ConfigError::InvalidBool(enable_metrics))?;
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate shared secret
        if self.shared_secret == "change-me-in-production" && self.environment == "production" {
            return Err(ConfigError::InsecureProductionSecret);
        }

        if self.shared_secret.len() < 16 {
            return Err(ConfigError::SharedSecretTooShort);
        }

        // Validate port
        if self.port == 0 || self.port > 65535 {
            return Err(ConfigError::InvalidPort(self.port.to_string()));
        }

        // Validate database URL
        if self.database_url.is_empty() {
            return Err(ConfigError::EmptyDatabaseUrl);
        }

        // Validate data directory
        if self.data_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyDataDir);
        }

        // Validate frontend directory
        if self.frontend_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyFrontendDir);
        }

        // Validate WebSocket settings
        if self.websocket_heartbeat_interval == 0 {
            return Err(ConfigError::InvalidWebSocketHeartbeat(
                self.websocket_heartbeat_interval.to_string()
            ));
        }

        if self.websocket_timeout == 0 {
            return Err(ConfigError::InvalidWebSocketTimeout(
                self.websocket_timeout.to_string()
            ));
        }

        if self.max_websocket_connections == 0 {
            return Err(ConfigError::InvalidMaxConnections(
                self.max_websocket_connections.to_string()
            ));
        }

        Ok(())
    }

    /// Get server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get server URL
    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Get WebSocket heartbeat interval in milliseconds
    pub fn websocket_heartbeat_interval_ms(&self) -> u64 {
        self.websocket_heartbeat_interval * 1000
    }

    /// Get WebSocket timeout in milliseconds
    pub fn websocket_timeout_ms(&self) -> u64 {
        self.websocket_timeout * 1000
    }

    /// Get request timeout in milliseconds
    pub fn request_timeout_ms(&self) -> u64 {
        self.request_timeout * 1000
    }

    /// Create data directory if it doesn't exist
    pub fn ensure_data_dir(&self) -> Result<(), ConfigError> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| ConfigError::DataDirCreationFailed(e.to_string()))?;
        Ok(())
    }

    /// Get full database path if using SQLite file
    pub fn database_path(&self) -> Option<PathBuf> {
        if self.database_url.starts_with("sqlite:") {
            let path = self.database_url.strip_prefix("sqlite:")
                .unwrap_or(&self.database_url);
            let path = PathBuf::from(path);

            if path.is_relative() {
                let mut full_path = self.data_dir.clone();
                full_path.push(path);
                Some(full_path)
            } else {
                Some(path)
            }
        } else {
            None
        }
    }

    /// Log configuration (excluding sensitive data)
    pub fn log_config(&self) {
        info!("Configuration loaded:");
        info!("  Environment: {}", self.environment);
        info!("  Bind address: {}", self.bind_address());
        info!("  Database URL: {}", self.mask_database_url());
        info!("  Data directory: {:?}", self.data_dir);
        info!("  Frontend directory: {:?}", self.frontend_dir);
        info!("  Log level: {}", self.log_level);
        info!("  CORS origins: {:?}", self.cors_origins);
        info!("  WebSocket heartbeat: {}s", self.websocket_heartbeat_interval);
        info!("  WebSocket timeout: {}s", self.websocket_timeout);
        info!("  Max WebSocket connections: {}", self.max_websocket_connections);
        info!("  Request timeout: {}s", self.request_timeout);
        info!("  Request logging: {}", self.enable_request_logging);
        info!("  Metrics: {}", self.enable_metrics);

        if self.shared_secret == "change-me-in-production" {
            warn!("⚠️  Using default shared secret - CHANGE IN PRODUCTION!");
        }
    }

    /// Mask database URL for logging
    fn mask_database_url(&self) -> String {
        if self.database_url.starts_with("sqlite:") {
            self.database_url.clone()
        } else if self.database_url.contains("://") {
            let parts: Vec<&str> = self.database_url.split("://").collect();
            if parts.len() >= 2 {
                format!("{}://***", parts[0])
            } else {
                "***".to_string()
            }
        } else {
            "***".to_string()
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port: {0}")]
    InvalidPort(String),

    #[error("Invalid WebSocket heartbeat interval: {0}")]
    InvalidWebSocketHeartbeat(String),

    #[error("Invalid WebSocket timeout: {0}")]
    InvalidWebSocketTimeout(String),

    #[error("Invalid max WebSocket connections: {0}")]
    InvalidMaxConnections(String),

    #[error("Invalid request timeout: {0}")]
    InvalidRequestTimeout(String),

    #[error("Invalid boolean value: {0}")]
    InvalidBool(String),

    #[error("Insecure shared secret for production environment")]
    InsecureProductionSecret,

    #[error("Shared secret too short (minimum 16 characters)")]
    SharedSecretTooShort,

    #[error("Empty database URL")]
    EmptyDatabaseUrl,

    #[error("Empty data directory")]
    EmptyDataDir,

    #[error("Empty frontend directory")]
    EmptyFrontendDir,

    #[error("Data directory creation failed: {0}")]
    DataDirCreationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.shared_secret, "change-me-in-production");
        assert_eq!(config.environment, "development");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid port should fail
        config.port = 0;
        assert!(config.validate().is_err());
        config.port = 3000;

        // Too short secret should fail
        config.shared_secret = "short".to_string();
        assert!(config.validate().is_err());
        config.shared_secret = "a-sufficiently-long-secret-key".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_production_secret_validation() {
        let mut config = Config::default();
        config.environment = "production".to_string();

        // Default secret should fail in production
        assert!(config.validate().is_err());

        // Custom secret should pass
        config.shared_secret = "a-sufficiently-long-secret-key".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_helper_methods() {
        let config = Config::default();

        assert_eq!(config.bind_address(), "0.0.0.0:3000");
        assert_eq!(config.server_url(), "http://0.0.0.0:3000");
        assert!(config.is_development());
        assert!(!config.is_production());

        assert_eq!(config.websocket_heartbeat_interval_ms(), 30000);
        assert_eq!(config.websocket_timeout_ms(), 300000);
        assert_eq!(config.request_timeout_ms(), 30000);
    }

    #[test]
    fn test_database_url_masking() {
        let mut config = Config::default();

        config.database_url = "sqlite:roma-timer.db".to_string();
        assert_eq!(config.mask_database_url(), "sqlite:roma-timer.db");

        config.database_url = "postgresql://user:pass@localhost/db".to_string();
        assert_eq!(config.mask_database_url(), "postgresql://***");

        config.database_url = "mongodb://localhost:27017".to_string();
        assert_eq!(config.mask_database_url(), "mongodb://***");
    }

    #[test]
    fn test_environment_loading() {
        // Test that config can be loaded without panicking
        // In real tests, you would set environment variables
        let config = Config::from_env();
        assert!(config.is_ok());
    }
}