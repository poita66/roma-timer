# Daily Session Reset - Quick Start Guide

**Purpose**: Implementation guide for developers working on the Daily Session Reset feature
**Date**: 2025-01-07
**Dependencies**: Phase 1 design documents (research.md, data-model.md, api.yaml)

## Overview

This guide provides a quick start for implementing the Daily Session Reset feature, which allows users to automatically reset their session count at a configurable time each day and manually adjust session counts as needed.

## Feature Summary

- **Automatic Daily Reset**: Session counts reset to 0 at user-configured time
- **Timezone Awareness**: Resets work correctly across different timezones and DST transitions
- **Manual Override**: Users can manually set session counts (0-1000)
- **Real-time Sync**: Changes sync instantly across all user devices
- **Analytics**: Track daily session statistics and reset events

## Prerequisites

### Development Environment

```bash
# Rust requirements
rustc 1.83+ (backend)
tokio async runtime
SQLite database support

# Frontend requirements
React Native (PWA framework)
WebSocket client support

# Development tools
cargo test
cargo clippy
npm test (frontend)
```

### Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
chrono-tz = { version = "0.8", features = ["serde"] }
tokio-cron-scheduler = { version = "0.15", features = ["signal"] }

[dev-dependencies]
mocktime = "0.11"
```

## Quick Implementation Steps

### Step 1: Database Schema Migration

```bash
# Apply the database migrations
./scripts/migrate-daily-reset.sh

# Or manually execute SQL
sqlite3 roma-timer.db < specs/002-session-reset/migrations.sql
```

**Key Tables Added**:
- `user_configurations` (extended with timezone, daily_reset_time fields)
- `daily_session_stats` (daily analytics)
- `scheduled_tasks` (background job tracking)
- `session_reset_events` (audit log)

### Step 2: Core Data Structures

```rust
// Add to models/user_configuration.rs
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DailyResetTime {
    Midnight,
    Hour(u8), // 0-23
    Custom(String), // HH:MM format
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfiguration {
    // ... existing fields ...
    pub timezone: String, // IANA timezone identifier
    pub daily_reset_time: DailyResetTime,
    pub last_daily_reset: Option<u64>,
    pub today_session_count: u32,
    pub manual_session_override: Option<u32>,
}
```

### Step 3: Time Provider Service

```rust
// services/time_provider.rs
pub trait TimeProvider: Send + Sync + 'static {
    fn now(&self) -> u64;
}

pub struct SystemTimeProvider;
impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

#[cfg(test)]
pub struct MockTimeProvider {
    current_time: Arc<Mutex<u64>>,
}

#[cfg(test)]
impl MockTimeProvider {
    pub async fn advance_time(&self, seconds: u64) {
        *self.current_time.lock().await += seconds;
    }
}
```

### Step 4: Daily Reset Service

```rust
// services/daily_reset_service.rs
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct DailyResetService {
    scheduler: Arc<RwLock<JobScheduler>>,
    time_provider: Arc<dyn TimeProvider>,
    db_manager: Arc<DatabaseManager>,
}

impl DailyResetService {
    pub async fn new(
        time_provider: Arc<dyn TimeProvider>,
        db_manager: Arc<DatabaseManager>,
    ) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            scheduler: Arc::new(RwLock::new(scheduler)),
            time_provider,
            db_manager,
        })
    }

    pub async fn schedule_daily_reset(&self, config: &UserConfiguration) -> Result<()> {
        let tz: Tz = config.timezone.parse()?;

        // Create cron expression for daily reset
        let cron_expr = match &config.daily_reset_time {
            DailyResetTime::Midnight => "0 0 * * *".to_string(),
            DailyResetTime::Hour(hour) => format!("0 {} * * *", hour),
            DailyResetTime::Custom(time_str) => {
                let parts: Vec<&str> = time_str.split(':').collect();
                format!("0 {} * * *", parts[1]) // At MM:HH every day
            }
        };

        let job = Job::new_async(&cron_expr, move |_uuid, _l| {
            // Reset session count logic here
            Box::pin(async move {
                // Implementation details...
            })
        })?
        .with_timezone(config.timezone.clone());

        let mut scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        Ok(())
    }

    pub async fn check_and_perform_reset(&self, user_id: &str) -> Result<bool> {
        let config = self.db_manager.get_user_configuration(user_id).await?;

        if let Some(config) = config {
            let should_reset = self.should_reset_now(&config).await?;

            if should_reset {
                self.perform_session_reset(&config).await?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn should_reset_now(&self, config: &UserConfiguration) -> Result<bool> {
        let now = self.time_provider.now();
        let now_utc = DateTime::from_timestamp(now as i64, 0).unwrap();

        // Calculate today's reset time in user's timezone
        let tz: Tz = config.timezone.parse()?;
        let today_local = now_utc.with_timezone(&tz).date_naive();

        let local_reset_time = match config.daily_reset_time {
            DailyResetTime::Midnight => today_local.and_hms_opt(0, 0, 0).unwrap(),
            DailyResetTime::Hour(hour) => today_local.and_hms_opt(hour as u32, 0, 0).unwrap(),
            DailyResetTime::Custom(time_str) => {
                let parts: Vec<&str> = time_str.split(':').collect();
                today_local.and_hms_opt(
                    parts[0].parse().unwrap(),
                    parts[1].parse().unwrap(),
                    0
                ).unwrap()
            }
        };

        let utc_reset_time = local_reset_time
            .and_local_timezone(tz)
            .unwrap()
            .with_timezone(&Utc);

        // Check if we should reset
        match config.last_daily_reset {
            Some(last_reset_ts) => {
                let last_reset = DateTime::from_timestamp(last_reset_ts as i64, 0).unwrap();
                now_utc >= utc_reset_time && last_reset < utc_reset_time
            }
            None => now_utc >= utc_reset_time,
        }
    }

    async fn perform_session_reset(&self, config: &UserConfiguration) -> Result<()> {
        // Reset session count
        let mut updated_config = config.clone();
        updated_config.today_session_count = 0;
        updated_config.manual_session_override = None;
        updated_config.last_daily_reset = Some(self.time_provider.now());
        updated_config.touch();

        self.db_manager.save_user_configuration(&updated_config).await?;

        // Log reset event
        self.log_reset_event(config, SessionResetType::ScheduledDaily).await?;

        // Broadcast reset notification
        self.broadcast_reset_notification(&updated_config).await?;

        Ok(())
    }
}
```

### Step 5: API Endpoints

```rust
// api/session_reset.rs
use axum::{extract::State, response::Json, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct SetSessionCountRequest {
    count: u32,
    clear_override: Option<bool>,
}

// GET /api/session/count
pub async fn get_session_count(
    State(app_state): State<AppState>,
) -> Result<Json<SessionCountResponse>, AppError> {
    let config = app_state.db_manager.get_user_configuration("default").await?
        .ok_or(AppError::NotFound)?;

    let next_reset = app_state.daily_reset_service
        .get_next_reset_time(&config).await?;

    Ok(Json(SessionCountResponse {
        current_count: config.today_session_count,
        manual_override: config.manual_session_override,
        today_date: Utc::now().format("%Y-%m-%d").to_string(),
        next_reset_utc: next_reset.map(|dt| dt.to_rfc3339()),
        next_reset_local: next_reset.map(|dt| {
            dt.with_timezone(&config.timezone.parse::<Tz>().unwrap())
                .format("%H:%M").to_string()
        }),
    }))
}

// PUT /api/session/count
pub async fn set_session_count(
    State(app_state): State<AppState>,
    Json(request): Json<SetSessionCountRequest>,
) -> Result<Json<SessionCountResponse>, AppError> {
    // Validate input
    if request.count > 1000 {
        return Err(AppError::BadRequest("Session count must be 0-1000".to_string()));
    }

    // Check for daily reset first
    app_state.daily_reset_service.check_and_perform_reset("default").await?;

    // Update session count
    let mut config = app_state.db_manager.get_user_configuration("default").await?
        .ok_or(AppError::NotFound)?;

    let previous_count = config.today_session_count;
    config.today_session_count = request.count;

    if request.clear_override.unwrap_or(false) {
        config.manual_session_override = None;
    } else {
        config.manual_session_override = Some(request.count);
    }

    config.touch();
    app_state.db_manager.save_user_configuration(&config).await?;

    // Broadcast update
    app_state.websocket_service.broadcast_session_update(&config).await?;

    Ok(Json(SessionCountResponse {
        current_count: config.today_session_count,
        manual_override: config.manual_session_override,
        today_date: Utc::now().format("%Y-%m-%d").to_string(),
        next_reset_utc: None, // Would calculate this
        next_reset_local: None,
    }))
}

// POST /api/session/reset
pub async fn reset_session_count(
    State(app_state): State<AppState>,
) -> Result<Json<SessionResetResponse>, AppError> {
    let config = app_state.db_manager.get_user_configuration("default").await?
        .ok_or(AppError::NotFound)?;

    let previous_count = config.today_session_count;

    // Perform reset
    app_state.daily_reset_service.perform_manual_reset(&config).await?;

    Ok(Json(SessionResetResponse {
        previous_count,
        new_count: 0,
        reset_timestamp: Utc::now().to_rfc3339(),
        reset_type: "manual".to_string(),
    }))
}
```

### Step 6: WebSocket Integration

```rust
// services/websocket_service.rs
use serde_json::json;

impl WebSocketService {
    pub async fn broadcast_session_update(&self, config: &UserConfiguration) -> Result<()> {
        let message = json!({
            "type": "session_count_update",
            "timestamp": Utc::now().to_rfc3339(),
            "user_id": config.user_id,
            "data": {
                "current_count": config.today_session_count,
                "manual_override": config.manual_session_override.is_some(),
                "change_type": if config.manual_session_override.is_some() { "manual_set" } else { "increment" },
                "today_date": Utc::now().format("%Y-%m-%d").to_string()
            }
        });

        self.broadcast_to_user(&config.user_id, &message.to_string()).await?;
        Ok(())
    }

    pub async fn broadcast_session_reset(&self, config: &UserConfiguration, reset_type: &str) -> Result<()> {
        let message = json!({
            "type": "session_reset_occurred",
            "timestamp": Utc::now().to_rfc3339(),
            "user_id": config.user_id,
            "data": {
                "reset_type": reset_type,
                "previous_count": 0, // Would come from before reset
                "new_count": 0,
                "reset_timestamp": Utc::now().to_rfc3339(),
                "local_reset_time": "00:00", // Would calculate this
                "timezone": config.timezone
            }
        });

        self.broadcast_to_user(&config.user_id, &message.to_string()).await?;
        Ok(())
    }
}
```

### Step 7: Frontend Integration

```typescript
// Frontend - Daily Reset Service
class DailyResetService {
    private ws: WebSocket;
    private sessionCount = 0;
    private nextResetTime: Date | null = null;

    async getSessionCount(): Promise<SessionCountResponse> {
        const response = await fetch('/api/session/count');
        return response.json();
    }

    async setSessionCount(count: number): Promise<SessionCountResponse> {
        const response = await fetch('/api/session/count', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ count })
        });
        return response.json();
    }

    async resetSession(): Promise<SessionResetResponse> {
        const response = await fetch('/api/session/reset', {
            method: 'POST'
        });
        return response.json();
    }

    // WebSocket message handling
    handleWebSocketMessage(message: any) {
        switch (message.type) {
            case 'session_count_update':
                this.sessionCount = message.data.current_count;
                this.updateUI();
                break;
            case 'session_reset_occurred':
                this.sessionCount = 0;
                this.showResetNotification(message.data.reset_type);
                break;
            case 'timezone_changed':
                this.updateTimezoneDisplay(message.data.new_timezone);
                break;
        }
    }

    private updateUI() {
        // Update session count display
        document.getElementById('session-count').textContent = this.sessionCount.toString();

        // Update next reset time display
        if (this.nextResetTime) {
            const timeUntil = this.getTimeUntilReset(this.nextResetTime);
            document.getElementById('next-reset').textContent = timeUntil;
        }
    }

    private showResetNotification(resetType: string) {
        const message = resetType === 'scheduled_daily'
            ? 'Daily session count has been reset'
            : 'Session count has been reset';

        this.showNotification(message, 'info');
    }
}

// React component example
function SessionCountDisplay() {
    const [sessionCount, setSessionCount] = useState(0);
    const [isEditing, setIsEditing] = useState(false);
    const [editValue, setEditValue] = useState('');

    const handleManualSet = async () => {
        try {
            const response = await dailyResetService.setSessionCount(parseInt(editValue));
            setSessionCount(response.data.current_count);
            setIsEditing(false);
        } catch (error) {
            showError('Failed to set session count');
        }
    };

    return (
        <div className="session-count-display">
            <h3>Today's Sessions</h3>
            {isEditing ? (
                <div className="manual-edit">
                    <input
                        type="number"
                        min="0"
                        max="1000"
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                    />
                    <button onClick={handleManualSet}>Set</button>
                    <button onClick={() => setIsEditing(false)}>Cancel</button>
                </div>
            ) : (
                <div className="display-mode">
                    <span className="count">{sessionCount}</span>
                    <button onClick={() => setIsEditing(true)}>Set</button>
                    <button onClick={handleReset}>Reset</button>
                </div>
            )}
        </div>
    );
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mocktime::mock_time;

    #[tokio::test]
    async fn test_daily_reset_at_midnight() {
        // Mock time to 2025-01-01 23:59:59 UTC
        mock_time("2025-01-01T23:59:59Z").unwrap();

        let mock_time_provider = Arc::new(MockTimeProvider::new(
            "2025-01-01T23:59:59Z".parse::<DateTime<Utc>>().unwrap()
                .timestamp() as u64
        ));

        let service = DailyResetService::new(
            mock_time_provider.clone(),
            test_db_manager().await
        ).await;

        // Create test configuration with midnight reset
        let config = create_test_config_with_midnight_reset();

        // Advance time to after midnight
        mock_time_provider.advance_time(120).await; // +2 minutes

        // Check if reset should occur
        let should_reset = service.check_and_perform_reset("test_user").await.unwrap();
        assert!(should_reset);
    }

    #[tokio::test]
    async fn test_manual_session_override() {
        let service = create_timer_service().await;

        // Set manual count
        service.set_session_count(15).await.unwrap();
        assert_eq!(service.get_session_count().await, 15);

        // Verify manual override is set
        let config = service.get_user_configuration().await.unwrap();
        assert_eq!(config.manual_session_override, Some(15));
    }

    #[tokio::test]
    async fn test_timezone_dst_transition() {
        let mock_time = MockTimeProvider::new(
            "2025-03-09T06:59:59Z".parse::<DateTime<Utc>>().unwrap() // 1:59:59 EST
                .timestamp() as u64
        );

        let service = DailyResetService::new(
            Arc::new(mock_time.clone()),
            test_db_manager().await
        ).await;

        // Test DST spring forward transition
        let config = create_test_config_with_timezone("America/New_York");
        service.schedule_daily_reset(&config).await.unwrap();

        // Advance past DST transition (skips 2:00 AM local time)
        mock_time.set_time(
            "2025-03-09T11:01:00Z".parse::<DateTime<Utc>>().unwrap() // 7:01 AM EDT
                .timestamp() as u64
        ).await;

        // Verify reset still works correctly
        let should_reset = service.check_and_perform_reset("test_user").await.unwrap();
        // Implementation dependent...
    }
}
```

### Integration Tests

```bash
# Run the full test suite
cargo test daily_reset

# Run with coverage
cargo test --coverage daily_reset

# Run integration tests
cargo test --test integration daily_reset
```

### Manual Testing Checklist

- [ ] Daily reset works at configured time
- [ ] Manual session count override works
- [ ] Timezone changes update reset times correctly
- [ ] DST transitions handled properly
- [ ] Real-time sync across multiple devices
- [ ] Offline scenarios recover correctly
- [ ] Analytics data is accurate

## Deployment

### Configuration

```bash
# Environment variables for daily reset feature
export DAILY_RESET_ENABLED=true
export DEFAULT_TIMEZONE=UTC
export RESET_CHECK_INTERVAL=300  # 5 minutes in seconds
export MAX_SESSION_COUNT=1000
```

### Migration Script

```bash
#!/bin/bash
# scripts/deploy-daily-reset.sh

echo "Deploying Daily Session Reset feature..."

# 1. Backup current database
cp roma-timer.db roma-timer.db.backup.$(date +%Y%m%d_%H%M%S)

# 2. Run database migrations
sqlite3 roma-timer.db < migrations/002_session_reset.sql

# 3. Update existing user configurations with defaults
sqlite3 roma-timer.db << EOF
UPDATE user_configurations
SET timezone = 'UTC',
    daily_reset_time = 'Midnight',
    today_session_count = 0
WHERE timezone IS NULL;
EOF

# 4. Restart application with new feature
systemctl restart roma-timer

echo "Deployment complete!"
```

### Monitoring

```bash
# Check daily reset service health
curl http://localhost:8080/api/health/daily-reset

# View recent reset events
curl "http://localhost:8080/api/session/reset-events?limit=10"

# Monitor session counts
curl http://localhost:8080/api/session/count
```

## Troubleshooting

### Common Issues

**Issue**: Daily reset not occurring at expected time
**Solution**: Check timezone configuration and server system time

```bash
# Verify timezone configuration
sqlite3 roma-timer.db "SELECT timezone, daily_reset_time FROM user_configurations;"

# Check system timezone
timedatectl status
```

**Issue**: Manual session override not working
**Solution**: Check validation and database write permissions

```bash
# Check application logs for errors
journalctl -u roma-timer -f

# Verify database permissions
sqlite3 roma-timer.db ".schema user_configurations"
```

**Issue**: WebSocket messages not received
**Solution**: Check WebSocket connection status and authentication

```bash
# Test WebSocket connection
wscat -c ws://localhost:8080/ws

# Check authentication token validity
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/session/count
```

### Debug Commands

```rust
// Enable debug logging
env RUST_LOG=debug cargo run

// Check scheduler status
// Add to service:
pub async fn get_scheduler_status(&self) -> SchedulerStatus {
    let scheduler = self.scheduler.read().await;
    SchedulerStatus {
        job_count: scheduler.list().await.len(),
        is_running: scheduler.is_started(),
        next_jobs: scheduler.list().await.iter()
            .filter_map(|job| job.next_tick())
            .collect()
    }
}
```

## Next Steps

1. **Complete Implementation**: Follow the code examples above to implement each component
2. **Add Comprehensive Tests**: Ensure 95% coverage for critical paths
3. **Performance Testing**: Verify <200ms API response times
4. **Documentation**: Update API documentation and user guides
5. **Integration Testing**: Test across different timezones and devices
6. **Monitoring Setup**: Implement alerts for daily reset failures

## Support

For implementation questions or issues:
1. Check the [research.md](./research.md) for technical decisions
2. Review the [data-model.md](./data-model.md) for data structure details
3. Refer to the [API contracts](./api.yaml) for endpoint specifications
4. Contact the development team for architecture questions

This quick start guide provides the essential information needed to implement the Daily Session Reset feature. The full specification and research documents contain additional details for comprehensive implementation.