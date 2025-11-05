import { useState, useCallback, useEffect, useRef } from 'react';
import { webSocketService } from '../services/websocket';
import { UseWebSocketReturn, WebSocketMessage, ClientWebSocketMessage, ConnectionStatus } from '../types';

const useWebSocket = (): UseWebSocketReturn => {
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('disconnected');
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [deviceCount, setDeviceCount] = useState<number>(0);
  const [reconnectAttempts, setReconnectAttempts] = useState<number>(0);

  const mountedRef = useRef<boolean>(true);

  // Handle WebSocket messages
  const handleMessage = useCallback((message: WebSocketMessage) => {
    if (!mountedRef.current) return;

    setLastMessage(message);
    setError(null);

    // Handle specific message types
    switch (message.type) {
      case 'ConnectionStatus':
        if (message.payload?.device_count !== undefined) {
          setDeviceCount(message.payload.device_count);
        }
        break;

      case 'Notification':
        // Handle notifications - could trigger local notifications
        if (message.payload?.message && 'Notification' in window) {
          // Request permission if not granted
          if (Notification.permission === 'default') {
            Notification.requestPermission().then(permission => {
              if (permission === 'granted') {
                new Notification('Roma Timer', {
                  body: message.payload.message,
                  icon: '/icon-192x192.png',
                  tag: 'roma-timer',
                });
              }
            });
          } else if (Notification.permission === 'granted') {
            new Notification('Roma Timer', {
              body: message.payload.message,
              icon: '/icon-192x192.png',
              tag: 'roma-timer',
            });
          }
        }
        break;

      case 'TimerStateUpdate':
        // Timer state updates will be handled by useTimer hook
        break;

      case 'ConfigurationUpdate':
        // Configuration updates could trigger re-renders
        break;

      default:
        console.log('Unknown WebSocket message type:', message.type);
    }
  }, []);

  // Handle connection status changes
  const handleConnectionStatusChange = useCallback((connected: boolean) => {
    if (!mountedRef.current) return;

    setIsConnected(connected);
    setConnectionStatus(connected ? 'connected' : 'disconnected');

    if (connected) {
      setError(null);
      setReconnectAttempts(0);
    } else {
      // When disconnected, check if we're reconnecting
      const stats = webSocketService.getConnectionStats();
      if (stats.reconnectAttempts > 0) {
        setConnectionStatus('reconnecting');
        setReconnectAttempts(stats.reconnectAttempts);
      }
    }
  }, []);

  // Handle errors
  const handleError = useCallback((wsError: Error) => {
    if (!mountedRef.current) return;

    console.error('WebSocket error:', wsError);
    setError(wsError.message);
  }, []);

  // Send message function
  const sendMessage = useCallback((message: ClientWebSocketMessage) => {
    try {
      webSocketService.sendMessage(message);
    } catch (error) {
      console.error('Failed to send WebSocket message:', error);
      setError((error as Error).message);
    }
  }, []);

  // Force reconnection
  const forceReconnect = useCallback(() => {
    try {
      setError(null);
      webSocketService.forceReconnect();
    } catch (error) {
      console.error('Failed to force reconnection:', error);
      setError((error as Error).message);
    }
  }, []);

  // Get connection statistics
  const getConnectionStats = useCallback(() => {
    return webSocketService.getConnectionStats();
  }, []);

  // Setup WebSocket event listeners
  useEffect(() => {
    // Register event listeners
    webSocketService.onMessage(handleMessage);
    webSocketService.onConnectionStatusChange(handleConnectionStatusChange);
    webSocketService.onError(handleError);

    // Connect if not already connected
    if (!webSocketService.isConnected()) {
      webSocketService.connect();
    }

    // Cleanup
    return () => {
      webSocketService.removeMessageListener(handleMessage);
      webSocketService.removeConnectionStatusListener(handleConnectionStatusChange);
      webSocketService.removeErrorListener(handleError);
    };
  }, [handleMessage, handleConnectionStatusChange, handleError]);

  // Handle page visibility changes for reconnection
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible' && !webSocketService.isConnected()) {
        // Page became visible and we're disconnected, try to reconnect
        webSocketService.connect();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  // Handle online/offline events
  useEffect(() => {
    const handleOnline = () => {
      if (!webSocketService.isConnected()) {
        webSocketService.connect();
      }
    };

    const handleOffline = () => {
      // Browser is offline, will automatically reconnect when online
      setError('Network connection lost');
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  // Setup periodic connection health check
  useEffect(() => {
    const healthCheckInterval = setInterval(() => {
      const stats = webSocketService.getConnectionStats();

      // Update reconnect attempts if changed
      if (stats.reconnectAttempts !== reconnectAttempts) {
        setReconnectAttempts(stats.reconnectAttempts);
      }

      // If disconnected for too long, show error
      if (!stats.isConnected && stats.reconnectAttempts >= stats.maxReconnectAttempts) {
        setError('Unable to establish connection. Please refresh the page.');
      }
    }, 5000);

    return () => {
      clearInterval(healthCheckInterval);
    };
  }, [reconnectAttempts]);

  return {
    isConnected,
    connectionStatus,
    lastMessage,
    sendMessage,
    error,
    deviceCount,
    reconnectAttempts,
    forceReconnect,
    getConnectionStats,
  };
};

export { useWebSocket };
export default useWebSocket;