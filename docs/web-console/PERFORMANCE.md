# Performance Optimization - Quick Start Guide

This guide shows how to use the performance optimization features in Artemis Console.

## 1. Using VirtualTable for Large Lists

Replace standard Material-UI tables with VirtualTable when displaying 100+ items:

### Before (Standard Table)
```typescript
<TableContainer>
  <Table>
    <TableHead>
      <TableRow>
        <TableCell>ID</TableCell>
        <TableCell>Status</TableCell>
      </TableRow>
    </TableHead>
    <TableBody>
      {instances.map((instance) => (
        <TableRow key={instance.id}>
          <TableCell>{instance.id}</TableCell>
          <TableCell>{instance.status}</TableCell>
        </TableRow>
      ))}
    </TableBody>
  </Table>
</TableContainer>
```

### After (VirtualTable)
```typescript
import { VirtualTable, VirtualTableColumn } from '@/components';

const columns: VirtualTableColumn<Instance>[] = [
  {
    id: 'id',
    label: 'Instance ID',
    width: 200,
    render: (instance) => instance.id,
  },
  {
    id: 'status',
    label: 'Status',
    width: 150,
    render: (instance) => <Chip label={instance.status} />,
  },
];

<VirtualTable
  data={instances}
  columns={columns}
  rowHeight={52}
  height={600}
  onRowClick={(instance) => navigate(`/instances/${instance.id}`)}
/>
```

## 2. Using API Cache

Cache API responses to reduce redundant calls:

### Before
```typescript
const fetchServices = async () => {
  const services = await discoveryApi.getAllServices();
  setServices(services);
};

useEffect(() => {
  fetchServices();
}, []);
```

### After
```typescript
import { cacheApi, CacheKeys, CacheInvalidators } from '@/utils/cache';

const fetchServices = async () => {
  const services = await cacheApi(
    CacheKeys.services.all(),
    () => discoveryApi.getAllServices(),
    5 * 60 * 1000 // Cache for 5 minutes
  );
  setServices(services);
};

// Invalidate cache after mutations
const registerInstance = async (instance) => {
  await discoveryApi.register(instance);
  CacheInvalidators.services(); // Clear all service caches
};
```

## 3. Using Performance Monitoring

Monitor critical operations:

### API Calls
```typescript
import { measureAsync, MetricType } from '@/utils/performance';

const fetchData = async () => {
  const data = await measureAsync(
    'get_all_services',
    MetricType.API_CALL,
    () => discoveryApi.getAllServices(),
    { endpoint: '/api/discovery/services' }
  );
  return data;
};
```

### Component Render Performance
```typescript
import { Profiler } from 'react';
import { createProfilerCallback } from '@/utils/performance';

function Dashboard() {
  return (
    <Profiler id="Dashboard" onRender={createProfilerCallback('Dashboard')}>
      {/* Dashboard content */}
    </Profiler>
  );
}
```

### View Performance Summary
```typescript
import { getPerformanceSummary } from '@/utils/performance';

// In dev console
console.table(getPerformanceSummary());
```

## 4. Bundle Analysis

Analyze bundle size and composition:

```bash
# Generate bundle visualization
npm run analyze

# This will:
# 1. Build the application
# 2. Generate stats.html in dist/
# 3. Open the visualization in your browser
```

## 5. Production Configuration

Update `/artemis-console/src/main.tsx` to configure performance monitoring:

```typescript
import { configurePerformanceMonitoring } from '@/utils/performance';

// Configure performance monitoring for production
if (import.meta.env.PROD) {
  configurePerformanceMonitoring({
    enabled: true,
    logToConsole: false, // Disable console logs in production
    sendToBackend: true, // Send metrics to backend
    backendUrl: '/api/metrics', // Metrics endpoint
    sampleRate: 0.1, // Sample 10% of metrics
  });
}
```

## 6. Lazy Loading (Already Configured)

All routes are already configured with lazy loading in `/src/routes/index.tsx`:

```typescript
// Pages are automatically lazy-loaded
const Dashboard = lazy(() => import('@/pages/Dashboard/Dashboard'));
const Services = lazy(() => import('@/pages/Services/Services'));
// ... etc

// Wrapped with Suspense
<Suspense fallback={<LoadingFallback />}>
  <Dashboard />
</Suspense>
```

## Performance Best Practices

### 1. Cache Configuration

| Data Type | Recommended TTL |
|-----------|----------------|
| Static config (regions, zones) | 10 minutes |
| Services list | 5 minutes |
| Instance status | 30 seconds |
| Real-time metrics | No cache |

### 2. Virtual Scrolling

Use VirtualTable when:
- List has 100+ items
- Row height is consistent
- Simple row interactions

Don't use when:
- List has <50 items
- Variable row heights
- Complex row interactions (drag-drop, nested components)

### 3. Performance Monitoring

In development:
- Enable all metrics
- Log to console
- Use React Profiler on slow components

In production:
- Sample 10-20% of metrics
- Send to backend
- Alert on P95 > threshold

## Troubleshooting

### Issue: VirtualTable rows not rendering
**Solution**: Ensure `rowHeight` matches actual row height. Use browser DevTools to measure.

### Issue: Cache not invalidating
**Solution**: Use `CacheInvalidators` after mutations:
```typescript
CacheInvalidators.services(); // Invalidate all service caches
```

### Issue: Bundle still too large
**Solution**:
1. Run `npm run analyze`
2. Identify largest chunks
3. Consider lazy loading or removing unused dependencies

### Issue: Performance metrics not showing
**Solution**: Check configuration:
```typescript
import { configurePerformanceMonitoring } from '@/utils/performance';

configurePerformanceMonitoring({
  enabled: true,
  logToConsole: true, // Enable for development
});
```

## Additional Resources

- **Full Documentation**: `/docs/performance-optimization.md`
- **VirtualTable API**: `/src/components/VirtualTable.tsx`
- **Cache API**: `/src/utils/cache.ts`
- **Performance API**: `/src/utils/performance.ts`
