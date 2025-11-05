//! Services module for Roma Timer
//!
//! Contains all business logic and service implementations.

pub mod timer_service;
pub mod websocket_service;

// Re-export commonly used services
pub use timer_service::{TimerService, TimerServiceError, TimerState, TimerOperation};
pub use websocket_service::{WebSocketService, WebSocketMessage, ClientMessage};