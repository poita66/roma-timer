//! Daily Reset Database Extensions
//!
//! Extension methods for DatabaseManager to handle daily session reset operations.
//! Provides database access for analytics, scheduling, and audit logging.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Row, SqlitePool};
use uuid::Uuid;

use crate::models::{
    DailyResetTime, DailyResetTimeType, SessionResetEvent, UserConfiguration,
    UserConfigurationError,
};

/// Extension trait for DatabaseManager to support daily reset operations
pub trait DailyResetDatabaseExt {
    /// Get user configuration by ID
    async fn get_user_configuration(&self, id: &str) -> Result<Option<UserConfiguration>>;

    /// Save or update user configuration
    async fn save_user_configuration(&self, config: &UserConfiguration) -> Result<()>;

    /// Get current session count for user
    async fn get_current_session_count(&self, user_id: &str) -> Result<u32>;

    /// Update session count for user
    async fn update_session_count(&self, user_id: &str, count: u32) -> Result<()>;

    /// Increment session count for user
    async fn increment_session_count(&self, user_id: &str) -> Result<u32>;

    /// Reset session count for user
    async fn reset_session_count(&self, user_id: &str) -> Result<()>;

    /// Set manual session override
    async fn set_manual_session_override(&self, user_id: &str, count: Option<u32>) -> Result<()>;

    /// Get daily statistics for a date range
    async fn get_daily_stats(
        &self,
        user_id: &str,
        start_date: &str,
        end_date: &str,
        timezone: &str,
    ) -> Result<Vec<DailyStatsRow>>;

    /// Save daily statistics
    async fn save_daily_stats(&self, stats: &DailyStatsRow) -> Result<()>;

    /// Get recent session reset events
    async fn get_reset_events(
        &self,
        user_id: &str,
        limit: Option<i64>,
        start_date: Option<&str>,
    ) -> Result<Vec<SessionResetEventRow>>;

    /// Log session reset event
    async fn log_reset_event(&self, event: &SessionResetEventRow) -> Result<()>;

    /// Get scheduled tasks
    async fn get_scheduled_tasks(&self, task_type: &str) -> Result<Vec<ScheduledTaskRow>>;

    /// Save scheduled task
    async fn save_scheduled_task(&self, task: &ScheduledTaskRow) -> Result<()>;

    /// Delete scheduled task
    async fn delete_scheduled_task(&self, task_id: &str) -> Result<()>;

    /// Get users with daily reset enabled
    async fn get_users_with_daily_reset_enabled(&self) -> Result<Vec<UserConfiguration>>;

    /// Update last daily reset timestamp
    async fn update_last_daily_reset(&self, user_id: &str, timestamp: u64) -> Result<()>;
}

/// Daily statistics database row
#[derive(Debug, sqlx::FromRow)]
pub struct DailyStatsRow {
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

/// Session reset event database row
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

/// Scheduled task database row
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

impl DailyResetDatabaseExt for crate::database::DatabaseManager {
    async fn get_user_configuration(&self, id: &str) -> Result<Option<UserConfiguration>> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let row = query_as!(
            UserConfigRow,
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
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let config = self.row_to_user_configuration(row)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    async fn save_user_configuration(&self, config: &UserConfiguration) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        query!(
            r#"
            INSERT OR REPLACE INTO user_configurations (
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            config.id,
            config.work_duration as i64,
            config.short_break_duration as i64,
            config.long_break_duration as i64,
            config.long_break_frequency as i64,
            config.notifications_enabled,
            config.webhook_url,
            config.wait_for_interaction,
            serde_json::to_string(&config.theme)?,
            config.timezone,
            serde_json::to_string(&config.daily_reset_time_type)?,
            config.daily_reset_time_hour.map(|h| h as i64),
            config.daily_reset_time_custom,
            config.daily_reset_enabled,
            config.last_daily_reset_utc.map(|t| t as i64),
            config.today_session_count as i64,
            config.manual_session_override.map(|c| c as i64),
            config.created_at as i64,
            config.updated_at as i64
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_current_session_count(&self, user_id: &str) -> Result<u32> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let row = query!(
            r#"
            SELECT
                today_session_count,
                manual_session_override
            FROM user_configurations
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        let count = row.manual_session_override
            .unwrap_or(row.today_session_count);
        Ok(count as u32)
    }

    async fn update_session_count(&self, user_id: &str, count: u32) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE user_configurations
            SET today_session_count = ?, updated_at = ?
            WHERE id = ?
            "#,
            count as i64,
            now as i64,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn increment_session_count(&self, user_id: &str) -> Result<u32> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE user_configurations
            SET today_session_count = CASE
                WHEN manual_session_override IS NULL THEN today_session_count + 1
                ELSE today_session_count
            END,
            updated_at = ?
            WHERE id = ?
            "#,
            now as i64,
            user_id
        )
        .execute(pool)
        .await?;

        self.get_current_session_count(user_id).await
    }

    async fn reset_session_count(&self, user_id: &str) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE user_configurations
            SET today_session_count = 0,
                manual_session_override = NULL,
                last_daily_reset_utc = ?,
                updated_at = ?
            WHERE id = ?
            "#,
            now as i64,
            now as i64,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn set_manual_session_override(&self, user_id: &str, count: Option<u32>) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE user_configurations
            SET manual_session_override = ?, updated_at = ?
            WHERE id = ?
            "#,
            count.map(|c| c as i64),
            now as i64,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_daily_stats(
        &self,
        user_id: &str,
        start_date: &str,
        end_date: &str,
        timezone: &str,
    ) -> Result<Vec<DailyStatsRow>> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let rows = query_as!(
            DailyStatsRow,
            r#"
            SELECT *
            FROM daily_session_stats
            WHERE user_configuration_id = ?
            AND date >= ? AND date <= ?
            AND timezone = ?
            ORDER BY date DESC
            "#,
            user_id,
            start_date,
            end_date,
            timezone
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn save_daily_stats(&self, stats: &DailyStatsRow) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        query!(
            r#"
            INSERT OR REPLACE INTO daily_session_stats (
                id, user_configuration_id, date, timezone,
                work_sessions_completed, total_work_seconds, total_break_seconds,
                manual_overrides, final_session_count, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            stats.id,
            stats.user_configuration_id,
            stats.date,
            stats.timezone,
            stats.work_sessions_completed,
            stats.total_work_seconds,
            stats.total_break_seconds,
            stats.manual_overrides,
            stats.final_session_count,
            stats.created_at,
            stats.updated_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_reset_events(
        &self,
        user_id: &str,
        limit: Option<i64>,
        start_date: Option<&str>,
    ) -> Result<Vec<SessionResetEventRow>> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM session_reset_events WHERE user_configuration_id = ",
        );
        query_builder.push_bind(user_id);

        if let Some(start_date) = start_date {
            query_builder.push(" AND date >= ");
            query_builder.push_bind(start_date);
        }

        query_builder.push(" ORDER BY reset_timestamp_utc DESC");

        if let Some(limit) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }

        let query = query_builder.build_query_as::<SessionResetEventRow>();
        let rows = query.fetch_all(pool).await?;

        Ok(rows)
    }

    async fn log_reset_event(&self, event: &SessionResetEventRow) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        query!(
            r#"
            INSERT INTO session_reset_events (
                id, user_configuration_id, reset_type, previous_count, new_count,
                reset_timestamp_utc, user_timezone, local_reset_time, device_id,
                trigger_source, context, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            event.id,
            event.user_configuration_id,
            event.reset_type,
            event.previous_count,
            event.new_count,
            event.reset_timestamp_utc,
            event.user_timezone,
            event.local_reset_time,
            event.device_id,
            event.trigger_source,
            event.context,
            event.created_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn get_scheduled_tasks(&self, task_type: &str) -> Result<Vec<ScheduledTaskRow>> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let rows = query_as!(
            ScheduledTaskRow,
            r#"
            SELECT *
            FROM scheduled_tasks
            WHERE task_type = ? AND is_active = TRUE
            ORDER BY next_run_utc ASC
            "#,
            task_type
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn save_scheduled_task(&self, task: &ScheduledTaskRow) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        query!(
            r#"
            INSERT OR REPLACE INTO scheduled_tasks (
                id, task_type, user_configuration_id, cron_expression, timezone,
                next_run_utc, last_run_utc, is_active, run_count, failure_count,
                task_data, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            task.id,
            task.task_type,
            task.user_configuration_id,
            task.cron_expression,
            task.timezone,
            task.next_run_utc,
            task.last_run_utc,
            task.is_active,
            task.run_count,
            task.failure_count,
            task.task_data,
            task.created_at,
            task.updated_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_scheduled_task(&self, task_id: &str) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        query!("DELETE FROM scheduled_tasks WHERE id = ?", task_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn get_users_with_daily_reset_enabled(&self) -> Result<Vec<UserConfiguration>> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let rows = query_as!(
            UserConfigRow,
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
            WHERE daily_reset_enabled = TRUE
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(self.row_to_user_configuration(row)?);
        }

        Ok(configs)
    }

    async fn update_last_daily_reset(&self, user_id: &str, timestamp: u64) -> Result<()> {
        let pool = match &self.pool {
            crate::database::DatabasePool::Sqlite(pool) => pool,
        };

        let now = Utc::now().timestamp() as u64;

        query!(
            r#"
            UPDATE user_configurations
            SET last_daily_reset_utc = ?, updated_at = ?
            WHERE id = ?
            "#,
            timestamp as i64,
            now as i64,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Internal helper row structure for user configuration queries
#[derive(Debug, sqlx::FromRow)]
struct UserConfigRow {
    pub id: String,
    pub work_duration: i64,
    pub short_break_duration: i64,
    pub long_break_duration: i64,
    pub long_break_frequency: i64,
    pub notifications_enabled: bool,
    pub webhook_url: Option<String>,
    pub wait_for_interaction: bool,
    pub theme: String,
    pub timezone: String,
    pub daily_reset_time_type: String,
    pub daily_reset_time_hour: Option<i64>,
    pub daily_reset_time_custom: Option<String>,
    pub daily_reset_enabled: bool,
    pub last_daily_reset_utc: Option<i64>,
    pub today_session_count: i64,
    pub manual_session_override: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl crate::database::DatabaseManager {
    /// Helper method to convert database row to UserConfiguration
    fn row_to_user_configuration(&self, row: UserConfigRow) -> Result<UserConfiguration> {
        let theme: crate::models::Theme = serde_json::from_str(&row.theme)?;
        let daily_reset_time_type: DailyResetTimeType = serde_json::from_str(&row.daily_reset_time_type)?;

        Ok(UserConfiguration {
            id: row.id,
            work_duration: row.work_duration as u32,
            short_break_duration: row.short_break_duration as u32,
            long_break_duration: row.long_break_duration as u32,
            long_break_frequency: row.long_break_frequency as u32,
            notifications_enabled: row.notifications_enabled,
            webhook_url: row.webhook_url,
            wait_for_interaction: row.wait_for_interaction,
            theme,
            timezone: row.timezone,
            daily_reset_time_type,
            daily_reset_time_hour: row.daily_reset_time_hour.map(|h| h as u8),
            daily_reset_time_custom: row.daily_reset_time_custom,
            daily_reset_enabled: row.daily_reset_enabled,
            last_daily_reset_utc: row.last_daily_reset_utc.map(|t| t as u64),
            today_session_count: row.today_session_count as u32,
            manual_session_override: row.manual_session_override.map(|c| c as u32),
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
        })
    }
}