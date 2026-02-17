/**
 * WebSocket Manager for Real-time Updates
 *
 * Features:
 * - Automatic reconnection with exponential backoff
 * - Heartbeat mechanism to keep connection alive
 * - Event subscription/unsubscription with pub-sub pattern
 * - Connection state management
 * - Error handling and recovery
 *
 * Events:
 * - service.registered - Service registration
 * - service.unregistered - Service deregistration
 * - instance.registered - Instance registration
 * - instance.unregistered - Instance deregistration
 * - instance.status_changed - Instance status change
 * - cluster.node_added - Cluster node added
 * - cluster.node_removed - Cluster node removed
 */

// ===== Type Definitions =====

/**
 * WebSocket message structure from server
 */
export interface WebSocketMessage {
  event: string;
  data: unknown;
  timestamp: number;
}

/**
 * WebSocket configuration options
 */
export interface WebSocketOptions {
  url: string;
  autoReconnect?: boolean;
  maxReconnectAttempts?: number;
  reconnectInterval?: number;
  heartbeatInterval?: number;
}

/**
 * Connection states
 */
export const ConnectionState = {
  DISCONNECTED: 'disconnected',
  CONNECTING: 'connecting',
  CONNECTED: 'connected',
  RECONNECTING: 'reconnecting',
  FAILED: 'failed',
} as const;

export type ConnectionState = typeof ConnectionState[keyof typeof ConnectionState];

/**
 * Event handler callback type
 */
export type EventHandler = (data: unknown) => void;

/**
 * Connection state change callback
 */
export type StateChangeHandler = (state: ConnectionState) => void;

// ===== Constants =====

const DEFAULT_OPTIONS: Required<Omit<WebSocketOptions, 'url'>> = {
  autoReconnect: true,
  maxReconnectAttempts: 10,
  reconnectInterval: 5000, // 5 seconds
  heartbeatInterval: 30000, // 30 seconds
};

const HEARTBEAT_MESSAGE = JSON.stringify({ type: 'ping' });

// ===== WebSocket Manager Class =====

/**
 * WebSocket Manager
 *
 * Manages WebSocket connection, reconnection, heartbeat, and event subscriptions
 *
 * Usage:
 * ```typescript
 * const wsManager = new WebSocketManager({
 *   url: 'ws://localhost:8080/ws'
 * });
 *
 * wsManager.connect();
 *
 * wsManager.subscribe('service.registered', (data) => {
 *   console.log('Service registered:', data);
 * });
 *
 * wsManager.unsubscribe('service.registered');
 * wsManager.disconnect();
 * ```
 */
export class WebSocketManager {
  private ws: WebSocket | null = null;
  private options: Required<WebSocketOptions>;
  private state: ConnectionState = ConnectionState.DISCONNECTED;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;

  // Event handlers stored by event type
  private eventHandlers = new Map<string, Set<EventHandler>>();

  // State change handlers
  private stateChangeHandlers = new Set<StateChangeHandler>();

  /**
   * Create a WebSocket manager
   *
   * @param options - WebSocket configuration
   */
  constructor(options: WebSocketOptions) {
    this.options = {
      ...DEFAULT_OPTIONS,
      ...options,
    };
  }

  // ===== Connection Management =====

  /**
   * Connect to WebSocket server
   */
  public connect(): void {
    if (
      this.state === 'connected' ||
      this.state === 'connecting'
    ) {
      // This is normal in React StrictMode (dev), just return silently
      return;
    }

    this.setState('connecting');

    try {
      this.ws = new WebSocket(this.options.url);
      this.setupEventHandlers();
    } catch (error) {
      console.error('[WebSocket] Connection failed:', error);
      this.handleConnectionError();
    }
  }

  /**
   * Disconnect from WebSocket server
   */
  public disconnect(): void {
    this.options.autoReconnect = false; // Disable auto-reconnect on manual disconnect
    this.clearTimers();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.setState('disconnected');
  }

  /**
   * Check if connected
   */
  public isConnected(): boolean {
    return this.state === 'connected';
  }

  /**
   * Get current connection state
   */
  public getState(): ConnectionState {
    return this.state;
  }

  // ===== Event Subscription =====

  /**
   * Subscribe to an event
   *
   * @param eventType - Event type to subscribe to
   * @param handler - Callback function to handle event
   */
  public subscribe(eventType: string, handler: EventHandler): void {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, new Set());
    }

    this.eventHandlers.get(eventType)!.add(handler);
    console.log(`[WebSocket] Subscribed to event: ${eventType}`);
  }

  /**
   * Unsubscribe from an event
   *
   * @param eventType - Event type to unsubscribe from
   * @param handler - Handler to remove (if not provided, removes all handlers for this event)
   */
  public unsubscribe(eventType: string, handler?: EventHandler): void {
    if (handler) {
      this.eventHandlers.get(eventType)?.delete(handler);
      console.log(`[WebSocket] Unsubscribed handler from event: ${eventType}`);
    } else {
      this.eventHandlers.delete(eventType);
      console.log(`[WebSocket] Unsubscribed all handlers from event: ${eventType}`);
    }
  }

  /**
   * Subscribe to connection state changes
   *
   * @param handler - Callback function to handle state changes
   */
  public onStateChange(handler: StateChangeHandler): void {
    this.stateChangeHandlers.add(handler);
  }

  /**
   * Unsubscribe from connection state changes
   *
   * @param handler - Handler to remove
   */
  public offStateChange(handler: StateChangeHandler): void {
    this.stateChangeHandlers.delete(handler);
  }

  // ===== Private Methods =====

  /**
   * Setup WebSocket event handlers
   */
  private setupEventHandlers(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      console.log('[WebSocket] Connected');
      this.setState('connected');
      this.reconnectAttempts = 0;
      this.startHeartbeat();
    };

    this.ws.onclose = (event) => {
      console.log('[WebSocket] Disconnected:', event.code, event.reason);
      this.clearTimers();
      this.handleConnectionError();
    };

    this.ws.onerror = (error) => {
      console.error('[WebSocket] Error:', error);
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        this.handleMessage(message);
      } catch (error) {
        console.error('[WebSocket] Failed to parse message:', error);
      }
    };
  }

  /**
   * Handle incoming WebSocket message
   *
   * @param message - WebSocket message
   */
  private handleMessage(message: WebSocketMessage): void {
    const { event: eventType, data } = message;

    // Special handling for pong messages
    if (eventType === 'pong') {
      return;
    }

    // Dispatch event to all subscribed handlers
    const handlers = this.eventHandlers.get(eventType);
    if (handlers && handlers.size > 0) {
      handlers.forEach((handler) => {
        try {
          handler(data);
        } catch (error) {
          console.error(`[WebSocket] Error handling event ${eventType}:`, error);
        }
      });
    }
  }

  /**
   * Handle connection errors and attempt reconnection
   */
  private handleConnectionError(): void {
    if (!this.options.autoReconnect) {
      this.setState('disconnected');
      return;
    }

    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('[WebSocket] Max reconnection attempts reached');
      this.setState('failed');
      return;
    }

    this.setState('reconnecting');
    this.reconnectAttempts += 1;

    const delay = this.options.reconnectInterval * this.reconnectAttempts;
    console.log(
      `[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.options.maxReconnectAttempts})`
    );

    this.reconnectTimer = setTimeout(() => {
      console.log('[WebSocket] Attempting to reconnect...');
      this.connect();
    }, delay);
  }

  /**
   * Start heartbeat mechanism
   */
  private startHeartbeat(): void {
    this.clearHeartbeat();

    this.heartbeatTimer = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        try {
          this.ws.send(HEARTBEAT_MESSAGE);
        } catch (error) {
          console.error('[WebSocket] Failed to send heartbeat:', error);
        }
      }
    }, this.options.heartbeatInterval);
  }

  /**
   * Clear heartbeat timer
   */
  private clearHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  /**
   * Clear all timers
   */
  private clearTimers(): void {
    this.clearHeartbeat();

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  /**
   * Set connection state and notify handlers
   *
   * @param newState - New connection state
   */
  private setState(newState: ConnectionState): void {
    if (this.state === newState) return;

    this.state = newState;
    console.log(`[WebSocket] State changed: ${newState}`);

    // Notify state change handlers
    this.stateChangeHandlers.forEach((handler) => {
      try {
        handler(newState);
      } catch (error) {
        console.error('[WebSocket] Error in state change handler:', error);
      }
    });
  }
}

// ===== Singleton Instance =====

let wsManagerInstance: WebSocketManager | null = null;

/**
 * Get or create WebSocket manager singleton
 *
 * @param url - WebSocket URL (only used on first call)
 * @returns WebSocket manager instance
 */
export function getWebSocketManager(url?: string): WebSocketManager {
  if (!wsManagerInstance) {
    if (!url) {
      // Get URL from environment variable
      const wsUrl = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080';
      url = `${wsUrl}/ws`;
    }

    wsManagerInstance = new WebSocketManager({ url });
  }

  return wsManagerInstance;
}

/**
 * Reset WebSocket manager singleton (mainly for testing)
 */
export function resetWebSocketManager(): void {
  if (wsManagerInstance) {
    wsManagerInstance.disconnect();
    wsManagerInstance = null;
  }
}
