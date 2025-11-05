//! Database abstraction layer
//!
//! Provides database-agnostic support for SQLite and PostgreSQL using SQLx.

pub mod connection;
pub mod types;

pub use connection::{DatabaseManager};
pub use types::DatabaseType;
pub use types::*;