//! User Configuration Model
//!
//! Represents user preferences and settings for the pomodoro timer.
//! Includes validation rules and default values.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

/// UI theme options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum Theme {
    #[serde(rename = "Light")]
    #[sqlx(rename = "Light")]
    Light,
    #[serde(rename = "Dark")]
    #[sqlx(rename = "Dark")]
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}

impl Theme {
    /// Get display name for this theme
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }
}

/// User configuration for pomodoro timer settings
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserConfiguration {
    /// Unique identifier for the configuration
    pub id: String,

    /// Work session duration in seconds (default: 25 minutes)
    #[sqlx(rename = "work_duration")]
    pub work_duration: u32,

    /// Short break duration in seconds (default: 5 minutes)
    #[sqlx(rename = "short_break_duration")]
    pub short_break_duration: u32,

    /// Long break duration in seconds (default: 15 minutes)
    #[sqlx(rename = "long_break_duration")]
    pub long_break_duration: u32,

    /// Number of work sessions before long break (default: 4)
    #[sqlx(rename = "long_break_frequency")]
    pub long_break_frequency: u32,

    /// Whether browser notifications are enabled
    #[sqlx(rename = "notifications_enabled")]
    pub notifications_enabled: bool,

    /// Optional webhook URL for timer completion notifications
    #[sqlx(rename = "webhook_url")]
    pub webhook_url: Option<String>,

    /// Whether to wait for user interaction before starting next session
    #[sqlx(rename = "wait_for_interaction")]
    pub wait_for_interaction: bool,

    /// UI theme preference
    pub theme: Theme,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: u64,

    /// Last update timestamp (Unix timestamp)
    #[sqlx(rename = "updated_at")]
    pub updated_at: u64,
}

impl UserConfiguration {
    /// Create a new user configuration with default values
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: "default-config".to_string(),
            work_duration: 1500,        // 25 minutes
            short_break_duration: 300,  // 5 minutes
            long_break_duration: 900,   // 15 minutes
            long_break_frequency: 4,    // Long break after 4 work sessions
            notifications_enabled: true,
            webhook_url: None,
            wait_for_interaction: false,
            theme: Theme::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a user configuration with custom ID
    pub fn with_id(id: String) -> Self {
        let mut config = Self::new();
        config.id = id;
        config
    }

    /// Validate work duration bounds
    fn validate_work_duration(duration: u32) -> Result<(), UserConfigurationError> {
        if duration < 300 || duration > 3600 {
            // 5 minutes to 1 hour
            return Err(UserConfigurationError::InvalidWorkDuration(duration));
        }
        Ok(())
    }

    /// Validate short break duration bounds
    fn validate_short_break_duration(duration: u32) -> Result<(), UserConfigurationError> {
        if duration < 60 || duration > 900 {
            // 1 minute to 15 minutes
            return Err(UserConfigurationError::InvalidShortBreakDuration(duration));
        }
        Ok(())
    }

    /// Validate long break duration bounds
    fn validate_long_break_duration(duration: u32) -> Result<(), UserConfigurationError> {
        if duration < 300 || duration > 1800 {
            // 5 minutes to 30 minutes
            return Err(UserConfigurationError::InvalidLongBreakDuration(duration));
        }
        Ok(())
    }

    /// Validate long break frequency bounds
    fn validate_long_break_frequency(frequency: u32) -> Result<(), UserConfigurationError> {
        if frequency < 2 || frequency > 10 {
            // 2 to 10 work sessions
            return Err(UserConfigurationError::InvalidLongBreakFrequency(frequency));
        }
        Ok(())
    }

    /// Validate webhook URL if provided
    fn validate_webhook_url(url: &Option<String>) -> Result<(), UserConfigurationError> {
        if let Some(webhook_url) = url {
            Url::parse(webhook_url)
                .map_err(|_| UserConfigurationError::InvalidWebhookUrl(webhook_url.clone()))?;

            // Ensure URL uses HTTP or HTTPS
            let parsed_url = Url::parse(webhook_url).unwrap();
            if !matches!(parsed_url.scheme(), "http" | "https") {
                return Err(UserConfigurationError::InvalidWebhookUrl(webhook_url.clone()));
            }
        }
        Ok(())
    }

    /// Validate the user configuration
    pub fn validate(&self) -> Result<(), UserConfigurationError> {
        Self::validate_work_duration(self.work_duration)?;
        Self::validate_short_break_duration(self.short_break_duration)?;
        Self::validate_long_break_duration(self.long_break_duration)?;
        Self::validate_long_break_frequency(self.long_break_frequency)?;
        Self::validate_webhook_url(&self.webhook_url)?;

        // Check timestamp consistency
        if self.updated_at < self.created_at {
            return Err(UserConfigurationError::InvalidTimestamps);
        }

        Ok(())
    }

    /// Update work duration with validation
    pub fn set_work_duration(&mut self, duration: u32) -> Result<(), UserConfigurationError> {
        Self::validate_work_duration(duration)?;
        self.work_duration = duration;
        self.touch();
        Ok(())
    }

    /// Update short break duration with validation
    pub fn set_short_break_duration(&mut self, duration: u32) -> Result<(), UserConfigurationError> {
        Self::validate_short_break_duration(duration)?;
        self.short_break_duration = duration;
        self.touch();
        Ok(())
    }

    /// Update long break duration with validation
    pub fn set_long_break_duration(&mut self, duration: u32) -> Result<(), UserConfigurationError> {
        Self::validate_long_break_duration(duration)?;
        self.long_break_duration = duration;
        self.touch();
        Ok(())
    }

    /// Update long break frequency with validation
    pub fn set_long_break_frequency(&mut self, frequency: u32) -> Result<(), UserConfigurationError> {
        Self::validate_long_break_frequency(frequency)?;
        self.long_break_frequency = frequency;
        self.touch();
        Ok(())
    }

    /// Update webhook URL with validation
    pub fn set_webhook_url(&mut self, url: Option<String>) -> Result<(), UserConfigurationError> {
        Self::validate_webhook_url(&url)?;
        self.webhook_url = url;
        self.touch();
        Ok(())
    }

    /// Update theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.touch();
    }

    /// Toggle notifications
    pub fn toggle_notifications(&mut self) {
        self.notifications_enabled = !self.notifications_enabled;
        self.touch();
    }

    /// Toggle wait for interaction
    pub fn toggle_wait_for_interaction(&mut self) {
        self.wait_for_interaction = !self.wait_for_interaction;
        self.touch();
    }

    /// Update the updated_at timestamp
    fn touch(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if notifications are enabled and configured
    pub fn should_send_notifications(&self) -> bool {
        self.notifications_enabled
    }

    /// Check if webhook notifications should be sent
    pub fn should_send_webhook(&self) -> bool {
        self.notifications_enabled && self.webhook_url.is_some()
    }

    /// Get work duration in minutes (for display)
    pub fn work_duration_minutes(&self) -> u32 {
        self.work_duration / 60
    }

    /// Get short break duration in minutes (for display)
    pub fn short_break_duration_minutes(&self) -> u32 {
        self.short_break_duration / 60
    }

    /// Get long break duration in minutes (for display)
    pub fn long_break_duration_minutes(&self) -> u32 {
        self.long_break_duration / 60
    }

    /// Set work duration from minutes
    pub fn set_work_duration_from_minutes(&mut self, minutes: u32) -> Result<(), UserConfigurationError> {
        self.set_work_duration(minutes * 60)
    }

    /// Set short break duration from minutes
    pub fn set_short_break_duration_from_minutes(&mut self, minutes: u32) -> Result<(), UserConfigurationError> {
        self.set_short_break_duration(minutes * 60)
    }

    /// Set long break duration from minutes
    pub fn set_long_break_duration_from_minutes(&mut self, minutes: u32) -> Result<(), UserConfigurationError> {
        self.set_long_break_duration(minutes * 60)
    }
}

impl Default for UserConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// User configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum UserConfigurationError {
    #[error("Work duration {0} minutes is invalid (must be 5-60 minutes)")]
    InvalidWorkDuration(u32),

    #[error("Short break duration {0} minutes is invalid (must be 1-15 minutes)")]
    InvalidShortBreakDuration(u32),

    #[error("Long break duration {0} minutes is invalid (must be 5-30 minutes)")]
    InvalidLongBreakDuration(u32),

    #[error("Long break frequency {0} is invalid (must be 2-10 work sessions)")]
    InvalidLongBreakFrequency(u32),

    #[error("Webhook URL '{0}' is invalid")]
    InvalidWebhookUrl(String),

    #[error("Configuration timestamps are inconsistent")]
    InvalidTimestamps,

    #[error("Configuration not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_configuration_creation() {
        let config = UserConfiguration::new();
        assert_eq!(config.id, "default-config");
        assert_eq!(config.work_duration, 1500); // 25 minutes
        assert_eq!(config.short_break_duration, 300); // 5 minutes
        assert_eq!(config.long_break_duration, 900); // 15 minutes
        assert_eq!(config.long_break_frequency, 4);
        assert!(config.notifications_enabled);
        assert!(!config.wait_for_interaction);
        assert_eq!(config.theme, Theme::Light);
    }

    #[test]
    fn test_user_configuration_validation() {
        let config = UserConfiguration::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_work_duration() {
        let mut config = UserConfiguration::new();

        // Too short (less than 5 minutes)
        assert!(config.set_work_duration(299).is_err());

        // Too long (more than 1 hour)
        assert!(config.set_work_duration(3601).is_err());

        // Valid
        assert!(config.set_work_duration(1800).is_ok()); // 30 minutes
    }

    #[test]
    fn test_invalid_short_break_duration() {
        let mut config = UserConfiguration::new();

        // Too short (less than 1 minute)
        assert!(config.set_short_break_duration(59).is_err());

        // Too long (more than 15 minutes)
        assert!(config.set_short_break_duration(901).is_err());

        // Valid
        assert!(config.set_short_break_duration(600).is_ok()); // 10 minutes
    }

    #[test]
    fn test_invalid_long_break_frequency() {
        let mut config = UserConfiguration::new();

        // Too small (less than 2)
        assert!(config.set_long_break_frequency(1).is_err());

        // Too large (more than 10)
        assert!(config.set_long_break_frequency(11).is_err());

        // Valid
        assert!(config.set_long_break_frequency(6).is_ok());
    }

    #[test]
    fn test_webhook_url_validation() {
        let mut config = UserConfiguration::new();

        // Valid HTTPS URL
        assert!(config.set_webhook_url(Some("https://example.com/webhook".to_string())).is_ok());

        // Valid HTTP URL
        assert!(config.set_webhook_url(Some("http://localhost:3000/webhook".to_string())).is_ok());

        // Invalid URL
        assert!(config.set_webhook_url(Some("not-a-url".to_string())).is_err());

        // Invalid scheme
        assert!(config.set_webhook_url(Some("ftp://example.com/webhook".to_string())).is_err());
    }

    #[test]
    fn test_theme_operations() {
        let mut config = UserConfiguration::new();
        assert_eq!(config.theme, Theme::Light);

        config.set_theme(Theme::Dark);
        assert_eq!(config.theme, Theme::Dark);
        assert!(config.updated_at > config.created_at);
    }

    #[test]
    fn test_toggle_operations() {
        let mut config = UserConfiguration::new();

        // Toggle notifications
        assert!(config.notifications_enabled);
        config.toggle_notifications();
        assert!(!config.notifications_enabled);
        config.toggle_notifications();
        assert!(config.notifications_enabled);

        // Toggle wait for interaction
        assert!(!config.wait_for_interaction);
        config.toggle_wait_for_interaction();
        assert!(config.wait_for_interaction);
    }

    #[test]
    fn test_duration_conversions() {
        let config = UserConfiguration::new();

        assert_eq!(config.work_duration_minutes(), 25);
        assert_eq!(config.short_break_duration_minutes(), 5);
        assert_eq!(config.long_break_duration_minutes(), 15);

        let mut config = UserConfiguration::new();
        config.set_work_duration_from_minutes(30).unwrap();
        assert_eq!(config.work_duration, 1800);
        assert_eq!(config.work_duration_minutes(), 30);
    }

    #[test]
    fn test_notification_settings() {
        let mut config = UserConfiguration::new();

        // Initially notifications enabled but no webhook
        assert!(config.should_send_notifications());
        assert!(!config.should_send_webhook());

        // Add webhook URL
        config.set_webhook_url(Some("https://example.com/webhook".to_string())).unwrap();
        assert!(config.should_send_webhook());

        // Disable notifications
        config.notifications_enabled = false;
        assert!(!config.should_send_notifications());
        assert!(!config.should_send_webhook());
    }

    #[test]
    fn test_theme_display_names() {
        assert_eq!(Theme::Light.display_name(), "Light");
        assert_eq!(Theme::Dark.display_name(), "Dark");
    }
}