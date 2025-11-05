//! Database connection manager
//!
//! Provides database-agnostic connection management for SQLite and PostgreSQL.

use anyhow::Result;
use sqlx::{query, AnyPool, query_as};
use sqlx::any::AnyQueryResult;
use tracing::{debug, info};
use chrono::Utc;
use uuid::Uuid;

use super::types::DatabaseType;

// Database row structures
#[derive(Debug, sqlx::FromRow)]
struct TimerStateRow {
    is_running: bool,
    remaining_seconds: i64,
    session_type: String,
    session_count: i64,
    work_duration: i64,
    short_break_duration: i64,
    long_break_duration: i64,
    last_updated: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Database connection manager
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pub pool: AnyPool,
    pub database_type: DatabaseType,
}

impl DatabaseManager {
    /// Create a new database manager with the given connection URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let database_type = DatabaseType::from_url(database_url);

        info!("Connecting to database: {} ({})", database_type, database_url);

        let pool = AnyPool::connect(database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        debug!("Successfully connected to {} database", database_type);

        Ok(Self {
            pool,
            database_type,
        })
    }

    /// Create a new database manager with environment variable fallback
    pub async fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                if std::env::var("POSTGRES_URL").is_ok() {
                    std::env::var("POSTGRES_URL").unwrap()
                } else {
                    "sqlite:roma-timer.db".to_string()
                }
            });

        Self::new(&database_url).await
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations for {}", self.database_type);

        // Run migrations using sqlx migrate if migration files exist
        // For now, we'll create the tables directly

        self.create_tables().await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Create database tables
    async fn create_tables(&self) -> Result<()> {
        match self.database_type {
            DatabaseType::Sqlite => {
                self.create_sqlite_tables().await?;
            }
            DatabaseType::Postgres => {
                self.create_postgres_tables().await?;
            }
        }
        Ok(())
    }

    /// Create SQLite-specific tables
    async fn create_sqlite_tables(&self) -> Result<()> {
        // Timer state table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS timer_state (
                id TEXT PRIMARY KEY,
                is_running BOOLEAN NOT NULL DEFAULT FALSE,
                remaining_seconds INTEGER NOT NULL DEFAULT 1500,
                session_type TEXT NOT NULL DEFAULT 'work',
                session_count INTEGER NOT NULL DEFAULT 1,
                work_duration INTEGER NOT NULL DEFAULT 1500,
                short_break_duration INTEGER NOT NULL DEFAULT 300,
                long_break_duration INTEGER NOT NULL DEFAULT 900,
                last_updated INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Users table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // User configurations table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS user_configurations (
                id TEXT PRIMARY KEY,
                work_duration INTEGER NOT NULL DEFAULT 1500,
                short_break_duration INTEGER NOT NULL DEFAULT 300,
                long_break_duration INTEGER NOT NULL DEFAULT 900,
                long_break_frequency INTEGER NOT NULL DEFAULT 4,
                notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                webhook_url TEXT,
                wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
                theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Timer sessions table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS timer_sessions (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                timer_type TEXT NOT NULL,
                duration INTEGER NOT NULL,
                elapsed INTEGER NOT NULL DEFAULT 0,
                is_running BOOLEAN NOT NULL DEFAULT FALSE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Notification events table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_events (
                id TEXT PRIMARY KEY,
                timer_session_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                message TEXT,
                delivered BOOLEAN NOT NULL DEFAULT FALSE,
                created_at INTEGER NOT NULL,
                delivered_at INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("SQLite tables created successfully");
        Ok(())
    }

    /// Create PostgreSQL-specific tables
    async fn create_postgres_tables(&self) -> Result<()> {
        // Timer state table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS timer_state (
                id TEXT PRIMARY KEY,
                is_running BOOLEAN NOT NULL DEFAULT FALSE,
                remaining_seconds INTEGER NOT NULL DEFAULT 1500,
                session_type TEXT NOT NULL DEFAULT 'work',
                session_count INTEGER NOT NULL DEFAULT 1,
                work_duration INTEGER NOT NULL DEFAULT 1500,
                short_break_duration INTEGER NOT NULL DEFAULT 300,
                long_break_duration INTEGER NOT NULL DEFAULT 900,
                last_updated BIGINT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Users table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // User configurations table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS user_configurations (
                id TEXT PRIMARY KEY,
                work_duration INTEGER NOT NULL DEFAULT 1500,
                short_break_duration INTEGER NOT NULL DEFAULT 300,
                long_break_duration INTEGER NOT NULL DEFAULT 900,
                long_break_frequency INTEGER NOT NULL DEFAULT 4,
                notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                webhook_url TEXT,
                wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
                theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Timer sessions table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS timer_sessions (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                timer_type TEXT NOT NULL,
                duration INTEGER NOT NULL,
                elapsed INTEGER NOT NULL DEFAULT 0,
                is_running BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                completed_at BIGINT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Notification events table
        query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_events (
                id TEXT PRIMARY KEY,
                timer_session_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                message TEXT,
                delivered BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL,
                delivered_at BIGINT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("PostgreSQL tables created successfully");
        Ok(())
    }

    /// Get connection pool statistics
    pub async fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database connection test failed: {}", e))?;

        debug!("Database connection test successful");
        Ok(())
    }

    /// Save timer state to database
    pub async fn save_timer_state(&self, state: &crate::TimerState) -> Result<()> {
        query(
            r#"
            INSERT OR REPLACE INTO timer_state (id, is_running, remaining_seconds, session_type, session_count, work_duration, short_break_duration, long_break_duration, last_updated)
            VALUES ('default', ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(state.is_running)
        .bind(state.remaining_seconds as i64)
        .bind(&state.session_type)
        .bind(state.session_count as i64)
        .bind(state.work_duration as i64)
        .bind(state.short_break_duration as i64)
        .bind(state.long_break_duration as i64)
        .bind(state.last_updated as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save timer state: {}", e))?;

        Ok(())
    }

    /// Get current timer state from database
    pub async fn get_current_timer_state(&self) -> Result<Option<crate::TimerState>> {
        let row = sqlx::query_as::<_, TimerStateRow>(
            r#"
            SELECT is_running, remaining_seconds, session_type, session_count, work_duration, short_break_duration, long_break_duration, last_updated
            FROM timer_state
            WHERE id = 'default'
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get timer state: {}", e))?;

        Ok(row.map(|r| crate::TimerState {
            is_running: r.is_running,
            remaining_seconds: r.remaining_seconds as u32,
            session_type: r.session_type,
            session_count: r.session_count as u32,
            work_duration: r.work_duration as u32,
            short_break_duration: r.short_break_duration as u32,
            long_break_duration: r.long_break_duration as u32,
            last_updated: r.last_updated as u64,
        }))
    }

    /// Create a new user
    pub async fn create_user(&self, username: &str, password_hash: &str, salt: &str) -> Result<String> {
        let user_id = uuid::Uuid::new_v4().to_string();
        
        query(
            r#"
            INSERT INTO users (id, username, password_hash, salt, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&user_id)
        .bind(username)
        .bind(password_hash)
        .bind(salt)
        .bind(chrono::Utc::now().timestamp())
        .bind(chrono::Utc::now().timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))?;
        
        Ok(user_id)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, salt, created_at, updated_at
            FROM users
            WHERE username = ?
            "#
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get user by username: {}", e))?;

        Ok(row)
    }

    /// Get the database URL for logging (masked for security)
    pub fn masked_database_url(&self) -> String {
        // This is a simplified version - you might want to add more sophisticated masking
        match self.database_type {
            DatabaseType::Sqlite => {
                // For SQLite, show the filename
                "sqlite:roma-timer.db".to_string()
            }
            DatabaseType::Postgres => {
                // For PostgreSQL, mask the password
                "postgres://user:***@localhost/roma_timer".to_string()
            }
        }
    }
}