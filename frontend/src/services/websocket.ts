//! WebSocket Service
//!
//! Real-time communication service for timer state synchronization.

import React, { useState, useCallback, useEffect } from 'react';
import {
  WebSocketMessage,
  ClientWebSocketMessage,
  UseWebSocketReturn,
  ConnectionStatus
} from '../types';

class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second
  private maxReconnectDelay = 30000; // Max 30 seconds
  private isConnecting = false;
  private isDestroyed = false;

  // Event listeners
  private messageListeners: ((message: WebSocketMessage) => void)[] = [];
  private connectionStatusListeners: ((connected: boolean) => void)[] = [];
  private errorListeners: ((error: Error) => void)[] = [];

  // WebSocket URL with authentication
  private getWebSocketUrl(): string {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = process.env.REACT_APP_WS_URL ||
                  process.env.EXPO_PUBLIC_WS_URL ||
                  window.location.host;

    const sharedSecret = process.env.REACT_APP_SHARED_SECRET ||
                        process.env.EXPO_PUBLIC_SHARED_SECRET;

    let url = `${wsProtocol}//${wsHost}/ws`;

    if (sharedSecret) {
      url += `?token=${encodeURIComponent(sharedSecret)}`;
    }

    return url;
  }

  // Connect to WebSocket
  connect(): void {
    if (this.isDestroyed || this.isConnecting || this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.isConnecting = true;

    try {
      const url = this.getWebSocketUrl();
      this.ws = new WebSocket(url);

      // Setup event handlers
      this.ws.onopen = this.handleOpen.bind(this);
      this.ws.onmessage = this.handleMessage.bind(this);
      this.ws.onclose = this.handleClose.bind(this);
      this.ws.onerror = this.handleError.bind(this);

    } catch (error) {
      this.isConnecting = false;
      console.error('Failed to create WebSocket connection:', error);
      this.notifyError(error as Error);
      this.scheduleReconnect();
    }
  }

  // Disconnect from WebSocket
  disconnect(): void {
    this.isDestroyed = true;

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.clearReconnectTimeout();
  }

  // Send message to server
  sendMessage(message: ClientWebSocketMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message));
      } catch (error) {
        console.error('Failed to send WebSocket message:', error);
        this.notifyError(error as Error);
      }
    } else {
      console.warn('WebSocket is not connected. Message not sent:', message);
    }
  }

  // Get connection status
  getConnectionStatus(): ConnectionStatus {
    if (!this.ws) return 'disconnected';

    switch (this.ws.readyState) {
      case WebSocket.OPEN:
        return 'connected';
      case WebSocket.CONNECTING:
        return 'reconnecting';
      case WebSocket.CLOSING:
      case WebSocket.CLOSED:
        return 'disconnected';
      default:
        return 'disconnected';
    }
  }

  // Check if connected
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  // Event listener registration
  onMessage(listener: (message: WebSocketMessage) => void): void {
    this.messageListeners.push(listener);
  }

  onConnectionStatusChange(listener: (connected: boolean) => void): void {
    this.connectionStatusListeners.push(listener);
  }

  onError(listener: (error: Error) => void): void {
    this.errorListeners.push(listener);
  }

  // Remove event listener
  removeMessageListener(listener: (message: WebSocketMessage) => void): void {
    const index = this.messageListeners.indexOf(listener);
    if (index > -1) {
      this.messageListeners.splice(index, 1);
    }
  }

  removeConnectionStatusListener(listener: (connected: boolean) => void): void {
    const index = this.connectionStatusListeners.indexOf(listener);
    if (index > -1) {
      this.connectionStatusListeners.splice(index, 1);
    }
  }

  removeErrorListener(listener: (error: Error) => void): void {
    const index = this.errorListeners.indexOf(listener);
    if (index > -1) {
      this.errorListeners.splice(index, 1);
    }
  }

  // Private methods
  private handleOpen(): void {
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    this.reconnectDelay = 1000;

    console.log('WebSocket connected');
    this.notifyConnectionStatusChange(true);
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const message: WebSocketMessage = JSON.parse(event.data);
      this.notifyMessage(message);
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
      this.notifyError(error as Error);
    }
  }

  private handleClose(event: CloseEvent): void {
    this.isConnecting = false;
    this.ws = null;

    console.log('WebSocket disconnected:', event.code, event.reason);
    this.notifyConnectionStatusChange(false);

    if (!this.isDestroyed && event.code !== 1000) { // 1000 = normal closure
      this.scheduleReconnect();
    }
  }

  private handleError(event: Event): void {
    this.isConnecting = false;
    console.error('WebSocket error:', event);
    this.notifyError(new Error('WebSocket connection error'));
  }

  private notifyMessage(message: WebSocketMessage): void {
    this.messageListeners.forEach(listener => {
      try {
        listener(message);
      } catch (error) {
        console.error('Error in message listener:', error);
      }
    });
  }

  private notifyConnectionStatusChange(connected: boolean): void {
    this.connectionStatusListeners.forEach(listener => {
      try {
        listener(connected);
      } catch (error) {
        console.error('Error in connection status listener:', error);
      }
    });
  }

  private notifyError(error: Error): void {
    this.errorListeners.forEach(listener => {
      try {
        listener(error);
      } catch (error) {
        console.error('Error in error listener:', error);
      }
    });
  }

  private scheduleReconnect(): void {
    if (this.isDestroyed || this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.log('Max reconnection attempts reached. Giving up.');
      return;
    }

    this.clearReconnectTimeout();

    const delay = Math.min(
      this.reconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay
    );

    console.log(`Scheduling reconnect in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);

    this.reconnectTimeout = window.setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  private clearReconnectTimeout(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }

  private reconnectTimeout: number | null = null;

  // Force immediate reconnection
  public forceReconnect(): void {
    this.disconnect();
    this.reconnectAttempts = 0;
    this.connect();
  }

  // Get current connection statistics
  public getConnectionStats(): {
    reconnectAttempts: number;
    maxReconnectAttempts: number;
    isConnected: boolean;
    connectionStatus: ConnectionStatus;
  } {
    return {
      reconnectAttempts: this.reconnectAttempts,
      maxReconnectAttempts: this.maxReconnectAttempts,
      isConnected: this.isConnected(),
      connectionStatus: this.getConnectionStatus(),
    };
  }

  // Send heartbeat/ping to keep connection alive
  public sendPing(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify({ type: 'Ping' }));
      } catch (error) {
        console.error('Failed to send ping:', error);
        this.notifyError(error as Error);
      }
    }
  }

  // Start automatic heartbeat
  private startHeartbeat(): void {
    // Send ping every 30 seconds
    this.heartbeatInterval = window.setInterval(() => {
      this.sendPing();
    }, 30000);
  }

  // Stop automatic heartbeat
  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  // Enhanced connection establishment with device identification
  private establishConnection(): void {
    if (this.isDestroyed || this.isConnecting || this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.isConnecting = true;

    try {
      const url = this.getWebSocketUrlWithDeviceInfo();
      this.ws = new WebSocket(url);

      // Setup event handlers
      this.ws.onopen = this.handleOpen.bind(this);
      this.ws.onmessage = this.handleMessage.bind(this);
      this.ws.onclose = this.handleClose.bind(this);
      this.ws.onerror = this.handleError.bind(this);

    } catch (error) {
      this.isConnecting = false;
      console.error('Failed to create WebSocket connection:', error);
      this.notifyError(error as Error);
      this.scheduleReconnect();
    }
  }

  // Enhanced WebSocket URL with device information
  private getWebSocketUrlWithDeviceInfo(): string {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = process.env.REACT_APP_WS_URL ||
                  process.env.EXPO_PUBLIC_WS_URL ||
                  window.location.host;

    const sharedSecret = process.env.REACT_APP_SHARED_SECRET ||
                        process.env.EXPO_PUBLIC_SHARED_SECRET;

    let url = `${wsProtocol}//${wsHost}/ws`;

    // Add query parameters for device identification
    const params = new URLSearchParams();

    if (sharedSecret) {
      params.append('token', sharedSecret);
    }

    // Add device identification
    const deviceId = this.getOrCreateDeviceId();
    params.append('device_id', deviceId);

    const userAgent = navigator.userAgent;
    params.append('user_agent', userAgent);

    url += `?${params.toString()}`;
    return url;
  }

  // Get or create persistent device ID
  private getOrCreateDeviceId(): string {
    const storageKey = 'roma-timer-device-id';
    let deviceId = localStorage.getItem(storageKey);

    if (!deviceId) {
      deviceId = `device-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem(storageKey, deviceId);
    }

    return deviceId;
  }

  // Enhanced message handling with device synchronization
  private handleMessage(event: MessageEvent): void {
    try {
      const message: WebSocketMessage = JSON.parse(event.data);

      // Handle different message types
      switch (message.type) {
        case 'TimerStateUpdate':
          // Validate timestamp to prevent stale updates
          if (message.payload?.timestamp) {
            const now = Date.now();
            const messageAge = now - message.payload.timestamp;

            // Reject messages older than 10 seconds
            if (messageAge > 10000) {
              console.warn('Received stale timer state update, ignoring');
              return;
            }
          }
          break;

        case 'ConnectionStatus':
          // Update device count and connection info
          if (message.payload?.device_count !== undefined) {
            console.log(`Connected devices: ${message.payload.device_count}`);
          }
          break;

        case 'Notification':
          // Handle notifications (timer completion, etc.)
          this.handleNotification(message);
          break;

        case 'ConfigurationUpdate':
          // Handle configuration changes from other devices
          this.handleConfigurationUpdate(message);
          break;
      }

      this.notifyMessage(message);
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
      this.notifyError(error as Error);
    }
  }

  // Handle notification messages
  private handleNotification(message: WebSocketMessage): void {
    if (message.type === 'Notification' && message.payload?.message) {
      // Show browser notification if page is not visible
      if (!document.hidden && 'Notification' in window && Notification.permission === 'granted') {
        new Notification('Roma Timer', {
          body: message.payload.message,
          icon: '/icon-192x192.png',
          tag: 'roma-timer',
        });
      }

      // Also show in-app notification
      console.log('Timer Notification:', message.payload.message);
    }
  }

  // Handle configuration updates from other devices
  private handleConfigurationUpdate(message: WebSocketMessage): void {
    if (message.type === 'ConfigurationUpdate' && message.payload) {
      // Emit custom event for configuration updates
      window.dispatchEvent(new CustomEvent('configurationUpdate', {
        detail: message.payload
      }));
    }
  }

  private heartbeatInterval: number | null = null;

  // Override the connect method to use enhanced connection
  public connect(): void {
    this.establishConnection();
  }
}

// Export singleton instance
export const webSocketService = new WebSocketService();

// Export hook for React components
export const useWebSocket = (): UseWebSocketReturn => {
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('disconnected');
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Register event listeners
    const handleMessage = (message: WebSocketMessage) => {
      setLastMessage(message);
      setError(null);
    };

    const handleConnectionStatusChange = (connected: boolean) => {
      setIsConnected(connected);
      setConnectionStatus(connected ? 'connected' : 'disconnected');
      if (connected) {
        setError(null);
      }
    };

    const handleError = (wsError: Error) => {
      setError(wsError.message);
    };

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
  }, []);

  const sendMessage = useCallback((message: ClientWebSocketMessage) => {
    webSocketService.sendMessage(message);
  }, []);

  return {
    isConnected,
    connectionStatus,
    lastMessage,
    sendMessage,
    error,
  };
};

export { WebSocketService };
export default webSocketService;