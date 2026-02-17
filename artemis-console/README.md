# Artemis Console - React Frontend

Modern web-based management console for Artemis service registry, built with React, TypeScript, and Material-UI.

## Features

- **Service Discovery**: Browse and search registered services and instances
- **Cluster Management**: Monitor cluster nodes and replication status
- **Routing Configuration**: Configure service routing rules and groups
- **Zone Operations**: Manage zone-level batch operations
- **Canary Deployment**: Configure canary releases and IP whitelists
- **Audit Logging**: Track all management operations
- **Real-time Updates**: WebSocket-based live updates
- **Performance Optimized**: Lazy loading, code splitting, virtual scrolling

## Quick Start

### Development

```bash
# Install dependencies
npm install

# Start dev server (with HMR)
npm run dev

# Access at http://localhost:5173
```

**Default Login Credentials**:
- Username: `admin`
- Password: `admin123`
- Role: Administrator

> **Security Note**: Change the default password immediately after first login. For production deployments, use strong passwords and configure via environment variables.

### Production Build

```bash
# Build for production
npm run build

# Preview production build
npm run preview

# Analyze bundle size
npm run analyze
```

## Performance Optimizations

### 1. Lazy Loading
All route components are lazy-loaded for faster initial page load:
- Initial bundle: ~320 KB (gzipped)
- First Contentful Paint: ~0.9s
- Time to Interactive: ~1.4s

### 2. Code Splitting
Optimized chunk strategy:
- `react-vendor`: React core (rarely changes)
- `mui-core`: MUI components
- `mui-icons`: Icon library
- `charts`: Recharts library
- `api-*`: API modules by domain
- `page-*`: Individual page components

### 3. Virtual Scrolling
VirtualTable component for efficient rendering of large lists:
- Handles 10,000+ items without performance issues
- Constant memory usage
- Smooth 60 FPS scrolling

### 4. API Caching
In-memory cache with TTL support:
- Reduces redundant API calls
- Automatic expiration
- Pattern-based invalidation

### 5. Performance Monitoring
Track and analyze application performance:
- Page load times
- API call durations
- Component render performance
- Metrics export to console/backend

**See [PERFORMANCE.md](PERFORMANCE.md) for detailed usage guide.**

## Project Structure

```
artemis-console/
├── src/
│   ├── api/              # API client modules
│   │   ├── client.ts     # Base HTTP client
│   │   ├── auth.ts       # Authentication
│   │   ├── discovery.ts  # Service discovery
│   │   ├── cluster.ts    # Cluster management
│   │   ├── routing.ts    # Routing rules
│   │   ├── zone.ts       # Zone operations
│   │   ├── canary.ts     # Canary deployment
│   │   └── audit.ts      # Audit logs
│   │
│   ├── components/       # Reusable components
│   │   ├── Layout/       # Layout components
│   │   ├── VirtualTable.tsx    # Virtual scrolling table
│   │   └── LoadingFallback.tsx # Loading indicators
│   │
│   ├── pages/            # Page components (lazy-loaded)
│   │   ├── Dashboard/    # Dashboard page
│   │   ├── Services/     # Services management
│   │   ├── Instances/    # Instance management
│   │   ├── Cluster/      # Cluster status
│   │   ├── Routing/      # Routing configuration
│   │   ├── AuditLog/     # Audit logs
│   │   ├── ZoneOps/      # Zone operations
│   │   ├── Canary/       # Canary deployment
│   │   └── Users/        # User management
│   │
│   ├── routes/           # Route configuration
│   │   ├── index.tsx     # Main router (with lazy loading)
│   │   └── PrivateRoute.tsx # Auth guard
│   │
│   ├── hooks/            # Custom React hooks
│   │   └── useServiceCache.ts # Cached API hooks
│   │
│   ├── utils/            # Utility functions
│   │   ├── cache.ts      # API response cache
│   │   ├── performance.ts # Performance monitoring
│   │   ├── token.ts      # JWT token management
│   │   └── websocket.ts  # WebSocket client
│   │
│   ├── store/            # Zustand state management
│   │   └── authStore.ts  # Authentication state
│   │
│   └── main.tsx          # Application entry point
│
├── public/               # Static assets
├── dist/                 # Production build output
├── vite.config.ts        # Vite configuration
├── tsconfig.json         # TypeScript configuration
└── package.json          # Dependencies and scripts
```

## Technology Stack

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| **Framework** | React | 19.2.0 | UI framework |
| **Language** | TypeScript | 5.9.3 | Type safety |
| **Build Tool** | Vite | 7.3.1 | Fast builds and HMR |
| **UI Library** | MUI (Material-UI) | 7.3.8 | Component library |
| **Routing** | React Router | 7.13.0 | Client-side routing |
| **State** | Zustand | 5.0.11 | Lightweight state management |
| **HTTP** | Axios | 1.13.5 | HTTP client |
| **Charts** | Recharts | 3.7.0 | Data visualization |
| **Forms** | React Hook Form | 7.71.1 | Form validation |
| **Virtual Scroll** | react-window | 1.8.10 | List virtualization |

## Configuration

### Vite Proxy

The dev server proxies API requests to the backend:

```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8080', // Artemis server
      changeOrigin: true,
    },
  },
}
```

### Environment Variables

Create `.env.local` for custom configuration:

```bash
# API endpoint (if not using proxy)
VITE_API_URL=http://localhost:8080

# Enable performance monitoring
VITE_ENABLE_PERFORMANCE=true

# Performance metrics endpoint
VITE_METRICS_URL=http://localhost:8080/api/metrics
```

## Development Workflow

### Code Quality

```bash
# Lint code
npm run lint

# Format code
npm run format

# Type check
npm run type-check
```

### Performance Analysis

```bash
# Generate bundle visualization
npm run analyze

# View performance metrics (in browser console)
import { getPerformanceSummary } from '@/utils/performance';
console.table(getPerformanceSummary());
```

### Testing

```bash
# Run unit tests
npm test

# Run with coverage
npm run test:coverage
```

## Usage Examples

### Using VirtualTable

```typescript
import { VirtualTable } from '@/components';

<VirtualTable
  data={instances}
  columns={[
    { id: 'id', label: 'ID', render: (row) => row.id },
    { id: 'status', label: 'Status', render: (row) => <StatusChip status={row.status} /> },
  ]}
  rowHeight={52}
  height={600}
/>
```

### Using API Cache

```typescript
import { useServices } from '@/hooks/useServiceCache';

const { services, loading, refresh } = useServices({
  refreshInterval: 30000 // Auto-refresh every 30s
});
```

### Performance Monitoring

```typescript
import { measureAsync, MetricType } from '@/utils/performance';

const data = await measureAsync(
  'get_services',
  MetricType.API_CALL,
  () => api.getAllServices()
);
```

## Browser Support

- Chrome/Edge: Latest 2 versions
- Firefox: Latest 2 versions
- Safari: Latest 2 versions

## Build Output

Production build generates:
- `dist/index.html` - Entry HTML
- `dist/assets/*.js` - JavaScript chunks
- `dist/assets/*.css` - CSS files
- `dist/stats.html` - Bundle analysis (if using `npm run analyze`)

## Performance Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Initial bundle (gzipped) | <500 KB | ~320 KB |
| First Contentful Paint | <1.5s | ~0.9s |
| Time to Interactive | <2.0s | ~1.4s |
| Largest Contentful Paint | <2.5s | ~1.2s |
| Long list render (10k items) | <200ms | <100ms |

## Documentation

- **Performance Guide**: [PERFORMANCE.md](PERFORMANCE.md)
- **Cluster Page**: [CLUSTER_PAGE_IMPLEMENTATION.md](CLUSTER_PAGE_IMPLEMENTATION.md)
- **Full Docs**: [../docs/performance-optimization.md](../docs/performance-optimization.md)

## Contributing

1. Follow TypeScript strict mode
2. Use Material-UI components
3. Lazy load route components
4. Cache API responses appropriately
5. Monitor performance impact
6. Update documentation

## License

MIT OR Apache-2.0
