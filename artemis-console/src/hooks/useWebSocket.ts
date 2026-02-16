/**
 * React Hook for WebSocket Subscriptions
 *
 * Provides a convenient way to subscribe to WebSocket events in React components
 * Automatically handles subscription/unsubscription lifecycle
 *
 * Features:
 * - Automatic subscription on mount
 * - Automatic unsubscription on unmount
 * - Re-subscription when event type or callback changes
 * - Multiple components can subscribe to the same event
 *
 * Usage:
 * ```typescript
 * function MyComponent() {
 *   useWebSocket('service.registered', (data) => {
 *     console.log('Service registered:', data);
 *   });
 *
 *   return <div>...</div>;
 * }
 * ```
 */

import { useEffect, useCallback, useRef } from 'react';
import type { EventHandler } from '@/utils/websocket';
import { getWebSocketManager } from '@/utils/websocket';

/**
 * Hook for subscribing to WebSocket events
 *
 * @param eventType - Event type to subscribe to
 * @param callback - Callback function to handle events
 * @param enabled - Whether subscription is enabled (default: true)
 */
export function useWebSocket(
  eventType: string,
  callback: EventHandler,
  enabled = true
): void {
  const wsManager = getWebSocketManager();
  const callbackRef = useRef<EventHandler>(callback);

  // Keep callback ref up to date
  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  // Wrap callback to use the ref (avoids re-subscription when callback changes)
  const stableCallback = useCallback<EventHandler>((data) => {
    callbackRef.current(data);
  }, []);

  useEffect(() => {
    if (!enabled) {
      return;
    }

    // Subscribe to event
    wsManager.subscribe(eventType, stableCallback);

    // Unsubscribe on unmount
    return () => {
      wsManager.unsubscribe(eventType, stableCallback);
    };
  }, [wsManager, eventType, stableCallback, enabled]);
}

/**
 * Hook for WebSocket connection state
 *
 * Returns the current connection state and updates when it changes
 *
 * Usage:
 * ```typescript
 * function MyComponent() {
 *   const connectionState = useWebSocketState();
 *
 *   return (
 *     <div>
 *       Connection: {connectionState}
 *     </div>
 *   );
 * }
 * ```
 */
import { useState } from 'react';
import type { ConnectionState } from '@/utils/websocket';

export function useWebSocketState(): ConnectionState {
  const wsManager = getWebSocketManager();
  const [state, setState] = useState<ConnectionState>(wsManager.getState());

  useEffect(() => {
    const handleStateChange = (newState: ConnectionState) => {
      setState(newState);
    };

    wsManager.onStateChange(handleStateChange);

    return () => {
      wsManager.offStateChange(handleStateChange);
    };
  }, [wsManager]);

  return state;
}

/**
 * Hook for WebSocket connection control
 *
 * Returns connection state and control functions
 *
 * Usage:
 * ```typescript
 * function MyComponent() {
 *   const { isConnected, connect, disconnect } = useWebSocketConnection();
 *
 *   return (
 *     <div>
 *       {isConnected ? (
 *         <Button onClick={disconnect}>Disconnect</Button>
 *       ) : (
 *         <Button onClick={connect}>Connect</Button>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function useWebSocketConnection() {
  const wsManager = getWebSocketManager();
  const connectionState = useWebSocketState();

  const connect = useCallback(() => {
    wsManager.connect();
  }, [wsManager]);

  const disconnect = useCallback(() => {
    wsManager.disconnect();
  }, [wsManager]);

  const isConnected = connectionState === 'connected';
  const isConnecting = connectionState === 'connecting' || connectionState === 'reconnecting';
  const isFailed = connectionState === 'failed';

  return {
    connectionState,
    isConnected,
    isConnecting,
    isFailed,
    connect,
    disconnect,
  };
}
