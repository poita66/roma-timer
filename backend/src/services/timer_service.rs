//! Timer Service
//!
//! Core business logic for timer operations including countdown,
//! session transitions, and real-time state management.

use crate::models::timer_session::{TimerSession, TimerType, TimerSessionError};
use crate::models::user_configuration::UserConfiguration;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Timer service for managing pomodoro sessions
#[derive(Debug, Clone)]
pub struct TimerService {
    /// Current timer session
    session: Arc<RwLock<TimerSession>>,

    /// User configuration
    config: Arc<RwLock<UserConfiguration>>,

    /// Work sessions completed in current cycle
    work_sessions_completed: Arc<Mutex<u32>>,

    /// Last update timestamp for accurate time tracking
    last_update: Arc<Mutex<u64>>,
}

/// Timer state for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimerState {
    pub id: String,
    pub duration: u32,
    pub elapsed: u32,
    pub timer_type: String,
    pub is_running: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub remaining_seconds: u32,
    pub progress_percentage: f64,
    pub session_count: u32,
}

impl TimerService {
    /// Create a new timer service with default configuration
    pub fn new() -> Self {
        Self::with_config(UserConfiguration::default())
    }

    /// Create a new timer service with custom configuration
    pub fn with_config(config: UserConfiguration) -> Self {
        let session = TimerSession::new_work_session();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            session: Arc::new(RwLock::new(session)),
            config: Arc::new(RwLock::new(config)),
            work_sessions_completed: Arc::new(Mutex::new(0)),
            last_update: Arc::new(Mutex::new(now)),
        }
    }

    /// Get current timer state
    pub async fn get_timer_state(&self) -> TimerState {
        let session = self.session.read().await;
        let work_sessions = *self.work_sessions_completed.lock().await;

        TimerState {
            id: session.id.clone(),
            duration: session.duration,
            elapsed: session.elapsed,
            timer_type: format!("{:?}", session.timer_type),
            is_running: session.is_running,
            created_at: session.created_at,
            updated_at: session.updated_at,
            remaining_seconds: session.remaining_seconds(),
            progress_percentage: session.progress() * 100.0,
            session_count: work_sessions,
        }
    }

    /// Start the timer
    pub async fn start_timer(&self) -> Result<(), TimerServiceError> {
        let mut session = self.session.write().await;

        if session.is_running {
            return Err(TimerServiceError::AlreadyRunning);
        }

        session.start()?;

        // Update last update timestamp
        *self.last_update.lock().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Pause the timer
    pub async fn pause_timer(&self) -> Result<(), TimerServiceError> {
        let mut session = self.session.write().await;

        if !session.is_running {
            return Err(TimerServiceError::NotRunning);
        }

        // Update elapsed time based on when timer was started
        self.update_elapsed_time(&mut session).await;

        session.pause()?;

        Ok(())
    }

    /// Reset the timer
    pub async fn reset_timer(&self) -> Result<(), TimerServiceError> {
        let mut session = self.session.write().await;

        session.reset();

        // Reset to work session
        session.timer_type = TimerType::Work;
        session.duration = session.timer_type.default_duration();

        *self.work_sessions_completed.lock().await = 0;
        *self.last_update.lock().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Skip to next session
    pub async fn skip_timer(&self) -> Result<(), TimerServiceError> {
        let mut session = self.session.write().await;
        let config = self.config.read().await;
        let mut work_sessions = self.work_sessions_completed.lock().await;

        // Update elapsed time before skipping
        self.update_elapsed_time(&mut session).await;

        // Handle session completion if work session was active
        if session.timer_type == TimerType::Work && session.elapsed > 0 {
            *work_sessions += 1;
        }

        session.skip_to_next(*work_sessions, config.long_break_frequency);

        *self.last_update.lock().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Get current session type
    pub async fn get_session_type(&self) -> TimerType {
        let session = self.session.read().await;
        session.timer_type.clone()
    }

    /// Set session type
    pub async fn set_session_type(&self, timer_type: TimerType) {
        let mut session = self.session.write().await;

        self.update_elapsed_time(&mut session).await;

        session.timer_type = timer_type;
        session.duration = timer_type.default_duration();
        session.elapsed = 0;
        session.is_running = false;

        *self.last_update.lock().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get elapsed time
    pub async fn get_elapsed_time(&self) -> u32 {
        let mut session = self.session.write().await;
        self.update_elapsed_time(&mut session).await;
        session.elapsed
    }

    /// Set timer duration
    pub async fn set_duration(&self, duration: u32) {
        let mut session = self.session.write().await;

        self.update_elapsed_time(&mut session).await;

        session.duration = duration;

        // Ensure elapsed doesn't exceed new duration
        if session.elapsed > duration {
            session.elapsed = duration;
        }
    }

    /// Check if timer is running
    pub async fn is_running(&self) -> bool {
        let session = self.session.read().await;
        session.is_running
    }

    /// Complete current session and transition to next
    pub async fn complete_current_session(&self) {
        let mut session = self.session.write().await;
        let config = self.config.read().await;
        let mut work_sessions = self.work_sessions_completed.lock().await;

        if session.timer_type == TimerType::Work {
            *work_sessions += 1;
        }

        session.skip_to_next(*work_sessions, config.long_break_frequency);

        *self.last_update.lock().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Increment work session count (for testing)
    pub async fn increment_work_session_count(&self) {
        let mut work_sessions = self.work_sessions_completed.lock().await;
        *work_sessions += 1;
    }

    /// Set long break frequency
    pub async fn set_long_break_frequency(&self, frequency: u32) {
        let mut config = self.config.write().await;
        config.long_break_frequency = frequency;
    }

    /// Update elapsed time based on current time
    async fn update_elapsed_time(&self, session: &mut TimerSession) -> bool {
        if session.is_running {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let last_update = *self.last_update.lock().await;
            let elapsed_since_update = (now - last_update) as u32;

            if elapsed_since_update > 0 {
                let completed = session.add_elapsed(elapsed_since_update);
                *self.last_update.lock().await = now;
                return completed;
            }
        }
        false
    }

    /// Start background timer task for automatic updates
    pub async fn start_background_timer(&self) {
        let service = self.clone();
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let session_clone = service.session.clone();
            let mut session = session_clone.write().await;
            if session.is_running {
                let completed = service.update_elapsed_time(&mut session).await;

                if completed {
                    // Timer completed, trigger completion logic
                    drop(session); // Release lock before calling complete_current_session
                    service.complete_current_session().await;

                    // Send completion notification if needed
                    // This could integrate with notification service
                }
            }
        }
    }
}

/// Timer service errors
#[derive(Debug, thiserror::Error)]
pub enum TimerServiceError {
    #[error("Timer is already running")]
    AlreadyRunning,

    #[error("Timer is not running")]
    NotRunning,

    #[error("Timer session error: {0}")]
    SessionError(#[from] TimerSessionError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_timer_service_creation() {
        let service = TimerService::new();

        assert!(!service.is_running().await);
        assert_eq!(service.get_session_type().await, TimerType::Work);
    }

    #[tokio::test]
    async fn test_timer_start_stop() {
        let service = TimerService::new();

        // Start timer
        assert!(service.start_timer().await.is_ok());
        assert!(service.is_running().await);

        // Pause timer
        assert!(service.pause_timer().await.is_ok());
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_timer_reset() {
        let service = TimerService::new();

        // Start timer and let it run
        service.start_timer().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Reset timer
        assert!(service.reset_timer().await.is_ok());
        assert!(!service.is_running().await);
        assert_eq!(service.get_elapsed_time().await, 0);
    }

    #[tokio::test]
    async fn test_timer_skip() {
        let service = TimerService::new();

        let initial_type = service.get_session_type().await;
        assert_eq!(initial_type, TimerType::Work);

        // Skip timer
        assert!(service.skip_timer().await.is_ok());
        assert_eq!(service.get_session_type().await, TimerType::ShortBreak);
    }

    #[tokio::test]
    async fn test_timer_state() {
        let service = TimerService::new();
        let state = service.get_timer_state().await;

        assert_eq!(state.timer_type, "Work");
        assert_eq!(state.duration, 1500); // 25 minutes
        assert_eq!(state.elapsed, 0);
        assert!(!state.is_running);
        assert_eq!(state.remaining_seconds, 1500);
        assert_eq!(state.progress_percentage, 0.0);
    }

    #[tokio::test]
    async fn test_elapsed_time_tracking() {
        let service = TimerService::new();

        service.set_duration(2).await; // 2 seconds for testing
        service.start_timer().await.unwrap();

        tokio::time::sleep(Duration::from_millis(1100)).await; // 1.1 seconds

        let elapsed = service.get_elapsed_time().await;
        assert!(elapsed >= 1);
        assert!(elapsed < 2);
    }
}