//! Models module for Roma Timer
//!
//! Contains all data models and their validation logic.

pub mod timer_session;
pub mod user_configuration;
pub mod notification_event;
pub mod device_connection;

// Re-export commonly used types
pub use timer_session::{TimerSession, TimerType, TimerSessionError};
pub use user_configuration::{UserConfiguration, UserConfigurationError};
pub use notification_event::{NotificationEvent, NotificationType, NotificationError};
pub use device_connection::{DeviceConnection, ConnectionPool, ConnectionStats, DeviceType, Browser, ConnectionStatus};