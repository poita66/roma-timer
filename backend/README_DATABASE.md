# Database Configuration Guide

The Roma Timer backend supports both SQLite and PostgreSQL databases through SQLx's database-agnostic features.

## Quick Setup

### SQLite (Default)
```bash
# Default configuration - no setup needed
cargo run

# Or explicitly set SQLite
DATABASE_URL="sqlite:roma-timer.db" cargo run

# Custom SQLite file location
DATABASE_URL="sqlite:/path/to/your/database.db" cargo run
```

### PostgreSQL
```bash
# Install PostgreSQL (Ubuntu/Debian)
sudo apt update
sudo apt install postgresql postgresql-contrib

# Create database
sudo -u postgres psql
CREATE DATABASE roma_timer;
CREATE USER roma_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE roma_timer TO roma_user;
\q

# Run with PostgreSQL
DATABASE_URL="postgres://roma_user:your_password@localhost/roma_timer" cargo run

# Or build with PostgreSQL feature
cargo build --features postgres
./target/release/roma-timer
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Full database connection URL | `sqlite:roma-timer.db` |
| `ROMA_TIMER_DATABASE_URL` | Roma Timer specific database URL | `sqlite:roma-timer.db` |
| `POSTGRES_URL` | PostgreSQL connection URL (alternative) | - |
| `ROMA_TIMER_HOST` | Server bind address | `0.0.0.0` |
| `ROMA_TIMER_PORT` | Server port | `3000` |

## Database URL Formats

### SQLite
```
sqlite:filename.db
sqlite:/absolute/path/to/database.db
sqlite::memory:   # In-memory database
```

### PostgreSQL
```
postgres://user:password@localhost:5432/database_name
postgresql://user:password@host:port/database_name
```

## Build Features

The application supports conditional compilation features:

```bash
# SQLite only (default)
cargo build --features sqlite

# PostgreSQL only
cargo build --features postgres

# Both databases supported
cargo build --features "sqlite postgres"

# Release build
cargo build --release --features "sqlite postgres"
```

## Docker Compose Example

Create a `docker-compose.yml` file:

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: roma_timer
      POSTGRES_USER: roma_user
      POSTGRES_PASSWORD: timer_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  roma-timer:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: "postgres://roma_user:timer_password@postgres:5432/roma_timer"
      ROMA_TIMER_HOST: "0.0.0.0"
      ROMA_TIMER_PORT: "3000"
    depends_on:
      - postgres

volumes:
  postgres_data:
```

## Database Migration

The application automatically creates database tables on startup. No manual migration is needed.

### Schema

#### user_configurations table
```sql
CREATE TABLE user_configurations (
    id TEXT PRIMARY KEY,
    work_duration INTEGER NOT NULL DEFAULT 1500,
    short_break_duration INTEGER NOT NULL DEFAULT 300,
    long_break_duration INTEGER NOT NULL DEFAULT 900,
    long_break_frequency INTEGER NOT NULL DEFAULT 4,
    notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    webhook_url TEXT,
    wait_for_interaction BOOLEAN NOT NULL DEFAULT FALSE,
    theme TEXT NOT NULL DEFAULT 'Light' CHECK (theme IN ('Light', 'Dark')),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
```

#### timer_sessions table
```sql
CREATE TABLE timer_sessions (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    timer_type TEXT NOT NULL,
    duration INTEGER NOT NULL,
    elapsed INTEGER NOT NULL DEFAULT 0,
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    completed_at BIGINT
);
```

#### notification_events table
```sql
CREATE TABLE notification_events (
    id TEXT PRIMARY KEY,
    timer_session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    message TEXT,
    delivered BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    delivered_at BIGINT
);
```

## Performance Considerations

### SQLite
- **Pros**: Zero configuration, portable, good for development/small deployments
- **Cons**: Limited concurrent writes, single-writer limitation
- **Best for**: Development, single-user deployments, small installations

### PostgreSQL
- **Pros**: Full concurrent support, advanced features, better for production
- **Cons**: Requires setup, external dependency
- **Best for**: Production, multi-user deployments, large installations

## Connection Pooling

Both databases use SQLx's built-in connection pooling:

- Default pool size: 10 connections
- Configurable via environment variables if needed
- Automatic connection cleanup and management

## Troubleshooting

### SQLite Permission Issues
```bash
# Ensure the application can write to the database file
chmod 664 roma-timer.db
chown $USER:$USER roma-timer.db
```

### PostgreSQL Connection Issues
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql "postgres://user:password@localhost/database_name"

# Check connection limits
sudo -u postgres psql -c "SHOW max_connections;"
```

### Database Migration Issues
The application will automatically create tables. If you encounter issues:

1. Check database permissions
2. Verify connection string format
3. Ensure the database server is running
4. Check application logs for detailed error messages