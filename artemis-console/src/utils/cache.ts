/**
 * API response caching utility
 *
 * Provides a simple in-memory cache for API responses with TTL support
 * Helps reduce unnecessary API calls and improve performance
 */

/**
 * Cache entry with expiration time
 */
interface CacheEntry<T> {
  data: T;
  expiresAt: number;
}

/**
 * Cache manager for API responses
 */
class CacheManager {
  private cache = new Map<string, CacheEntry<unknown>>();
  private defaultTTL = 5 * 60 * 1000; // 5 minutes

  /**
   * Get cached data if available and not expired
   *
   * @param key - Cache key
   * @returns Cached data or null if not found/expired
   */
  get<T>(key: string): T | null {
    const entry = this.cache.get(key) as CacheEntry<T> | undefined;

    if (!entry) {
      return null;
    }

    // Check if expired
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Set cached data with TTL
   *
   * @param key - Cache key
   * @param data - Data to cache
   * @param ttl - Time to live in milliseconds (optional, uses default if not provided)
   */
  set<T>(key: string, data: T, ttl?: number): void {
    const expiresAt = Date.now() + (ttl ?? this.defaultTTL);
    this.cache.set(key, { data, expiresAt });
  }

  /**
   * Invalidate specific cache key
   *
   * @param key - Cache key to invalidate
   */
  invalidate(key: string): void {
    this.cache.delete(key);
  }

  /**
   * Invalidate all cache keys matching a pattern
   *
   * @param pattern - RegExp pattern to match keys
   */
  invalidatePattern(pattern: RegExp): void {
    const keys = Array.from(this.cache.keys());
    for (const key of keys) {
      if (pattern.test(key)) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Clear all cached data
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache size (number of entries)
   */
  size(): number {
    return this.cache.size;
  }

  /**
   * Clean up expired entries
   */
  cleanup(): void {
    const now = Date.now();
    const keys = Array.from(this.cache.keys());
    for (const key of keys) {
      const entry = this.cache.get(key);
      if (entry && now > entry.expiresAt) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Set default TTL for all cache entries
   *
   * @param ttl - Default TTL in milliseconds
   */
  setDefaultTTL(ttl: number): void {
    this.defaultTTL = ttl;
  }
}

// Global cache instance
export const apiCache = new CacheManager();

// Cleanup expired entries every 5 minutes
if (typeof window !== 'undefined') {
  setInterval(() => {
    apiCache.cleanup();
  }, 5 * 60 * 1000);
}

/**
 * Cache wrapper for API calls
 *
 * Automatically caches API responses and returns cached data if available
 *
 * @param key - Unique cache key
 * @param fetcher - Async function to fetch data if not cached
 * @param ttl - Time to live in milliseconds (optional)
 * @returns Cached or fresh data
 *
 * @example
 * ```ts
 * const services = await cacheApi(
 *   'services',
 *   () => discoveryApi.getAllServices(),
 *   60000 // 1 minute
 * );
 * ```
 */
export async function cacheApi<T>(
  key: string,
  fetcher: () => Promise<T>,
  ttl?: number,
): Promise<T> {
  // Try to get from cache
  const cached = apiCache.get<T>(key);
  if (cached !== null) {
    return cached;
  }

  // Fetch fresh data
  const data = await fetcher();

  // Cache the result
  apiCache.set(key, data, ttl);

  return data;
}

/**
 * Cache key builders for common API patterns
 */
export const CacheKeys = {
  /**
   * Service discovery cache keys
   */
  services: {
    all: () => 'services:all',
    byId: (serviceId: string) => `services:${serviceId}`,
    instances: (serviceId: string) => `services:${serviceId}:instances`,
  },

  /**
   * Instance cache keys
   */
  instances: {
    all: () => 'instances:all',
    byId: (instanceId: string) => `instances:${instanceId}`,
  },

  /**
   * Cluster cache keys
   */
  cluster: {
    nodes: () => 'cluster:nodes',
    status: () => 'cluster:status',
  },

  /**
   * Routing cache keys
   */
  routing: {
    groups: () => 'routing:groups',
    rules: (groupId: string) => `routing:groups:${groupId}:rules`,
  },

  /**
   * Zone operations cache keys
   */
  zone: {
    operations: () => 'zone:operations',
    byId: (operationId: string) => `zone:operations:${operationId}`,
  },

  /**
   * Canary cache keys
   */
  canary: {
    configs: () => 'canary:configs',
    byService: (serviceId: string) => `canary:${serviceId}`,
  },

  /**
   * Audit log cache keys
   */
  audit: {
    logs: (page: number) => `audit:logs:page:${page}`,
  },
};

/**
 * Cache invalidation helpers
 */
export const CacheInvalidators = {
  /**
   * Invalidate all service-related caches
   */
  services: () => {
    apiCache.invalidatePattern(/^services:/);
  },

  /**
   * Invalidate all instance-related caches
   */
  instances: () => {
    apiCache.invalidatePattern(/^instances:/);
  },

  /**
   * Invalidate specific service cache
   */
  service: (serviceId: string) => {
    apiCache.invalidate(CacheKeys.services.byId(serviceId));
    apiCache.invalidate(CacheKeys.services.instances(serviceId));
  },

  /**
   * Invalidate cluster caches
   */
  cluster: () => {
    apiCache.invalidatePattern(/^cluster:/);
  },

  /**
   * Invalidate routing caches
   */
  routing: () => {
    apiCache.invalidatePattern(/^routing:/);
  },

  /**
   * Invalidate zone operation caches
   */
  zone: () => {
    apiCache.invalidatePattern(/^zone:/);
  },

  /**
   * Invalidate canary caches
   */
  canary: () => {
    apiCache.invalidatePattern(/^canary:/);
  },

  /**
   * Invalidate audit log caches
   */
  audit: () => {
    apiCache.invalidatePattern(/^audit:/);
  },
};

export default apiCache;
