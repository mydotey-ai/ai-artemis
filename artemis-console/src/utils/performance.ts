/**
 * Performance monitoring utilities
 *
 * Provides tools for monitoring and reporting application performance metrics:
 * - Page load times
 * - API call durations
 * - Component render performance
 * - User interactions
 */

/**
 * Performance metric types
 */
export enum MetricType {
  PAGE_LOAD = 'page_load',
  API_CALL = 'api_call',
  COMPONENT_RENDER = 'component_render',
  USER_INTERACTION = 'user_interaction',
}

/**
 * Performance metric data
 */
export interface PerformanceMetric {
  type: MetricType;
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, unknown>;
}

/**
 * Performance monitoring configuration
 */
interface PerformanceConfig {
  enabled: boolean;
  logToConsole: boolean;
  sendToBackend: boolean;
  backendUrl?: string;
  sampleRate: number; // 0-1, percentage of metrics to report
}

/**
 * Default configuration
 */
const defaultConfig: PerformanceConfig = {
  enabled: true,
  logToConsole: import.meta.env.DEV, // Only log in development
  sendToBackend: false,
  sampleRate: 1.0, // Report 100% of metrics by default
};

let config = { ...defaultConfig };

/**
 * Performance metrics buffer
 */
const metricsBuffer: PerformanceMetric[] = [];
const MAX_BUFFER_SIZE = 100;

/**
 * Configure performance monitoring
 */
export function configurePerformanceMonitoring(options: Partial<PerformanceConfig>): void {
  config = { ...config, ...options };
}

/**
 * Record a performance metric
 */
export function recordMetric(metric: PerformanceMetric): void {
  if (!config.enabled) return;

  // Sample metrics based on sample rate
  if (Math.random() > config.sampleRate) return;

  // Add to buffer
  metricsBuffer.push(metric);

  // Trim buffer if too large
  if (metricsBuffer.length > MAX_BUFFER_SIZE) {
    metricsBuffer.shift();
  }

  // Log to console in development
  if (config.logToConsole) {
    console.log(`[Performance] ${metric.type} - ${metric.name}: ${metric.duration.toFixed(2)}ms`, {
      ...metric.metadata,
      timestamp: new Date(metric.timestamp).toISOString(),
    });
  }

  // Send to backend if configured
  if (config.sendToBackend && config.backendUrl) {
    sendMetricToBackend(metric);
  }
}

/**
 * Send metric to backend (async, non-blocking)
 */
async function sendMetricToBackend(metric: PerformanceMetric): Promise<void> {
  if (!config.backendUrl) return;

  try {
    await fetch(config.backendUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(metric),
      keepalive: true, // Allow request to complete even if page is closing
    });
  } catch (error) {
    // Silently fail - don't disrupt user experience
    if (import.meta.env.DEV) {
      console.error('Failed to send performance metric:', error);
    }
  }
}

/**
 * Get all recorded metrics
 */
export function getMetrics(): PerformanceMetric[] {
  return [...metricsBuffer];
}

/**
 * Clear all recorded metrics
 */
export function clearMetrics(): void {
  metricsBuffer.length = 0;
}

/**
 * Measure execution time of a function
 */
export async function measureAsync<T>(
  name: string,
  type: MetricType,
  fn: () => Promise<T>,
  metadata?: Record<string, unknown>,
): Promise<T> {
  const startTime = performance.now();

  try {
    const result = await fn();
    const duration = performance.now() - startTime;

    recordMetric({
      type,
      name,
      duration,
      timestamp: Date.now(),
      metadata: {
        ...metadata,
        success: true,
      },
    });

    return result;
  } catch (error) {
    const duration = performance.now() - startTime;

    recordMetric({
      type,
      name,
      duration,
      timestamp: Date.now(),
      metadata: {
        ...metadata,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      },
    });

    throw error;
  }
}

/**
 * Measure execution time of a synchronous function
 */
export function measureSync<T>(
  name: string,
  type: MetricType,
  fn: () => T,
  metadata?: Record<string, unknown>,
): T {
  const startTime = performance.now();

  try {
    const result = fn();
    const duration = performance.now() - startTime;

    recordMetric({
      type,
      name,
      duration,
      timestamp: Date.now(),
      metadata: {
        ...metadata,
        success: true,
      },
    });

    return result;
  } catch (error) {
    const duration = performance.now() - startTime;

    recordMetric({
      type,
      name,
      duration,
      timestamp: Date.now(),
      metadata: {
        ...metadata,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      },
    });

    throw error;
  }
}

/**
 * Start a performance timer
 */
export function startTimer(name: string): () => void {
  const startTime = performance.now();

  return () => {
    const duration = performance.now() - startTime;
    return duration;
  };
}

/**
 * Monitor page load performance using Navigation Timing API
 */
export function monitorPageLoad(): void {
  if (typeof window === 'undefined' || !window.performance) return;

  // Wait for page to fully load
  window.addEventListener('load', () => {
    setTimeout(() => {
      const perfData = window.performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;

      if (!perfData) return;

      // DNS lookup time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'dns_lookup',
        duration: perfData.domainLookupEnd - perfData.domainLookupStart,
        timestamp: Date.now(),
      });

      // TCP connection time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'tcp_connection',
        duration: perfData.connectEnd - perfData.connectStart,
        timestamp: Date.now(),
      });

      // Request time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'request',
        duration: perfData.responseStart - perfData.requestStart,
        timestamp: Date.now(),
      });

      // Response time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'response',
        duration: perfData.responseEnd - perfData.responseStart,
        timestamp: Date.now(),
      });

      // DOM processing time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'dom_processing',
        duration: perfData.domComplete - perfData.domLoading,
        timestamp: Date.now(),
      });

      // Total page load time
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'total_load',
        duration: perfData.loadEventEnd - perfData.fetchStart,
        timestamp: Date.now(),
      });

      // DOM Content Loaded
      recordMetric({
        type: MetricType.PAGE_LOAD,
        name: 'dom_content_loaded',
        duration: perfData.domContentLoadedEventEnd - perfData.fetchStart,
        timestamp: Date.now(),
      });
    }, 0);
  });
}

/**
 * Get performance summary statistics
 */
export function getPerformanceSummary(): Record<string, {
  count: number;
  avg: number;
  min: number;
  max: number;
  p95: number;
}> {
  const summary: Record<string, {
    count: number;
    avg: number;
    min: number;
    max: number;
    p95: number;
  }> = {};

  // Group metrics by type and name
  const groups: Record<string, number[]> = {};

  for (const metric of metricsBuffer) {
    const key = `${metric.type}:${metric.name}`;
    if (!groups[key]) {
      groups[key] = [];
    }
    groups[key].push(metric.duration);
  }

  // Calculate statistics for each group
  for (const [key, durations] of Object.entries(groups)) {
    const sorted = [...durations].sort((a, b) => a - b);
    const sum = durations.reduce((a, b) => a + b, 0);
    const p95Index = Math.floor(sorted.length * 0.95);

    summary[key] = {
      count: durations.length,
      avg: sum / durations.length,
      min: sorted[0],
      max: sorted[sorted.length - 1],
      p95: sorted[p95Index] || sorted[sorted.length - 1],
    };
  }

  return summary;
}

/**
 * React Profiler callback type
 */
export type ProfilerCallback = (
  id: string,
  phase: 'mount' | 'update',
  actualDuration: number,
  baseDuration: number,
  startTime: number,
  commitTime: number,
) => void;

/**
 * Create a React Profiler callback that records component render metrics
 */
export function createProfilerCallback(componentName: string): ProfilerCallback {
  return (id, phase, actualDuration) => {
    recordMetric({
      type: MetricType.COMPONENT_RENDER,
      name: componentName,
      duration: actualDuration,
      timestamp: Date.now(),
      metadata: {
        id,
        phase,
      },
    });
  };
}

// Initialize page load monitoring
if (typeof window !== 'undefined') {
  monitorPageLoad();
}

export default {
  configure: configurePerformanceMonitoring,
  record: recordMetric,
  measureAsync,
  measureSync,
  startTimer,
  getMetrics,
  clearMetrics,
  getSummary: getPerformanceSummary,
  createProfilerCallback,
};
