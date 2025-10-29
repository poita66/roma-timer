-- Initial database schema for Roma Timer
-- This migration creates the core tables for timer sessions, user configurations, and notification events

-- Timer sessions table
-- Stores the current and historical timer session states
CREATE TABLE IF NOT EXISTS timer_sessions (
    id TEXT PRIMARY KEY,
    duration INTEGER NOT NULL,
    elapsed INTEGER NOT NULL DEFAULT 0,
    timer_type TEXT NOT NULL CHECK (timer_type IN ('Work', 'ShortBreak', 'LongBreak')),
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- User configurations table
-- Stores user preferences and timer settings
CREATE TABLE IF NOT EXISTS user_configurations (
    id TEXT PRIMARY KEY,
    work_duration INTEGER NOT NULL DEFAULT 1500,    -- 25 minutes in seconds
    short_break_duration INTEGER NOT NULL DEFAULT 300,  -- 5 minutes in seconds
    long_break_duration INTEGER NOT NULL DEFAULT 900,   -- 15 minutes in seconds
    long_break_frequency INTEGER NOT NULL DEFAULT 4,
    notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    webhook_url TEXT,
    wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
    theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Notification events table
-- Stores timer completion notifications for delivery tracking
CREATE TABLE IF NOT EXISTS notification_events (
    id TEXT PRIMARY KEY,
    timer_session_id TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('WorkSessionComplete', 'BreakSessionComplete', 'TimerSkipped', 'TimerReset')),
    message TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    delivered_at INTEGER,
    FOREIGN KEY (timer_session_id) REFERENCES timer_sessions(id)
);

-- Create indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_timer_sessions_updated_at ON timer_sessions(updated_at);
CREATE INDEX IF NOT EXISTS idx_timer_sessions_is_running ON timer_sessions(is_running);
CREATE INDEX IF NOT EXISTS idx_notification_events_timer_session_id ON notification_events(timer_session_id);
CREATE INDEX IF NOT EXISTS idx_notification_events_created_at ON notification_events(created_at);

-- Insert default user configuration
INSERT OR IGNORE INTO user_configurations (
    id,
    work_duration,
    short_break_duration,
    long_break_duration,
    long_break_frequency,
    notifications_enabled,
    wait_for_interaction,
    theme,
    created_at,
    updated_at
) VALUES (
    'default-config',
    1500,  -- 25 minutes
    300,   -- 5 minutes
    900,   -- 15 minutes
    4,     -- Long break after 4 work sessions
    TRUE,  -- Notifications enabled
    FALSE, -- Don't wait for interaction
    'Light',
    strftime('%s', 'now'),
    strftime('%s', 'now')
);