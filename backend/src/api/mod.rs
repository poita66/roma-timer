//! API module for Roma Timer
//!
//! Contains all REST API endpoints and routing.

pub mod configuration;
pub mod timer;

// Re-export commonly used API components
pub use configuration::*;
pub use timer::*;