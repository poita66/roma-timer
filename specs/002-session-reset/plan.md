# Implementation Plan: Daily Session Reset

**Branch**: `002-session-reset` | **Date**: 2025-01-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-session-reset/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Daily session reset feature adds configurable automatic session count reset at user-defined times and manual session count adjustment. Researched and designed with tokio-cron-scheduler for background tasks, chrono-tz for timezone handling, SQLite for persistence, and WebSocket for real-time synchronization. Phase 0 research completed with all technical unknowns resolved. Phase 1 design completed with data model, API contracts, and implementation guide.

## Technical Context

**Language/Version**: Rust 1.83+ (backend), React Native (frontend PWA)
**Primary Dependencies**: Tokio (async runtime), SQLite (storage), React Native PWA framework, tokio-cron-scheduler v0.15+, chrono-tz v0.8+
**Storage**: SQLite database for persistent user preferences and session state
**Testing**: cargo test (backend), Jest/React Testing Library (frontend), mocktime v0.11+ for time mocking
**Target Platform**: Linux server (backend), Web PWA (frontend), iOS/Android (via React Native)
**Project Type**: Web application with PWA frontend and Rust backend
**Performance Goals**: Sub-100ms UI interactions, <200ms API responses, 50+ concurrent timer sessions
**Constraints**: <200ms API p95, <100MB memory usage, offline-capable with local state persistence, WCAG 2.1 AA accessibility
**Scale/Scope**: Individual users with multi-device synchronization, single binary deployment

**Technical Decisions** (Phase 0 Complete):
- Background task scheduling: tokio-cron-scheduler with SQLite persistence
- Time zone handling: chrono-tz with UTC storage and user timezone preferences
- Testing: TimeProvider trait with MockTimeProvider for deterministic testing
- Database: Extended user_configurations table with timezone and reset fields
- API: RESTful endpoints with WebSocket real-time synchronization

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **Phase 0 Gate Passed**: All technical unknowns resolved

✅ **Phase 1 Gate Passed**: Design completed and validated against constitution

**Code Quality Excellence**: ✅
- TimeProvider trait enables clean separation of concerns
- tokio-cron-scheduler and chrono-tz are well-maintained libraries
- Comprehensive error handling and validation included
- Clear documentation patterns established

**Test-First Development**: ✅
- MockTimeProvider enables deterministic testing without time dependencies
- Comprehensive unit test patterns for timezone and scheduling logic
- Integration test patterns for end-to-end scenarios
- 95% test coverage requirements established

**User Experience Consistency**: ✅
- WebSocket real-time synchronization across all devices
- WCAG 2.1 AA accessibility requirements specified
- Timezone-aware UI updates and notifications
- Graceful handling of network interruptions with local persistence

**Performance Requirements**: ✅
- <100ms UI interactions through efficient session count updates
- <200ms API responses through optimized database queries
- 5-minute check intervals for background scheduling efficiency
- <100MB memory usage with minimal dependency overhead

**Simplicity Focus**: ✅
- Direct alignment with core timer functionality (session counting)
- Minimal additional technology (2 new dependencies)
- Simple database schema extensions
- Clean separation of daily reset logic from core timer logic

## Project Structure

### Documentation (this feature)

```text
specs/002-session-reset/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output ✅ COMPLETED
├── data-model.md        # Phase 1 output ✅ COMPLETED
├── quickstart.md        # Phase 1 output ✅ COMPLETED
├── contracts/           # Phase 1 output ✅ COMPLETED
│   ├── api.yaml         # REST API contracts
│   └── websocket.yaml   # WebSocket message contracts
├── checklists/          # Quality assurance
│   └── requirements.md  # Specification quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Web application (Rust backend + React Native PWA frontend)
backend/
├── src/
│   ├── models/
│   │   ├── user_configuration.rs     # Extended with timezone & daily reset
│   │   ├── daily_session_stats.rs    # New analytics model
│   │   ├── scheduled_task.rs         # New background task model
│   │   └── session_reset_event.rs    # New audit model
│   ├── services/
│   │   ├── daily_reset_service.rs    # Core daily reset logic
│   │   ├── timezone_service.rs       # Timezone handling
│   │   ├── scheduling_service.rs     # Background task management
│   │   └── time_provider.rs          # Time abstraction for testing
│   ├── api/
│   │   ├── session_reset.rs          # REST API endpoints
│   │   └── analytics.rs              # Statistics endpoints
│   └── main.rs
├── migrations/
│   └── 002_session_reset.sql         # Database migrations
└── tests/
    ├── unit/
    │   ├── daily_reset_service_tests.rs
    │   └── timezone_service_tests.rs
    └── integration/
        ├── session_reset_api_tests.rs
        └── websocket_sync_tests.rs

frontend/
├── src/
│   ├── components/
│   │   ├── SessionCountDisplay.tsx   # Session count UI
│   │   ├── TimezonePicker.tsx        # Timezone configuration
│   │   └── DailyResetConfig.tsx      # Reset time configuration
│   ├── services/
│   │   ├── dailyResetApi.ts          # API client
│   │   └── websocketService.ts       # Real-time updates
│   └── hooks/
│       ├── useSessionCount.ts        # Session count state
│       └── useDailyReset.ts          # Daily reset configuration
├── public/
└── __tests__/
    ├── components/
    └── integration/
```

**Structure Decision**: Single repository with backend/frontend separation, extending existing Roma Timer architecture with minimal changes.

## Complexity Tracking

No constitution violations requiring justification. Design maintains simplicity while adding essential functionality.

## Implementation Status

### Phase 0: Research ✅ COMPLETED
- [x] Background task scheduling research
- [x] Timezone handling analysis
- [x] Time-based testing strategies
- [x] Database schema design
- [x] API design patterns

### Phase 1: Design ✅ COMPLETED
- [x] Data model specification
- [x] API contracts (REST + WebSocket)
- [x] Implementation quickstart guide
- [x] Constitution validation
- [x] Agent context updates

### Phase 2: Tasks 🚧 READY FOR /speckit.tasks
- [ ] Generate detailed task breakdown
- [ ] Create implementation timeline
- [ ] Define testing strategy
- [ ] Plan deployment approach

## Next Steps

**Command**: Run `/speckit.tasks` to proceed with Phase 2 detailed task planning.

**Ready for Implementation**: All research and design phases complete. The feature is ready for implementation with:
- Clear technical decisions and architecture
- Comprehensive API contracts
- Complete data model specification
- Implementation quickstart guide
- Quality assurance checklist
