# Roma Timer Makefile
# Provides convenient commands for development and deployment

.PHONY: help build dev dev-backend dev-backend test lint clean docker-build docker-run

# Default target
help:
	@echo "Roma Timer - Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  dev          - Run development servers (backend + frontend)"
	@echo "  dev-backend  - Run backend development server only"
	@echo "  dev-frontend - Run frontend development server only"
	@echo ""
	@echo "Building:"
	@echo "  build        - Build backend (release mode)"
	@echo "  build-frontend - Build frontend for production"
	@echo ""
	@echo "Testing:"
	@echo "  test         - Run backend tests"
	@echo "  test-backend - Run backend tests"
	@echo "  lint         - Run linting (clippy + eslint)"
	@echo "  lint-backend - Run clippy linting"
	@echo "  lint-frontend - Run eslint linting"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run Docker container"
	@echo "  docker-compose-up - Start with docker-compose"
	@echo "  docker-compose-down - Stop docker-compose"
	@echo ""
	@echo "Maintenance:"
	@echo "  clean        - Clean build artifacts"

# Development
dev: dev-backend dev-frontend

dev-backend:
	@echo "Starting backend development server..."
	cd backend && cargo run

dev-frontend:
	@echo "Starting frontend development server..."
	cd frontend && npm start

# Building
build:
	@echo "Building backend..."
	cd backend && cargo build --release

build-frontend:
	@echo "Building frontend..."
	cd frontend && npm run build

# Testing
test: test-backend

test-backend:
	@echo "Running backend tests..."
	cd backend && cargo test

# Linting
lint: lint-backend lint-frontend

lint-backend:
	@echo "Running clippy..."
	cd backend && cargo clippy -- -D warnings

lint-frontend:
	@echo "Running eslint..."
	cd frontend && npm run lint

# Docker
docker-build:
	@echo "Building Docker image..."
	docker build -t roma-timer:latest .

docker-run:
	@echo "Running Docker container..."
	docker run -d \
		--name roma-timer \
		-p 3000:3000 \
		-e ROMA_TIMER_SECRET="dev-secret" \
		-v $(PWD)/data:/app/data \
		roma-timer:latest

docker-compose-up:
	@echo "Starting with docker-compose..."
	docker-compose up -d

docker-compose-down:
	@echo "Stopping docker-compose..."
	docker-compose down

# Maintenance
clean:
	@echo "Cleaning build artifacts..."
	cd backend && cargo clean
	cd frontend && rm -rf node_modules dist build web-build
	rm -rf data/

# Database
migrate:
	@echo "Running database migrations..."
	cd backend && cargo run --bin roma-timer

# Install dependencies
install:
	@echo "Installing dependencies..."
	cd backend && cargo build
	cd frontend && npm install