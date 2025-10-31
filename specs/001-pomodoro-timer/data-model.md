# Roma Timer Data Model

**Date**: 2025-10-29
**Purpose**: Data structures and relationships for timer functionality

## Core Entities

### TimerSession

Represents the current timer state and configuration.

```rust
// Backend Rust structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSession {
    pub id: String,                    // UUID for session identification
    pub duration: u32,                 // Total duration in seconds
    pub elapsed: u32,                  // Elapsed time in seconds
    pub timer_type: TimerType,         // Work, ShortBreak, or LongBreak
    pub is_running: bool,              // Timer active state
    pub created_at: u64,               // Unix timestamp
    pub updated_at: u64,               // Last update timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerType {
    Work,
    ShortBreak,
    LongBreak,
}
```

```javascript
// Frontend TypeScript interfaces
interface TimerSession {
    id: string;
    duration: number;
    elapsed: number;
    timerType: 'Work' | 'ShortBreak' | 'LongBreak';
    isRunning: boolean;
    createdAt: number;
    updatedAt: number;
}
```

### UserConfiguration

Stores user preferences for timer behavior.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfiguration {
    pub id: String,
    pub work_duration: u32,            // Work session duration (seconds)
    pub short_break_duration: u32,     // Short break duration (seconds)
    pub long_break_duration: u32,      // Long break duration (seconds)
    pub long_break_frequency: u32,     // Work sessions before long break
    pub notifications_enabled: bool,   // Browser notifications enabled
    pub webhook_url: Option<String>,   // Optional webhook for notifications
    pub wait_for_interaction: bool,    // Wait for user input before next session
    pub theme: Theme,                  // Light or dark theme
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}
```

```javascript
interface UserConfiguration {
    id: string;
    workDuration: number;      // seconds
    shortBreakDuration: number; // seconds
    longBreakDuration: number;  // seconds
    longBreakFrequency: number; // number of work sessions
    notificationsEnabled: boolean;
    webhookUrl?: string;
    waitForInteraction: boolean;
    theme: 'Light' | 'Dark';
    createdAt: number;
    updatedAt: number;
}
```

### DeviceConnection

Represents an active device connection for synchronization.

```rust
#[derive(Debug, Clone)]
pub struct DeviceConnection {
    pub id: String,                    // Connection identifier
    pub user_agent: String,             // Browser/device identifier
    pub connected_at: u64,              // Connection timestamp
    pub last_ping: u64,                 // Last activity timestamp
    pub websocket_sender: Option<tokio::sync::mpsc::UnboundedSender<TimerMessage>>,
}
```

### NotificationEvent

Represents timer completion notifications.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub id: String,
    pub timer_session_id: String,
    pub event_type: NotificationType,
    pub message: String,
    pub created_at: u64,
    pub delivered_at: Option<u64>,     // Delivery confirmation timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    WorkSessionComplete,
    BreakSessionComplete,
    TimerSkipped,
    TimerReset,
}
```

## State Transitions

### Timer State Machine

```
[Stopped] -> Play -> [Running]
[Running] -> Pause -> [Stopped]
[Running] -> Complete -> [Auto-transition to next session type]
[Running/Stopped] -> Reset -> [Stopped with Work session, elapsed = 0]
[Running/Stopped] -> Skip -> [Stopped with next session type, elapsed = 0]
```

### Session Type Transitions

```
Work (N times) -> Short Break -> Work (N+1 times)
Work (N = long_break_frequency) -> Long Break -> Work (1)
```

## Database Schema (SQLite)

### timer_sessions table

```sql
CREATE TABLE timer_sessions (
    id TEXT PRIMARY KEY,
    duration INTEGER NOT NULL,
    elapsed INTEGER NOT NULL DEFAULT 0,
    timer_type TEXT NOT NULL CHECK (timer_type IN ('Work', 'ShortBreak', 'LongBreak')),
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### user_configurations table

```sql
CREATE TABLE user_configurations (
    id TEXT PRIMARY KEY,
    work_duration INTEGER NOT NULL DEFAULT 1500,    -- 25 minutes
    short_break_duration INTEGER NOT NULL DEFAULT 300,  -- 5 minutes
    long_break_duration INTEGER NOT NULL DEFAULT 900,   -- 15 minutes
    long_break_frequency INTEGER NOT NULL DEFAULT 4,
    notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    webhook_url TEXT,
    wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
    theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### notification_events table

```sql
CREATE TABLE notification_events (
    id TEXT PRIMARY KEY,
    timer_session_id TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('WorkSessionComplete', 'BreakSessionComplete', 'TimerSkipped', 'TimerReset')),
    message TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    delivered_at INTEGER,
    FOREIGN KEY (timer_session_id) REFERENCES timer_sessions(id)
);
```

## Validation Rules

### Timer Session Validation

- `duration` must be > 0 and ≤ 7200 (2 hours max)
- `elapsed` must be ≥ 0 and ≤ `duration`
- `timer_type` must be one of: Work, ShortBreak, LongBreak
- `updated_at` must be ≥ `created_at`

### User Configuration Validation

- `work_duration` must be between 300 (5 min) and 3600 (1 hour)
- `short_break_duration` must be between 60 (1 min) and 900 (15 min)
- `long_break_duration` must be between 300 (5 min) and 1800 (30 min)
- `long_break_frequency` must be between 2 and 10
- `webhook_url` must be valid URL if provided
- `theme` must be either Light or Dark

### Business Logic Rules

1. **Timer Completion**: When `elapsed` reaches `duration` and `is_running` is true, automatically transition to next session type
2. **Session Counter**: Track completed work sessions to determine when to schedule long breaks
3. **Auto-transition**: Only auto-transition if `wait_for_interaction` is false in user configuration
4. **Webhook Validation**: Validate webhook URL format before storing
5. **Connection Management**: Remove inactive connections after 5 minutes of no activity

## API Message Formats

### WebSocket Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimerMessage {
    // Server to client messages
    TimerStateUpdate { session: TimerSession },
    Notification { event: NotificationEvent },

    // Client to server messages
    StartTimer,
    PauseTimer,
    ResetTimer,
    SkipTimer,
    UpdateConfiguration { config: UserConfiguration },
}
```

### REST API Endpoints

- `GET /api/timer` - Get current timer state
- `POST /api/timer/start` - Start timer
- `POST /api/timer/pause` - Pause timer
- `POST /api/timer/reset` - Reset timer
- `POST /api/timer/skip` - Skip to next session
- `GET /api/configuration` - Get user configuration
- `PUT /api/configuration` - Update user configuration
- `GET /api/health` - Health check

## Error Handling

### Error Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum TimerError {
    InvalidTimerState(String),
    ConfigurationNotFound,
    InvalidConfiguration(String),
    TimerNotFound,
    WebSocketError(String),
    DatabaseError(String),
}
```

### Error Responses

All API errors return JSON with error details:
```json
{
    "error": "InvalidTimerState",
    "message": "Cannot start timer that is already running",
    "timestamp": 1698569400
}
```