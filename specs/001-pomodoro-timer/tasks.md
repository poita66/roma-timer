---

description: "Task list template for feature implementation"
---

# Tasks: Roma Timer Application

**Input**: Design documents from `/specs/001-pomodoro-timer/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are MANDATORY based on Roma Timer constitution requirements for comprehensive testing.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume web application structure based on plan.md

<!--
  ============================================================================
  IMPORTANT: The tasks below are GENERATED TASKS based on the Roma Timer specification.

  Tasks are organized by user story to enable:
  - Independent implementation and testing of each story
  - MVP delivery after P1 stories (Basic Timer Control + Cross-Device Sync)
  - Incremental delivery with configuration (P2) and themes (P3)

  All tasks follow the checklist format: - [ ] [TaskID] [P?] [Story?] Description with file path
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize Rust backend with Tokio dependencies
- [ ] T003 Initialize React Native PWA frontend with Expo
- [ ] T004 [P] Configure clippy (Rust) and ESLint (React Native) with zero-warning policies
- [ ] T005 [P] Setup SQLite database migrations framework
- [ ] T006 [P] Setup basic Axum web server with health check endpoint
- [ ] T007 [P] Setup development build scripts and Docker configuration

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Setup SQLite database schema and migrations for timer sessions
- [ ] T009 [P] Implement shared-secret authentication middleware
- [ ] T010 [P] Setup WebSocket connection handling with tokio-tungstenite
- [ ] T011 [P] Setup API routing and middleware structure with <200ms response goals
- [ ] T012 Create base TimerSession model with validation rules
- [ ] T013 Create base UserConfiguration model with default values
- [ ] T014 Configure error handling and structured logging for debugging
- [ ] T015 Setup environment configuration management for shared secret and database
- [ ] T016 Create basic WebSocket message broadcasting system
- [ ] T017 [P] Setup embedded PWA serving using include_dir crate

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Pomodoro Timer Control (Priority: P1) 🎯 MVP

**Goal**: Core timer functionality with play/pause/reset/skip controls and automatic session transitions

**Independent Test**: Can be tested by operating timer controls and verifying accurate countdown and state transitions without any other devices connected

### Tests for User Story 1 (MANDATORY) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T018 [P] [US1] Unit test for TimerSession model in backend/tests/unit/test_timer_session.rs
- [ ] T019 [P] [US1] Unit test for timer control logic in backend/tests/unit/test_timer_service.rs
- [ ] T020 [P] [US1] Integration test for timer API endpoints in backend/tests/integration/test_timer_api.rs
- [ ] T021 [P] [US1] Component test for TimerDisplay component in frontend/src/components/TimerDisplay/__tests__/TimerDisplay.test.tsx
- [ ] T022 [P] [US1] Component test for TimerControls component in frontend/src/components/TimerControls/__tests__/TimerControls.test.tsx
- [ ] T023 [P] [US1] E2E test for complete timer flow in frontend/src/__tests__/e2e/timer-flow.test.ts

### Implementation for User Story 1

**Backend Timer Logic**
- [ ] T024 [P] [US1] Create TimerSession model in backend/src/models/timer_session.rs
- [ ] T025 [P] [US1] Create TimerService with countdown logic in backend/src/services/timer_service.rs
- [ ] T026 [US1] Implement timer state machine with session transitions in backend/src/services/timer_service.rs
- [ ] T027 [US1] Create background task for timer countdown updates in backend/src/services/timer_service.rs

**Backend API Endpoints**
- [ ] T028 [US1] Implement GET /api/timer endpoint in backend/src/api/timer.rs
- [ ] T029 [US1] Implement POST /api/timer/start endpoint in backend/src/api/timer.rs
- [ ] T030 [US1] Implement POST /api/timer/pause endpoint in backend/src/api/timer.rs
- [ ] T031 [US1] Implement POST /api/timer/reset endpoint in backend/src/api/timer.rs
- [ ] T032 [US1] Implement POST /api/timer/skip endpoint in backend/src/api/timer.rs

**Backend WebSocket Integration**
- [ ] T033 [US1] Add timer state broadcasting to WebSocket service in backend/src/services/websocket_service.rs
- [ ] T034 [US1] Integrate timer service with WebSocket message broadcasting in backend/src/main.rs

**Frontend Timer Display**
- [ ] T035 [P] [US1] Create TimerSession TypeScript interface in frontend/src/types/index.ts
- [ ] T036 [P] [US1] Create TimerDisplay component with countdown in frontend/src/components/TimerDisplay/TimerDisplay.tsx
- [ ] T037 [P] [US1] Create useTimer hook for timer state management in frontend/src/hooks/useTimer.ts
- [ ] T038 [US1] Add timer session type display (Work/ShortBreak/LongBreak) in frontend/src/components/TimerDisplay/TimerDisplay.tsx

**Frontend Timer Controls**
- [ ] T039 [P] [US1] Create TimerControls component with play/pause buttons in frontend/src/components/TimerControls/TimerControls.tsx
- [ ] T040 [P] [US1] Create reset and skip controls in frontend/src/components/TimerControls/TimerControls.tsx
- [ ] T041 [P] [US1] Add keyboard accessibility (Space, R, S keys) in frontend/src/components/TimerControls/TimerControls.tsx
- [ ] T042 [P] [US1] Add ARIA labels and screen reader support in frontend/src/components/TimerControls/TimerControls.tsx

**Frontend API Integration**
- [ ] T043 [P] [US1] Create API client for timer endpoints in frontend/src/services/api.ts
- [ ] T044 [P] [US1] Create WebSocket client for real-time updates in frontend/src/services/websocket.ts
- [ ] T045 [US1] Integrate API calls with timer controls in frontend/src/hooks/useTimer.ts

**Frontend Main Screen**
- [ ] T046 [US1] Create TimerScreen with timer display and controls in frontend/src/pages/TimerScreen.tsx
- [ ] T047 [US1] Add responsive design for various screen sizes in frontend/src/pages/TimerScreen.tsx
- [ ] T048 [US1] Add loading and error states in frontend/src/pages/TimerScreen.tsx

**Checkpoint**: At this point, User Story 1 should be fully functional and independently testable

---

## Phase 4: User Story 2 - Cross-Device Synchronization (Priority: P1) 🎯 MVP

**Goal**: Real-time timer state synchronization across multiple connected devices

**Independent Test**: Can be tested by starting a timer on one device, opening the app on another device, and verifying the timer state synchronizes in real-time

### Tests for User Story 2 (MANDATORY) ⚠️

- [ ] T049 [P] [US2] Integration test for WebSocket synchronization in backend/tests/integration/test_websocket_sync.rs
- [ ] T050 [P] [US2] Integration test for multi-device connection handling in backend/tests/integration/test_multi_device.rs
- [ ] T051 [P] [US2] Component test for WebSocket reconnection in frontend/src/services/__tests__/websocket.test.ts
- [ ] T052 [P] [US2] E2E test for cross-device synchronization in frontend/src/__tests__/e2e/sync-flow.test.ts

### Implementation for User Story 2

**Backend WebSocket Enhancements**
- [ ] T053 [P] [US2] Implement device connection tracking in backend/src/models/device_connection.rs
- [ ] T054 [P] [US2] Add connection lifecycle management in backend/src/services/websocket_service.rs
- [ ] T055 [P] [US2] Implement connection heartbeat monitoring in backend/src/services/websocket_service.rs
- [ ] T056 [US2] Add graceful connection cleanup on disconnect in backend/src/services/websocket_service.rs

**Backend Multi-Device Logic**
- [ ] T057 [US2] Implement timer state broadcasting to all connected devices in backend/src/services/timer_service.rs
- [ ] T058 [US2] Add concurrent device support with connection pooling in backend/src/services/websocket_service.rs
- [ ] T059 [US2] Handle simultaneous control requests from multiple devices in backend/src/services/timer_service.rs

**Frontend WebSocket Client**
- [ ] T060 [P] [US2] Enhance WebSocket client with reconnection logic in frontend/src/services/websocket.ts
- [ ] T061 [P] [US2] Add connection status indicators in frontend/src/hooks/useWebSocket.ts
- [ ] T062 [P] [US2] Implement exponential backoff reconnection in frontend/src/services/websocket.ts

**Frontend Sync Integration**
- [ ] T063 [P] [US2] Integrate WebSocket updates with timer state in frontend/src/hooks/useTimer.ts
- [ ] T064 [P] [US2] Add sync status UI elements in frontend/src/pages/TimerScreen.tsx
- [ ] T065 [P] [US2] Handle sync conflicts and resolution in frontend/src/hooks/useTimer.ts

**Performance Optimization**
- [ ] T066 [US2] Optimize WebSocket message broadcasting for 50+ concurrent sessions in backend/src/services/websocket_service.rs
- [ ] T067 [US2] Add message batching for efficient state updates in backend/src/services/timer_service.rs
- [ ] T068 [US2] Implement sub-500ms synchronization timing in backend/src/services/websocket_service.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently and together

---

## Phase 5: User Story 3 - Timer Configuration (Priority: P2)

**Goal**: User-configurable timer durations, notification preferences, and break scheduling

**Independent Test**: Can be tested by modifying settings and verifying the timer uses the new configurations in subsequent sessions

### Tests for User Story 3 (MANDATORY) ⚠️

- [ ] T069 [P] [US3] Unit test for UserConfiguration model in backend/tests/unit/test_user_configuration.rs
- [ ] T070 [P] [US3] Integration test for configuration API endpoints in backend/tests/integration/test_config_api.rs
- [ ] T071 [P] [US3] Component test for Settings component in frontend/src/components/Settings/__tests__/Settings.test.tsx
- [ ] T072 [P] [US3] E2E test for configuration changes in frontend/src/__tests__/e2e/config-flow.test.ts

### Implementation for User Story 3

**Backend Configuration Model**
- [ ] T073 [P] [US3] Complete UserConfiguration model implementation in backend/src/models/user_configuration.rs
- [ ] T074 [P] [US3] Add configuration validation rules in backend/src/models/user_configuration.rs
- [ ] T075 [P] [US3] Create default configuration initialization in backend/src/services/configuration_service.rs

**Backend Configuration API**
- [ ] T076 [P] [US3] Implement GET /api/configuration endpoint in backend/src/api/configuration.rs
- [ ] T077 [P] [US3] Implement PUT /api/configuration endpoint in backend/src/api/configuration.rs
- [ ] T078 [P] [US3] Add configuration update broadcasting via WebSocket in backend/src/services/configuration_service.rs

**Backend Timer Integration**
- [ ] T079 [US3] Integrate user configuration with timer session logic in backend/src/services/timer_service.rs
- [ ] T080 [US3] Implement custom duration support in timer countdown logic in backend/src/services/timer_service.rs
- [ ] T081 [US3] Add long break frequency tracking in backend/src/services/timer_service.rs

**Frontend Settings Components**
- [ ] T082 [P] [US3] Create SettingsScreen component in frontend/src/pages/SettingsScreen.tsx
- [ ] T083 [P] [US3] Create DurationSettings component for time configuration in frontend/src/components/Settings/DurationSettings.tsx
- [ ] T084 [P] [US3] Create NotificationSettings component in frontend/src/components/Settings/NotificationSettings.tsx
- [ ] T085 [P] [US3] Create GeneralSettings component for other preferences in frontend/src/components/Settings/GeneralSettings.tsx

**Frontend Configuration Management**
- [ ] T086 [P] [US3] Create useConfiguration hook for settings management in frontend/src/hooks/useConfiguration.ts
- [ ] T087 [P] [US3] Add configuration form validation in frontend/src/components/Settings/SettingsForm.tsx
- [ ] T088 [P] [US3] Implement settings persistence and synchronization in frontend/src/hooks/useConfiguration.ts

**Frontend Navigation**
- [ ] T089 [US3] Add settings navigation button to timer screen in frontend/src/pages/TimerScreen.tsx
- [ ] T090 [US3] Create app navigation structure in frontend/src/App.tsx
- [ ] T091 [US3] Add navigation accessibility support in frontend/src/components/Navigation/Navigation.tsx

**Checkpoint**: All user stories should now be independently functional with configuration support

---

## Phase 6: User Story 4 - Theme Selection (Priority: P3)

**Goal**: Light and dark theme switching with persistent preferences

**Independent Test**: Can be tested by switching themes and verifying the UI updates appropriately across all screens

### Tests for User Story 4 (MANDATORY) ⚠️

- [ ] T092 [P] [US4] Component test for theme switching in frontend/src/components/Theme/__tests__/ThemeToggle.test.tsx
- [ ] T093 [P] [US4] Component test for theme persistence in frontend/src/hooks/__tests__/useTheme.test.ts
- [ ] T094 [P] [US4] Visual regression test for theme changes in frontend/src/__tests__/visual/theme.test.ts

### Implementation for User Story 4

**Backend Theme Support**
- [ ] T095 [P] [US4] Add theme field to UserConfiguration model in backend/src/models/user_configuration.rs
- [ ] T096 [P] [US4] Add theme validation (Light/Dark only) in backend/src/models/user_configuration.rs

**Frontend Theme System**
- [ ] T097 [P] [US4] Create theme context and provider in frontend/src/contexts/ThemeContext.tsx
- [ ] T098 [P] [US4] Create theme constants and color definitions in frontend/src/themes/index.ts
- [ ] T099 [P] [US4] Create useTheme hook for theme management in frontend/src/hooks/useTheme.ts

**Frontend Theme Components**
- [ ] T100 [P] [US4] Create ThemeToggle component in frontend/src/components/Theme/ThemeToggle.tsx
- [ ] T101 [P] [US4] Add theme switching to Settings screen in frontend/src/pages/SettingsScreen.tsx
- [ ] T102 [P] [US4] Apply theme styling to all components in frontend/src/components/

**Frontend Theme Integration**
- [ ] T103 [US4] Integrate theme persistence with configuration in frontend/src/hooks/useTheme.ts
- [ ] T104 [US4] Add theme transitions and animations in frontend/src/styles/themes.css
- [ ] T105 [US4] Ensure WCAG color contrast compliance in frontend/src/themes/index.ts

**Checkpoint**: All user stories should now be independently functional with theme support

---

## Phase 7: Notification System (Cross-Cutting Concern)

**Purpose**: Timer completion notifications for all connected devices

- [ ] T106 [P] Create NotificationEvent model in backend/src/models/notification_event.rs
- [ ] T107 [P] Implement browser notification service in backend/src/services/notification_service.rs
- [ ] T108 [P] Add webhook delivery service in backend/src/services/notification_service.rs
- [ ] T109 [P] Create notification broadcasting via WebSocket in backend/src/services/notification_service.rs
- [ ] T110 [P] Create frontend notification service in frontend/src/services/notifications.ts
- [ ] T111 [P] Add browser notification API integration in frontend/src/services/notifications.ts
- [ ] T112 [P] Create notification settings management in frontend/src/components/Settings/NotificationSettings.tsx

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T113 [P] Documentation updates in README.md and docs/
- [ ] T114 [P] Code cleanup and refactoring for performance
- [ ] T115 [P] Performance optimization for <100ms UI interactions across all components
- [ ] T116 [P] Accessibility compliance verification (WCAG 2.1 AA) across all UI elements
- [ ] T117 [P] Security hardening for shared-secret authentication
- [ ] T118 [P] Load testing for 50+ concurrent timer sessions in backend/tests/load/
- [ ] T119 [P] Cross-browser compatibility testing in frontend/src/__tests__/compatibility/
- [ ] T120 [P] Network interruption and reconnection scenario testing in backend/tests/integration/test_network.rs
- [ ] T121 [P] Docker image optimization and production deployment scripts
- [ ] T122 [P] Final integration testing and bug fixes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User Story 1 & 2 (P1): Can proceed in parallel after Phase 2 - Core MVP functionality
  - User Story 3 (P2): Depends on Stories 1 & 2 - Configuration features
  - User Story 4 (P3): Can proceed independently after Phase 2 - Theme features
- **Notification System (Phase 7)**: Depends on Stories 1 & 2 - Cross-cutting feature
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (Basic Timer Control)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (Cross-Device Sync)**: Can start after Foundational (Phase 2) - Enhances Story 1 but can be developed independently
- **User Story 3 (Timer Configuration)**: Depends on Stories 1 & 2 being functional - Builds upon core timer functionality
- **User Story 4 (Theme Selection)**: Can start after Foundational (Phase 2) - Independent UI enhancement

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints/components
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, User Stories 1 & 2 can start in parallel
- User Story 4 (Themes) can be developed in parallel with Stories 1 & 2
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1 (MVP)

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for TimerSession model in backend/tests/unit/test_timer_session.rs"
Task: "Unit test for timer control logic in backend/tests/unit/test_timer_service.rs"
Task: "Integration test for timer API endpoints in backend/tests/integration/test_timer_api.rs"
Task: "Component test for TimerDisplay component in frontend/src/components/TimerDisplay/__tests__/TimerDisplay.test.tsx"

# Launch all models for User Story 1 together:
Task: "Create TimerSession model in backend/src/models/timer_session.rs"
Task: "Create TimerSession TypeScript interface in frontend/src/types/index.ts"
Task: "Create useTimer hook for timer state management in frontend/src/hooks/useTimer.ts"

# Launch all components for User Story 1 together:
Task: "Create TimerDisplay component with countdown in frontend/src/components/TimerDisplay/TimerDisplay.tsx"
Task: "Create TimerControls component with play/pause buttons in frontend/src/components/TimerControls/TimerControls.tsx"
Task: "Create TimerScreen with timer display and controls in frontend/src/pages/TimerScreen.tsx"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Basic Timer Control)
4. Complete Phase 4: User Story 2 (Cross-Device Sync)
5. **STOP and VALIDATE**: Test both stories independently and together
6. Deploy/demo core MVP functionality

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (Basic MVP)
3. Add User Story 2 → Test independently + integration → Deploy/Demo (Complete MVP)
4. Add User Story 3 → Test independently + integration → Deploy/Demo (Enhanced MVP)
5. Add User Story 4 → Test independently + integration → Deploy/Demo (Polished MVP)
6. Complete Notification System & Polish → Final release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Timer Control)
   - Developer B: User Story 2 (Synchronization)
   - Developer C: User Story 4 (Themes) - can start early as it's independent
3. Stories complete and integrate independently
4. Developer A/B: User Story 3 (Configuration) - needs Stories 1 & 2 functional
5. All developers: Polish & testing phase

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach per constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Total tasks: 122
- P1 (MVP) tasks: 48 (Stories 1 & 2)
- P2 tasks: 21 (Story 3 + Notifications)
- P3 tasks: 14 (Story 4)
- Polish tasks: 13 (Final phase)
- Parallel opportunities identified throughout for team acceleration