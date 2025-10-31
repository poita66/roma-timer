# Roma Timer Technical Research

**Date**: 2025-10-29
**Purpose**: Research findings for technical implementation decisions

## Real-time Synchronization Technology

**Decision**: WebSocket
**Rationale**:
- Bidirectional communication required for timer controls from any device
- Sub-500ms synchronization requirement achievable with WebSocket persistent connections
- React Native PWA has excellent WebSocket support
- Efficient for 50+ concurrent sessions with frequent state updates (every second)
- Tokio ecosystem provides robust WebSocket support via `tokio-tungstenite`

**Implementation**: Use WebSocket for real-time timer state broadcasting and control command handling. All timer state changes will be broadcast to connected clients within 500ms requirement.

## PWA Packaging Strategy

**Decision**: `include_dir` crate with Rust binary embedding
**Rationale**:
- Most mature and reliable approach for static file embedding in Rust
- Compile-time embedding ensures single binary deployment
- Maintains full PWA capabilities (manifest, service worker, offline support)
- Simple build process integration with React Native/Expo build output

**Implementation**: React Native PWA built via Expo, then embedded into Rust binary at compile time using `include_dir!`. Axum serves embedded static files with proper MIME types and SPA routing fallback.

## Shared-Secret Authentication

**Decision**: Simple shared-secret token system
**Rationale**:
- No PII stored, security requirements are minimal
- Simpler than JWT for self-hosted use case
- Easy configuration for single-user or small team deployment
- Sufficient for authenticating WebSocket connections and API requests

**Implementation**: Shared secret configured via environment variable. Token-based authentication for WebSocket connections and API requests. Simple token validation without complex JWT parsing.

## Offline Storage Strategy

**Decision**: Online-only architecture
**Rationale**:
- Frontend does not need local database
- All operations happen online with backend
- Simplifies architecture and reduces complexity
- Focus on core timer functionality rather than offline capabilities

**Implementation**: Frontend maintains minimal in-memory state, all persistence handled by backend SQLite database. Network interruptions handled gracefully with reconnection logic.

## Notification Delivery

**Decision**: WebSocket broadcast + optional webhooks
**Rationale**:
- Real-time delivery to all connected devices via WebSocket
- Browser notification API for client-side notifications
- Optional webhook integration for external notification systems
- Simple and reliable for timer completion notifications

**Implementation**: Timer completion triggers WebSocket broadcast to all connected devices. Browser displays notifications using Notification API. Optional webhook calls for external integrations.

## Technology Stack Summary

- **Backend**: Rust 1.75+ with Tokio async runtime, Axum web framework, SQLite storage
- **Frontend**: React Native PWA via Expo for cross-platform compatibility
- **Real-time**: WebSocket connections for sub-500ms synchronization
- **Authentication**: Simple shared-secret token system
- **Deployment**: Single binary with embedded PWA assets
- **Notifications**: WebSocket broadcast + browser notifications + optional webhooks