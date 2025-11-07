//! Database abstraction layer
//!
//! Provides database-agnostic support for SQLite and PostgreSQL using SQLx.

pub mod connection;
pub mod types;
// pub mod daily_reset_extensions; // Temporarily disabled for compilation

pub use connection::{DatabaseManager};
pub use types::DatabaseType;
// pub use daily_reset_extensions::DailyResetDatabaseExtensions; // Temporarily disabled for compilation
