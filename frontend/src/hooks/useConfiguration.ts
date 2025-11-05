//! Configuration Hook
//!
//! Custom hook for managing user configuration state and API calls

import { useState, useEffect, useCallback } from 'react';
import { UserConfiguration, UseConfigurationReturn, ApiError } from '../types';

// API base URL - should match backend server
const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3000';

// Get auth token from localStorage or environment
const getAuthToken = (): string => {
  return localStorage.getItem('roma-timer-auth-token') ||
         process.env.REACT_APP_AUTH_TOKEN ||
         'change-me';
};

/**
 * Custom hook for managing user configuration
 * Provides functionality to load, update, and reset configuration settings
 */
export const useConfiguration = (): UseConfigurationReturn => {
  const [config, setConfig] = useState<UserConfiguration | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [saving, setSaving] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Load configuration from API
  const loadConfiguration = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/configuration`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'X-Auth-Token': getAuthToken(),
        },
      });

      if (!response.ok) {
        const errorData: ApiError = await response.json();
        throw new Error(errorData.message || `HTTP ${response.status}`);
      }

      const configuration: UserConfiguration = await response.json();
      setConfig(configuration);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load configuration';
      setError(errorMessage);
      console.error('Error loading configuration:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Update configuration
  const updateConfig = useCallback(async (updates: Partial<UserConfiguration>): Promise<void> => {
    if (!config) {
      throw new Error('No configuration loaded');
    }

    setSaving(true);
    setError(null);

    try {
      // Create update payload with only provided fields
      const updatePayload = {
        workDuration: updates.workDuration,
        shortBreakDuration: updates.shortBreakDuration,
        longBreakDuration: updates.longBreakDuration,
        longBreakFrequency: updates.longBreakFrequency,
        notificationsEnabled: updates.notificationsEnabled,
        webhookUrl: updates.webhookUrl !== undefined ? updates.webhookUrl : undefined,
        waitForInteraction: updates.waitForInteraction,
        theme: updates.theme,
      };

      const response = await fetch(`${API_BASE_URL}/api/configuration`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Auth-Token': getAuthToken(),
        },
        body: JSON.stringify(updatePayload),
      });

      if (!response.ok) {
        const errorData: ApiError = await response.json();
        throw new Error(errorData.message || `HTTP ${response.status}`);
      }

      const updatedConfig: UserConfiguration = await response.json();
      setConfig(updatedConfig);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update configuration';
      setError(errorMessage);
      console.error('Error updating configuration:', err);
      throw err;
    } finally {
      setSaving(false);
    }
  }, [config]);

  // Reset configuration to defaults
  const resetConfig = useCallback(async (): Promise<void> => {
    setSaving(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/configuration/reset`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Auth-Token': getAuthToken(),
        },
      });

      if (!response.ok) {
        const errorData: ApiError = await response.json();
        throw new Error(errorData.message || `HTTP ${response.status}`);
      }

      const defaultConfig: UserConfiguration = await response.json();
      setConfig(defaultConfig);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to reset configuration';
      setError(errorMessage);
      console.error('Error resetting configuration:', err);
      throw err;
    } finally {
      setSaving(false);
    }
  }, []);

  // Load configuration on mount
  useEffect(() => {
    loadConfiguration();
  }, [loadConfiguration]);

  // Handle configuration updates from WebSocket
  const handleConfigurationUpdate = useCallback((updatedConfig: UserConfiguration) => {
    setConfig(updatedConfig);
  }, []);

  return {
    config,
    loading,
    saving,
    error,
    updateConfig,
    resetConfig,
  };
};

// Utility functions for configuration validation
export const ConfigurationValidators = {
  /**
   * Validate work duration (5-60 minutes)
   */
  validateWorkDuration: (minutes: number): string | null => {
    if (minutes < 5 || minutes > 60) {
      return 'Work duration must be between 5 and 60 minutes';
    }
    return null;
  },

  /**
   * Validate short break duration (1-15 minutes)
   */
  validateShortBreakDuration: (minutes: number): string | null => {
    if (minutes < 1 || minutes > 15) {
      return 'Short break duration must be between 1 and 15 minutes';
    }
    return null;
  },

  /**
   * Validate long break duration (5-30 minutes)
   */
  validateLongBreakDuration: (minutes: number): string | null => {
    if (minutes < 5 || minutes > 30) {
      return 'Long break duration must be between 5 and 30 minutes';
    }
    return null;
  },

  /**
   * Validate long break frequency (2-10 work sessions)
   */
  validateLongBreakFrequency: (frequency: number): string | null => {
    if (frequency < 2 || frequency > 10) {
      return 'Long break frequency must be between 2 and 10 work sessions';
    }
    return null;
  },

  /**
   * Validate webhook URL format
   */
  validateWebhookUrl: (url: string): string | null => {
    if (!url.trim()) {
      return null; // Empty URL is allowed
    }

    try {
      const parsedUrl = new URL(url);
      if (!['http:', 'https:'].includes(parsedUrl.protocol)) {
        return 'Webhook URL must use HTTP or HTTPS protocol';
      }
    } catch {
      return 'Invalid webhook URL format';
    }

    return null;
  },

  /**
   * Validate complete configuration object
   */
  validateConfiguration: (config: Partial<UserConfiguration>): Record<string, string> => {
    const errors: Record<string, string> = {};

    if (config.workDuration !== undefined) {
      const minutes = config.workDuration / 60;
      const error = ConfigurationValidators.validateWorkDuration(minutes);
      if (error) errors.workDuration = error;
    }

    if (config.shortBreakDuration !== undefined) {
      const minutes = config.shortBreakDuration / 60;
      const error = ConfigurationValidators.validateShortBreakDuration(minutes);
      if (error) errors.shortBreakDuration = error;
    }

    if (config.longBreakDuration !== undefined) {
      const minutes = config.longBreakDuration / 60;
      const error = ConfigurationValidators.validateLongBreakDuration(minutes);
      if (error) errors.longBreakDuration = error;
    }

    if (config.longBreakFrequency !== undefined) {
      const error = ConfigurationValidators.validateLongBreakFrequency(config.longBreakFrequency);
      if (error) errors.longBreakFrequency = error;
    }

    if (config.webhookUrl !== undefined) {
      const error = ConfigurationValidators.validateWebhookUrl(config.webhookUrl);
      if (error) errors.webhookUrl = error;
    }

    if (config.theme !== undefined && !['Light', 'Dark'].includes(config.theme)) {
      errors.theme = 'Theme must be either "Light" or "Dark"';
    }

    return errors;
  },
};

// Utility functions for configuration formatting
export const ConfigurationFormatters = {
  /**
   * Convert seconds to minutes for display
   */
  secondsToMinutes: (seconds: number): number => {
    return Math.round(seconds / 60);
  },

  /**
   * Convert minutes to seconds for API
   */
  minutesToSeconds: (minutes: number): number => {
    return minutes * 60;
  },

  /**
   * Format duration display
   */
  formatDuration: (seconds: number): string => {
    const minutes = Math.floor(seconds / 60);
    return `${minutes} minute${minutes !== 1 ? 's' : ''}`;
  },

  /**
   * Get theme display name
   */
  getThemeDisplayName: (theme: 'Light' | 'Dark'): string => {
    return theme === 'Light' ? 'Light Mode' : 'Dark Mode';
  },
};

export default useConfiguration;