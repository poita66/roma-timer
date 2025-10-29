# Roma Timer - Simple Pomodoro Timer 🍅

A simple, beautiful, and functional Pomodoro timer web application built with Rust backend and vanilla JavaScript frontend.

## Features

- ✅ **Pomodoro Technique**: 25-minute work sessions with 5-minute breaks
- ✅ **Customizable Durations**: Configure work, short break, and long break durations
- ✅ **Beautiful UI**: Clean, responsive design with light/dark themes
- ✅ **Progress Visualization**: Circular progress indicator
- ✅ **Session Counter**: Track your completed work sessions
- ✅ **Audio Notifications**: Sound alerts when sessions complete
- ✅ **PWA Support**: Install as a native app on supported devices
- ✅ **Settings Persistence**: Your preferences are saved locally
- ✅ **Keyboard Accessible**: Full keyboard navigation support

## Quick Start

### Using the Pre-built Binary (Recommended)

1. Download the latest binary for your platform from the [Releases](https://github.com/your-username/roma-timer/releases) page
2. Extract and run the binary:
   ```bash
   ./roma-timer
   ```
3. Open your browser and navigate to `http://localhost:3000`

### Building from Source

#### Prerequisites

- Rust 1.70+
- Git

#### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/roma-timer.git
   cd roma-timer
   ```

2. Build the application:
   ```bash
   cd backend
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/roma-timer
   ```

4. Open your browser and navigate to `http://localhost:3000`

## Usage

### Basic Timer Controls

- **Start**: Begin the current timer session
- **Pause**: Pause the running timer
- **Reset**: Reset the current session to its full duration
- **Skip**: Skip to the next session type

### Settings

Configure your Pomodoro sessions:

- **Work Duration**: Length of work sessions (default: 25 minutes)
- **Short Break**: Length of short breaks (default: 5 minutes)
- **Long Break**: Length of long breaks (default: 15 minutes)
- **Long Break Frequency**: Number of work sessions before a long break (default: 4)
- **Notifications**: Enable/disable browser notifications
- **Theme**: Choose between light and dark themes

### PWA Installation

On supported browsers, you can install Roma Timer as a Progressive Web App:

1. Open the app in Chrome, Edge, or Firefox
2. Look for the "Install" icon in the address bar
3. Click "Install" to add it to your applications

## API Endpoints

The application exposes a simple REST API:

- `GET /api/timer` - Get current timer state
- `POST /api/timer` - Control timer (start/pause/reset/skip)
- `GET /api/settings` - Get current settings
- `POST /api/settings` - Update settings
- `GET /api/health` - Health check

### Timer Control

Send POST requests to `/api/timer` with JSON payload:

```json
{
  "action": "start"  // or "pause", "reset", "skip"
}
```

### Settings Update

Send POST requests to `/api/settings` with JSON payload:

```json
{
  "work_duration": 1500,
  "short_break_duration": 300,
  "long_break_duration": 900,
  "long_break_frequency": 4
}
```

## Configuration

### Environment Variables

- `PORT`: Server port (default: 3000)
- `HOST`: Server host (default: 0.0.0.0)

### Example

```bash
PORT=8080 HOST=127.0.0.1 ./roma-timer
```

## Development

### Running in Development Mode

```bash
cd backend
cargo run
```

### Building Frontend Assets

The frontend is built with vanilla HTML, CSS, and JavaScript - no build step required!

### File Structure

```
roma-timer/
├── backend/
│   ├── src/
│   │   └── main.rs              # Main application server
│   └── Cargo.toml               # Rust dependencies
├── frontend/
│   ├── index.html               # Main HTML page
│   ├── styles.css               # Styles and themes
│   ├── script.js                # Main application logic
│   ├── manifest.json            # PWA manifest
│   └── sw.js                    # Service worker for offline support
└── README_SIMPLE.md             # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the Pomodoro Technique® developed by Francesco Cirillo
- Built with Rust (Axum) and vanilla JavaScript
- Icons and design inspired by modern productivity apps

---

**Roma Timer** - Stay focused, be productive! 🍅