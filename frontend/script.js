class PomodoroTimer {
    constructor() {
        // Default settings (fallback)
        this.settings = {
            workDuration: 25 * 60,        // 25 minutes in seconds
            shortBreakDuration: 5 * 60,   // 5 minutes in seconds
            longBreakDuration: 15 * 60,   // 15 minutes in seconds
            longBreakFrequency: 4,        // Long break after 4 sessions
            notificationsEnabled: true,
            theme: 'light'
        };

        // Timer state (synchronized with server)
        this.currentSession = {
            type: 'work',                  // 'work', 'shortBreak', 'longBreak'
            duration: this.settings.workDuration,
            remaining: this.settings.workDuration,
            isRunning: false,
            sessionCount: 1,
            totalSessions: 0,
            lastUpdated: null
        };

        // WebSocket connection
        this.ws = null;
        this.reconnectInterval = null;
        this.connectionStatus = 'disconnected'; // 'disconnected', 'connecting', 'connected'
        this.deviceCount = 0;
        this.connectionId = null;

        // API base URL
        this.apiBaseUrl = window.location.origin + '/api';

        // Authentication state
        this.currentUser = null;
        this.authToken = null;

        // Initialize
        this.init();
    }

    async init() {
        this.loadSettings();
        this.setupAuthEventListeners();
        this.setupEventListeners();
        this.requestNotificationPermission();

        // Check if user is already logged in
        await this.checkAuthStatus();

        // Fetch initial state from backend
        await this.fetchInitialState();

        // Initialize WebSocket
        this.initWebSocket();
    }

    async fetchInitialState() {
        // Only fetch if user is authenticated
        if (!this.currentUser || !this.authToken) {
            console.log('User not authenticated, skipping initial state fetch');
            return;
        }

        try {
            console.log('Fetching initial state from backend...');
            const response = await fetch(`${this.apiBaseUrl}/timer`, {
                headers: {
                    'Authorization': `Bearer ${this.authToken}`
                }
            });
            if (response.ok) {
                const serverState = await response.json();
                this.updateTimerFromServer(serverState);
                console.log('Initial state loaded from backend:', serverState);
            } else if (response.status === 401) {
                console.log('Token expired or invalid, showing login modal');
                this.showAuthModal();
            } else {
                console.error('Failed to fetch initial state:', response.status);
                this.showNotification('Failed to connect to server', 'error');
            }
        } catch (error) {
            console.error('Error fetching initial state:', error);
            this.showNotification('Failed to connect to server', 'error');
        }
    }

    initWebSocket() {
        // Only connect WebSocket if user is authenticated
        if (!this.currentUser || !this.authToken) {
            console.log('User not authenticated, skipping WebSocket connection');
            return;
        }

        // Determine WebSocket URL based on current location
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws?token=${encodeURIComponent(this.authToken)}`;

        console.log('Connecting to WebSocket:', wsUrl);
        this.connectionStatus = 'connecting';
        this.updateConnectionDisplay();

        try {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log('WebSocket connected');
                this.connectionStatus = 'connected';
                this.updateConnectionDisplay();

                // Clear any reconnect interval
                if (this.reconnectInterval) {
                    clearInterval(this.reconnectInterval);
                    this.reconnectInterval = null;
                }

                // Start periodic ping to keep connection alive
                this.startPingInterval();

                // Show connection success notification
                this.showNotification('Connected to server! Timer synchronized.', 'success');
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleWebSocketMessage(message);
                } catch (error) {
                    console.error('Error parsing WebSocket message:', error);
                }
            };

            this.ws.onclose = (event) => {
                console.log('WebSocket disconnected:', event.code, event.reason);
                this.connectionStatus = 'disconnected';
                this.updateConnectionDisplay();

                // Stop ping interval
                this.stopPingInterval();

                // Attempt to reconnect after 3 seconds
                if (!this.reconnectInterval) {
                    this.reconnectInterval = setInterval(() => {
                        if (this.connectionStatus === 'disconnected') {
                            console.log('Attempting to reconnect...');
                            this.initWebSocket();
                        }
                    }, 3000);
                }

                this.showNotification('Lost connection to server. Trying to reconnect...', 'warning');
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.connectionStatus = 'disconnected';
                this.updateConnectionDisplay();
            };

        } catch (error) {
            console.error('Error creating WebSocket connection:', error);
            this.connectionStatus = 'disconnected';
            this.updateConnectionDisplay();
        }
    }

    startPingInterval() {
        this.pingInterval = setInterval(() => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.sendWebSocketMessage({ type: 'Ping' });
            }
        }, 30000); // Ping every 30 seconds
    }

    stopPingInterval() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
    }

    sendWebSocketMessage(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
            return true;
        }
        return false;
    }

    async sendApiRequest(action, data = {}) {
        try {
            const headers = {
                'Content-Type': 'application/json',
            };

            // Add auth token if available
            if (this.authToken) {
                headers['Authorization'] = `Bearer ${this.authToken}`;
            }

            const response = await fetch(`${this.apiBaseUrl}/timer`, {
                method: 'POST',
                headers,
                body: JSON.stringify({ action, ...data })
            });

            if (response.ok) {
                const serverState = await response.json();
                this.updateTimerFromServer(serverState);
                return true;
            } else if (response.status === 401) {
                // Token expired or invalid, show login modal
                this.showAuthModal();
                return false;
            } else {
                console.error('API request failed:', response.status);
                return false;
            }
        } catch (error) {
            console.error('Error sending API request:', error);
            return false;
        }
    }

    // Authentication methods
    setupAuthEventListeners() {
        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const targetTab = e.target.dataset.tab;
                this.switchAuthTab(targetTab);
            });
        });

        // Login form
        document.getElementById('loginForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.login();
        });

        // Register form
        document.getElementById('registerForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.register();
        });

        // Logout button
        document.getElementById('logoutBtn').addEventListener('click', () => {
            this.logout();
        });
    }

    switchAuthTab(tabName) {
        // Update tab buttons
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

        // Update tab content
        document.querySelectorAll('.auth-tab').forEach(tab => {
            tab.classList.remove('active');
        });
        document.getElementById(`${tabName}Tab`).classList.add('active');

        // Clear any error messages
        this.clearAuthError();
    }

    async login() {
        const username = document.getElementById('loginUsername').value;
        const password = document.getElementById('loginPassword').value;

        if (!username || !password) {
            this.showAuthError('Please fill in all fields');
            return;
        }

        try {
            const response = await fetch(`${this.apiBaseUrl}/auth/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password })
            });

            if (response.ok) {
                const data = await response.json();
                this.setCurrentUser(data);
                this.hideAuthModal();
                this.showNotification('Login successful!', 'success');

                // Fetch data and initialize WebSocket after successful login
                await this.fetchInitialState();
                this.initWebSocket();
            } else if (response.status === 401) {
                this.showAuthError('Invalid username or password');
            } else {
                this.showAuthError('Login failed. Please try again.');
            }
        } catch (error) {
            console.error('Login error:', error);
            this.showAuthError('Network error. Please try again.');
        }
    }

    async register() {
        const username = document.getElementById('registerUsername').value;
        const password = document.getElementById('registerPassword').value;
        const confirmPassword = document.getElementById('confirmPassword').value;

        if (!username || !password || !confirmPassword) {
            this.showAuthError('Please fill in all fields');
            return;
        }

        if (username.length < 3) {
            this.showAuthError('Username must be at least 3 characters');
            return;
        }

        if (password.length < 6) {
            this.showAuthError('Password must be at least 6 characters');
            return;
        }

        if (password !== confirmPassword) {
            this.showAuthError('Passwords do not match');
            return;
        }

        try {
            const response = await fetch(`${this.apiBaseUrl}/auth/register`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password })
            });

            if (response.ok) {
                const data = await response.json();
                this.showAuthError('Registration successful! Please login.', 'success');

                // Switch to login tab
                setTimeout(() => {
                    this.switchAuthTab('login');
                    // Clear form
                    document.getElementById('registerForm').reset();
                }, 1500);
            } else if (response.status === 409) {
                this.showAuthError('Username already exists');
            } else {
                this.showAuthError('Registration failed. Please try again.');
            }
        } catch (error) {
            console.error('Registration error:', error);
            this.showAuthError('Network error. Please try again.');
        }
    }

    logout() {
        this.clearCurrentUser();
        this.showAuthModal();
        this.showNotification('Logged out successfully', 'success');
    }

    async checkAuthStatus() {
        const savedToken = localStorage.getItem('romaTimerToken');
        const savedUser = localStorage.getItem('romaTimerUser');

        if (savedToken && savedUser) {
            try {
                // Verify token by making a simple API call
                const response = await fetch(`${this.apiBaseUrl}/health`, {
                    headers: {
                        'Authorization': `Bearer ${savedToken}`
                    }
                });

                if (response.ok) {
                    this.currentUser = JSON.parse(savedUser);
                    this.authToken = savedToken;
                    this.updateUserDisplay();
                } else {
                    // Token is invalid, clear saved data
                    this.clearCurrentUser();
                }
            } catch (error) {
                console.error('Auth check error:', error);
                this.clearCurrentUser();
            }
        }

        // Show auth modal if not logged in
        if (!this.currentUser) {
            this.showAuthModal();
        }
    }

    setCurrentUser(authResponse) {
        this.currentUser = {
            id: authResponse.user_id,
            username: authResponse.username
        };
        this.authToken = authResponse.token;

        // Save to localStorage
        localStorage.setItem('romaTimerToken', authResponse.token);
        localStorage.setItem('romaTimerUser', JSON.stringify(this.currentUser));

        this.updateUserDisplay();
    }

    clearCurrentUser() {
        this.currentUser = null;
        this.authToken = null;

        // Remove from localStorage
        localStorage.removeItem('romaTimerToken');
        localStorage.removeItem('romaTimerUser');

        this.updateUserDisplay();
    }

    updateUserDisplay() {
        const userDisplay = document.getElementById('userDisplay');
        const logoutBtn = document.getElementById('logoutBtn');

        if (this.currentUser) {
            userDisplay.textContent = `👤 ${this.currentUser.username}`;
            logoutBtn.style.display = 'block';
        } else {
            userDisplay.textContent = '';
            logoutBtn.style.display = 'none';
        }
    }

    showAuthModal() {
        document.getElementById('authModal').classList.add('show');
        this.clearAuthError();
    }

    hideAuthModal() {
        document.getElementById('authModal').classList.remove('show');
        this.clearAuthError();
    }

    showAuthError(message, type = 'error') {
        const errorElement = document.getElementById('authError');
        errorElement.textContent = message;
        errorElement.className = type === 'success' ? 'success-message' : 'error-message';
    }

    clearAuthError() {
        const errorElement = document.getElementById('authError');
        errorElement.textContent = '';
        errorElement.className = '';
    }

    handleWebSocketMessage(message) {
        switch (message.type) {
            case 'TimerStateUpdate':
                this.updateTimerFromServer(message.data);
                break;
            case 'ConnectionStatus':
                this.updateConnectionStatus(message.data);
                break;
            case 'Pong':
                // Ping received, connection is alive
                break;
            default:
                console.log('Unknown WebSocket message type:', message.type);
        }
    }

    updateTimerFromServer(serverState) {
        const wasRunning = this.currentSession.isRunning;
        const oldType = this.currentSession.type;
        const oldRemaining = this.currentSession.remaining;

        // Update local state with server state
        this.currentSession = {
            type: serverState.session_type,
            duration: serverState.work_duration, // Will be adjusted based on session type
            remaining: serverState.remaining_seconds,
            isRunning: serverState.is_running,
            sessionCount: serverState.session_count,
            totalSessions: this.currentSession.totalSessions,
            lastUpdated: serverState.last_updated
        };

        // Adjust duration based on session type
        if (this.currentSession.type === 'work') {
            this.currentSession.duration = serverState.work_duration;
        } else if (this.currentSession.type === 'shortBreak') {
            this.currentSession.duration = serverState.short_break_duration;
        } else if (this.currentSession.type === 'longBreak') {
            this.currentSession.duration = serverState.long_break_duration;
        }

        // Update settings if they changed
        this.settings.workDuration = serverState.work_duration;
        this.settings.shortBreakDuration = serverState.short_break_duration;
        this.settings.longBreakDuration = serverState.long_break_duration;

        // Handle session completion notification
        if (wasRunning && !this.currentSession.isRunning && this.currentSession.remaining === 0) {
            this.showNotification();
            this.playSound();
        }

        // Handle session type change
        if (oldType !== this.currentSession.type && oldRemaining !== 0) {
            this.showNotification(this.getSessionCompleteMessage(oldType), 'success');
        }

        this.updateDisplay();
        this.updateButtons();
    }

    updateConnectionStatus(status) {
        this.deviceCount = status.device_count;
        this.connectionId = status.connection_id;
        this.updateConnectionDisplay();
    }

    updateConnectionDisplay() {
        const statusElement = document.getElementById('connectionStatus');
        const deviceCountElement = document.getElementById('deviceCount');

        if (statusElement) {
            const statusText = {
                'disconnected': '🔴 Offline',
                'connecting': '🟡 Connecting...',
                'connected': '🟢 Online'
            };

            statusElement.textContent = statusText[this.connectionStatus] || statusText['disconnected'];
            statusElement.className = `connection-status ${this.connectionStatus}`;
        }

        if (deviceCountElement) {
            if (this.deviceCount > 1) {
                deviceCountElement.textContent = `${this.deviceCount} devices connected`;
                deviceCountElement.style.display = 'block';
            } else {
                deviceCountElement.style.display = 'none';
            }
        }
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

    async start() {
        if (this.currentSession.isRunning) return;

        // Try WebSocket first, fall back to API
        if (this.sendWebSocketMessage({ type: 'TimerControl', data: { action: 'start' } })) {
            console.log('Start command sent via WebSocket');
            return;
        }

        // Fallback to API if WebSocket is not available
        console.log('WebSocket not available, using API');
        await this.sendApiRequest('start');
    }

    async pause() {
        if (!this.currentSession.isRunning) return;

        // Try WebSocket first, fall back to API
        if (this.sendWebSocketMessage({ type: 'TimerControl', data: { action: 'pause' } })) {
            console.log('Pause command sent via WebSocket');
            return;
        }

        // Fallback to API if WebSocket is not available
        console.log('WebSocket not available, using API');
        await this.sendApiRequest('pause');
    }

    async reset() {
        // Try WebSocket first, fall back to API
        if (this.sendWebSocketMessage({ type: 'TimerControl', data: { action: 'reset' } })) {
            console.log('Reset command sent via WebSocket');
            return;
        }

        // Fallback to API if WebSocket is not available
        console.log('WebSocket not available, using API');
        await this.sendApiRequest('reset');
    }

    async skip() {
        // Try WebSocket first, fall back to API
        if (this.sendWebSocketMessage({ type: 'TimerControl', data: { action: 'skip' } })) {
            console.log('Skip command sent via WebSocket');
            return;
        }

        // Fallback to API if WebSocket is not available
        console.log('WebSocket not available, using API');
        await this.sendApiRequest('skip');
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

    async saveSettings() {
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

        // Send settings update via WebSocket if connected
        const settingsData = {
            work_duration: this.settings.workDuration,
            short_break_duration: this.settings.shortBreakDuration,
            long_break_duration: this.settings.longBreakDuration,
            long_break_frequency: this.settings.longBreakFrequency
        };

        if (this.sendWebSocketMessage({ type: 'SettingsUpdate', data: settingsData })) {
            console.log('Settings update sent via WebSocket');
            this.showNotification('Settings synced across devices!', 'success');
            return;
        }

        // Fallback to API if WebSocket is not available
        console.log('WebSocket not available, sending settings via API');
        try {
            const headers = {
                'Content-Type': 'application/json',
            };

            // Add auth token if available
            if (this.authToken) {
                headers['Authorization'] = `Bearer ${this.authToken}`;
            }

            const response = await fetch(`${this.apiBaseUrl}/settings`, {
                method: 'POST',
                headers,
                body: JSON.stringify(settingsData)
            });

            if (response.ok) {
                this.showNotification('Settings saved successfully!', 'success');
            } else if (response.status === 401) {
                // Token expired or invalid, show login modal
                this.showAuthModal();
                this.showNotification('Please login to save settings', 'error');
            } else {
                this.showNotification('Failed to save settings', 'error');
            }
        } catch (error) {
            console.error('Error saving settings:', error);
            this.showNotification('Failed to save settings', 'error');
        }
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

    getSessionCompleteMessage(fromType = null) {
        const sessionType = fromType || this.currentSession.type;
        const messages = {
            'work': 'Work session complete! Time for a break.',
            'shortBreak': 'Short break over! Ready to focus?',
            'longBreak': 'Long break complete! Ready to be productive?'
        };
        return messages[sessionType] || messages['work'];
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

// Add connection status display to the HTML
document.addEventListener('DOMContentLoaded', () => {
    // Add connection status element to the header
    const header = document.querySelector('header');
    const connectionStatus = document.createElement('div');
    connectionStatus.id = 'connectionStatus';
    connectionStatus.className = 'connection-status';
    connectionStatus.textContent = '🔴 Offline';
    header.appendChild(connectionStatus);

    const deviceCount = document.createElement('div');
    deviceCount.id = 'deviceCount';
    deviceCount.className = 'device-count';
    deviceCount.style.display = 'none';
    header.appendChild(deviceCount);

    // Initialize the timer
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