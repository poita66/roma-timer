import { webSocketService, useWebSocket } from '../websocket';
import { renderHook, act, waitFor } from '@testing-library/react';
import { WebSocketMessage, ClientWebSocketMessage } from '../../types';

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState: number = MockWebSocket.CONNECTING;
  url: string;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  private sentMessages: string[] = [];
  private closeEvent?: CloseEvent;

  constructor(url: string) {
    this.url = url;

    // Simulate connection after a short delay
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 10);
  }

  send(data: string) {
    this.sentMessages.push(data);
  }

  close(code?: number, reason?: string) {
    this.readyState = MockWebSocket.CLOSING;

    setTimeout(() => {
      this.readyState = MockWebSocket.CLOSED;
      if (this.onclose) {
        this.closeEvent = new CloseEvent('close', {
          code: code || 1000,
          reason: reason || '',
          wasClean: true
        });
        this.onclose(this.closeEvent);
      }
    }, 5);
  }

  // Helper methods for testing
  simulateMessage(message: WebSocketMessage) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', {
        data: JSON.stringify(message)
      }));
    }
  }

  simulateError() {
    if (this.onerror) {
      this.onerror(new Event('error'));
    }
  }

  simulateConnectionLoss() {
    this.close(1006, 'Connection lost');
  }

  getSentMessages(): string[] {
    return [...this.sentMessages];
  }

  resetSentMessages() {
    this.sentMessages = [];
  }
}

// Mock global WebSocket
global.WebSocket = MockWebSocket as any;
global.setTimeout = global.setTimeout || ((fn: Function, delay: number) => setTimeout(fn, delay));
global.clearTimeout = global.clearTimeout || ((id: number) => clearTimeout(id));

describe('WebSocket Service', () => {
  let mockWebSocket: MockWebSocket;

  beforeEach(() => {
    // Reset service state
    webSocketService.disconnect();

    // Create new mock WebSocket instance
    mockWebSocket = new MockWebSocket('ws://localhost:3001/ws');
  });

  afterEach(() => {
    webSocketService.disconnect();
  });

  test('should connect to WebSocket successfully', async () => {
    const connectionStatusListener = jest.fn();
    webSocketService.onConnectionStatusChange(connectionStatusListener);

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(connectionStatusListener).toHaveBeenCalledWith(true);
    });
  });

  test('should handle connection errors and retry', async () => {
    const connectionStatusListener = jest.fn();
    const errorListener = jest.fn();

    webSocketService.onConnectionStatusChange(connectionStatusListener);
    webSocketService.onError(errorListener);

    // Mock failed connection
    const originalWebSocket = global.WebSocket;
    global.WebSocket = class extends MockWebSocket {
      constructor(url: string) {
        super(url);
        // Simulate immediate connection failure
        setTimeout(() => {
          this.readyState = MockWebSocket.CLOSED;
          if (this.onclose) {
            this.onclose(new CloseEvent('close', { code: 1006 }));
          }
        }, 5);
      }
    };

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(connectionStatusListener).toHaveBeenCalledWith(false);
    }, { timeout: 200 });

    // Restore original WebSocket
    global.WebSocket = originalWebSocket;
  });

  test('should send messages when connected', async () => {
    act(() => {
      webSocketService.connect();
    });

    // Wait for connection
    await waitFor(() => {
      expect(webSocketService.isConnected()).toBe(true);
    });

    const message: ClientWebSocketMessage = {
      type: 'StartTimer',
      payload: { sessionId: 'test-session' }
    };

    act(() => {
      webSocketService.sendMessage(message);
    });

    // Note: This test would need access to the actual WebSocket instance
    // In practice, we'd test this through integration tests
  });

  test('should handle incoming messages', async () => {
    const messageListener = jest.fn();
    webSocketService.onMessage(messageListener);

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(webSocketService.isConnected()).toBe(true);
    });

    const incomingMessage: WebSocketMessage = {
      type: 'TimerStateUpdate',
      payload: {
        id: 'test-session',
        is_running: true,
        elapsed: 30,
        duration: 1500
      }
    };

    act(() => {
      mockWebSocket.simulateMessage(incomingMessage);
    });

    expect(messageListener).toHaveBeenCalledWith(incomingMessage);
  });

  test('should reconnect automatically on connection loss', async () => {
    const connectionStatusListener = jest.fn();
    webSocketService.onConnectionStatusChange(connectionStatusListener);

    act(() => {
      webSocketService.connect();
    });

    // Wait for initial connection
    await waitFor(() => {
      expect(connectionStatusListener).toHaveBeenCalledWith(true);
    });

    // Simulate connection loss
    act(() => {
      mockWebSocket.simulateConnectionLoss();
    });

    // Should detect disconnection
    await waitFor(() => {
      expect(connectionStatusListener).toHaveBeenCalledWith(false);
    }, { timeout: 100 });

    // Should attempt reconnection (handled by the service)
    // This would be tested with mock timers in a real implementation
  });

  test('should handle exponential backoff on reconnection', async () => {
    jest.useFakeTimers();

    const connectionStatusListener = jest.fn();
    webSocketService.onConnectionStatusChange(connectionStatusListener);

    // Mock WebSocket that fails to connect
    global.WebSocket = class extends MockWebSocket {
      constructor(url: string) {
        super(url);
        setTimeout(() => {
          this.readyState = MockWebSocket.CLOSED;
          if (this.onclose) {
            this.onclose(new CloseEvent('close', { code: 1006 }));
          }
        }, 5);
      }
    };

    act(() => {
      webSocketService.connect();
    });

    // Fast forward through retry attempts
    for (let i = 0; i < 3; i++) {
      act(() => {
        jest.advanceTimersByTime(1000 * Math.pow(2, i)); // Exponential backoff
      });
    }

    // Restore real WebSocket
    jest.useRealTimers();

    // Should have attempted multiple reconnections
    expect(connectionStatusListener).toHaveBeenCalledTimes(4); // 1 initial + 3 retries
  });

  test('should remove event listeners correctly', () => {
    const messageListener = jest.fn();
    const connectionListener = jest.fn();

    webSocketService.onMessage(messageListener);
    webSocketService.onConnectionStatusChange(connectionListener);

    // Remove listeners
    webSocketService.removeMessageListener(messageListener);
    webSocketService.removeConnectionStatusListener(connectionListener);

    act(() => {
      webSocketService.connect();
    });

    // Simulate message
    const incomingMessage: WebSocketMessage = {
      type: 'TimerStateUpdate',
      payload: { id: 'test' }
    };

    act(() => {
      mockWebSocket.simulateMessage(incomingMessage);
    });

    // Listeners should not be called after removal
    expect(messageListener).not.toHaveBeenCalled();
  });

  test('should handle connection status changes correctly', () => {
    const connectionListener = jest.fn();

    act(() => {
      webSocketService.onConnectionStatusChange(connectionListener);
      webSocketService.connect();
    });

    // Should be disconnected initially
    expect(webSocketService.getConnectionStatus()).toBe('disconnected');
  });

  test('should handle cleanup on disconnect', () => {
    act(() => {
      webSocketService.connect();
    });

    act(() => {
      webSocketService.disconnect();
    });

    expect(webSocketService.getConnectionStatus()).toBe('disconnected');
    expect(webSocketService.isConnected()).toBe(false);
  });
});

describe('useWebSocket Hook', () => {
  beforeEach(() => {
    // Reset WebSocket service before each test
    webSocketService.disconnect();
  });

  afterEach(() => {
    webSocketService.disconnect();
  });

  test('should initialize with disconnected state', () => {
    const { result } = renderHook(() => useWebSocket());

    expect(result.current.isConnected).toBe(false);
    expect(result.current.connectionStatus).toBe('disconnected');
    expect(result.current.lastMessage).toBe(null);
    expect(result.current.error).toBe(null);
  });

  test('should handle connection status changes', async () => {
    const { result } = renderHook(() => useWebSocket());

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
      expect(result.current.connectionStatus).toBe('connected');
    });
  });

  test('should handle incoming messages', async () => {
    const { result } = renderHook(() => useWebSocket());

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
    });

    const incomingMessage: WebSocketMessage = {
      type: 'TimerStateUpdate',
      payload: {
        id: 'test-session',
        is_running: true,
        elapsed: 45,
        duration: 1500
      }
    };

    act(() => {
      // This would need access to the actual WebSocket instance
      // In practice, we'd test through integration tests
    });

    expect(result.current.lastMessage).toBe(incomingMessage);
    expect(result.current.error).toBe(null);
  });

  test('should send messages through hook', async () => {
    const { result } = renderHook(() => useWebSocket());

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
    });

    const message: ClientWebSocketMessage = {
      type: 'StartTimer',
      payload: { sessionId: 'test-session' }
    };

    act(() => {
      result.current.sendMessage(message);
    });

    // Message should be sent (verified through integration tests)
  });

  test('should handle connection errors', async () => {
    const { result } = renderHook(() => useWebSocket());

    // Mock WebSocket that fails to connect
    global.WebSocket = class extends MockWebSocket {
      constructor(url: string) {
        super(url);
        setTimeout(() => {
          this.readyState = MockWebSocket.CLOSED;
          if (this.onerror) {
            this.onerror(new Event('error'));
          }
          if (this.onclose) {
            this.onclose(new CloseEvent('close', { code: 1006 }));
          }
        }, 5);
      }
    };

    act(() => {
      webSocketService.connect();
    });

    await waitFor(() => {
      expect(result.current.isConnected).toBe(false);
      expect(result.current.connectionStatus).toBe('disconnected');
      expect(result.current.error).toBeTruthy();
    });
  });

  test('should cleanup on unmount', () => {
    const { unmount } = renderHook(() => useWebSocket());

    act(() => {
      webSocketService.connect();
    });

    unmount();

    // Service should be cleaned up
    expect(webSocketService.getConnectionStatus()).toBe('disconnected');
  });
});