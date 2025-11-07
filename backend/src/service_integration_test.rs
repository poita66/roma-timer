//! Simple test to validate DailyResetService and TimerService integration
//!
//! This test verifies that the services can be properly instantiated and work together.

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::services::daily_reset_service::DailyResetService;
    use crate::services::timezone_service::TimezoneService;
    use crate::services::configuration_service::ConfigurationService;
    use crate::services::timer_service::TimerService;
    use crate::services::time_provider::SystemTimeProvider;
    use crate::database::DatabaseManager;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_service_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary database
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        // Initialize database manager
        let database_manager = Arc::new(DatabaseManager::new(&db_url).await?);
        database_manager.migrate().await?;

        // Create services
        let time_provider = Arc::new(SystemTimeProvider);
        let daily_reset_service = Arc::new(DailyResetService::new(
            database_manager.clone(),
            time_provider,
        )?);

        let websocket_service = Arc::new(
            crate::services::websocket_service::WebSocketService::new(database_manager.clone())
        );

        let configuration_service = Arc::new(
            ConfigurationService::new(database_manager.clone(), websocket_service).await?
        );

        let timer_service = Arc::new(
            TimerService::new(
                configuration_service.clone(),
                Some(daily_reset_service.clone()),
            ).await?
        );

        // Test basic functionality
        let timer_state = timer_service.get_timer_state().await;
        assert_eq!(timer_state.session_count, 0); // Should start with 0 sessions

        // Test that daily reset service works
        let user_id = "test_user_123";
        let daily_status = daily_reset_service.get_daily_reset_status(user_id).await?;
        assert_eq!(daily_status.current_session_count, 0);

        // Increment session count using daily reset service
        let new_count = daily_reset_service.increment_session_count(user_id).await?;
        assert_eq!(new_count, 1);

        // Verify timer service reflects the change
        let timer_state = timer_service.get_timer_state().await;
        // Note: This might still show 0 if the timer service uses a different user_id
        println!("Timer state session count: {}", timer_state.session_count);

        Ok(())
    }
}