//! Daily Reset WebSocket Service
//!
//! WebSocket service for communicating with daily reset backend handlers.
//! Handles all daily reset operations including configuration, session count,
//! and analytics via WebSocket messages.

import { useWebSocket } from '../hooks/useWebSocket';

// Types for daily reset WebSocket messages
export interface ConfigureDailyResetRequest {
  type: 'configure_daily_reset';
  message_id: string;
  user_id: string;
  enabled: boolean;
  reset_time_type: 'midnight' | 'hour' | 'custom';
  reset_hour?: number;
  custom_time?: string;
  timezone: string;
  timestamp: string;
}

export interface GetDailyResetStatusRequest {
  type: 'get_daily_reset_status';
  message_id: string;
  user_id: string;
  timestamp: string;
}

export interface SetSessionCountRequest {
  type: 'set_session_count';
  message_id: string;
  user_id: string;
  session_count: number;
  manual_override: boolean;
  timestamp: string;
}

export interface ResetSessionRequest {
  type: 'reset_session';
  message_id: string;
  user_id: string;
  timestamp: string;
}

export interface GetDailyStatsRequest {
  type: 'get_daily_stats';
  message_id: string;
  user_id: string;
  date?: string;
  days?: number;
  timestamp: string;
}

export interface ConfigureDailyResetResponse {
  type: 'configure_daily_reset_response';
  message_id: string;
  success: boolean;
  configuration?: any;
  next_reset_time_utc?: number;
  error?: string;
  timestamp: string;
}

export interface DailyResetStatusResponse {
  type: 'daily_reset_status_response';
  message_id: string;
  success: boolean;
  configuration?: any;
  next_reset_time_utc?: number;
  reset_due_today?: boolean;
  current_session_count?: number;
  manual_session_override?: number;
  error?: string;
  timestamp: string;
}

export interface SessionCountResponse {
  type: 'session_count_response';
  message_id: string;
  success: boolean;
  current_session_count: number;
  manual_session_override?: number;
  last_reset_utc?: number;
  error?: string;
  timestamp: string;
}

export interface SessionSetResponse {
  type: 'session_set_response';
  message_id: string;
  success: boolean;
  current_session_count: number;
  manual_session_override?: number;
  error?: string;
  timestamp: string;
}

export interface SessionResetResponse {
  type: 'session_reset_response';
  message_id: string;
  success: boolean;
  previous_session_count: number;
  new_session_count: number;
  reset_time_utc: number;
  error?: string;
  timestamp: string;
}

export interface DailyStatsResponse {
  type: 'daily_stats_response';
  message_id: string;
  success: boolean;
  stats: any[];
  error?: string;
  timestamp: string;
}

export interface ErrorResponse {
  type: 'error';
  message_id: string;
  error_code: string;
  error_message: string;
  timestamp: string;
}

// Response union type
export type DailyResetWebSocketResponse =
  | ConfigureDailyResetResponse
  | DailyResetStatusResponse
  | SessionCountResponse
  | SessionSetResponse
  | SessionResetResponse
  | DailyStatsResponse
  | ErrorResponse;

// Configuration types
export interface DailyResetConfig {
  enabled: boolean;
  timezone: string;
  reset_time_type: 'midnight' | 'hour' | 'custom';
  reset_hour?: number;
  custom_time?: string;
}

export interface DailyResetStatus {
  next_reset_time_utc?: number;
  reset_due_today?: boolean;
  current_session_count: number;
  manual_session_override?: number;
  last_reset_utc?: number;
}

class DailyResetWebSocketService {
  private pendingRequests: Map<string, {
    resolve: (response: DailyResetWebSocketResponse) => void;
    reject: (error: Error) => void;
    timeout?: NodeJS.Timeout;
  }> = new Map();
  private messageHandler: ((message: any) => void) | null = null;

  constructor() {
    // Constructor no longer takes WebSocketService as a dependency
  }

  setMessageHandler(handler: (message: any) => void) {
    this.messageHandler = handler;
  }

  private setupMessageHandler(handler: (message: any) => void) {
    this.messageHandler = handler;
  }

  private isDailyResetResponse(message: any): message is DailyResetWebSocketResponse {
    return message.type && [
      'configure_daily_reset_response',
      'daily_reset_status_response',
      'session_count_response',
      'session_set_response',
      'session_reset_response',
      'daily_stats_response',
      'error'
    ].includes(message.type);
  }

  private handleResponse(response: DailyResetWebSocketResponse) {
    const messageId = response.message_id;
    const pendingRequest = this.pendingRequests.get(messageId);

    if (pendingRequest) {
      const { resolve, timeout } = pendingRequest;

      // Clear timeout
      if (timeout) {
        clearTimeout(timeout);
      }

      // Remove from pending requests
      this.pendingRequests.delete(messageId);

      // Resolve promise
      resolve(response);
    }
  }

  private generateMessageId(): string {
    return `daily_reset_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async sendMessage<T>(
    message: T,
    timeoutMs: number = 10000
  ): Promise<DailyResetWebSocketResponse> {
    const messageId = message.message_id || this.generateMessageId();

    return new Promise((resolve, reject) => {
      // Set up timeout
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(messageId);
        reject(new Error('Request timeout'));
      }, timeoutMs);

      // Store pending request
      this.pendingRequests.set(messageId, {
        resolve,
        reject,
        timeout
      });

      // Send message (would be injected from parent)
      try {
        if (this.messageHandler) {
          this.messageHandler(message);
        } else {
          throw new Error('WebSocket message handler not set');
        }
      } catch (error) {
        // Clean up on send error
        this.pendingRequests.delete(messageId);
        clearTimeout(timeout);
        reject(error);
      }
    });
  }

  // Configure daily reset settings
  async configureDailyReset(
    userId: string,
    config: DailyResetConfig
  ): Promise<ConfigureDailyResetResponse> {
    const message: ConfigureDailyResetRequest = {
      type: 'configure_daily_reset',
      message_id: this.generateMessageId(),
      user_id: userId,
      enabled: config.enabled,
      reset_time_type: config.reset_time_type,
      reset_hour: config.reset_hour,
      custom_time: config.custom_time,
      timezone: config.timezone,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'configure_daily_reset_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    return response;
  }

  // Get current daily reset status
  async getDailyResetStatus(
    userId: string
  ): Promise<DailyResetStatusResponse> {
    const message: GetDailyResetStatusRequest = {
      type: 'get_daily_reset_status',
      message_id: this.generateMessageId(),
      user_id: userId,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'daily_reset_status_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    return response;
  }

  // Get current session count
  async getSessionCount(
    userId: string
  ): Promise<SessionCountResponse> {
    const message: GetDailyResetStatusRequest = {
      type: 'get_daily_reset_status',
      message_id: this.generateMessageId(),
      user_id: userId,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'daily_reset_status_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    // Convert status response to session count response
    return {
      type: 'session_count_response',
      message_id: response.message_id,
      success: response.success,
      current_session_count: response.current_session_count || 0,
      manual_session_override: response.manual_session_override,
      last_reset_utc: undefined, // Would need to extract from config
      error: response.error,
      timestamp: response.timestamp
    };
  }

  // Set session count (with optional manual override)
  async setSessionCount(
    userId: string,
    sessionCount: number,
    manualOverride: boolean = false
  ): Promise<SessionSetResponse> {
    const message: SetSessionCountRequest = {
      type: 'set_session_count',
      message_id: this.generateMessageId(),
      user_id: userId,
      session_count: sessionCount,
      manual_override: manualOverride,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'session_set_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    return response;
  }

  // Reset session count to zero
  async resetSession(
    userId: string
  ): Promise<SessionResetResponse> {
    const message: ResetSessionRequest = {
      type: 'reset_session',
      message_id: this.generateMessageId(),
      user_id: userId,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'session_reset_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    return response;
  }

  // Get daily statistics
  async getDailyStats(
    userId: string,
    date?: string,
    days?: number
  ): Promise<DailyStatsResponse> {
    const message: GetDailyStatsRequest = {
      type: 'get_daily_stats',
      message_id: this.generateMessageId(),
      user_id: userId,
      date,
      days,
      timestamp: new Date().toISOString()
    };

    const response = await this.sendMessage(message);

    if (response.type !== 'daily_stats_response') {
      throw new Error(`Unexpected response type: ${response.type}`);
    }

    return response;
  }

  // Clean up pending requests
  cleanup() {
    // Clear all timeouts
    for (const [messageId, pending] of this.pendingRequests) {
      if (pending.timeout) {
        clearTimeout(pending.timeout);
      }
      pending.reject(new Error('Service cleanup'));
    }

    // Clear pending requests
    this.pendingRequests.clear();
  }
}

// Hook for using Daily Reset WebSocket Service
export const useDailyResetWebSocket = () => {
  const [dailyResetService] = React.useState<DailyResetWebSocketService | null>(null);

  React.useEffect(() => {
    const service = new DailyResetWebSocketService();
    setDailyResetService(service);

    return () => {
      service.cleanup();
    };
  }, []);

  return dailyResetService;
}

export default DailyResetWebSocketService;