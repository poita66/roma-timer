import { useState, useEffect, useCallback, useRef } from 'react';
import { TimerSession, UseTimerReturn, TimerState, ApiError } from '../types';
import { timerApi } from '../services/api';
import { webSocketService } from '../services/websocket';

const useTimer = (): UseTimerReturn => {
  const [session, setSession] = useState<TimerSession | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);

  // Track last update to prevent stale state
  const lastUpdateRef = useRef<number>(0);
  const mountedRef = useRef<boolean>(true);

  // Transform backend TimerState to frontend TimerSession
  const transformTimerState = (timerState: TimerState): TimerSession => ({
    id: timerState.id,
    duration: timerState.duration,
    elapsed: timerState.elapsed,
    timerType: timerState.timer_type as TimerSession['timerType'],
    isRunning: timerState.is_running,
    createdAt: timerState.created_at,
    updatedAt: timerState.updated_at,
  });

  // Update session state with validation
  const updateSession = useCallback((newSession: TimerSession | null) => {
    if (!mountedRef.current) return;

    // Validate session timestamp to prevent stale updates
    if (newSession && newSession.updated_at <= lastUpdateRef.current) {
      return;
    }

    if (newSession) {
      lastUpdateRef.current = newSession.updated_at;
    }

    setSession(newSession);
  }, []);

  // Handle WebSocket messages
  const handleWebSocketMessage = useCallback((message: any) => {
    if (!mountedRef.current) return;

    try {
      if (message.type === 'TimerStateUpdate' && message.payload) {
        const timerState = message.payload as TimerState;
        const transformedSession = transformTimerState(timerState);
        updateSession(transformedSession);
      }
    } catch (err) {
      console.error('Error processing WebSocket message:', err);
      if (mountedRef.current) {
        setError('Failed to process timer update');
      }
    }
  }, [updateSession]);

  // Handle WebSocket connection status changes
  const handleConnectionStatusChange = useCallback((connected: boolean) => {
    if (!mountedRef.current) return;
    setIsConnected(connected);

    if (!connected && mountedRef.current) {
      setError('Connection lost. Attempting to reconnect...');
    } else if (connected && mountedRef.current) {
      setError(null);
      // Refresh timer state when reconnected
      fetchTimerState();
    }
  }, []);

  // Fetch current timer state from API
  const fetchTimerState = useCallback(async () => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.getTimer();

      if (mountedRef.current) {
        const transformedSession = transformTimerState(response);
        updateSession(transformedSession);
      }
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
  }, [updateSession]);

  // Start timer
  const startTimer = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.startTimer();

      if (mountedRef.current) {
        const transformedSession = transformTimerState(response);
        updateSession(transformedSession);
      }
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || 'Failed to start timer');
        console.error('Failed to start timer:', err);
      }
      throw err;
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [updateSession]);

  // Pause timer
  const pauseTimer = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.pauseTimer();

      if (mountedRef.current) {
        const transformedSession = transformTimerState(response);
        updateSession(transformedSession);
      }
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || 'Failed to pause timer');
        console.error('Failed to pause timer:', err);
      }
      throw err;
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [updateSession]);

  // Reset timer
  const resetTimer = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.resetTimer();

      if (mountedRef.current) {
        const transformedSession = transformTimerState(response);
        updateSession(transformedSession);
      }
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || 'Failed to reset timer');
        console.error('Failed to reset timer:', err);
      }
      throw err;
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [updateSession]);

  // Skip timer
  const skipTimer = useCallback(async (): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      setLoading(true);
      setError(null);

      const response = await timerApi.skipTimer();

      if (mountedRef.current) {
        const transformedSession = transformTimerState(response);
        updateSession(transformedSession);
      }
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err as ApiError;
        setError(apiError.message || 'Failed to skip timer');
        console.error('Failed to skip timer:', err);
      }
      throw err;
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [updateSession]);

  // Initialize timer and WebSocket connections
  useEffect(() => {
    mountedRef.current = true;

    // Initialize WebSocket connection
    webSocketService.connect();
    webSocketService.onMessage(handleWebSocketMessage);
    webSocketService.onConnectionStatusChange(handleConnectionStatusChange);

    // Fetch initial timer state
    fetchTimerState();

    // Cleanup
    return () => {
      mountedRef.current = false;
      webSocketService.disconnect();
    };
  }, [fetchTimerState, handleWebSocketMessage, handleConnectionStatusChange]);

  // Periodic timer state updates (backup for WebSocket)
  useEffect(() => {
    if (!isConnected && session?.isRunning) {
      const interval = setInterval(() => {
        if (mountedRef.current) {
          fetchTimerState();
        }
      }, 5000); // Update every 5 seconds when WebSocket is disconnected

      return () => clearInterval(interval);
    }
  }, [isConnected, session?.isRunning, fetchTimerState]);

  return {
    session,
    startTimer,
    pauseTimer,
    resetTimer,
    skipTimer,
    loading,
    error,
    isConnected,
  };
};

export { useTimer };
export default useTimer;