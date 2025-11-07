//! useDailyReset Hook
//!
//! React hook for managing daily reset functionality including configuration,
//! session count management, and status updates.

import { useState, useEffect, useCallback, useRef } from 'react';
import { Alert } from 'react-native';
import {
  DailyResetWebSocketService,
  useDailyResetWebSocket as useDailyResetWebSocketService,
  DailyResetConfig,
  DailyResetStatus,
  ConfigureDailyResetResponse,
  DailyResetStatusResponse,
  SessionCountResponse,
  SessionSetResponse,
  SessionResetResponse,
  ErrorResponse
} from '../services/dailyResetWebSocket';

// Hook options
interface UseDailyResetOptions {
  userId: string;
  autoRefresh?: boolean;
  refreshInterval?: number; // in milliseconds
}

// Hook return type
interface UseDailyResetReturn {
  // Configuration
  config: DailyResetConfig;
  setConfig: (config: DailyResetConfig) => Promise<void>;
  updateConfig: (updates: Partial<DailyResetConfig>) => Promise<void>;
  isConfiguring: boolean;
  configError: string | null;

  // Status
  status: DailyResetStatus | null;
  refreshStatus: () => Promise<void>;
  isRefreshing: boolean;
  statusError: string | null;

  // Session Count Management
  currentSessionCount: number;
  manualOverride: number | null;
  setSessionCount: (count: number, manual?: boolean) => Promise<void>;
  resetSession: () => Promise<void>;
  isUpdatingSession: boolean;
  sessionError: string | null;

  // Utilities
  isResetDueToday: boolean;
  nextResetTime: Date | null;
  formatNextResetTime: () => string;
  daysUntilNextReset: number;

  // Actions
  enableDailyReset: () => Promise<void>;
  disableDailyReset: () => Promise<void>;
}

// Default configuration
const DEFAULT_CONFIG: DailyResetConfig = {
  enabled: false,
  timezone: 'UTC',
  reset_time_type: 'midnight',
  reset_hour: 0,
  custom_time: '00:00'
};

export const useDailyReset = ({
  userId,
  autoRefresh = true,
  refreshInterval = 60000 // 1 minute
}: UseDailyResetOptions): UseDailyResetReturn => {
  const dailyResetService = useDailyResetWebSocketService();

  // State
  const [config, setConfigState] = useState<DailyResetConfig>(DEFAULT_CONFIG);
  const [status, setStatus] = useState<DailyResetStatus | null>(null);
  const [isConfiguring, setIsConfiguring] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isUpdatingSession, setIsUpdatingSession] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [sessionError, setSessionError] = useState<string | null>(null);

  // Refs for cleanup
  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Set up auto-refresh
  useEffect(() => {
    if (autoRefresh && dailyResetService) {
      refreshStatus();

      // Set up interval for auto-refresh
      if (refreshInterval > 0) {
        refreshIntervalRef.current = setInterval(() => {
          refreshStatus();
        }, refreshInterval);
      }

      return () => {
        if (refreshIntervalRef.current) {
          clearInterval(refreshIntervalRef.current);
        }
      };
    }
  }, [autoRefresh, refreshInterval, dailyResetService]);

  // Refresh status
  const refreshStatus = useCallback(async () => {
    if (!dailyResetService || isRefreshing) return;

    setIsRefreshing(true);
    setStatusError(null);

    try {
      const response = await dailyResetService.getDailyResetStatus(userId);

      if (!response.success) {
        throw new Error(response.error || 'Failed to get daily reset status');
      }

      // Update status if configuration is available
      if (response.configuration) {
        setConfigState({
          enabled: response.configuration.daily_reset_enabled || false,
          timezone: response.configuration.timezone || 'UTC',
          reset_time_type: response.configuration.daily_reset_time_type || 'midnight',
          reset_hour: response.configuration.daily_reset_time_hour,
          custom_time: response.configuration.daily_reset_time_custom
        });
      }

      // Update status
      setStatus({
        next_reset_time_utc: response.next_reset_time_utc,
        reset_due_today: response.reset_due_today,
        current_session_count: response.current_session_count || 0,
        manual_session_override: response.manual_session_override,
        last_reset_utc: response.last_reset_utc
      });

    } catch (error) {
      console.error('Failed to refresh daily reset status:', error);
      setStatusError(error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setIsRefreshing(false);
    }
  }, [dailyResetService, userId, isRefreshing]);

  // Set configuration
  const setConfig = useCallback(async (newConfig: DailyResetConfig) => {
    if (!dailyResetService || isConfiguring) return;

    setIsConfiguring(true);
    setConfigError(null);

    try {
      const response = await dailyResetService.configureDailyReset(userId, newConfig);

      if (!response.success) {
        throw new Error(response.error || 'Failed to configure daily reset');
      }

      // Update local state
      setConfigState(newConfig);

      // Refresh status to get updated next reset time
      await refreshStatus();

    } catch (error) {
      console.error('Failed to configure daily reset:', error);
      setConfigError(error instanceof Error ? error.message : 'Unknown error');

      // Revert to previous configuration
      await refreshStatus();
    } finally {
      setIsConfiguring(false);
    }
  }, [dailyResetService, userId, isConfiguring, refreshStatus]);

  // Update configuration partially
  const updateConfig = useCallback(async (updates: Partial<DailyResetConfig>) => {
    const newConfig = { ...config, ...updates };
    await setConfig(newConfig);
  }, [config, setConfig]);

  // Set session count
  const setSessionCount = useCallback(async (
    count: number,
    manualOverride: boolean = false
  ) => {
    if (!dailyResetService || isUpdatingSession) return;

    // Validate input
    if (count < 0 || count > 100) {
      const error = 'Session count must be between 0 and 100';
      setSessionError(error);
      Alert.alert('Invalid Input', error);
      return;
    }

    setIsUpdatingSession(true);
    setSessionError(null);

    try {
      const response = await dailyResetService.setSessionCount(userId, count, manualOverride);

      if (!response.success) {
        throw new Error(response.error || 'Failed to set session count');
      }

      // Update local status
      if (status) {
        setStatus({
          ...status,
          current_session_count: response.current_session_count,
          manual_session_override: response.manual_session_override
        });
      }

      // Show success feedback for manual overrides
      if (manualOverride) {
        Alert.alert(
          'Session Count Updated',
          `Session count set to ${count} (manual override)`
        );
      }

    } catch (error) {
      console.error('Failed to set session count:', error);
      setSessionError(error instanceof Error ? error.message : 'Unknown error');
      Alert.alert('Error', 'Failed to update session count');
    } finally {
      setIsUpdatingSession(false);
    }
  }, [dailyResetService, userId, status, isUpdatingSession]);

  // Reset session
  const resetSession = useCallback(async () => {
    if (!dailyResetService || isUpdatingSession) return;

    setIsUpdatingSession(true);
    setSessionError(null);

    try {
      const response = await dailyResetService.resetSession(userId);

      if (!response.success) {
        throw new Error(response.error || 'Failed to reset session');
      }

      // Update local status
      if (status) {
        setStatus({
          ...status,
          current_session_count: response.new_session_count,
          manual_session_override: undefined
        });
      }

      Alert.alert(
        'Session Reset',
        `Session count reset from ${response.previous_session_count} to 0`
      );

    } catch (error) {
      console.error('Failed to reset session:', error);
      setSessionError(error instanceof Error ? error.message : 'Unknown error');
      Alert.alert('Error', 'Failed to reset session');
    } finally {
      setIsUpdatingSession(false);
    }
  }, [dailyResetService, userId, status, isUpdatingSession]);

  // Enable daily reset
  const enableDailyReset = useCallback(async () => {
    await updateConfig({ enabled: true });
  }, [updateConfig]);

  // Disable daily reset
  const disableDailyReset = useCallback(async () => {
    await updateConfig({ enabled: false });
  }, [updateConfig]);

  // Utility functions
  const isResetDueToday = Boolean(status?.reset_due_today);

  const nextResetTime = status?.next_reset_time_utc
    ? new Date(status.next_reset_time_utc * 1000)
    : null;

  const formatNextResetTime = useCallback(() => {
    if (!nextResetTime) return 'Not scheduled';

    return nextResetTime.toLocaleString('en-US', {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }, [nextResetTime]);

  const daysUntilNextReset = useCallback(() => {
    if (!nextResetTime) return -1;

    const now = new Date();
    const diffTime = nextResetTime.getTime() - now.getTime();
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }, [nextResetTime]);

  // Current session count (prefer manual override)
  const currentSessionCount = status?.manual_session_override ?? status?.current_session_count ?? 0;

  // Manual override status
  const manualOverride = status?.manual_session_override ?? null;

  return {
    // Configuration
    config,
    setConfig,
    updateConfig,
    isConfiguring,
    configError,

    // Status
    status,
    refreshStatus,
    isRefreshing,
    statusError,

    // Session Count Management
    currentSessionCount,
    manualOverride,
    setSessionCount,
    resetSession,
    isUpdatingSession,
    sessionError,

    // Utilities
    isResetDueToday,
    nextResetTime,
    formatNextResetTime,
    daysUntilNextReset,

    // Actions
    enableDailyReset,
    disableDailyReset
  };
};

// Export types for external use
export type {
  DailyResetConfig,
  DailyResetStatus,
  UseDailyResetOptions,
  UseDailyResetReturn
};

export default useDailyReset;