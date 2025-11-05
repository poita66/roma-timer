//! Roma Timer Type Definitions
//!
//! TypeScript interfaces for timer sessions and API responses

export interface TimerSession {
  id: string;
  duration: number;           // Total duration in seconds
  elapsed: number;            // Elapsed time in seconds
  timerType: 'Work' | 'ShortBreak' | 'LongBreak';
  isRunning: boolean;
  createdAt: number;          // Unix timestamp
  updatedAt: number;          // Unix timestamp
}

export interface TimerState {
  id: string;
  duration: number;
  elapsed: number;
  timer_type: string;
  is_running: boolean;
  created_at: number;
  updated_at: number;
  remaining_seconds: number;
  progress_percentage: number;
  session_count: number;
}

export interface UserConfiguration {
  id: string;
  workDuration: number;           // seconds
  shortBreakDuration: number;     // seconds
  longBreakDuration: number;      // seconds
  longBreakFrequency: number;     // number of work sessions
  notificationsEnabled: boolean;
  webhookUrl?: string;
  waitForInteraction: boolean;
  theme: 'Light' | 'Dark';
  createdAt: number;
  updatedAt: number;
}

export interface TimerDisplayProps {
  session: TimerSession | null;
  theme?: 'light' | 'dark';
  className?: string;
}

export interface TimerControlsProps {
  session: TimerSession | null;
  onStart: () => void;
  onPause: () => void;
  onReset: () => void;
  onSkip: () => void;
  loading?: boolean;
  theme?: 'light' | 'dark';
  disabled?: boolean;
}

export interface ApiResponse<T> {
  data?: T;
  error?: string;
  message?: string;
  timestamp: number;
}

export interface WebSocketMessage {
  type: 'TimerStateUpdate' | 'Notification' | 'ConfigurationUpdate' | 'ConnectionStatus';
  payload?: any;
}

export interface ClientWebSocketMessage {
  type: 'StartTimer' | 'PauseTimer' | 'ResetTimer' | 'SkipTimer' | 'UpdateConfiguration';
  payload?: any;
}

export interface DeviceConnection {
  id: string;
  userAgent: string;
  connectedAt: number;
  lastPing: number;
}

export interface NotificationEvent {
  id: string;
  timerSessionId: string;
  eventType: 'WorkSessionComplete' | 'BreakSessionComplete' | 'TimerSkipped' | 'TimerReset';
  message: string;
  createdAt: number;
  deliveredAt?: number;
}

// Timer display format utilities
export interface TimerDisplayFormat {
  minutes: string;
  seconds: string;
  display: string;    // MM:SS format
  progress: number;   // 0-100 percentage
  remaining: number;  // seconds remaining
}

// Theme colors and configuration
export interface ThemeColors {
  primary: string;
  secondary: string;
  background: string;
  surface: string;
  text: string;
  accent: string;
}

export interface Theme {
  name: string;
  colors: {
    light: ThemeColors;
    dark: ThemeColors;
  };
}

// API error types
export interface ApiError {
  error: string;
  message: string;
  timestamp: number;
}

// Component state interfaces
export interface TimerScreenState {
  session: TimerSession | null;
  connectionStatus: 'connected' | 'disconnected' | 'reconnecting';
  error: string | null;
  loading: boolean;
}

export interface SettingsScreenState {
  config: UserConfiguration | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
}

// Hook return types
export interface UseTimerReturn {
  session: TimerSession | null;
  startTimer: () => Promise<void>;
  pauseTimer: () => Promise<void>;
  resetTimer: () => Promise<void>;
  skipTimer: () => Promise<void>;
  loading: boolean;
  error: string | null;
  isConnected: boolean;
}

export interface UseWebSocketReturn {
  isConnected: boolean;
  connectionStatus: 'connected' | 'disconnected' | 'reconnecting';
  lastMessage: WebSocketMessage | null;
  sendMessage: (message: ClientWebSocketMessage) => void;
  error: string | null;
}

export interface UseConfigurationReturn {
  config: UserConfiguration | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  updateConfig: (config: Partial<UserConfiguration>) => Promise<void>;
  resetConfig: () => Promise<void>;
}

// Utility function types
export type TimerSessionType = TimerSession['timerType'];
export type ThemeMode = 'light' | 'dark';
export type ConnectionStatus = 'connected' | 'disconnected' | 'reconnecting';

// Export commonly used enums
export enum TimerType {
  Work = 'Work',
  ShortBreak = 'ShortBreak',
  LongBreak = 'LongBreak',
}

export enum NotificationType {
  WorkSessionComplete = 'WorkSessionComplete',
  BreakSessionComplete = 'BreakSessionComplete',
  TimerSkipped = 'TimerSkipped',
  TimerReset = 'TimerReset',
}

export enum WebSocketMessageType {
  TimerStateUpdate = 'TimerStateUpdate',
  Notification = 'Notification',
  ConfigurationUpdate = 'ConfigurationUpdate',
  ConnectionStatus = 'ConnectionStatus',
  StartTimer = 'StartTimer',
  PauseTimer = 'PauseTimer',
  ResetTimer = 'ResetTimer',
  SkipTimer = 'SkipTimer',
  UpdateConfiguration = 'UpdateConfiguration',
}