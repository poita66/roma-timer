#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_timer_start() {
        let mut timer_service = TimerService::new();

        // Initially timer should be stopped
        assert!(!timer_service.is_running());

        // Start timer
        let result = timer_service.start_timer().await;
        assert!(result.is_ok());

        // Timer should be running
        assert!(timer_service.is_running());
    }

    #[tokio::test]
    async fn test_timer_pause() {
        let mut timer_service = TimerService::new();

        // Start timer first
        timer_service.start_timer().await.unwrap();
        assert!(timer_service.is_running());

        // Pause timer
        let result = timer_service.pause_timer().await;
        assert!(result.is_ok());

        // Timer should be paused
        assert!(!timer_service.is_running());
    }

    #[tokio::test]
    async fn test_timer_reset() {
        let mut timer_service = TimerService::new();

        // Start timer and let it run
        timer_service.start_timer().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Timer should have progressed
        let before_reset_elapsed = timer_service.get_elapsed_time();
        assert!(before_reset_elapsed > 0);

        // Reset timer
        let result = timer_service.reset_timer().await;
        assert!(result.is_ok());

        // Elapsed time should be 0 and timer should be stopped
        assert_eq!(timer_service.get_elapsed_time(), 0);
        assert!(!timer_service.is_running());
    }

    #[tokio::test]
    async fn test_timer_skip() {
        let mut timer_service = TimerService::new();

        // Set initial state as Work session
        timer_service.set_session_type(TimerType::Work);
        assert_eq!(timer_service.get_session_type(), TimerType::Work);

        // Skip to next session
        let result = timer_service.skip_timer().await;
        assert!(result.is_ok());

        // Should transition to ShortBreak (or appropriate next session)
        assert_ne!(timer_service.get_session_type(), TimerType::Work);
        assert_eq!(timer_service.get_elapsed_time(), 0);
        assert!(!timer_service.is_running());
    }

    #[tokio::test]
    async fn test_timer_countdown_progression() {
        let mut timer_service = TimerService::new();

        // Set a short duration for testing
        timer_service.set_duration(2); // 2 seconds
        timer_service.start_timer().await.unwrap();

        // Wait for 1 second
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should have progressed but not completed
        let elapsed = timer_service.get_elapsed_time();
        assert!(elapsed >= 1);
        assert!(elapsed < 2);
        assert!(timer_service.is_running());

        // Wait for completion
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Timer should be completed and stopped
        assert!(timer_service.get_elapsed_time() >= 2);
        assert!(!timer_service.is_running());
    }

    #[tokio::test]
    async fn test_session_type_transitions() {
        let mut timer_service = TimerService::new();

        // Start with Work session
        timer_service.set_session_type(TimerType::Work);
        assert_eq!(timer_service.get_session_type(), TimerType::Work);

        // Complete work session (simulate)
        timer_service.complete_current_session().await;

        // Should transition to ShortBreak
        assert_eq!(timer_service.get_session_type(), TimerType::ShortBreak);

        // Complete short break
        timer_service.complete_current_session().await;

        // Should transition back to Work
        assert_eq!(timer_service.get_session_type(), TimerType::Work);
    }

    #[tokio::test]
    async fn test_long_break_frequency() {
        let mut timer_service = TimerService::new();
        timer_service.set_long_break_frequency(3); // Long break every 3 work sessions

        // Complete 3 work sessions
        for i in 0..3 {
            timer_service.set_session_type(TimerType::Work);
            timer_service.increment_work_session_count();
            timer_service.complete_current_session().await;

            if i < 2 {
                // First 2 times should go to ShortBreak
                assert_eq!(timer_service.get_session_type(), TimerType::ShortBreak);
                timer_service.complete_current_session().await;
            }
        }

        // After 3rd work session, should go to LongBreak
        assert_eq!(timer_service.get_session_type(), TimerType::LongBreak);
    }

    #[tokio::test]
    async fn test_invalid_timer_operations() {
        let mut timer_service = TimerService::new();

        // Cannot pause when not running
        let pause_result = timer_service.pause_timer().await;
        assert!(pause_result.is_err());

        // Cannot start when already running (after starting)
        timer_service.start_timer().await.unwrap();
        let start_result = timer_service.start_timer().await;
        assert!(start_result.is_err());
    }

    #[tokio::test]
    async fn test_timer_state_persistence() {
        let mut timer_service = TimerService::new();

        // Start timer with specific state
        timer_service.set_duration(300);
        timer_service.set_session_type(TimerType::Work);
        timer_service.start_timer().await.unwrap();

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get state
        let state = timer_service.get_timer_state();

        assert_eq!(state.duration, 300);
        assert_eq!(state.session_type, "Work");
        assert!(state.is_running);
        assert!(state.elapsed > 0);
    }
}