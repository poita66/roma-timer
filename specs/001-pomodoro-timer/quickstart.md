# Roma Timer Quick Start Guide

**Version**: 1.0.0
**Updated**: 2025-10-29

## Overview

Roma Timer is a self-hostable pomodoro timer web UI with multi-device synchronization. This guide will help you get it running in minutes.

## Prerequisites

- Rust 1.75+ (for building from source)
- Node.js 18+ (for frontend development only)
- SQLite (included with Rust binary)

## Quick Start (Production Deployment)

### 1. Download Binary

Download the latest Roma Timer binary for your platform:
```bash
# Linux (x86_64)
wget https://github.com/your-repo/roma-timer/releases/latest/download/roma-timer-linux-x86_64

# macOS (Intel)
wget https://github.com/your-repo/roma-timer/releases/latest/download/roma-timer-macos-x86_64

# macOS (Apple Silicon)
wget https://github.com/your-repo/roma-timer/releases/latest/download/roma-timer-macos-aarch64
```

### 2. Configure

Set environment variables (optional):
```bash
export ROMA_TIMER_HOST=0.0.0.0
export ROMA_TIMER_PORT=3000
export ROMA_TIMER_SECRET="your-secret-here"
export ROMA_TIMER_DB_URL="sqlite://roma-timer.db"
```

### 3. Run

```bash
chmod +x roma-timer-linux-x86_64
./roma-timer-linux-x86_64
```

### 4. Access

Open your browser and navigate to:
- Web UI: `http://localhost:3000`
- API docs: `http://localhost:3000/api/health`

## Development Setup

### 1. Clone Repository

```bash
git clone https://github.com/your-repo/roma-timer.git
cd roma-timer
```

### 2. Install Dependencies

```bash
# Backend (Rust)
cd backend
cargo build
cd ..

# Frontend (Node.js)
cd frontend
npm install
cd ..
```

### 3. Development Mode

Run backend and frontend in separate terminals:

**Terminal 1 - Backend:**
```bash
cd backend
cargo run
```

**Terminal 2 - Frontend:**
```bash
cd frontend
npm start
```

### 4. Access Development

- Frontend: `http://localhost:19006` (Expo development server)
- Backend API: `http://localhost:3000`

## Docker Deployment

### 1. Using Docker Compose (Recommended)

```bash
docker-compose up -d
```

### 2. Manual Docker Build

```bash
# Build image
docker build -t roma-timer .

# Run container
docker run -d \
  --name roma-timer \
  -p 3000:3000 \
  -e ROMA_TIMER_SECRET="your-secret-here" \
  -v $(pwd)/data:/app/data \
  roma-timer
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ROMA_TIMER_HOST` | `0.0.0.0` | Server bind address |
| `ROMA_TIMER_PORT` | `3000` | Server port |
| `ROMA_TIMER_SECRET` | `change-me` | Shared secret for authentication |
| `ROMA_TIMER_DB_URL` | `sqlite://roma-timer.db` | Database connection string |

### Configuration File (Optional)

Create `config.toml` in the same directory as the binary:

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite://roma-timer.db"

[auth]
shared_secret = "your-secret-here"
```

## Usage

### Basic Timer Control

1. **Start Timer**: Click the play button or press Space
2. **Pause Timer**: Click the pause button or press Space
3. **Reset Timer**: Click the reset button or press R
4. **Skip Session**: Click the skip button or press S

### Multi-Device Sync

1. Open Roma Timer on multiple devices (PC, phone, tablet)
2. Use the same shared secret on all devices
3. Timer state automatically syncs across all connected devices

### Settings

Access settings by clicking the gear icon:

- **Work Duration**: Length of work sessions (default: 25 minutes)
- **Short Break**: Length of short breaks (default: 5 minutes)
- **Long Break**: Length of long breaks (default: 15 minutes)
- **Long Break After**: Number of work sessions before long break (default: 4)
- **Notifications**: Enable browser notifications
- **Webhook URL**: Optional webhook for timer events
- **Wait for Interaction**: Require user input before starting next session
- **Theme**: Light or dark mode

## Troubleshooting

### Common Issues

**Timer not syncing across devices:**
- Ensure all devices use the same shared secret
- Check network connectivity
- Verify WebSocket connection (browser console)

**Can't access the web UI:**
- Check if the service is running: `curl http://localhost:3000/api/health`
- Verify port is not blocked by firewall
- Check if another service is using port 3000

**Authentication errors:**
- Verify `ROMA_TIMER_SECRET` environment variable is set
- Check client-side authentication token
- Ensure shared secret matches across devices

### Logs

Check application logs for debugging:
```bash
# If running as systemd service
sudo journalctl -u roma-timer -f

# If running in Docker
docker logs -f roma-timer

# If running binary directly
./roma-timer 2>&1 | tee roma-timer.log
```

### Health Check

Verify the service is healthy:
```bash
curl http://localhost:3000/api/health
```

Expected response:
```json
{
  "status": "healthy",
  "timestamp": 1698569400
}
```

## API Usage

### Authentication

Include the shared secret in requests:
```bash
curl -H "X-Auth-Token: your-secret-here" \
     http://localhost:3000/api/timer
```

### Basic API Calls

```bash
# Get timer state
curl -H "X-Auth-Token: your-secret-here" \
     http://localhost:3000/api/timer

# Start timer
curl -X POST \
     -H "X-Auth-Token: your-secret-here" \
     http://localhost:3000/api/timer/start

# Pause timer
curl -X POST \
     -H "X-Auth-Token: your-secret-here" \
     http://localhost:3000/api/timer/pause

# Get configuration
curl -H "X-Auth-Token: your-secret-here" \
     http://localhost:3000/api/configuration
```

## Production Deployment

### Systemd Service

Create `/etc/systemd/system/roma-timer.service`:

```ini
[Unit]
Description=Roma Timer
After=network.target

[Service]
Type=simple
User=roma-timer
Group=roma-timer
WorkingDirectory=/opt/roma-timer
Environment=ROMA_TIMER_SECRET="your-production-secret"
Environment=ROMA_TIMER_HOST="0.0.0.0"
Environment=ROMA_TIMER_PORT="3000"
ExecStart=/opt/roma-timer/roma-timer
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable roma-timer
sudo systemctl start roma-timer
```

### Reverse Proxy (nginx)

Configure nginx to proxy to Roma Timer:

```nginx
server {
    listen 80;
    server_name timer.yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket proxy
    location /ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### SSL/TLS

Use Let's Encrypt for HTTPS:
```bash
sudo certbot --nginx -d timer.yourdomain.com
```

## Support

- **Documentation**: [Full Documentation](https://docs.roma-timer.com)
- **Issues**: [GitHub Issues](https://github.com/your-repo/roma-timer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-repo/roma-timer/discussions)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause |
| `R` | Reset |
| `S` | Skip Session |
| `Ctrl+,` | Settings |
| `Esc` | Close dialogs |