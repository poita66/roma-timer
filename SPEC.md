# Roma Timer

A self-hostable pomodoro timer web UI that you can use on multiple devices at once.
This way the user can start the timer on their PC and get notifications on their phone via the PWA.

## Stack

- React Native PWA frontend (allows for native apps later)
- Rust Tokio backend
- SQLite DB (for easy deployment)
- HTTP RESTful API
- Simple shared-secret auth
- Single binary (PWA packaged into Rust server binary)
- Container deployment (docker-compose.yaml included)

## UI

- Main screen: 
  - Simple play/pause, skip, reset buttons 
  - Timer countdown
  - Timer type (work, short break, long break)
  - Settings button
- Settings UI
  - Work/break time lengths
  - Long-break-after cycle-count
  - Notification enable
  - Notification webhook (for using pushbullet or similar)
  - Whether to wait for user interaction before starting the next timer
  - Light/dark mode
