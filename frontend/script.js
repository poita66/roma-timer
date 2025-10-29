class PomodoroTimer {
    constructor() {
        // Default settings
        this.settings = {
            workDuration: 25 * 60,        // 25 minutes in seconds
            shortBreakDuration: 5 * 60,   // 5 minutes in seconds
            longBreakDuration: 15 * 60,   // 15 minutes in seconds
            longBreakFrequency: 4,        // Long break after 4 sessions
            notificationsEnabled: true,
            theme: 'light'
        };

        // Timer state
        this.currentSession = {
            type: 'work',                  // 'work', 'shortBreak', 'longBreak'
            duration: this.settings.workDuration,
            remaining: this.settings.workDuration,
            isRunning: false,
            sessionCount: 1,
            totalSessions: 0
        };

        // Timer interval
        this.interval = null;

        // Initialize
        this.init();
    }

    init() {
        this.loadSettings();
        this.setupEventListeners();
        this.updateDisplay();
        this.requestNotificationPermission();
    }

    setupEventListeners() {
        // Timer controls
        document.getElementById('startBtn').addEventListener('click', () => this.start());
        document.getElementById('pauseBtn').addEventListener('click', () => this.pause());
        document.getElementById('resetBtn').addEventListener('click', () => this.reset());
        document.getElementById('skipBtn').addEventListener('click', () => this.skip());

        // Settings
        document.getElementById('saveSettings').addEventListener('click', () => this.saveSettings());
        document.getElementById('theme').addEventListener('change', (e) => this.changeTheme(e.target.value));
    }

    start() {
        if (this.currentSession.isRunning) return;

        this.currentSession.isRunning = true;
        this.updateButtons();

        this.interval = setInterval(() => this.tick(), 1000);

        // Update UI
        document.getElementById('startBtn').disabled = true;
        document.getElementById('pauseBtn').disabled = false;
    }

    pause() {
        if (!this.currentSession.isRunning) return;

        this.currentSession.isRunning = false;
        clearInterval(this.interval);

        // Update UI
        document.getElementById('startBtn').disabled = false;
        document.getElementById('pauseBtn').disabled = true;
    }

    reset() {
        this.pause();
        this.currentSession.remaining = this.currentSession.duration;
        this.updateDisplay();
    }

    skip() {
        this.pause();
        this.nextSession();
    }

    tick() {
        this.currentSession.remaining--;

        if (this.currentSession.remaining <= 0) {
            this.sessionComplete();
        } else {
            this.updateDisplay();
        }
    }

    sessionComplete() {
        this.pause();

        // Show notification
        if (this.settings.notificationsEnabled) {
            this.showNotification();
        }

        // Play sound (using Web Audio API)
        this.playSound();

        // Move to next session
        setTimeout(() => {
            this.nextSession();
        }, 2000); // 2 second delay before starting next session
    }

    nextSession() {
        // Update session count
        if (this.currentSession.type === 'work') {
            this.currentSession.sessionCount++;
            this.currentSession.totalSessions++;
        }

        // Determine next session type
        if (this.currentSession.type === 'work') {
            if (this.currentSession.sessionCount % this.settings.longBreakFrequency === 0) {
                // Long break
                this.currentSession.type = 'longBreak';
                this.currentSession.duration = this.settings.longBreakDuration;
            } else {
                // Short break
                this.currentSession.type = 'shortBreak';
                this.currentSession.duration = this.settings.shortBreakDuration;
            }
        } else {
            // Back to work
            this.currentSession.type = 'work';
            this.currentSession.duration = this.settings.workDuration;
        }

        this.currentSession.remaining = this.currentSession.duration;
        this.updateDisplay();
        this.updateButtons();
    }

    updateDisplay() {
        // Update timer text
        const minutes = Math.floor(this.currentSession.remaining / 60);
        const seconds = this.currentSession.remaining % 60;
        const timeString = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
        document.getElementById('timerText').textContent = timeString;

        // Update timer type
        const typeText = {
            'work': 'Work Session',
            'shortBreak': 'Short Break',
            'longBreak': 'Long Break'
        };
        document.getElementById('timerType').textContent = typeText[this.currentSession.type];

        // Update session counter
        if (this.currentSession.type === 'work') {
            const sessionsUntilLongBreak = this.settings.longBreakFrequency -
                (this.currentSession.sessionCount % this.settings.longBreakFrequency);
            document.getElementById('sessionCounter').textContent =
                `Session ${this.currentSession.sessionCount} of ${this.settings.longBreakFrequency}`;
        } else {
            document.getElementById('sessionCounter').textContent =
                `Break time! ${this.currentSession.sessionCount} work sessions completed`;
        }

        // Update progress circle
        this.updateProgressCircle();

        // Update page title
        document.title = `${timeString} - ${typeText[this.currentSession.type]} | Roma Timer`;
    }

    updateProgressCircle() {
        const circle = document.getElementById('progressCircle');
        const circumference = 2 * Math.PI * 90; // radius = 90
        const progress = this.currentSession.remaining / this.currentSession.duration;
        const offset = circumference * (1 - progress);
        circle.style.strokeDashoffset = offset;

        // Change color based on session type
        const colors = {
            'work': '#e74c3c',
            'shortBreak': '#3498db',
            'longBreak': '#27ae60'
        };
        circle.style.stroke = colors[this.currentSession.type];
    }

    updateButtons() {
        const startBtn = document.getElementById('startBtn');
        const pauseBtn = document.getElementById('pauseBtn');

        startBtn.disabled = this.currentSession.isRunning;
        pauseBtn.disabled = !this.currentSession.isRunning;
    }

    saveSettings() {
        // Get values from form
        this.settings.workDuration = parseInt(document.getElementById('workDuration').value) * 60;
        this.settings.shortBreakDuration = parseInt(document.getElementById('shortBreakDuration').value) * 60;
        this.settings.longBreakDuration = parseInt(document.getElementById('longBreakDuration').value) * 60;
        this.settings.longBreakFrequency = parseInt(document.getElementById('longBreakFrequency').value);
        this.settings.notificationsEnabled = document.getElementById('notificationsEnabled').checked;
        this.settings.theme = document.getElementById('theme').value;

        // Save to localStorage
        localStorage.setItem('pomodoroSettings', JSON.stringify(this.settings));

        // Apply theme
        this.changeTheme(this.settings.theme);

        // Update current session if it's not running
        if (!this.currentSession.isRunning) {
            if (this.currentSession.type === 'work') {
                this.currentSession.duration = this.settings.workDuration;
            } else if (this.currentSession.type === 'shortBreak') {
                this.currentSession.duration = this.settings.shortBreakDuration;
            } else if (this.currentSession.type === 'longBreak') {
                this.currentSession.duration = this.settings.longBreakDuration;
            }
            this.currentSession.remaining = this.currentSession.duration;
            this.updateDisplay();
        }

        // Show success notification
        this.showNotification('Settings saved successfully!', 'success');
    }

    loadSettings() {
        const saved = localStorage.getItem('pomodoroSettings');
        if (saved) {
            this.settings = { ...this.settings, ...JSON.parse(saved) };

            // Update form values
            document.getElementById('workDuration').value = this.settings.workDuration / 60;
            document.getElementById('shortBreakDuration').value = this.settings.shortBreakDuration / 60;
            document.getElementById('longBreakDuration').value = this.settings.longBreakDuration / 60;
            document.getElementById('longBreakFrequency').value = this.settings.longBreakFrequency;
            document.getElementById('notificationsEnabled').checked = this.settings.notificationsEnabled;
            document.getElementById('theme').value = this.settings.theme;

            // Apply theme
            this.changeTheme(this.settings.theme);
        }
    }

    changeTheme(theme) {
        if (theme === 'dark') {
            document.documentElement.setAttribute('data-theme', 'dark');
        } else {
            document.documentElement.removeAttribute('data-theme');
        }
        this.settings.theme = theme;
    }

    showNotification(message, type = 'info') {
        if (!this.settings.notificationsEnabled && type !== 'success') return;

        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message || this.getSessionCompleteMessage();

        document.body.appendChild(notification);

        // Remove after 3 seconds
        setTimeout(() => {
            notification.remove();
        }, 3000);

        // Browser notification if permitted
        if (this.settings.notificationsEnabled && 'Notification' in window &&
            Notification.permission === 'granted' && !message) {
            new Notification('Roma Timer', {
                body: this.getSessionCompleteMessage(),
                icon: '/favicon.ico'
            });
        }
    }

    getSessionCompleteMessage() {
        const messages = {
            'work': 'Work session complete! Time for a break.',
            'shortBreak': 'Short break over! Ready to focus?',
            'longBreak': 'Long break complete! Ready to be productive?'
        };
        return messages[this.currentSession.type];
    }

    playSound() {
        // Create a simple beep sound using Web Audio API
        const audioContext = new (window.AudioContext || window.webkitAudioContext)();
        const oscillator = audioContext.createOscillator();
        const gainNode = audioContext.createGain();

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.destination);

        oscillator.frequency.value = 800;
        oscillator.type = 'sine';

        gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);

        oscillator.start(audioContext.currentTime);
        oscillator.stop(audioContext.currentTime + 0.5);
    }

    requestNotificationPermission() {
        if ('Notification' in window && Notification.permission === 'default') {
            Notification.requestPermission();
        }
    }
}

// Initialize the timer when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    const timer = new PomodoroTimer();

    // Make timer globally available for debugging
    window.pomodoroTimer = timer;
});

// Service Worker registration for PWA
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/sw.js')
            .then(registration => {
                console.log('SW registered: ', registration);
            })
            .catch(registrationError => {
                console.log('SW registration failed: ', registrationError);
            });
    });
}