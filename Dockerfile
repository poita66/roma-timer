# Multi-stage build for Roma Timer
# Stage 1: Build the Rust backend using nightly for edition2024 support
FROM rustlang/rust:nightly as backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy backend source
COPY backend/ ./backend/

# Build the backend with SQLite support (default) using locked dependencies
WORKDIR /app/backend
RUN cargo build --release --features sqlite --locked

# Stage 2: Build the frontend (simple copy for vanilla JS)
FROM alpine:latest as frontend-builder

WORKDIR /app

# Copy frontend source (no build needed for vanilla JS)
COPY frontend/ ./frontend/

# Stage 3: Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 roma-timer

WORKDIR /app

# Copy the compiled backend
COPY --from=backend-builder /app/backend/target/release/roma-timer ./

# Copy the frontend (vanilla JS, no build step needed)
COPY --from=frontend-builder /app/frontend/ ./public/

# Also copy to expected location for backend and create symlink at root
RUN mkdir -p /app/frontend && cp -r /app/public/* /app/frontend/ && \
    ln -sf /app/frontend /frontend

# Copy database migrations
COPY backend/migrations ./migrations/

# Create data directory with correct ownership
RUN mkdir -p /app/data && chown -R roma-timer:roma-timer /app && \
    chmod 755 /app/data

# Create entrypoint script to handle permissions
RUN echo '#!/bin/sh\n\
# Ensure data directory has correct permissions\n\
if [ -d "/app/data" ]; then\n\
    chown -R roma-timer:roma-timer /app/data\n\
    chmod 755 /app/data\n\
fi\n\
# Create the database file if it doesn'\''t exist\n\
if [ ! -f "/app/data/roma-timer.db" ]; then\n\
    su-exec roma-timer touch /app/data/roma-timer.db\n\
fi\n\
# Switch to roma-timer user and start the application\n\
exec gosu roma-timer "$@"' > /docker-entrypoint.sh && \
    chmod +x /docker-entrypoint.sh

# Install gosu for user switching
RUN apt-get update && apt-get install -y gosu && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/docker-entrypoint.sh"]

# Expose port
EXPOSE 3000

# Environment variables for SQLite
ENV ROMA_TIMER_HOST="0.0.0.0"
ENV ROMA_TIMER_PORT="3000"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

# Run the application
CMD ["./roma-timer"]