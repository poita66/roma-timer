---

description: "Task list for Daily Session Reset feature implementation"
---

# Tasks: Daily Session Reset

**Input**: Design documents from `/specs/002-session-reset/`
**Branch**: `002-session-reset`
**MVP Scope**: User Story 1 (Configure Daily Reset Time) - delivers core daily reset functionality

**Tests**: Tests are REQUIRED for this feature (Test-First Development principle from constitution)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Backend**: `backend/src/` for Rust backend code
- **Frontend**: `frontend/src/` for React Native PWA frontend code
- **Database**: `backend/migrations/` for database schema changes
- **Tests**: `backend/tests/` for backend tests

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create backend/src directory structure for daily reset feature
- [X] T002 Create frontend/src/components and frontend/src/services directories for daily reset UI
- [X] T003 [P] Add chrono-tz and tokio-cron-scheduler dependencies to backend/Cargo.toml
- [X] T004 [P] Add mocktime to backend/Cargo.toml for time-based testing
- [X] T005 [P] Configure clippy with zero-warning policy for backend code quality
- [X] T006 [P] Configure Jest testing setup for frontend components

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Create TimeProvider trait in backend/src/services/time_provider.rs for time abstraction
- [X] T008 [P] Create MockTimeProvider implementation for deterministic testing
- [X] T009 [P] Create SystemTimeProvider implementation for production use
- [X] T010 Create database migration schema in backend/migrations/002_session_reset.sql
- [X] T011 Extend UserConfiguration model in backend/src/models/user_configuration.rs with timezone and reset fields
- [X] T012 Create DailyResetTime enum in backend/src/models/user_configuration.rs with validation
- [X] T013 [P] Create DatabaseManager extension methods for daily reset operations
- [X] T014 [P] Setup WebSocket message schemas for real-time synchronization
- [X] T015 Configure error handling and structured logging for daily reset operations
- [X] T016 Create unit test base infrastructure with mock time support

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Configure Daily Reset Time (Priority: P1) 🎯 MVP

**Goal**: Users can set a specific hour when session count automatically resets to 0 each day

**Independent Test**: Set reset time to "07:00" and verify session count resets at 7:00 AM, delivering fully functional daily reset

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T017 [P] [US1] Unit test for daily reset time calculation in backend/tests/unit/daily_reset_service_tests.rs
- [X] T018 [P] [US1] Unit test for timezone handling in backend/tests/unit/timezone_service_tests.rs
- [X] T019 [P] [US1] Unit test for background task scheduling in backend/tests/unit/scheduling_service_tests.rs
- [X] T020 [P] [US1] Integration test for complete daily reset cycle in backend/tests/integration/session_reset_websocket_tests.rs
- [X] T021 [P] [US1] Integration test for timezone-aware scheduling in backend/tests/integration/timezone_reset_tests.rs

### Implementation for User Story 1

#### Models and Data Layer
- [ ] T022 [P] [US1] Create DailySessionStats model in backend/src/models/daily_session_stats.rs
- [ ] T023 [P] [US1] Create ScheduledTask model in backend/src/models/scheduled_task.rs
- [ ] T024 [P] [US1] Create SessionResetEvent model in backend/src/models/session_reset_event.rs
- [ ] T025 [US1] Implement validation methods for DailyResetTime enum in backend/src/models/user_configuration.rs

#### Services Layer
- [ ] T026 [US1] Create DailyResetService in backend/src/services/daily_reset_service.rs (depends on T022-T025)
- [ ] T027 [US1] Create TimezoneService in backend/src/services/timezone_service.rs
- [ ] T028 [US1] Create SchedulingService in backend/src/services/scheduling_service.rs
- [ ] T029 [US1] Implement daily reset logic in DailyResetService (depends on T026-T028)

#### WebSocket Communication Layer
- [X] T030 [P] [US1] Create daily reset configuration message handlers in backend/src/websocket/handlers/daily_reset.rs
- [X] T031 [US1] Implement configure_daily_reset WebSocket message procedure
- [X] T032 [US1] Implement configure_timezone WebSocket message procedure
- [X] T033 [P] [US1] Add request/response message schemas for daily reset WebSocket procedures in backend/src/websocket/messages.rs

#### Frontend Components
- [X] T034 [P] [US1] Create TimezonePicker component in frontend/src/components/TimezonePicker.tsx
- [X] T035 [P] [US1] Create DailyResetConfig component in frontend/src/components/DailyResetConfig.tsx
- [X] T036 [US1] Create dailyResetWebSocket service in frontend/src/services/dailyResetWebSocket.ts
- [X] T037 [P] [US1] Create useDailyReset hook in frontend/src/hooks/useDailyReset.ts

#### Integration and Real-time
- [ ] T038 [US1] Integrate daily reset service with existing TimerService
- [ ] T039 [US1] Add WebSocket message handling for configuration changes
- [ ] T040 [US1] Implement real-time sync of daily reset configuration across devices
- [ ] T041 [US1] Add logging and error handling for daily reset operations

**Checkpoint**: User Story 1 complete - daily reset scheduling fully functional and independently testable

---

## Phase 4: User Story 2 - Manual Session Count Adjustment (Priority: P2)

**Goal**: Users can manually adjust session count at any time for flexibility and error correction

**Independent Test**: Set session count to 5 and verify it persists until next automated reset, delivering complete manual control

### Tests for User Story 2 ⚠️

- [ ] T042 [P] [US2] Unit test for manual session count validation in backend/tests/unit/session_count_tests.rs
- [ ] T043 [P] [US2] Unit test for session override behavior during reset in backend/tests/unit/session_override_tests.rs
- [ ] T044 [P] [US2] Integration test for manual session count WebSocket procedures in backend/tests/integration/session_count_websocket_tests.rs
- [ ] T045 [P] [US2] Integration test for WebSocket sync of manual changes in backend/tests/integration/session_sync_tests.rs

### Implementation for User Story 2

#### Backend Services
- [ ] T046 [P] [US2] Extend DailyResetService with manual override logic in backend/src/services/daily_reset_service.rs
- [ ] T047 [P] [US2] Add session count validation in backend/src/services/daily_reset_service.rs
- [ ] T048 [US2] Implement manual override persistence in backend/src/services/daily_reset_service.rs

#### WebSocket Message Procedures
- [ ] T049 [P] [US2] Implement get_session_count WebSocket message procedure in backend/src/websocket/handlers/session_count.rs
- [ ] T050 [P] [US2] Implement set_session_count WebSocket message procedure in backend/src/websocket/handlers/session_count.rs
- [ ] T051 [P] [US2] Implement reset_session WebSocket message procedure in backend/src/websocket/handlers/session_count.rs
- [ ] T052 [P] [US2] Add message validation and schema definitions for session count procedures

#### Frontend Components
- [ ] T053 [P] [US2] Create SessionCountDisplay component in frontend/src/components/SessionCountDisplay.tsx
- [ ] T054 [P] [US2] Add manual session count input UI to SessionCountDisplay component
- [ ] T055 [P] [US2] Extend dailyResetWebSocket with session count methods in frontend/src/services/dailyResetWebSocket.ts
- [ ] T056 [P] [US2] Create useSessionCount hook in frontend/src/hooks/useSessionCount.ts

#### Real-time Sync
- [ ] T057 [US2] Add WebSocket message handling for session count changes
- [ ] T058 [US2] Implement real-time sync of manual session overrides across devices
- [ ] T059 [US2] Add visual feedback for manual override status
- [ ] T060 [US2] Handle conflict resolution for concurrent manual changes

**Checkpoint**: User Story 2 complete - manual session adjustment fully functional

---

## Phase 5: User Story 3 - View Reset Schedule (Priority: P3)

**Goal**: Users can see when their next scheduled reset will occur for visibility and planning

**Independent Test**: Set reset time to 09:00 and verify countdown shows "Next reset in 30 minutes" at 08:30, delivering clear visibility

### Tests for User Story 3 ⚠️

- [ ] T061 [P] [US3] Unit test for reset time calculation in backend/tests/unit/reset_time_calculation_tests.rs
- [ ] T062 [P] [US3] Unit test for daily statistics aggregation in backend/tests/unit/daily_stats_tests.rs
- [ ] T063 [P] [US3] Integration test for analytics API in backend/tests/integration/analytics_api_tests.rs
- [ ] T064 [P] [US3] Integration test for reset events API in backend/tests/integration/reset_events_tests.rs

### Implementation for User Story 3

#### Backend Services
- [ ] T065 [P] [US3] Create analytics service in backend/src/services/analytics_service.rs
- [ ] T066 [P] [US3] Implement daily session statistics calculation in backend/src/services/analytics_service.rs
- [ ] T067 [P] [US3] Add next reset time calculation methods in backend/src/services/daily_reset_service.rs
- [ ] T068 [P] [US3] Create reset event logging service in backend/src/services/reset_event_service.rs

#### WebSocket Analytics Procedures
- [ ] T069 [P] [US3] Implement get_daily_stats WebSocket message procedure in backend/src/websocket/handlers/analytics.rs
- [ ] T070 [P] [US3] Implement get_reset_events WebSocket message procedure in backend/src/websocket/handlers/analytics.rs
- [ ] T071 [P] [US3] Add date range filtering and pagination for analytics WebSocket procedures
- [ ] T072 [P] [US3] Add timezone-aware statistics formatting for WebSocket responses

#### Frontend Components
- [ ] T073 [P] [US3] Create NextResetDisplay component in frontend/src/components/NextResetDisplay.tsx
- [ ] T074 [P] [US3] Create DailyStats component in frontend/src/components/DailyStats.tsx
- [ ] T075 [P] [US3] Create ResetEvents component in frontend/src/components/ResetEvents.tsx
- [ ] T076 [P] [US3] Add analytics WebSocket service in frontend/src/services/analyticsWebSocket.ts

#### Data Visualization
- [ ] T077 [US3] Implement countdown timer display in NextResetDisplay component
- [ ] T078 [US3] Add timezone-aware time formatting utilities
- [ ] T079 [US3] Create charts or graphs for daily session statistics
- [ ] T080 [US3] Add responsive design for analytics components

**Checkpoint**: User Story 3 complete - reset schedule visibility fully functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final polish, performance optimization, and cross-story improvements

### Performance & Reliability
- [ ] T081 [P] Optimize database queries for <200ms API response times
- [ ] T082 [P] Add database indexes for session reset queries
- [ ] T083 [P] Implement efficient WebSocket message batching
- [ ] T084 [P] Add connection pooling and timeout handling

### Accessibility & Internationalization
- [ ] T085 [P] Ensure WCAG 2.1 AA compliance for all daily reset components
- [ ] T086 [P] Add keyboard navigation support for timezone and time pickers
- [ ] T087 [P] Implement screen reader announcements for reset notifications
- [ ] T088 [P] Add proper ARIA labels and descriptions

### Error Handling & Edge Cases
- [ ] T089 [P] Handle device offline scenarios with local persistence
- [ ] T090 [P] Implement graceful handling of invalid timezone inputs
- [ ] T091 [P] Add DST transition edge case handling
- [ ] T092 [P] Create comprehensive error recovery procedures

### Documentation & Deployment
- [ ] T093 [P] Update WebSocket documentation with daily reset message procedures
- [ ] T094 [P] Create user guide for daily reset feature
- [ ] T095 [P] Add deployment instructions for timezone database updates
- [ ] T096 [P] Create monitoring and alerting for daily reset failures

### Final Testing & Quality Assurance
- [ ] T097 [P] Run full test suite and ensure 95% coverage requirement
- [ ] T098 [P] Perform cross-device synchronization testing
- [ ] T099 [P] Test timezone scenarios across different regions
- [ ] T100 [P] Validate WCAG 2.1 AA accessibility compliance

**Checkpoint**: Feature complete and production-ready

---

## Dependencies & Execution Order

### Story Dependencies
```
Phase 2 (Foundational) → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (Polish)
```

**Critical Path**: Phase 2 → Phase 3 (MVP)
- All foundational tasks (T007-T016) MUST complete before any user story
- User Story 1 is the MVP and delivers core value
- User Stories 2 and 3 can be implemented in parallel after US1

### Parallel Execution Opportunities

**Within User Stories**:
- **US1**: T017-T025 (tests) can run in parallel; T030-T041 (API/frontend) can run in parallel after T026-T029
- **US2**: T042-T045 (tests) can run in parallel; T049-T056 (API/frontend) can run in parallel
- **US3**: T061-T064 (tests) can run in parallel; T069-T080 (API/frontend) can run in parallel

**Cross-Story Parallel**:
- After US1 complete, US2 and US3 can be developed in parallel
- Phase 6 tasks can be done incrementally throughout development

## Independent Test Criteria

### User Story 1
**Test**: Configure reset time to "07:00" and verify session count resets automatically at 7:00 AM
**Criteria**: Session count resets within 1 minute of configured time, persists across restarts

### User Story 2
**Test**: Manually set session count to 5 and verify it persists until next automated reset
**Criteria**: Manual count reflects immediately in UI, maintains until scheduled reset overrides it

### User Story 3
**Test**: Set reset time to 09:00 and verify countdown shows correct time until next reset
**Criteria": Countdown updates accurately, handles timezone changes and DST transitions

## Implementation Strategy

### MVP (Minimum Viable Product)
**Scope**: User Story 1 only - Configure Daily Reset Time
**Timeline**: After Phase 2 foundation, implement Phase 3 completely
**Value**: Core daily reset functionality, immediate user benefit

### Incremental Delivery
1. **Release 1**: Daily reset configuration and automatic resets (US1)
2. **Release 2**: Manual session count adjustment (US2)
3. **Release 3**: Analytics and reset visibility (US3)
4. **Release 4**: Polish, accessibility, and performance optimization

## Quality Gates

### Completion Requirements
- All tests pass with 95% coverage
- API response times <200ms
- UI interactions <100ms
- WCAG 2.1 AA accessibility compliance
- Cross-device synchronization working
- Constitution principles validated

### Sign-off Criteria
- Independent test criteria met for all user stories
- Code quality gates passed (clippy zero warnings)
- Performance benchmarks met
- Documentation complete
- User acceptance testing approved

**Total Tasks**: 100
**Tasks per Story**:
- Setup: 6 tasks
- Foundational: 10 tasks
- US1 (MVP): 25 tasks
- US2: 18 tasks
- US3: 20 tasks
- Polish: 21 tasks

**Ready for Implementation**: Each task is specific enough for independent LLM completion with clear file paths and dependencies.