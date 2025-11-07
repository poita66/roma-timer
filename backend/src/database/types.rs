//! Database types and enums

use std::str::FromStr;
use tracing::warn;

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DatabaseType {
    #[serde(rename = "sqlite")]
    Sqlite,
    #[serde(rename = "postgres")]
    Postgres,
}

impl DatabaseType {
    /// Get database type from connection URL
    pub fn from_url(url: &str) -> Self {
        if url.starts_with("sqlite:") {
            DatabaseType::Sqlite
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            DatabaseType::Postgres
        } else {
            // Default to SQLite for backward compatibility
            warn!("Unknown database URL format: {}, defaulting to SQLite", url);
            DatabaseType::Sqlite
        }
    }

    /// Get the default port for this database type
    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::Sqlite => 0, // File-based, no port
            DatabaseType::Postgres => 5432,
        }
    }

    /// Get connection pool example URL for this database type
    pub fn example_url(&self) -> &'static str {
        match self {
            DatabaseType::Sqlite => "sqlite:roma-timer.db",
            DatabaseType::Postgres => "postgres://user:password@localhost/roma_timer",
        }
    }

    /// Get the SQLx feature name for this database type
    pub fn sqlx_feature(&self) -> &'static str {
        match self {
            DatabaseType::Sqlite => "sqlite",
            DatabaseType::Postgres => "postgres",
        }
    }
}

impl Default for DatabaseType {
    fn default() -> Self {
        DatabaseType::Sqlite
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Sqlite => write!(f, "sqlite"),
            DatabaseType::Postgres => write!(f, "postgres"),
        }
    }
}

impl FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(DatabaseType::Sqlite),
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            _ => Err(format!("Invalid database type: {}. Supported types: sqlite, postgres", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_from_url() {
        assert_eq!(DatabaseType::from_url("sqlite:test.db"), DatabaseType::Sqlite);
        assert_eq!(DatabaseType::from_url("postgres://user:pass@localhost/db"), DatabaseType::Postgres);
        assert_eq!(DatabaseType::from_url("postgresql://user:pass@localhost/db"), DatabaseType::Postgres);
        assert_eq!(DatabaseType::from_url("unknown://test"), DatabaseType::Sqlite); // Default fallback
    }

    #[test]
    fn test_database_type_display() {
        assert_eq!(DatabaseType::Sqlite.to_string(), "sqlite");
        assert_eq!(DatabaseType::Postgres.to_string(), "postgres");
    }

    #[test]
    fn test_database_type_from_str() {
        assert_eq!("sqlite".parse::<DatabaseType>().unwrap(), DatabaseType::Sqlite);
        assert_eq!("postgres".parse::<DatabaseType>().unwrap(), DatabaseType::Postgres);
        assert_eq!("postgresql".parse::<DatabaseType>().unwrap(), DatabaseType::Postgres);
        assert_eq!("POSTGRES".parse::<DatabaseType>().unwrap(), DatabaseType::Postgres);
        assert!("invalid".parse::<DatabaseType>().is_err());
    }

    #[test]
    fn test_database_type_default_port() {
        assert_eq!(DatabaseType::Sqlite.default_port(), 0);
        assert_eq!(DatabaseType::Postgres.default_port(), 5432);
    }

    #[test]
    fn test_database_type_example_url() {
        assert_eq!(DatabaseType::Sqlite.example_url(), "sqlite:roma-timer.db");
        assert_eq!(DatabaseType::Postgres.example_url(), "postgres://user:password@localhost/roma_timer");
    }
}