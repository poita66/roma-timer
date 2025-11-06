# Docker Variants for Roma Timer

This document explains the different Docker variants available for Roma Timer and how to use them.

## Overview

Roma Timer supports two database backends:
- **SQLite** (default): Simple, file-based database suitable for development and small deployments
- **PostgreSQL**: Full-featured database suitable for production and multi-user deployments

Each database variant has its own Dockerfile and docker-compose configuration.

## Docker Variants

### 1. SQLite Variant

**Files:**
- [`Dockerfile.sqlite`](Dockerfile.sqlite)
- [`docker-compose.sqlite.yml`](docker-compose.sqlite.yml)

**Use Cases:**
- Development environments
- Single-user deployments
- Small installations
- When you want zero database configuration

**Advantages:**
- Zero configuration required
- Portable (single file database)
- Lower resource usage
- Faster startup

**Disadvantages:**
- Limited concurrent writes
- Not suitable for high-traffic multi-user scenarios
- Single-writer limitation

**Quick Start:**
```bash
# Using the SQLite docker-compose
docker-compose -f docker-compose.sqlite.yml up -d

# Or build and run manually
docker build -f Dockerfile.sqlite -t roma-timer-sqlite .
docker run -d -p 3000:3000 -v roma-timer-data:/app/data roma-timer-sqlite
```

### 2. PostgreSQL Variant

**Files:**
- [`Dockerfile.postgres`](Dockerfile.postgres)
- [`docker-compose.postgres.yml`](docker-compose.postgres.yml)

**Use Cases:**
- Production environments
- Multi-user deployments
- Large installations
- When you need advanced database features

**Advantages:**
- Full concurrent support
- Advanced database features
- Better for production workloads
- Scalable for multiple users

**Disadvantages:**
- Requires additional setup
- Higher resource usage
- External dependency

**Quick Start:**
```bash
# Using the PostgreSQL docker-compose
docker-compose -f docker-compose.postgres.yml up -d

# Or build and run manually
docker build -f Dockerfile.postgres -t roma-timer-postgres .
docker run -d -p 3000:3000 \
  -e DATABASE_URL="postgres://user:password@host:5432/database" \
  roma-timer-postgres
```

## Default Configuration

The default [`docker-compose.yml`](docker-compose.yml) uses the SQLite variant for simplicity and ease of use.

## Environment Variables

### Common Variables (Both Variants)

| Variable | Description | Default |
|----------|-------------|---------|
| `ROMA_TIMER_HOST` | Server bind address | `0.0.0.0` |
| `ROMA_TIMER_PORT` | Server port | `3000` |
| `ROMA_TIMER_SECRET` | Application secret | `change-me-in-production` |
| `ROMA_TIMER_SHARED_SECRET` | JWT secret | `jwt-secret-change-me-in-production` |
| `ROMA_TIMER_PEPPER` | Password pepper | `pepper-change-me-in-production` |
| `ROMA_TIMER_LOG_LEVEL` | Log level | `info` |

### SQLite-Specific Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite database path | `sqlite:/app/data/roma-timer.db` |
| `ROMA_TIMER_DATA_DIR` | Data directory | `/app/data` |

### PostgreSQL-Specific Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | `postgres://roma_user:password@postgres:5432/roma_timer` |
| `POSTGRES_DB` | PostgreSQL database name | `roma_timer` |
| `POSTGRES_USER` | PostgreSQL username | `roma_user` |
| `POSTGRES_PASSWORD` | PostgreSQL password | `timer_password_change_me` |

## Security Considerations

### Production Deployment

1. **Change Default Secrets**: Always change the default secrets in production:
   ```bash
   # Generate secure secrets
   openssl rand -hex 32  # For ROMA_TIMER_SECRET
   openssl rand -hex 32  # For ROMA_TIMER_SHARED_SECRET
   openssl rand -hex 32  # For ROMA_TIMER_PEPPER
   ```

2. **Use Environment Files**: Create a `.env` file for production:
   ```bash
   # .env file
   ROMA_TIMER_SECRET=your-secure-secret-here
   ROMA_TIMER_SHARED_SECRET=your-jwt-secret-here
   ROMA_TIMER_PEPPER=your-pepper-here
   POSTGRES_PASSWORD=your-secure-postgres-password
   ```

3. **Network Security**: Use Docker networks or expose only necessary ports.

### Database Security

**SQLite:**
- Ensure proper file permissions on the database file
- Use volumes for data persistence
- Backup the database file regularly

**PostgreSQL:**
- Use strong passwords
- Limit database user permissions
- Enable SSL connections if needed
- Regular backups using pg_dump

## Migration Between Variants

### From SQLite to PostgreSQL

1. Export data from SQLite:
   ```bash
   docker exec -it roma-timer-container sqlite3 /app/data/roma-timer.db .dump > backup.sql
   ```

2. Import to PostgreSQL:
   ```bash
   # Convert SQLite dump to PostgreSQL format (may require manual adjustments)
   psql -h localhost -U roma_user -d roma_timer < backup.sql
   ```

3. Update docker-compose configuration to use PostgreSQL variant.

### From PostgreSQL to SQLite

1. Export from PostgreSQL:
   ```bash
   pg_dump -h localhost -U roma_user roma_timer > backup.sql
   ```

2. Convert and import to SQLite (may require manual adjustments)

3. Update docker-compose configuration to use SQLite variant.

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Ensure ports 3000 (and 5432 for PostgreSQL) are available
2. **Permission Issues**: Check Docker volume permissions
3. **Database Connection**: Verify database URLs and credentials
4. **Health Checks**: Check container logs for startup issues

### Debug Commands

```bash
# View container logs
docker-compose logs roma-timer
docker-compose logs postgres  # For PostgreSQL variant

# Execute commands in container
docker-compose exec roma-timer /bin/sh

# Check database connectivity
docker-compose exec roma-timer curl -f http://localhost:3000/api/health
```

## Performance Tuning

### SQLite
- Use SSD storage for better performance
- Consider WAL mode for better concurrency
- Regular VACUUM operations

### PostgreSQL
- Tune PostgreSQL configuration in `postgresql.conf`
- Use connection pooling
- Monitor and tune memory settings
- Consider read replicas for high-traffic scenarios

## Backup and Recovery

### SQLite
```bash
# Backup
docker exec roma-timer-container cp /app/data/roma-timer.db /backup/

# Restore
docker cp backup/roma-timer.db roma-timer-container:/app/data/
```

### PostgreSQL
```bash
# Backup
docker exec postgres-container pg_dump -U roma_user roma_timer > backup.sql

# Restore
docker exec -i postgres-container psql -U roma_user roma_timer < backup.sql
```

## Development Workflow

For development, the SQLite variant is recommended due to its simplicity:

```bash
# Development with SQLite
docker-compose -f docker-compose.sqlite.yml up

# Development with PostgreSQL (if testing multi-user features)
docker-compose -f docker-compose.postgres.yml up
```

For production, the PostgreSQL variant is recommended for better scalability and reliability.