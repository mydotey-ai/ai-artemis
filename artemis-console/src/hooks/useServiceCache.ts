/**
 * Custom hook for cached service discovery API calls
 *
 * Provides easy-to-use hooks that automatically cache API responses
 * and handle cache invalidation
 */

import { useState, useEffect, useCallback } from 'react';
import { cacheApi, CacheKeys, CacheInvalidators } from '@/utils/cache';
import { measureAsync, MetricType } from '@/utils/performance';
import * as discoveryApi from '@/api/discovery';
import { Service, ServiceInstance } from '@/api/types';

/**
 * Hook for fetching all services with caching
 *
 * @param options - Cache and refresh options
 * @returns Services data, loading state, and refresh function
 */
export function useServices(options?: {
  /** Cache TTL in milliseconds (default: 5 minutes) */
  ttl?: number;
  /** Auto-refresh interval in milliseconds (disabled by default) */
  refreshInterval?: number;
}) {
  const [services, setServices] = useState<Service[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const ttl = options?.ttl ?? 5 * 60 * 1000; // Default 5 minutes

  const fetchServices = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const data = await measureAsync(
        'get_all_services',
        MetricType.API_CALL,
        () =>
          cacheApi(CacheKeys.services.all(), () => discoveryApi.getAllServices(), ttl),
        { cached: true }
      );

      setServices(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to fetch services'));
    } finally {
      setLoading(false);
    }
  }, [ttl]);

  useEffect(() => {
    fetchServices();

    // Auto-refresh if configured
    if (options?.refreshInterval) {
      const interval = setInterval(fetchServices, options.refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchServices, options?.refreshInterval]);

  return {
    services,
    loading,
    error,
    refresh: fetchServices,
  };
}

/**
 * Hook for fetching service instances with caching
 *
 * @param serviceId - Service ID to fetch instances for
 * @param options - Cache and refresh options
 * @returns Instances data, loading state, and refresh function
 */
export function useServiceInstances(
  serviceId: string,
  options?: {
    /** Cache TTL in milliseconds (default: 30 seconds) */
    ttl?: number;
    /** Auto-refresh interval in milliseconds (disabled by default) */
    refreshInterval?: number;
  }
) {
  const [instances, setInstances] = useState<ServiceInstance[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const ttl = options?.ttl ?? 30 * 1000; // Default 30 seconds (more dynamic data)

  const fetchInstances = useCallback(async () => {
    if (!serviceId) return;

    try {
      setLoading(true);
      setError(null);

      const data = await measureAsync(
        'get_service_instances',
        MetricType.API_CALL,
        () =>
          cacheApi(
            CacheKeys.services.instances(serviceId),
            () => discoveryApi.getServiceInstances(serviceId),
            ttl
          ),
        { serviceId, cached: true }
      );

      setInstances(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to fetch instances'));
    } finally {
      setLoading(false);
    }
  }, [serviceId, ttl]);

  useEffect(() => {
    fetchInstances();

    // Auto-refresh if configured
    if (options?.refreshInterval) {
      const interval = setInterval(fetchInstances, options.refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchInstances, options?.refreshInterval]);

  return {
    instances,
    loading,
    error,
    refresh: fetchInstances,
  };
}

/**
 * Hook for service mutations with cache invalidation
 *
 * Provides functions for registering/deregistering instances
 * and automatically invalidates caches
 */
export function useServiceMutations() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const registerInstance = useCallback(async (instance: ServiceInstance) => {
    try {
      setLoading(true);
      setError(null);

      await measureAsync(
        'register_instance',
        MetricType.API_CALL,
        () => discoveryApi.registerInstance(instance)
      );

      // Invalidate caches
      CacheInvalidators.services();

      return true;
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to register instance'));
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const deregisterInstance = useCallback(async (serviceId: string, instanceId: string) => {
    try {
      setLoading(true);
      setError(null);

      await measureAsync(
        'deregister_instance',
        MetricType.API_CALL,
        () => discoveryApi.deregisterInstance(serviceId, instanceId)
      );

      // Invalidate caches
      CacheInvalidators.service(serviceId);

      return true;
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to deregister instance'));
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    registerInstance,
    deregisterInstance,
    loading,
    error,
  };
}

/**
 * Example usage in a component:
 *
 * ```typescript
 * function ServicesPage() {
 *   const { services, loading, refresh } = useServices({
 *     refreshInterval: 30000 // Auto-refresh every 30 seconds
 *   });
 *
 *   const { registerInstance } = useServiceMutations();
 *
 *   const handleRegister = async (instance) => {
 *     const success = await registerInstance(instance);
 *     if (success) {
 *       refresh(); // Refresh the list
 *     }
 *   };
 *
 *   if (loading) return <CircularProgress />;
 *
 *   return (
 *     <div>
 *       {services.map(service => (
 *         <ServiceCard key={service.id} service={service} />
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
