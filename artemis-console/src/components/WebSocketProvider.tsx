/**
 * WebSocket Provider Component
 *
 * Manages WebSocket connection lifecycle and integrates with notification system
 * Should be placed at app root level to ensure WebSocket is initialized
 *
 * Features:
 * - Automatic connection on mount
 * - Shows notifications for connection state changes
 * - Displays real-time event notifications
 */

import { useEffect } from 'react';
import { getWebSocketManager } from '@/utils/websocket';
import type { ConnectionState } from '@/utils/websocket';
import { useUIStore } from '@/store/uiStore';

/**
 * WebSocket Provider Component
 *
 * Initializes WebSocket connection and handles connection state notifications
 */
export const WebSocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const showNotification = useUIStore((state) => state.showNotification);

  useEffect(() => {
    const wsManager = getWebSocketManager();

    // Handle connection state changes
    const handleStateChange = (state: ConnectionState) => {
      switch (state) {
        case 'connected':
          showNotification({
            type: 'success',
            message: 'WebSocket connected',
            duration: 3000,
          });
          break;

        case 'reconnecting':
          showNotification({
            type: 'warning',
            message: 'WebSocket connection lost, reconnecting...',
            duration: 5000,
          });
          break;

        case 'failed':
          showNotification({
            type: 'error',
            message: 'WebSocket connection failed after multiple attempts',
            duration: 0, // Don't auto-close error messages
          });
          break;

        case 'disconnected':
          // Only show notification if it was an unexpected disconnect
          if (wsManager.getState() !== 'connecting') {
            showNotification({
              type: 'info',
              message: 'WebSocket disconnected',
              duration: 3000,
            });
          }
          break;

        default:
          break;
      }
    };

    // Subscribe to state changes
    wsManager.onStateChange(handleStateChange);

    // Connect to WebSocket
    wsManager.connect();

    // Cleanup on unmount
    return () => {
      wsManager.offStateChange(handleStateChange);
      // Note: We don't disconnect here to keep connection alive across route changes
      // Call wsManager.disconnect() explicitly if needed
    };
  }, [showNotification]);

  return <>{children}</>;
};

/**
 * Display name for debugging
 */
WebSocketProvider.displayName = 'WebSocketProvider';

export default WebSocketProvider;
