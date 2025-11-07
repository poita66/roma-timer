-- Migration 002: Daily Session Reset Feature
-- Adds support for daily session reset with timezone-aware scheduling
-- Includes analytics, audit logging, and manual session override functionality

-- Daily Session Reset Migration
-- Version: 002
-- Created: 2025-01-07
-- Description: Add daily session reset functionality to Roma Timer

-- Begin transaction
BEGIN;

-- Extend user_configurations table with daily reset fields
ALTER TABLE user_configurations
ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC',
ADD COLUMN daily_reset_time_type TEXT NOT NULL DEFAULT 'midnight',
ADD COLUMN daily_reset_time_hour INTEGER,
ADD COLUMN daily_reset_time_custom TEXT,
ADD COLUMN daily_reset_enabled BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN last_daily_reset_utc INTEGER,
ADD COLUMN today_session_count INTEGER NOT NULL DEFAULT 0,
ADD COLUMN manual_session_override INTEGER;

-- Create index on timezone for performance
CREATE INDEX idx_user_configurations_timezone ON user_configurations(timezone);

-- Create index on daily reset enabled for performance
CREATE INDEX idx_user_configurations_daily_reset_enabled ON user_configurations(daily_reset_enabled);

-- Create index on last daily reset for performance
CREATE INDEX idx_user_configurations_last_daily_reset ON user_configurations(last_daily_reset_utc);

-- Create daily_session_stats table for analytics
CREATE TABLE daily_session_stats (
    id TEXT PRIMARY KEY,
    user_configuration_id TEXT NOT NULL,
    date TEXT NOT NULL, -- Format: YYYY-MM-DD in UTC
    timezone TEXT NOT NULL,

    -- Session statistics
    work_sessions_completed INTEGER NOT NULL DEFAULT 0,
    total_work_seconds INTEGER NOT NULL DEFAULT 0,
    total_break_seconds INTEGER NOT NULL DEFAULT 0,

    -- Manual overrides tracking
    manual_overrides INTEGER NOT NULL DEFAULT 0,
    final_session_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (user_configuration_id) REFERENCES user_configurations(id) ON DELETE CASCADE,

    -- Unique constraint for one stat record per user per day
    UNIQUE(user_configuration_id, date, timezone)
);

-- Create indexes for daily_session_stats
CREATE INDEX idx_daily_session_stats_user_date ON daily_session_stats(user_configuration_id, date);
CREATE INDEX idx_daily_session_stats_date ON daily_session_stats(date);
CREATE INDEX idx_daily_session_stats_timezone ON daily_session_stats(timezone);
CREATE INDEX idx_daily_session_stats_created_at ON daily_session_stats(created_at);

-- Create scheduled_tasks table for background job persistence
CREATE TABLE scheduled_tasks (
    id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL, -- 'daily_reset', 'cleanup', etc.
    user_configuration_id TEXT,

    -- Scheduling information
    cron_expression TEXT NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    next_run_utc INTEGER NOT NULL,
    last_run_utc INTEGER,

    -- Task status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    run_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,

    -- Task data (JSON for flexibility)
    task_data TEXT, -- JSON string for additional task-specific data

    -- Timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (user_configuration_id) REFERENCES user_configurations(id) ON DELETE CASCADE
);

-- Create indexes for scheduled_tasks
CREATE INDEX idx_scheduled_tasks_type_active ON scheduled_tasks(task_type, is_active);
CREATE INDEX idx_scheduled_tasks_next_run ON scheduled_tasks(next_run_utc);
CREATE INDEX idx_scheduled_tasks_user_config ON scheduled_tasks(user_configuration_id);
CREATE INDEX idx_scheduled_tasks_cron_timezone ON scheduled_tasks(cron_expression, timezone);

-- Create session_reset_events table for audit logging
CREATE TABLE session_reset_events (
    id TEXT PRIMARY KEY,
    user_configuration_id TEXT NOT NULL,

    -- Reset event information
    reset_type TEXT NOT NULL, -- 'scheduled_daily', 'manual_reset', 'timezone_change', 'configuration_change'
    previous_count INTEGER NOT NULL DEFAULT 0,
    new_count INTEGER NOT NULL DEFAULT 0,

    -- Time information
    reset_timestamp_utc INTEGER NOT NULL,
    user_timezone TEXT NOT NULL,
    local_reset_time TEXT NOT NULL, -- Format: YYYY-MM-DD HH:MM:SS in user's timezone

    -- Source information
    device_id TEXT, -- Which device triggered the reset
    trigger_source TEXT NOT NULL, -- 'background_service', 'user_action', 'api_call', etc.

    -- Additional context (JSON for flexibility)
    context TEXT, -- JSON string for additional context

    -- Timestamps
    created_at INTEGER NOT NULL,

    -- Foreign key constraints
    FOREIGN KEY (user_configuration_id) REFERENCES user_configurations(id) ON DELETE CASCADE
);

-- Create indexes for session_reset_events
CREATE INDEX idx_session_reset_events_user_config ON session_reset_events(user_configuration_id);
CREATE INDEX idx_session_reset_events_reset_type ON session_reset_events(reset_type);
CREATE INDEX idx_session_reset_events_timestamp ON session_reset_events(reset_timestamp_utc);
CREATE INDEX idx_session_reset_events_created_at ON session_reset_events(created_at);
CREATE INDEX idx_session_reset_events_date ON session_reset_events(user_configuration_id, reset_timestamp_utc);

-- Insert initial daily reset configuration for existing users
UPDATE user_configurations
SET
    timezone = 'UTC',
    daily_reset_time_type = 'midnight',
    daily_reset_enabled = FALSE,
    today_session_count = 0,
    last_daily_reset_utc = CASE
        WHEN updated_at IS NOT NULL THEN updated_at
        ELSE CAST(strftime('%s', 'now') AS INTEGER)
    END
WHERE daily_reset_enabled IS NULL;

-- Create default daily session stats records for active users
INSERT INTO daily_session_stats (
    id,
    user_configuration_id,
    date,
    timezone,
    work_sessions_completed,
    total_work_seconds,
    total_break_seconds,
    manual_overrides,
    final_session_count,
    created_at,
    updated_at
)
SELECT
    'daily_stats_' || id || '_' || date('now'),
    id,
    date('now'),
    COALESCE(timezone, 'UTC'),
    0,
    0,
    0,
    0,
    0,
    CAST(strftime('%s', 'now') AS INTEGER),
    CAST(strftime('%s', 'now') AS INTEGER)
FROM user_configurations
WHERE id != 'default-config' -- Skip default config if it exists
AND NOT EXISTS (
    SELECT 1 FROM daily_session_stats
    WHERE user_configuration_id = user_configurations.id
    AND date = date('now')
);

-- Create a default daily reset task for users with daily reset enabled
INSERT INTO scheduled_tasks (
    id,
    task_type,
    user_configuration_id,
    cron_expression,
    timezone,
    next_run_utc,
    is_active,
    run_count,
    failure_count,
    created_at,
    updated_at
)
SELECT
    'daily_reset_task_' || id,
    'daily_reset',
    id,
    CASE
        WHEN daily_reset_time_type = 'midnight' THEN '0 0 * * *'  -- At midnight
        WHEN daily_reset_time_type = 'hour' AND daily_reset_time_hour IS NOT NULL
            THEN '0 ' || daily_reset_time_hour || ' * * *'  -- At specified hour
        WHEN daily_reset_time_type = 'custom' AND daily_reset_time_custom IS NOT NULL
            THEN '0 ' || REPLACE(daily_reset_time_custom, ':', ' ') || ' * * *'  -- Custom time
        ELSE '0 0 * * *'  -- Default to midnight
    END,
    COALESCE(timezone, 'UTC'),
    CASE
        WHEN daily_reset_enabled AND last_daily_reset_utc IS NOT NULL THEN last_daily_reset_utc + 86400  -- Next day
        WHEN daily_reset_enabled THEN CAST(strftime('%s', 'now', '+1 day') AS INTEGER)
        ELSE CAST(strftime('%s', 'now', '+1 year') AS INTEGER)  -- Far future if disabled
    END,
    daily_reset_enabled,
    0,
    0,
    CAST(strftime('%s', 'now') AS INTEGER),
    CAST(strftime('%s', 'now') AS INTEGER)
FROM user_configurations
WHERE daily_reset_enabled = TRUE
AND id != 'default-config';

-- Commit transaction
COMMIT;

-- Post-migration notes:
-- 1. All existing users now have timezone and daily reset fields added
-- 2. Daily reset is disabled by default for existing users
-- 3. Analytics tables are created for session statistics
-- 4. Background task persistence is set up for scheduled resets
-- 5. Audit logging is configured for all reset events
-- 6. Indexes are created for performance optimization
-- 7. Default daily stats records are created for current date
-- 8. Background tasks are created for users with daily reset enabled

-- To verify migration success:
-- SELECT COUNT(*) FROM user_configurations WHERE timezone IS NOT NULL;
-- SELECT COUNT(*) FROM daily_session_stats;
-- SELECT COUNT(*) FROM scheduled_tasks WHERE task_type = 'daily_reset';
-- SELECT COUNT(*) FROM session_reset_events;