//! Database Manager Extensions for Daily Session Reset
//!
//! Provides database operations for daily session reset functionality including:
//! - User configuration with timezone and reset settings
//! - Daily session statistics
//! - Scheduled tasks persistence
//! - Session reset event logging

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, SqlitePool, Row};
use uuid::Uuid;

use crate::models::{UserConfiguration, UserConfigurationError};

/// Extension trait for DatabaseManager to support daily reset operations
pub trait DailyResetDatabaseExtensions {
    /// Get user configuration with daily reset fields
    async fn get_user_configuration(&self, user_id: &str) -> Result<Option<UserConfiguration>>;

    /// Save user configuration with daily reset fields
    async fn save_user_configuration(&self, config: &UserConfiguration) -> Result<()>;

    /// Get or create default user configuration
    async fn get_or_create_user_config(&self, user_id: &str) -> Result<UserConfiguration>;

    /// Record a daily session statistic
    async fn record_daily_session_stat(
        &self,
        user_id: &str,
        date: &str,
        timezone: &str,
        work_sessions: u32,
        work_seconds: u64,
        break_seconds: u64,
        manual_overrides: u32,
        final_session_count: u32,
    ) -> Result<()>;

    /// Get daily session statistics for a user
    async fn get_daily_session_stats(
        &self,
        user_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailySessionStatRow>>;

    /// Create or update a scheduled task
    async fn upsert_scheduled_task(&self, task: &ScheduledTaskData) -> Result<()>;

    /// Get active scheduled tasks for execution
    async fn get_active_scheduled_tasks(&self) -> Result<Vec<ScheduledTaskRow>>;

    /// Update scheduled task execution status
    async fn update_scheduled_task_execution(
        &self,
        task_id: &str,
        last_run_utc: u64,
        run_count: u32,
        failure_count: u32,
    ) -> Result<()>;

    /// Record a session reset event
    async fn record_session_reset_event(&self, event: &SessionResetEventData) -> Result<()>;

    /// Get recent session reset events for a user
    async fn get_session_reset_events(
        &self,
        user_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SessionResetEventRow>>;

    /// Cleanup old records (for maintenance)
    async fn cleanup_old_records(&self, days_to_keep: u32) -> Result<()>;
}

/// Database row structures for daily reset operations

#[derive(Debug, sqlx::FromRow)]
pub struct DailySessionStatRow {
    pub id: String,
    pub user_configuration_id: String,
    pub date: String,
    pub timezone: String,
    pub work_sessions_completed: i64,
    pub total_work_seconds: i64,
    pub total_break_seconds: i64,
    pub manual_overrides: i64,
    pub final_session_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ScheduledTaskRow {
    pub id: String,
    pub task_type: String,
    pub user_configuration_id: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub next_run_utc: i64,
    pub last_run_utc: Option<i64>,
    pub is_active: bool,
    pub run_count: i64,
    pub failure_count: i64,
    pub task_data: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionResetEventRow {
    pub id: String,
    pub user_configuration_id: String,
    pub reset_type: String,
    pub previous_count: i64,
    pub new_count: i64,
    pub reset_timestamp_utc: i64,
    pub user_timezone: String,
    pub local_reset_time: String,
    pub device_id: Option<String>,
    pub trigger_source: String,
    pub context: Option<String>,
    pub created_at: i64,
}

/// Data structures for database operations

#[derive(Debug)]
pub struct ScheduledTaskData {
    pub id: String,
    pub task_type: String,
    pub user_configuration_id: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub next_run_utc: u64,
    pub last_run_utc: Option<u64>,
    pub is_active: bool,
    pub run_count: u32,
    pub failure_count: u32,
    pub task_data: Option<String>,
}

#[derive(Debug)]
pub struct SessionResetEventData {
    pub id: String,
    pub user_configuration_id: String,
    pub reset_type: String,
    pub previous_count: u32,
    pub new_count: u32,
    pub reset_timestamp_utc: u64,
    pub user_timezone: String,
    pub local_reset_time: String,
    pub device_id: Option<String>,
    pub trigger_source: String,
    pub context: Option<String>,
}

impl DailyResetDatabaseExtensions for super::DatabaseManager {
    fn get_pool(&self) -> &SqlitePool {
        match &self.pool {
            super::DatabasePool::Sqlite(pool) => pool,
        }
    }

    async fn get_user_configuration(&self, user_id: &str) -> Result<Option<UserConfiguration>> {
        let pool = self.get_pool();

        let row = query_as!(
            UserConfigurationRow,
            r#"
            SELECT
                id,
                work_duration,
                short_break_duration,
                long_break_duration,
                long_break_frequency,
                notifications_enabled,
                webhook_url,
                wait_for_interaction,
                theme,
                timezone,
                daily_reset_time_type,
                daily_reset_time_hour,
                daily_reset_time_custom,
                daily_reset_enabled,
                last_daily_reset_utc,
                today_session_count,
                manual_session_override,
                created_at,
                updated_at
            FROM user_configurations
            WHERE id = ?1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let config = UserConfiguration {
                    id: row.id,
                    work_duration: row.work_duration as u32,
                    short_break_duration: row.short_break_duration as u32,
                    long_break_duration: row.long_break_duration as u32,
                    long_break_frequency: row.long_break_frequency as u32,
                    notifications_enabled: row.notifications_enabled,
                    webhook_url: row.webhook_url,
                    wait_for_interaction: row.wait_for_interaction,
                    theme: crate::models::Theme::Light, // Parse from string
                    timezone: row.timezone,
                    daily_reset_time_type: crate::models::DailyResetTimeType::Midnight, // Parse from string
                    daily_reset_time_hour: row.daily_reset_time_hour.map(|h| h as u8),
                    daily_reset_time_custom: row.daily_reset_time_custom,
                    daily_reset_enabled: row.daily_reset_enabled,
                    last_daily_reset_utc: row.last_daily_reset_utc.map(|t| t as u64),
                    today_session_count: row.today_session_count as u32,
                    manual_session_override: row.manual_session_override.map(|c| c as u32),
                    created_at: row.created_at as u64,
                    updated_at: row.updated_at as u64,
                };
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    async fn save_user_configuration(&self, config: &UserConfiguration) -> Result<()> {
        let pool = self.get_pool();

        query!(
            r#"
            INSERT OR REPLACE INTO user_configurations (
                id, work_duration, short_break_duration, long_break_duration,
                long_break_frequency, notifications_enabled, webhook_url,
                wait_for_interaction, theme, timezone, daily_reset_time_type,
                daily_reset_time_hour, daily_reset_time_custom, daily_reset_enabled,
                last_daily_reset_utc, today_session_count, manual_session_override,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19
            )
            "#,
            config.id,
            config.work_duration as i64,
            config.short_break_duration as i64,
            config.long_break_duration as i64,
            config.long_break_frequency as i64,
            config.notifications_enabled,
            config.webhook_url,
            config.wait_for_interaction,
            "Light", // Convert theme to string
            config.timezone,
            "midnight", // Convert reset time type to string
            config.daily_reset_time_hour.map(|h| h as i64),
            config.daily_reset_time_custom,
            config.daily_reset_enabled,
            config.last_daily_reset_utc.map(|t| t as i64),
            config.today_session_count as i64,
            config.manual_session_override.map(|c| c as i64),
            config.created_at as i64,
            config.updated_at as i64,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_or_create_user_config(&self, user_id: &str) -> Result<UserConfiguration> {
        if let Some(config) = self.get_user_configuration(user_id).await? {
            return Ok(config);
        }

        // Create default configuration
        let now = Utc::now().timestamp() as u64;
        let mut config = UserConfiguration::with_id(user_id.to_string());
        config.created_at = now;
        config.updated_at = now;

        self.save_user_configuration(&config).await?;
        Ok(config)
    }

    async fn record_daily_session_stat(
        &self,
        user_id: &str,
        date: &str,
        timezone: &str,
        work_sessions: u32,
        work_seconds: u64,
        break_seconds: u64,
        manual_overrides: u32,
        final_session_count: u32,
    ) -> Result<()> {
        let pool = self.get_pool();
        let now = Utc::now().timestamp() as u64;
        let id = format!("daily_stats_{}_{}", user_id, date);

        query!(
            r#"
            INSERT OR REPLACE INTO daily_session_stats (
                id, user_configuration_id, date, timezone,
                work_sessions_completed, total_work_seconds, total_break_seconds,
                manual_overrides, final_session_count, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
            )
            "#,
            id,
            user_id,
            date,
            timezone,
            work_sessions as i64,
            work_seconds as i64,
            break_seconds as i64,
            manual_overrides as i64,
            final_session_count as i64,
            now as i64,
            now as i64,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_daily_session_stats(
        &self,
        user_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailySessionStatRow>> {
        let pool = self.get_pool();

        let rows = query_as!(
            DailySessionStatRow,
            r#"
            SELECT * FROM daily_session_stats
            WHERE user_configuration_id = ?1
            AND date BETWEEN ?2 AND ?3
            ORDER BY date DESC
            "#,
            user_id,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn upsert_scheduled_task(&self, task: &ScheduledTaskData) -> Result<()> {
        let pool = self.get_pool();
        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            INSERT OR REPLACE INTO scheduled_tasks (
                id, task_type, user_configuration_id, cron_expression, timezone,
                next_run_utc, last_run_utc, is_active, run_count, failure_count,
                task_data, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
            )
            "#,
            task.id,
            task.task_type,
            task.user_configuration_id,
            task.cron_expression,
            task.timezone,
            task.next_run_utc as i64,
            task.last_run_utc.map(|t| t as i64),
            task.is_active,
            task.run_count as i64,
            task.failure_count as i64,
            task.task_data,
            now as i64,
            now as i64,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_active_scheduled_tasks(&self) -> Result<Vec<ScheduledTaskRow>> {
        let pool = self.get_pool();
        let now = Utc::now().timestamp() as i64;

        let rows = query_as!(
            ScheduledTaskRow,
            r#"
            SELECT * FROM scheduled_tasks
            WHERE is_active = TRUE
            AND next_run_utc <= ?1
            ORDER BY next_run_utc ASC
            "#,
            now
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn update_scheduled_task_execution(
        &self,
        task_id: &str,
        last_run_utc: u64,
        run_count: u32,
        failure_count: u32,
    ) -> Result<()> {
        let pool = self.get_pool();
        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE scheduled_tasks
            SET last_run_utc = ?1,
                run_count = ?2,
                failure_count = ?3,
                updated_at = ?4
            WHERE id = ?5
            "#,
            last_run_utc as i64,
            run_count as i64,
            failure_count as i64,
            now as i64,
            task_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn record_session_reset_event(&self, event: &SessionResetEventData) -> Result<()> {
        let pool = self.get_pool();
        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            INSERT INTO session_reset_events (
                id, user_configuration_id, reset_type, previous_count,
                new_count, reset_timestamp_utc, user_timezone, local_reset_time,
                device_id, trigger_source, context, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
            )
            "#,
            event.id,
            event.user_configuration_id,
            event.reset_type,
            event.previous_count as i64,
            event.new_count as i64,
            event.reset_timestamp_utc as i64,
            event.user_timezone,
            event.local_reset_time,
            event.device_id,
            event.trigger_source,
            event.context,
            now as i64,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_session_reset_events(
        &self,
        user_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SessionResetEventRow>> {
        let pool = self.get_pool();

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let query_str = format!(
            r#"
            SELECT * FROM session_reset_events
            WHERE user_configuration_id = ?1
            ORDER BY reset_timestamp_utc DESC
            {}
            "#,
            limit_clause
        );

        let mut query = sqlx::query_as::<_, SessionResetEventRow>(&query_str)
            .bind(user_id);

        let rows = query.fetch_all(pool).await?;
        Ok(rows)
    }

    async fn cleanup_old_records(&self, days_to_keep: u32) -> Result<()> {
        let pool = self.get_pool();
        let cutoff_timestamp = (Utc::now().timestamp() - (days_to_keep as i64 * 24 * 60 * 60)) as i64;

        // Cleanup old daily session stats
        query!(
            "DELETE FROM daily_session_stats WHERE created_at < ?1",
            cutoff_timestamp
        )
        .execute(pool)
        .await?;

        // Cleanup old session reset events (keep more of these for audit trail)
        let events_cutoff = (Utc::now().timestamp() - (days_to_keep as i64 * 2 * 24 * 60 * 60)) as i64;
        query!(
            "DELETE FROM session_reset_events WHERE created_at < ?1",
            events_cutoff
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Helper struct for user configuration database row
#[derive(sqlx::FromRow)]
struct UserConfigurationRow {
    id: String,
    work_duration: i64,
    short_break_duration: i64,
    long_break_duration: i64,
    long_break_frequency: i64,
    notifications_enabled: bool,
    webhook_url: Option<String>,
    wait_for_interaction: bool,
    theme: String,
    timezone: String,
    daily_reset_time_type: String,
    daily_reset_time_hour: Option<i64>,
    daily_reset_time_custom: Option<String>,
    daily_reset_enabled: bool,
    last_daily_reset_utc: Option<i64>,
    today_session_count: i64,
    manual_session_override: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;

    #[tokio::test]
    async fn test_daily_reset_database_extensions() {
        // This would require setting up a test database
        // For now, we'll just ensure the trait compiles correctly
        fn _assert_implemented() {
            fn _impl<T: DailyResetDatabaseExtensions>(_: T) {}
            let db: DatabaseManager = unsafe { std::mem::zeroed() };
            _impl(db);
        }
    }
}