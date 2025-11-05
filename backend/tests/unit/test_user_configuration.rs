//! User Configuration Model Unit Tests
//!
//! Comprehensive tests for UserConfiguration model validation and business logic

use roma_timer::models::user_configuration::{UserConfiguration, UserConfigurationError, Theme};
use std::time::{SystemTime, UNIX_EPOCH};

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
        assert!(config.created_at > 0);
        assert!(config.updated_at > 0);
        assert_eq!(config.created_at, config.updated_at);
    }

    #[test]
    fn test_user_configuration_with_id() {
        let custom_id = "test-config-123".to_string();
        let config = UserConfiguration::with_id(custom_id.clone());

        assert_eq!(config.id, custom_id);
        assert_eq!(config.work_duration, 1500); // Should still have default values
    }

    #[test]
    fn test_user_configuration_validation() {
        let config = UserConfiguration::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_work_duration_validation() {
        let mut config = UserConfiguration::new();

        // Test valid work durations
        assert!(config.set_work_duration(300).is_ok()); // 5 minutes minimum
        assert!(config.set_work_duration(3600).is_ok()); // 1 hour maximum
        assert!(config.set_work_duration(1800).is_ok()); // 30 minutes

        // Test invalid work durations
        assert!(config.set_work_duration(299).is_err()); // Too short
        assert!(config.set_work_duration(3601).is_err()); // Too long

        // Verify config state is unchanged after failed validation
        assert_eq!(config.work_duration, 1500);
    }

    #[test]
    fn test_short_break_duration_validation() {
        let mut config = UserConfiguration::new();

        // Test valid short break durations
        assert!(config.set_short_break_duration(60).is_ok()); // 1 minute minimum
        assert!(config.set_short_break_duration(900).is_ok()); // 15 minutes maximum
        assert!(config.set_short_break_duration(300).is_ok()); // 5 minutes

        // Test invalid short break durations
        assert!(config.set_short_break_duration(59).is_err()); // Too short
        assert!(config.set_short_break_duration(901).is_err()); // Too long
    }

    #[test]
    fn test_long_break_duration_validation() {
        let mut config = UserConfiguration::new();

        // Test valid long break durations
        assert!(config.set_long_break_duration(300).is_ok()); // 5 minutes minimum
        assert!(config.set_long_break_duration(1800).is_ok()); // 30 minutes maximum
        assert!(config.set_long_break_duration(900).is_ok()); // 15 minutes

        // Test invalid long break durations
        assert!(config.set_long_break_duration(299).is_err()); // Too short
        assert!(config.set_long_break_duration(1801).is_err()); // Too long
    }

    #[test]
    fn test_long_break_frequency_validation() {
        let mut config = UserConfiguration::new();

        // Test valid long break frequencies
        assert!(config.set_long_break_frequency(2).is_ok()); // 2 minimum
        assert!(config.set_long_break_frequency(10).is_ok()); // 10 maximum
        assert!(config.set_long_break_frequency(6).is_ok()); // 6 sessions

        // Test invalid long break frequencies
        assert!(config.set_long_break_frequency(1).is_err()); // Too small
        assert!(config.set_long_break_frequency(11).is_err()); // Too large
    }

    #[test]
    fn test_webhook_url_validation() {
        let mut config = UserConfiguration::new();

        // Test valid URLs
        assert!(config.set_webhook_url(Some("https://example.com/webhook".to_string())).is_ok());
        assert!(config.set_webhook_url(Some("http://localhost:3000/webhook".to_string())).is_ok());
        assert!(config.set_webhook_url(None).is_ok()); // Empty URL is allowed

        // Test invalid URLs
        assert!(config.set_webhook_url(Some("not-a-url".to_string())).is_err());
        assert!(config.set_webhook_url(Some("ftp://example.com/webhook".to_string())).is_err());
        assert!(config.set_webhook_url(Some("javascript:alert('xss')".to_string())).is_err());
    }

    #[test]
    fn test_theme_operations() {
        let mut config = UserConfiguration::new();
        assert_eq!(config.theme, Theme::Light);

        // Set dark theme
        config.set_theme(Theme::Dark);
        assert_eq!(config.theme, Theme::Dark);
        assert!(config.updated_at > config.created_at);

        // Set light theme
        config.set_theme(Theme::Light);
        assert_eq!(config.theme, Theme::Light);
    }

    #[test]
    fn test_toggle_operations() {
        let mut config = UserConfiguration::new();

        // Test notifications toggle
        assert!(config.notifications_enabled);
        config.toggle_notifications();
        assert!(!config.notifications_enabled);
        config.toggle_notifications();
        assert!(config.notifications_enabled);

        // Test wait for interaction toggle
        assert!(!config.wait_for_interaction);
        config.toggle_wait_for_interaction();
        assert!(config.wait_for_interaction);
        config.toggle_wait_for_interaction();
        assert!(!config.wait_for_interaction);
    }

    #[test]
    fn test_duration_conversions() {
        let mut config = UserConfiguration::new();

        // Test getter methods
        assert_eq!(config.work_duration_minutes(), 25);
        assert_eq!(config.short_break_duration_minutes(), 5);
        assert_eq!(config.long_break_duration_minutes(), 15);

        // Test setter methods
        assert!(config.set_work_duration_from_minutes(30).is_ok());
        assert_eq!(config.work_duration, 1800);
        assert_eq!(config.work_duration_minutes(), 30);

        assert!(config.set_short_break_duration_from_minutes(10).is_ok());
        assert_eq!(config.short_break_duration, 600);
        assert_eq!(config.short_break_duration_minutes(), 10);

        assert!(config.set_long_break_duration_from_minutes(20).is_ok());
        assert_eq!(config.long_break_duration, 1200);
        assert_eq!(config.long_break_duration_minutes(), 20);
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

    #[test]
    fn test_timestamp_updates() {
        let mut config = UserConfiguration::new();
        let original_updated_at = config.updated_at;

        // Add small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Make a change that should update the timestamp
        config.set_work_duration(1800).unwrap();

        assert!(config.updated_at > original_updated_at);
    }

    #[test]
    fn test_invalid_timestamps() {
        let mut config = UserConfiguration::new();

        // Set updated_at to be earlier than created_at
        config.updated_at = config.created_at - 1;

        assert!(matches!(config.validate(), Err(UserConfigurationError::InvalidTimestamps)));
    }

    #[test]
    fn test_edge_case_durations() {
        let mut config = UserConfiguration::new();

        // Test boundary values for work duration
        assert!(config.set_work_duration(300).is_ok()); // Minimum
        assert_eq!(config.work_duration, 300);
        assert!(config.set_work_duration(3600).is_ok()); // Maximum
        assert_eq!(config.work_duration, 3600);

        // Test boundary values for short break
        assert!(config.set_short_break_duration(60).is_ok()); // Minimum
        assert_eq!(config.short_break_duration, 60);
        assert!(config.set_short_break_duration(900).is_ok()); // Maximum
        assert_eq!(config.short_break_duration, 900);

        // Test boundary values for long break
        assert!(config.set_long_break_duration(300).is_ok()); // Minimum
        assert_eq!(config.long_break_duration, 300);
        assert!(config.set_long_break_duration(1800).is_ok()); // Maximum
        assert_eq!(config.long_break_duration, 1800);

        // Test boundary values for frequency
        assert!(config.set_long_break_frequency(2).is_ok()); // Minimum
        assert_eq!(config.long_break_frequency, 2);
        assert!(config.set_long_break_frequency(10).is_ok()); // Maximum
        assert_eq!(config.long_break_frequency, 10);
    }

    #[test]
    fn test_configuration_with_webhook() {
        let mut config = UserConfiguration::new();

        // Set webhook URL
        let webhook_url = "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX";
        assert!(config.set_webhook_url(Some(webhook_url.to_string())).is_ok());
        assert_eq!(config.webhook_url, Some(webhook_url.to_string()));

        // Should send webhook when notifications are enabled
        assert!(config.should_send_webhook());

        // Should not send webhook when notifications are disabled
        config.notifications_enabled = false;
        assert!(!config.should_send_webhook());
    }

    #[test]
    fn test_default_implementation() {
        let config = UserConfiguration::default();

        assert_eq!(config.work_duration, 1500);
        assert_eq!(config.short_break_duration, 300);
        assert_eq!(config.long_break_duration, 900);
        assert_eq!(config.long_break_frequency, 4);
        assert!(config.notifications_enabled);
        assert!(!config.wait_for_interaction);
        assert_eq!(config.theme, Theme::Light);
    }

    #[test]
    fn test_complex_validation_scenario() {
        let mut config = UserConfiguration::new();

        // Make multiple changes
        assert!(config.set_work_duration(2700).is_ok()); // 45 minutes
        assert!(config.set_short_break_duration(420).is_ok()); // 7 minutes
        assert!(config.set_long_break_duration(1200).is_ok()); // 20 minutes
        assert!(config.set_long_break_frequency(5).is_ok());
        config.set_theme(Theme::Dark);

        // Validate complete configuration
        assert!(config.validate().is_ok());

        // Verify all values are set correctly
        assert_eq!(config.work_duration, 2700);
        assert_eq!(config.short_break_duration, 420);
        assert_eq!(config.long_break_duration, 1200);
        assert_eq!(config.long_break_frequency, 5);
        assert_eq!(config.theme, Theme::Dark);
    }
}