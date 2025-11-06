//! Notification Event Model
//!
//! Represents timer completion notifications for delivery tracking.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::time::{SystemTime, UNIX_EPOCH};

/// Notification event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum NotificationType {
    #[serde(rename = "WorkSessionComplete")]
    #[sqlx(rename = "WorkSessionComplete")]
    WorkSessionComplete,
    #[serde(rename = "BreakSessionComplete")]
    #[sqlx(rename = "BreakSessionComplete")]
    BreakSessionComplete,
    #[serde(rename = "TimerSkipped")]
    #[sqlx(rename = "TimerSkipped")]
    TimerSkipped,
    #[serde(rename = "TimerReset")]
    #[sqlx(rename = "TimerReset")]
    TimerReset,
}

impl NotificationType {
    /// Get display name for this notification type
    pub fn display_name(&self) -> &'static str {
        match self {
            NotificationType::WorkSessionComplete => "Work Session Complete",
            NotificationType::BreakSessionComplete => "Break Session Complete",
            NotificationType::TimerSkipped => "Timer Skipped",
            NotificationType::TimerReset => "Timer Reset",
        }
    }

    /// Get default message for this notification type
    pub fn default_message(&self) -> &'static str {
        match self {
            NotificationType::WorkSessionComplete => "Work session completed! Time for a break.",
            NotificationType::BreakSessionComplete => "Break completed! Ready to focus?",
            NotificationType::TimerSkipped => "Timer session skipped.",
            NotificationType::TimerReset => "Timer has been reset.",
        }
    }
}

/// Notification event representing timer completion notifications
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationEvent {
    /// Unique identifier for the notification event
    pub id: String,

    /// Associated timer session ID
    #[sqlx(rename = "timer_session_id")]
    pub timer_session_id: String,

    /// Type of notification event
    #[sqlx(rename = "event_type")]
    pub event_type: NotificationType,

    /// Notification message
    pub message: String,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: u64,

    /// Delivery confirmation timestamp (None if not yet delivered)
    #[sqlx(rename = "delivered_at")]
    pub delivered_at: Option<u64>,
}

impl NotificationEvent {
    /// Create a new notification event
    pub fn new(timer_session_id: String, event_type: NotificationType, message: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timer_session_id,
            event_type: event_type.clone(),
            message: message.unwrap_or_else(|| event_type.default_message().to_string()),
            created_at: now,
            delivered_at: None,
        }
    }

    /// Mark the notification as delivered
    pub fn mark_delivered(&mut self) {
        self.delivered_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    /// Check if the notification has been delivered
    pub fn is_delivered(&self) -> bool {
        self.delivered_at.is_some()
    }

    /// Get the time since creation in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(self.created_at)
    }

    /// Check if the notification is old (older than 5 minutes)
    pub fn is_old(&self) -> bool {
        self.age_seconds() > 300 // 5 minutes
    }

    /// Get delivery delay in seconds (None if not delivered)
    pub fn delivery_delay_seconds(&self) -> Option<u64> {
        self.delivered_at.map(|delivered_at| delivered_at.saturating_sub(self.created_at))
    }
}

/// Notification errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Notification event not found")]
    NotFound,

    #[error("Notification delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Invalid notification type")]
    InvalidType,

    #[error("Notification already delivered")]
    AlreadyDelivered,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_event_creation() {
        let event = NotificationEvent::new(
            "session-123".to_string(),
            NotificationType::WorkSessionComplete,
            None,
        );

        assert_eq!(event.timer_session_id, "session-123");
        assert_eq!(event.event_type, NotificationType::WorkSessionComplete);
        assert_eq!(event.message, "Work session completed! Time for a break.");
        assert!(!event.is_delivered());
        assert_eq!(event.age_seconds(), 0);
    }

    #[test]
    fn test_notification_with_custom_message() {
        let custom_message = "Custom work session complete message!";
        let event = NotificationEvent::new(
            "session-123".to_string(),
            NotificationType::WorkSessionComplete,
            Some(custom_message.to_string()),
        );

        assert_eq!(event.message, custom_message);
    }

    #[test]
    fn test_notification_delivery() {
        let mut event = NotificationEvent::new(
            "session-123".to_string(),
            NotificationType::BreakSessionComplete,
            None,
        );

        assert!(!event.is_delivered());
        assert!(event.delivery_delay_seconds().is_none());

        event.mark_delivered();

        assert!(event.is_delivered());
        assert!(event.delivery_delay_seconds().is_some());
    }

    #[test]
    fn test_notification_age() {
        let event = NotificationEvent::new(
            "session-123".to_string(),
            NotificationType::TimerSkipped,
            None,
        );

        // Should be 0 or very close to 0 when just created
        assert!(event.age_seconds() < 2);
    }

    #[test]
    fn test_notification_type_display_names() {
        assert_eq!(
            NotificationType::WorkSessionComplete.display_name(),
            "Work Session Complete"
        );
        assert_eq!(
            NotificationType::BreakSessionComplete.display_name(),
            "Break Session Complete"
        );
        assert_eq!(NotificationType::TimerSkipped.display_name(), "Timer Skipped");
        assert_eq!(NotificationType::TimerReset.display_name(), "Timer Reset");
    }

    #[test]
    fn test_notification_type_default_messages() {
        assert_eq!(
            NotificationType::WorkSessionComplete.default_message(),
            "Work session completed! Time for a break."
        );
        assert_eq!(
            NotificationType::BreakSessionComplete.default_message(),
            "Break completed! Ready to focus?"
        );
        assert_eq!(NotificationType::TimerSkipped.default_message(), "Timer session skipped.");
        assert_eq!(NotificationType::TimerReset.default_message(), "Timer has been reset.");
    }
}