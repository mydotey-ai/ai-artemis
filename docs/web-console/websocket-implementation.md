# WebSocket 实时推送实现

**文档状态**: ✅ 最新
**最后更新**: 2026-02-17
**相关 Phase**: Phase 3
**源代码**: `artemis-console/src/utils/websocket.ts`, `artemis-console/src/hooks/useWebSocket.ts`

---

## 概述

WebSocket 实现为 Artemis Console 提供实时更新功能，通过客户端解决方案实时推送服务、实例和集群事件。

## Architecture

### Core Components

1. **WebSocket Manager** (`src/utils/websocket.ts`)
   - Singleton WebSocket connection manager
   - Automatic reconnection with exponential backoff
   - Heartbeat mechanism (ping every 30 seconds)
   - Event subscription/unsubscription with pub-sub pattern
   - Connection state management

2. **React Hooks** (`src/hooks/useWebSocket.ts`)
   - `useWebSocket(eventType, callback)` - Subscribe to specific events
   - `useWebSocketState()` - Get current connection state
   - `useWebSocketConnection()` - Connection control functions

3. **WebSocket Provider** (`src/components/WebSocketProvider.tsx`)
   - Manages WebSocket lifecycle at app level
   - Shows connection state notifications
   - Wraps entire application

4. **Notification System** (`src/components/NotificationSnackbar.tsx`)
   - Displays event notifications using Material-UI Snackbar
   - Integrates with uiStore notification system
   - Auto-stacking and auto-dismiss

5. **Connection Status Indicator** (in Header component)
   - Visual indicator in the header
   - Shows connection state: Connected (green), Connecting/Reconnecting (orange), Failed (red)
   - Animated pulse effect during connection attempts

## Supported Events

| Event Type | Description |
|------------|-------------|
| `service.registered` | Service registration |
| `service.unregistered` | Service deregistration |
| `instance.registered` | Instance registration |
| `instance.unregistered` | Instance deregistration |
| `instance.status_changed` | Instance status change |
| `cluster.node_added` | Cluster node added |
| `cluster.node_removed` | Cluster node removed |

## Page Integration

### Dashboard (`src/pages/Dashboard/Dashboard.tsx`)
- Subscribes to all event types
- Updates statistics in real-time
- Shows notifications for each event

### Services (`src/pages/Services/Services.tsx`)
- Subscribes to `service.registered` and `service.unregistered`
- Automatically refreshes service list
- Shows notifications

### Instances (`src/pages/Instances/Instances.tsx`)
- Subscribes to `instance.*` events
- Updates instance list in real-time
- Shows notifications with instance details

### Cluster (`src/pages/Cluster/Cluster.tsx`)
- Subscribes to `cluster.*` events
- Updates cluster status and node list
- Shows notifications for node changes

## Configuration

Environment variables:

```bash
# Development (.env.development)
VITE_WS_BASE_URL=ws://localhost:8080

# Production (.env.production)
VITE_WS_BASE_URL=wss://artemis.example.com
```

WebSocket options (in `src/utils/websocket.ts`):

```typescript
{
  autoReconnect: true,              // Enable automatic reconnection
  maxReconnectAttempts: 10,         // Maximum reconnection attempts
  reconnectInterval: 5000,          // Reconnection interval (5 seconds)
  heartbeatInterval: 30000,         // Heartbeat interval (30 seconds)
}
```

## Usage Examples

### Subscribe to Events in a Component

```typescript
import { useWebSocket } from '@/hooks/useWebSocket';
import { useUIStore } from '@/store/uiStore';

function MyComponent() {
  const showNotification = useUIStore((state) => state.showNotification);

  useWebSocket('service.registered', (data) => {
    console.log('Service registered:', data);
    showNotification({
      type: 'success',
      message: 'New service registered',
    });
  });

  return <div>...</div>;
}
```

### Get Connection State

```typescript
import { useWebSocketState } from '@/hooks/useWebSocket';

function MyComponent() {
  const connectionState = useWebSocketState();

  return (
    <div>
      Connection: {connectionState}
    </div>
  );
}
```

### Control Connection

```typescript
import { useWebSocketConnection } from '@/hooks/useWebSocket';

function MyComponent() {
  const { isConnected, connect, disconnect } = useWebSocketConnection();

  return (
    <div>
      {isConnected ? (
        <Button onClick={disconnect}>Disconnect</Button>
      ) : (
        <Button onClick={connect}>Connect</Button>
      )}
    </div>
  );
}
```

## Features

### Automatic Reconnection
- Reconnects automatically on connection loss
- Exponential backoff strategy (5s, 10s, 15s, ...)
- Maximum 10 reconnection attempts
- Shows reconnection status in UI

### Heartbeat Mechanism
- Sends ping message every 30 seconds
- Keeps connection alive
- Server responds with pong

### Event Subscription
- Multiple components can subscribe to same event
- Automatic cleanup on unmount
- Type-safe event handlers

### Connection State Management
- Tracks connection state (disconnected, connecting, connected, reconnecting, failed)
- Notifies all subscribers on state changes
- Visual indicator in header

### Error Handling
- Graceful handling of connection errors
- User-friendly error messages
- Automatic recovery attempts

## TypeScript Support

All components and hooks are fully typed with TypeScript:

```typescript
interface WebSocketMessage {
  event: string;
  data: unknown;
  timestamp: number;
}

interface WebSocketOptions {
  url: string;
  autoReconnect?: boolean;
  maxReconnectAttempts?: number;
  reconnectInterval?: number;
  heartbeatInterval?: number;
}

type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'failed';

type EventHandler = (data: unknown) => void;
```

## Testing

To test WebSocket functionality:

1. Start the Artemis server with WebSocket support
2. Start the console: `npm run dev`
3. Open browser console to see WebSocket logs
4. Trigger events from server or API
5. Observe real-time updates and notifications

## Future Enhancements

Potential improvements:

1. **Event Filtering** - Allow filtering events by criteria
2. **Message Queuing** - Queue messages when offline
3. **Compression** - Enable WebSocket compression
4. **Binary Messages** - Support binary message formats
5. **Authentication** - Add WebSocket authentication
6. **Metrics** - Track connection metrics and latency

## References

- WebSocket API: https://developer.mozilla.org/en-US/docs/Web/API/WebSocket
- React Hooks: https://react.dev/reference/react
- Material-UI Snackbar: https://mui.com/material-ui/react-snackbar/
- Zustand State Management: https://github.com/pmndrs/zustand
