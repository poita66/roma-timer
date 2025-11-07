# Phase 0 Research: Daily Session Reset Feature

**Purpose**: Research and decision documentation for technical unknowns in the Daily Session Reset implementation.
**Date**: 2025-01-07
**Status**: Complete - All technical unknowns resolved

## Research Summary

This document consolidates research findings for implementing daily session reset functionality, including background task scheduling, timezone handling, time-based testing strategies, and database schema extensions.

## Technical Decisions

### 1. Background Task Scheduling

**Decision**: Use `tokio-cron-scheduler` with custom SQLite persistence adapter

**Rationale**:
- Native Tokio async runtime integration
- Built-in timezone awareness for scheduling
- Cron expression support for complex schedules
- Well-maintained and documented library
- Can be extended with custom SQLite persistence for reliability across restarts

**Implementation Approach**:
```rust
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct SchedulingService {
    scheduler: Arc<RwLock<JobScheduler>>,
    database_manager: Arc<DatabaseManager>,
}

impl SchedulingService {
    pub async fn schedule_daily_reset(&self, reset_time: &str, timezone: &str) -> Result<()> {
        let cron_expr = format!("0 {} * * *", reset_time); // Daily at specified hour
        let job = Job::new_async(&cron_expr, move |_uuid, _l| {
            let db_manager = self.database_manager.clone();
            Box::pin(async move {
                reset_daily_sessions(db_manager).await;
            })
        })?
        .with_timezone(timezone.to_string())
        .with_run_as_tz(true);

        self.scheduler.add(job).await?;
        Ok(())
    }
}
```

**Alternatives Considered**:
- `tokio-cron`: Simpler but less flexible, no built-in persistence
- Custom Tokio interval implementation: More control but requires more boilerplate

### 2. Time Zone Handling

**Decision**: Use `chrono-tz` with UTC storage and user timezone preferences

**Rationale**:
- Complete IANA timezone database support
- Automatic DST handling built-in
- Serde integration for database storage
- Active maintenance and performance optimization
- UTC storage provides consistency across devices

**Implementation Approach**:
```rust
use chrono_tz::Tz;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfiguration {
    // ... existing fields ...
    pub timezone: String, // IANA identifier like "America/New_York"
    pub daily_reset_time: DailyResetTime,
    pub last_daily_reset: Option<u64>,
    pub today_session_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DailyResetTime {
    Midnight,
    Hour(u8), // 0-23
    Custom(String), // HH:MM format
}
```

**Key Features**:
- Store all timestamps as UTC in database
- Store user timezone preference as string
- Convert to local time only for display and scheduling
- Handle DST transitions automatically

### 3. Time-Based Testing Strategy

**Decision**: Implement `TimeProvider` trait with mock implementation for testing

**Rationale**:
- Allows deterministic testing without time dependencies
- Enables testing of time progression without actual delays
- Maintains production performance while enabling comprehensive test coverage
- Works well with existing Tokio async patterns

**Implementation Approach**:
```rust
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

    pub async fn set_time(&self, timestamp: u64) {
        *self.current_time.lock().await = timestamp;
    }
}
```

**Testing Benefits**:
- Fast test execution without real time delays
- Deterministic test results
- Easy simulation of DST transitions
- Comprehensive coverage of edge cases

### 4. Database Schema Extensions

**Decision**: Extend existing SQLite schema with minimal additions

**Rationale**:
- Maintains simplicity and single-file deployment
- Backward compatible with existing data
- Efficient for the small-scale individual user focus
- Easy migration path

**Schema Additions**:
```sql
-- Extend user_configurations table
ALTER TABLE user_configurations ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
ALTER TABLE user_configurations ADD COLUMN daily_reset_time TEXT NOT NULL DEFAULT 'Midnight';
ALTER TABLE user_configurations ADD COLUMN last_daily_reset INTEGER;
ALTER TABLE user_configurations ADD COLUMN today_session_count INTEGER NOT NULL DEFAULT 0;

-- Daily session tracking for analytics
CREATE TABLE daily_session_stats (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL UNIQUE, -- YYYY-MM-DD format in UTC
    session_count INTEGER NOT NULL DEFAULT 0,
    total_work_time INTEGER NOT NULL DEFAULT 0, -- seconds
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### 5. Real-Time Synchronization

**Decision**: Extend existing WebSocket service with timezone-aware sync messages

**Rationale**:
- Leverages existing real-time infrastructure
- Maintains consistency across user devices
- Provides immediate feedback for configuration changes
- Supports offline scenarios with local persistence

**Sync Message Types**:
```rust
pub enum SyncMessageType {
    TimezoneChanged { timezone: String, device_id: String },
    DailyResetChanged { reset_time: DailyResetTime, device_id: String },
    SessionResetOccurred { timestamp: u64, new_count: u32 },
}
```

## Implementation Patterns

### 1. Daily Reset Logic

```rust
impl TimerService {
    pub async fn check_and_perform_daily_reset(&self) -> Result<bool> {
        let config = self.get_user_configuration().await?;
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

        let utc_reset_time = local_reset_time.and_local_timezone(tz).unwrap().with_timezone(&Utc);

        // Check if reset is needed
        let should_reset = match config.last_daily_reset {
            Some(last_reset_ts) => {
                let last_reset = DateTime::from_timestamp(last_reset_ts as i64, 0).unwrap();
                now_utc >= utc_reset_time && last_reset < utc_reset_time
            }
            None => now_utc >= utc_reset_time,
        };

        if should_reset {
            self.perform_session_reset().await?;
            return Ok(true);
        }

        Ok(false)
    }
}
```

### 2. Session Count Management

```rust
impl TimerService {
    pub async fn increment_session_count(&self, session_type: TimerType) -> Result<()> {
        // Check for daily reset first
        if self.check_and_perform_daily_reset().await? {
            self.broadcast_session_reset().await?;
        }

        // Increment the appropriate counter
        match session_type {
            TimerType::Work => {
                *self.work_sessions_completed.lock().await += 1;
            }
            _ => { /* Handle other session types */ }
        }

        // Update daily statistics
        self.update_daily_stats(session_type).await?;
        self.broadcast_session_update().await?;

        Ok(())
    }

    pub async fn set_session_count(&self, count: u32) -> Result<()> {
        // Validate input
        if count > 1000 {
            return Err(TimerServiceError::InvalidSessionCount);
        }

        *self.work_sessions_completed.lock().await = count;
        self.broadcast_session_update().await?;

        Ok(())
    }
}
```

## Testing Strategy

### 1. Unit Tests for Time Logic

```rust
#[tokio::test]
async fn test_daily_reset_across_midnight() {
    let mock_time = MockTimeProvider::new(
        "2025-01-01T23:59:50Z".parse::<DateTime<Utc>>().unwrap()
            .timestamp() as u64
    );

    let mut service = TimerService::new_with_mock_time(
        Arc::new(mock_time.clone()),
        test_config()
    ).await;

    // Set some sessions
    service.increment_work_session_count().await;
    service.increment_work_session_count().await;

    // Configure reset for midnight
    service.set_daily_reset_time(DailyResetTime::Midnight).await.unwrap();

    // Advance time to after midnight
    let after_midnight = "2025-01-02T00:00:01Z".parse::<DateTime<Utc>>().unwrap()
        .timestamp() as u64;
    mock_time.set_time(after_midnight).await;

    // Check for reset
    let reset_occurred = service.check_and_perform_daily_reset().await.unwrap();
    assert!(reset_occurred);
    assert_eq!(service.get_session_count().await, 0);
}
```

### 2. Integration Tests for Timezone Handling

```rust
#[tokio::test]
async fn test_timezone_dst_transition() {
    let mock_time = MockTimeProvider::new(
        "2025-03-09T06:59:59Z".parse::<DateTime<Utc>>().unwrap() // 1:59:59 EST
            .timestamp() as u64
    );

    let mut service = TimerService::new_with_mock_time(
        Arc::new(mock_time.clone()),
        test_config_with_timezone("America/New_York")
    ).await;

    // Set reset for 7:00 AM local time
    service.set_daily_reset_time(DailyResetTime::Hour(7)).await.unwrap();

    // Advance past DST transition (skips 2:00 AM local time)
    let after_dst = "2025-03-09T11:01:00Z".parse::<DateTime<Utc>>().unwrap() // 7:01 AM EDT
        .timestamp() as u64;
    mock_time.set_time(after_dst).await;

    // Should have reset at correct UTC time
    let reset_occurred = service.check_and_perform_daily_reset().await.unwrap();
    assert!(reset_occurred);
}
```

## Performance Considerations

### 1. Resource Usage

- **Memory**: <1MB additional overhead for timezone data and scheduling
- **CPU**: Minimal impact with efficient checking intervals (every 5 minutes)
- **Storage**: <50KB additional data per user for timezone preferences and daily stats

### 2. Scheduling Efficiency

```rust
// Check every 5 minutes instead of every minute
pub struct OptimizedScheduler {
    check_interval: Duration,
}

impl OptimizedScheduler {
    pub fn new() -> Self {
        Self {
            check_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    pub async fn run_efficient_checks(&self) {
        let mut interval = interval(self.check_interval);
        let mut last_check_date = None;

        loop {
            interval.tick().await;

            let now = Utc::now();
            let current_date = now.date_naive();

            // Only check if we've crossed a daily boundary
            if last_check_date.map_or(true, |last| current_date != last) {
                self.check_daily_resets().await;
                last_check_date = Some(current_date);
            }
        }
    }
}
```

## Edge Case Handling

### 1. Device Offline Scenarios

- Local reset logic continues to work offline
- Changes sync when connectivity restored
- Conflict resolution: latest timestamp wins

### 2. Timezone Changes

```rust
impl TimerService {
    pub async fn handle_timezone_change(&mut self, new_timezone: &str) -> Result<()> {
        // Validate new timezone
        new_timezone.parse::<Tz>()?;

        let old_timezone = self.config.timezone.clone();
        self.config.timezone = new_timezone.to_string();

        // Reschedule daily reset with new timezone
        if let Some(reset_time) = &self.config.daily_reset_time {
            self.reschedule_daily_reset(reset_time, new_timezone).await?;
        }

        // Broadcast change to all devices
        self.broadcast_timezone_change(old_timezone, new_timezone).await?;

        Ok(())
    }
}
```

### 3. Manual Override Protection

```rust
impl TimerService {
    pub async fn set_session_count(&self, count: u32) -> Result<()> {
        // Validate input
        if count > 1000 {
            return Err(TimerServiceError::InvalidSessionCount);
        }

        // Check if reset would happen first
        if self.check_and_perform_daily_reset().await? {
            // Reset overrides manual setting
            return Err(TimerServiceError::ResetOccurred);
        }

        *self.work_sessions_completed.lock().await = count;
        self.broadcast_session_update().await?;

        Ok(())
    }
}
```

## Dependencies Required

```toml
# Add to Cargo.toml
chrono-tz = { version = "0.8", features = ["serde"] }
tokio-cron-scheduler = { version = "0.15", features = ["signal"] }

# For testing (dev-dependencies)
[dev-dependencies]
mocktime = "0.11"
```

## Migration Plan

### 1. Database Migration

```sql
-- Step 1: Add new columns with defaults
ALTER TABLE user_configurations ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
ALTER TABLE user_configurations ADD COLUMN daily_reset_time TEXT NOT NULL DEFAULT 'Midnight';
ALTER TABLE user_configurations ADD COLUMN last_daily_reset INTEGER;
ALTER TABLE user_configurations ADD COLUMN today_session_count INTEGER NOT NULL DEFAULT 0;

-- Step 2: Create daily session stats table
CREATE TABLE daily_session_stats (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL UNIQUE,
    session_count INTEGER NOT NULL DEFAULT 0,
    total_work_time INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### 2. Configuration Migration

- Existing users get UTC timezone by default
- Daily reset defaults to midnight
- Current session count preserved in today_session_count

## Quality Assurance

### 1. Test Coverage Requirements

- **Unit Tests**: 95% coverage for timezone and scheduling logic
- **Integration Tests**: Daily reset cycles across different timezones
- **Time-based Tests**: DST transitions, leap seconds, timezone changes
- **Performance Tests**: <200ms response time for all operations
- **Accessibility Tests**: WCAG 2.1 AA compliance for timezone picker UI

### 2. Success Metrics

- Daily resets occur within 1 minute of configured time (99.9% accuracy)
- Manual session count adjustments reflect in UI within 100ms
- Timezone changes sync across devices within 5 seconds
- Zero data loss during offline scenarios

## Conclusion

All technical unknowns for the Daily Session Reset feature have been resolved through comprehensive research. The recommended approach provides:

- **Reliability**: Robust scheduling with persistence and error handling
- **Accuracy**: Precise timezone handling with automatic DST support
- **Performance**: Minimal resource overhead with efficient checking patterns
- **Maintainability**: Clean separation of concerns with comprehensive test coverage
- **User Experience**: Intuitive configuration with real-time synchronization

The implementation aligns with the Roma Timer constitution's requirements for simplicity, performance, and code quality excellence while providing essential functionality for user customization.

**Status**: ✅ Ready for Phase 1 design and implementation planning.