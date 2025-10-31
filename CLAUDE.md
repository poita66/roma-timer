# roma-timer Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-29

## Active Technologies

- Rust 1.83+ (backend), React Native (frontend PWA) + Tokio (async runtime), JSON file storage, React Native PWA framework (001-pomodoro-timer)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.83+ (backend), React Native (frontend PWA): Follow standard conventions

## Recent Changes

- Updated Rust version requirement from 1.75+ to 1.83+ (backend)
- Added configurable persistence directory with ROMA_TIMER_DATA_DIR
- Updated Docker configuration to use named volumes
- Added comprehensive CI/CD pipeline with GitHub Actions
- Fixed cargo format and clippy warnings
- Added rust-version constraint to Cargo.toml

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
