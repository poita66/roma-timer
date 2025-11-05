//! Timer Session Model
//!
//! Represents the current timer state and configuration for pomodoro sessions.
//! Includes validation rules and business logic for timer operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Timer session types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum TimerType {
    #[serde(rename = "Work")]
    #[sqlx(rename = "Work")]
    Work,
    #[serde(rename = "ShortBreak")]
    #[sqlx(rename = "ShortBreak")]
    ShortBreak,
    #[serde(rename = "LongBreak")]
    #[sqlx(rename = "LongBreak")]
    LongBreak,
}

impl TimerType {
    /// Get the default duration in seconds for this timer type
    pub fn default_duration(&self) -> u32 {
        match self {
            TimerType::Work => 1500,      // 25 minutes
            TimerType::ShortBreak => 300,  // 5 minutes
            TimerType::LongBreak => 900,   // 15 minutes
        }
    }

    /// Get display name for this timer type
    pub fn display_name(&self) -> &'static str {
        match self {
            TimerType::Work => "Work Session",
            TimerType::ShortBreak => "Short Break",
            TimerType::LongBreak => "Long Break",
        }
    }
}

/// Timer session representing current pomodoro state
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TimerSession {
    /// Unique identifier for the session
    pub id: String,

    /// Total duration in seconds
    pub duration: u32,

    /// Elapsed time in seconds
    pub elapsed: u32,

    /// Type of timer session
    #[sqlx(rename = "timer_type")]
    pub timer_type: TimerType,

    /// Whether the timer is currently running
    #[sqlx(rename = "is_running")]
    pub is_running: bool,

    /// Creation timestamp (Unix timestamp)
    #[sqlx(rename = "created_at")]
    pub created_at: u64,

    /// Last update timestamp (Unix timestamp)
    #[sqlx(rename = "updated_at")]
    pub updated_at: u64,
}

impl TimerSession {
    /// Create a new timer session
    pub fn new(timer_type: TimerType, duration: Option<u32>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            duration: duration.unwrap_or_else(|| timer_type.default_duration()),
            elapsed: 0,
            timer_type,
            is_running: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a default work session
    pub fn new_work_session() -> Self {
        Self::new(TimerType::Work, None)
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> u32 {
        self.duration.saturating_sub(self.elapsed)
    }

    /// Check if the timer session is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Get progress as a percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.duration == 0 {
            0.0
        } else {
            self.elapsed as f64 / self.duration as f64
        }
    }

    /// Add elapsed time and return if session is complete
    pub fn add_elapsed(&mut self, seconds: u32) -> bool {
        self.elapsed = self.elapsed.saturating_add(seconds);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.is_complete()
    }

    /// Start the timer
    pub fn start(&mut self) -> Result<(), TimerSessionError> {
        if self.is_running {
            return Err(TimerSessionError::AlreadyRunning);
        }

        self.is_running = true;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Pause the timer
    pub fn pause(&mut self) -> Result<(), TimerSessionError> {
        if !self.is_running {
            return Err(TimerSessionError::NotRunning);
        }

        self.is_running = false;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Reset the timer to initial state
    pub fn reset(&mut self) {
        self.elapsed = 0;
        self.is_running = false;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Skip to next session type
    pub fn skip_to_next(&mut self, work_sessions_completed: u32, long_break_frequency: u32) {
        let next_type = match self.timer_type {
            TimerType::Work => {
                // After work, check if it's time for a long break
                if work_sessions_completed % long_break_frequency == 0 {
                    TimerType::LongBreak
                } else {
                    TimerType::ShortBreak
                }
            }
            TimerType::ShortBreak | TimerType::LongBreak => TimerType::Work,
        };

        self.timer_type = next_type.clone();
        self.duration = next_type.default_duration();
        self.elapsed = 0;
        self.is_running = false;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Validate the timer session
    pub fn validate(&self) -> Result<(), TimerSessionError> {
        // Check duration bounds (1 second to 2 hours)
        if self.duration == 0 || self.duration > 7200 {
            return Err(TimerSessionError::InvalidDuration(self.duration));
        }

        // Check elapsed bounds
        if self.elapsed > self.duration {
            return Err(TimerSessionError::InvalidElapsed(self.elapsed));
        }

        // Check timestamp consistency
        if self.updated_at < self.created_at {
            return Err(TimerSessionError::InvalidTimestamps);
        }

        Ok(())
    }
}

/// Timer session validation errors
#[derive(Debug, thiserror::Error)]
pub enum TimerSessionError {
    #[error("Timer session duration {0} is invalid (must be 1-7200 seconds)")]
    InvalidDuration(u32),

    #[error("Timer session elapsed time {0} is invalid (cannot exceed duration)")]
    InvalidElapsed(u32),

    #[error("Timer session timestamps are inconsistent")]
    InvalidTimestamps,

    #[error("Timer session is already running")]
    AlreadyRunning,

    #[error("Timer session is not running")]
    NotRunning,

    #[error("Timer session not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_session_creation() {
        let session = TimerSession::new_work_session();
        assert_eq!(session.timer_type, TimerType::Work);
        assert_eq!(session.duration, 1500); // 25 minutes
        assert_eq!(session.elapsed, 0);
        assert!(!session.is_running);
    }

    #[test]
    fn test_timer_session_progress() {
        let mut session = TimerSession::new_work_session();
        assert_eq!(session.progress(), 0.0);

        session.elapsed = 750; // Half of 1500
        assert_eq!(session.progress(), 0.5);
    }

    #[test]
    fn test_timer_session_completion() {
        let mut session = TimerSession::new_work_session();
        assert!(!session.is_complete());

        session.elapsed = 1500;
        assert!(session.is_complete());
    }

    #[test]
    fn test_timer_session_start_pause() {
        let mut session = TimerSession::new_work_session();

        // Can start when not running
        assert!(session.start().is_ok());
        assert!(session.is_running);

        // Cannot start when already running
        assert!(session.start().is_err());

        // Can pause when running
        assert!(session.pause().is_ok());
        assert!(!session.is_running);

        // Cannot pause when not running
        assert!(session.pause().is_err());
    }

    #[test]
    fn test_timer_session_skip() {
        let mut session = TimerSession::new_work_session();
        session.elapsed = 1500; // Complete work session

        // Skip to short break (default frequency of 4)
        session.skip_to_next(1, 4);
        assert_eq!(session.timer_type, TimerType::ShortBreak);
        assert_eq!(session.duration, 300); // 5 minutes
        assert_eq!(session.elapsed, 0);
    }

    #[test]
    fn test_timer_session_skip_to_long_break() {
        let mut session = TimerSession::new_work_session();
        session.elapsed = 1500; // Complete work session

        // Skip to long break after 4 work sessions
        session.skip_to_next(4, 4);
        assert_eq!(session.timer_type, TimerType::LongBreak);
        assert_eq!(session.duration, 900); // 15 minutes
    }

    #[test]
    fn test_timer_session_validation() {
        let session = TimerSession::new_work_session();
        assert!(session.validate().is_ok());

        // Test invalid duration
        let mut invalid_session = session.clone();
        invalid_session.duration = 0;
        assert!(invalid_session.validate().is_err());

        invalid_session.duration = 7201; // Over 2 hours
        assert!(invalid_session.validate().is_err());

        // Test invalid elapsed
        let mut invalid_session = session.clone();
        invalid_session.elapsed = 1600; // More than duration
        assert!(invalid_session.validate().is_err());
    }

    #[test]
    fn test_timer_type_defaults() {
        assert_eq!(TimerType::Work.default_duration(), 1500);
        assert_eq!(TimerType::ShortBreak.default_duration(), 300);
        assert_eq!(TimerType::LongBreak.default_duration(), 900);

        assert_eq!(TimerType::Work.display_name(), "Work Session");
        assert_eq!(TimerType::ShortBreak.display_name(), "Short Break");
        assert_eq!(TimerType::LongBreak.display_name(), "Long Break");
    }
}