//! API Service
//!
//! HTTP client for communicating with the Roma Timer backend API.

import { TimerState, UserConfiguration, ApiError } from '../types';

// Base API URL - can be configured via environment variables
const API_BASE_URL = process.env.REACT_APP_API_URL ||
                     process.env.EXPO_PUBLIC_API_URL ||
                     'http://localhost:3001';

class ApiService {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
  }

  // Generic HTTP request method
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const defaultHeaders = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };

    // Add authentication headers if available
    const authHeaders = this.getAuthHeaders();

    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          ...defaultHeaders,
          ...authHeaders,
          ...options.headers,
        },
      });

      // Handle HTTP errors
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        const apiError: ApiError = {
          error: response.statusText,
          message: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
          timestamp: Date.now(),
        };
        throw apiError;
      }

      // Handle empty responses
      if (response.status === 204) {
        return null as T;
      }

      return await response.json();
    } catch (error) {
      // Network or fetch errors
      if (error instanceof TypeError) {
        const networkError: ApiError = {
          error: 'NetworkError',
          message: 'Unable to connect to the server. Please check your connection.',
          timestamp: Date.now(),
        };
        throw networkError;
      }

      // Re-throw API errors
      throw error;
    }
  }

  // Get authentication headers (shared secret authentication)
  private getAuthHeaders(): Record<string, string> {
    const sharedSecret = process.env.REACT_APP_SHARED_SECRET ||
                        process.env.EXPO_PUBLIC_SHARED_SECRET;

    if (sharedSecret) {
      // Simple bearer token authentication
      return {
        'Authorization': `Bearer ${sharedSecret}`,
      };
    }

    return {};
  }

  // Timer API methods
  async getTimer(): Promise<TimerState> {
    return this.request<TimerState>('/api/timer');
  }

  async startTimer(): Promise<TimerState> {
    return this.request<TimerState>('/api/timer/start', {
      method: 'POST',
    });
  }

  async pauseTimer(): Promise<TimerState> {
    return this.request<TimerState>('/api/timer/pause', {
      method: 'POST',
    });
  }

  async resetTimer(): Promise<TimerState> {
    return this.request<TimerState>('/api/timer/reset', {
      method: 'POST',
    });
  }

  async skipTimer(): Promise<TimerState> {
    return this.request<TimerState>('/api/timer/skip', {
      method: 'POST',
    });
  }

  // Configuration API methods
  async getConfiguration(): Promise<UserConfiguration> {
    return this.request<UserConfiguration>('/api/configuration');
  }

  async updateConfiguration(config: Partial<UserConfiguration>): Promise<UserConfiguration> {
    return this.request<UserConfiguration>('/api/configuration', {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  // Health check
  async healthCheck(): Promise<{ status: string; timestamp: number }> {
    return this.request<{ status: string; timestamp: number }>('/api/health');
  }

  // Utility method to test connection
  async testConnection(): Promise<boolean> {
    try {
      await this.healthCheck();
      return true;
    } catch {
      return false;
    }
  }
}

// Export singleton instance
export const apiService = new ApiService();

// Export typed API services for better developer experience
export const timerApi = {
  getTimer: () => apiService.getTimer(),
  startTimer: () => apiService.startTimer(),
  pauseTimer: () => apiService.pauseTimer(),
  resetTimer: () => apiService.resetTimer(),
  skipTimer: () => apiService.skipTimer(),
};

export const configurationApi = {
  getConfiguration: () => apiService.getConfiguration(),
  updateConfiguration: (config: Partial<UserConfiguration>) =>
    apiService.updateConfiguration(config),
};

export const healthApi = {
  check: () => apiService.healthCheck(),
  testConnection: () => apiService.testConnection(),
};

export { ApiService };
export default apiService;