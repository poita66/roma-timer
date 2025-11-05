import { useState, useEffect, useCallback, useRef } from 'react';
import { TimerSession, UseTimerReturn, TimerState, ApiError } from '../types';
import { timerApi } from '../services/api';
import { useWebSocket } from './useWebSocket';

const useTimer = (): UseTimerReturn => {
  const [session, setSession] = useState<TimerSession | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [syncStatus, setSyncStatus] = useState<'synced' | 'syncing' | 'conflict' | 'offline'>('offline');

  // Track last update timestamp for conflict resolution
  const lastUpdateRef = useRef<number>(0);
  const localOperationRef = useRef<boolean>(false);
  const mountedRef = useRef<boolean>(true);

  // Use WebSocket hook for real-time communication
  const {
    isConnected: wsConnected,
    connectionStatus,
    lastMessage,
    sendMessage,
    error: wsError,
    forceReconnect,
  } = useWebSocket();

  // Transform backend TimerState to frontend TimerSession
  const transformTimerState = useCallback((timerState: TimerState): TimerSession => {
    return {
      id: timerState.id,
      duration: timerState.duration,
      elapsed: timerState.elapsed,
      timerType: timerState.timer_type as TimerSession['timerType'],
      isRunning: timerState.is_running,
      createdAt: timerState.created_at,
      updatedAt: timerState.updated_at,
    };
  }, []);

  // Update session state with conflict resolution
  const updateSession = useCallback((newSession: TimerSession | null, isLocalUpdate: boolean = false) => {
    if (!mountedRef.current) return;

    if (newSession) {
      // Check for conflicts
      if (!isLocalUpdate && lastUpdateRef.current > 0) {
        const currentTime = Date.now();
        const timeSinceLastUpdate = currentTime - lastUpdateRef.current;

        // If we received an update shortly after a local operation, check for conflicts
        if (timeSinceLastUpdate < 1000 && session) {
          const conflict = detectConflict(session, newSession);
          if (conflict) {
            setSyncStatus('conflict');
            console.warn('Timer state conflict detected:', conflict);
          }
        }
      }

      lastUpdateRef.current = Date.now();
      setSession(newSession);

      if (isLocalUpdate) {
        setSyncStatus('syncing');
      } else if (wsConnected) {
        setSyncStatus('synced');
      }
    } else {
      setSession(newSession);
    }
  }, [session, wsConnected]);

  // Detect conflicts between local and remote state
  const detectConflict = (local: TimerSession, remote: TimerSession): boolean => {
    // Check for basic state inconsistencies
    if (local.id !== remote.id) return true;

    // Check if both think they're in control simultaneously
    if (local.isRunning && remote.isRunning && Math.abs(local.elapsed - remote.elapsed) > 2) {
      return true;
    }

    // Check for type mismatches
    if (local.timerType !== remote.timerType && local.elapsed > 0 && remote.elapsed > 0) {
      return true;
    }

    return false;
  };

  // Resolve conflicts by choosing the most recent state
  const resolveConflict = useCallback((local: TimerSession, remote: TimerSession): TimerSession => {
    // Choose the state with the most recent timestamp
    if (local.updated_at > remote.updated_at) {
      return local;
    } else {
      return remote;
    }
  }, []);

  // Fetch current timer state from API
  const fetchTimerState = useCallback(async () => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.getTimer();
      const transformedSession = transformTimerState(response);

      updateSession(transformedSession, false);
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || 'Failed to fetch timer state');
        console.error('Failed to fetch timer state:', err);
      }
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [transformTimerState, updateSession]);

  // Execute timer operation with WebSocket fallback
  const executeOperation = useCallback(async (
    operation: () => Promise<TimerState>,
    wsMessage?: any
  ): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      // Mark as local operation to prevent conflict warnings
      localOperationRef.current = true;

      // Send WebSocket message first if connected
      if (wsConnected && wsMessage) {
        sendMessage(wsMessage);
      }

      // Execute API call
      const response = await operation();
      const transformedSession = transformTimerState(response);

      updateSession(transformedSession, true);
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || `Operation failed: ${apiError.error}`);
        console.error('Timer operation failed:', err);

        // Reset local operation flag on error
        localOperationRef.current = false;
      }
      throw err;
    } finally {
      if (mountedRef.current) {
        setLoading(false);
        // Reset local operation flag after a short delay
        setTimeout(() => {
          if (mountedRef.current) {
            localOperationRef.current = false;
          }
        }, 500);
      }
    }
  }, [wsConnected, sendMessage, transformTimerState, updateSession]);

  // Start timer
  const startTimer = useCallback(async (): Promise<void> => {
    await executeOperation(
      () => timerApi.startTimer(),
      { type: 'StartTimer' }
    );
  }, [executeOperation]);

  // Pause timer
  const pauseTimer = useCallback(async (): Promise<void> => {
    await executeOperation(
      () => timerApi.pauseTimer(),
      { type: 'PauseTimer' }
    );
  }, [executeOperation]);

  // Reset timer
  const resetTimer = useCallback(async (): Promise<void> => {
    await executeOperation(
      () => timerApi.resetTimer(),
      { type: 'ResetTimer' }
    );
  }, [executeOperation]);

  // Skip timer
  const skipTimer = useCallback(async (): Promise<void> => {
    await executeOperation(
      () => timerApi.skipTimer(),
      { type: 'SkipTimer' }
    );
  }, [executeOperation]);

  // Handle WebSocket messages
  useEffect(() => {
    if (!lastMessage) return;

    switch (lastMessage.type) {
      case 'TimerStateUpdate':
        if (lastMessage.payload) {
          const timerState = lastMessage.payload as TimerState;
          const transformedSession = transformTimerState(timerState);

          // Don't update if this was our own operation
          if (!localOperationRef.current) {
            updateSession(transformedSession, false);
          }
        }
        break;

      case 'ConnectionStatus':
        // Update connection status based on device count
        if (lastMessage.payload?.device_count !== undefined) {
          // Could show multi-device indicators here
        }
        break;

      case 'Notification':
        // Handle timer completion notifications
        if (lastMessage.payload?.event_type) {
          console.log('Timer notification:', lastMessage.payload);
        }
        break;
    }
  }, [lastMessage, transformTimerState, updateSession]);

  // Update connection status
  useEffect(() => {
    setIsConnected(wsConnected);

    if (wsConnected) {
      setError(null);
      if (syncStatus === 'offline') {
        setSyncStatus('synced');
      }
    } else {
      setSyncStatus('offline');
      if (connectionStatus === 'reconnecting') {
        setError('Reconnecting...');
      } else if (connectionStatus === 'disconnected') {
        setError('Connection lost');
      }
    }
  }, [wsConnected, connectionStatus, syncStatus]);

  // Handle WebSocket errors
  useEffect(() => {
    if (wsError) {
      setError(wsError);
    }
  }, [wsError]);

  // Initialize timer and WebSocket connections
  useEffect(() => {
    mountedRef.current = true;

    // Fetch initial timer state
    fetchTimerState();

    // Cleanup
    return () => {
      mountedRef.current = false;
    };
  }, [fetchTimerState]);

  // Periodic state sync when disconnected
  useEffect(() => {
    if (!wsConnected && session?.isRunning) {
      const interval = setInterval(() => {
        if (mountedRef.current && !wsConnected) {
          fetchTimerState();
        }
      }, 5000); // Sync every 5 seconds when WebSocket is disconnected

      return () => clearInterval(interval);
    }
  }, [wsConnected, session?.isRunning, fetchTimerState]);

  // Handle browser online/offline events
  useEffect(() => {
    const handleOnline = () => {
      if (!wsConnected) {
        forceReconnect();
      }
    };

    window.addEventListener('online', handleOnline);
    return () => window.removeEventListener('online', handleOnline);
  }, [wsConnected, forceReconnect]);

  return {
    session,
    startTimer,
    pauseTimer,
    resetTimer,
    skipTimer,
    loading,
    error,
    isConnected,
    syncStatus,
  };
};

export { useTimer };
export default useTimer;