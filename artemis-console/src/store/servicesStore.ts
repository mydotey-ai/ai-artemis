/**
 * 服务状态管理 (Zustand Store)
 *
 * 管理所有注册的服务实例和相关数据
 * - 服务列表（使用 Map 数据结构）
 * - 加载和错误状态
 * - 服务的获取、更新、删除操作
 */

import { create } from 'zustand';
import type { Service, Instance } from '../api/types';
import { InstanceStatus } from '../api/types';

// ===== Store State Interface =====

interface ServicesStoreState {
  // 状态
  services: Map<string, Service>;
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;

  // 操作
  fetchServices: (regionId: string, zoneId: string) => Promise<void>;
  updateService: (service: Service) => void;
  addService: (service: Service) => void;
  removeService: (serviceId: string) => void;
  clearServices: () => void;
  getService: (serviceId: string) => Service | undefined;
  getServicesByStatus: (status: InstanceStatus) => Service[];
  updateInstance: (
    serviceId: string,
    instanceId: string,
    instance: Partial<Instance>
  ) => void;
  removeInstance: (serviceId: string, instanceId: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

// ===== Store Implementation =====

export const useServicesStore = create<ServicesStoreState>((set, get) => ({
  // 初始状态
  services: new Map(),
  loading: false,
  error: null,
  lastUpdated: null,

  // ===== 获取所有服务 =====
  fetchServices: async (regionId: string, zoneId: string) => {
    set({ loading: true, error: null });
    try {
      // TODO: 调用实际的服务发现 API
      // const response = await getServicesAPI({ region_id: regionId, zone_id: zoneId });
      // const services = response.data.services || [];

      // 模拟获取服务 (用于演示)
      const mockServices: Service[] = [
        {
          service_id: 'user-service',
          instances: [
            {
              region_id: regionId,
              zone_id: zoneId,
              service_id: 'user-service',
              instance_id: 'user-service-1',
              ip: '192.168.1.10',
              port: 8001,
              url: 'http://192.168.1.10:8001',
              status: InstanceStatus.UP,
              metadata: { version: '1.0.0' },
            },
            {
              region_id: regionId,
              zone_id: zoneId,
              service_id: 'user-service',
              instance_id: 'user-service-2',
              ip: '192.168.1.11',
              port: 8001,
              url: 'http://192.168.1.11:8001',
              status: InstanceStatus.UP,
              metadata: { version: '1.0.0' },
            },
          ],
        },
        {
          service_id: 'order-service',
          instances: [
            {
              region_id: regionId,
              zone_id: zoneId,
              service_id: 'order-service',
              instance_id: 'order-service-1',
              ip: '192.168.1.20',
              port: 8002,
              url: 'http://192.168.1.20:8002',
              status: InstanceStatus.UP,
              metadata: { version: '2.0.0' },
            },
          ],
        },
        {
          service_id: 'payment-service',
          instances: [
            {
              region_id: regionId,
              zone_id: zoneId,
              service_id: 'payment-service',
              instance_id: 'payment-service-1',
              ip: '192.168.1.30',
              port: 8003,
              url: 'http://192.168.1.30:8003',
              status: InstanceStatus.DOWN,
              metadata: { version: '1.5.0' },
            },
          ],
        },
      ];

      // 将服务列表转换为 Map
      const servicesMap = new Map<string, Service>();
      mockServices.forEach((service) => {
        servicesMap.set(service.service_id, service);
      });

      set({
        services: servicesMap,
        lastUpdated: Date.now(),
        loading: false,
        error: null,
      });
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Failed to fetch services';
      set({
        error: errorMessage,
        loading: false,
      });
      throw err;
    }
  },

  // ===== 更新服务 =====
  updateService: (service: Service) => {
    const { services } = get();
    const updatedServices = new Map(services);
    updatedServices.set(service.service_id, service);
    set({
      services: updatedServices,
      lastUpdated: Date.now(),
    });
  },

  // ===== 添加服务 =====
  addService: (service: Service) => {
    const { services } = get();
    if (!services.has(service.service_id)) {
      const updatedServices = new Map(services);
      updatedServices.set(service.service_id, service);
      set({
        services: updatedServices,
        lastUpdated: Date.now(),
      });
    }
  },

  // ===== 删除服务 =====
  removeService: (serviceId: string) => {
    const { services } = get();
    const updatedServices = new Map(services);
    updatedServices.delete(serviceId);
    set({
      services: updatedServices,
      lastUpdated: Date.now(),
    });
  },

  // ===== 清空所有服务 =====
  clearServices: () => {
    set({
      services: new Map(),
      lastUpdated: null,
      error: null,
    });
  },

  // ===== 获取指定服务 =====
  getService: (serviceId: string): Service | undefined => {
    const { services } = get();
    return services.get(serviceId);
  },

  // ===== 按状态获取服务 =====
  getServicesByStatus: (status: InstanceStatus): Service[] => {
    const { services } = get();
    const result: Service[] = [];

    services.forEach((service) => {
      const matchingInstances = service.instances.filter(
        (instance) => instance.status === status
      );

      if (matchingInstances.length > 0) {
        result.push({
          ...service,
          instances: matchingInstances,
        });
      }
    });

    return result;
  },

  // ===== 更新实例 =====
  updateInstance: (
    serviceId: string,
    instanceId: string,
    instanceUpdate: Partial<Instance>
  ) => {
    const { services } = get();
    const service = services.get(serviceId);

    if (service) {
      const updatedService = {
        ...service,
        instances: service.instances.map((instance) =>
          instance.instance_id === instanceId
            ? { ...instance, ...instanceUpdate }
            : instance
        ),
      };

      const updatedServices = new Map(services);
      updatedServices.set(serviceId, updatedService);

      set({
        services: updatedServices,
        lastUpdated: Date.now(),
      });
    }
  },

  // ===== 删除实例 =====
  removeInstance: (serviceId: string, instanceId: string) => {
    const { services } = get();
    const service = services.get(serviceId);

    if (service) {
      const updatedService = {
        ...service,
        instances: service.instances.filter(
          (instance) => instance.instance_id !== instanceId
        ),
      };

      // 如果没有实例了，删除整个服务
      if (updatedService.instances.length === 0) {
        const updatedServices = new Map(services);
        updatedServices.delete(serviceId);
        set({
          services: updatedServices,
          lastUpdated: Date.now(),
        });
      } else {
        const updatedServices = new Map(services);
        updatedServices.set(serviceId, updatedService);
        set({
          services: updatedServices,
          lastUpdated: Date.now(),
        });
      }
    }
  },

  // ===== 设置加载状态 =====
  setLoading: (loading: boolean) => {
    set({ loading });
  },

  // ===== 设置错误 =====
  setError: (error: string | null) => {
    set({ error });
  },
}));

// ===== Store Selectors (性能优化) =====

/**
 * 选择器：获取所有服务（作为数组）
 */
export const selectServicesArray = (state: ServicesStoreState): Service[] => {
  return Array.from(state.services.values());
};

/**
 * 选择器：获取服务数量
 */
export const selectServiceCount = (state: ServicesStoreState): number => {
  return state.services.size;
};

/**
 * 选择器：获取所有实例数量
 */
export const selectInstanceCount = (state: ServicesStoreState): number => {
  let count = 0;
  state.services.forEach((service) => {
    count += service.instances.length;
  });
  return count;
};

/**
 * 选择器：获取加载状态
 */
export const selectLoading = (state: ServicesStoreState): boolean =>
  state.loading;

/**
 * 选择器：获取错误信息
 */
export const selectError = (state: ServicesStoreState): string | null =>
  state.error;

/**
 * 选择器：获取最后更新时间
 */
export const selectLastUpdated = (state: ServicesStoreState): number | null =>
  state.lastUpdated;

/**
 * 获取统计信息
 */
export const getServicesStats = (
  state: ServicesStoreState
): {
  totalServices: number;
  totalInstances: number;
  upInstances: number;
  downInstances: number;
} => {
  let totalInstances = 0;
  let upInstances = 0;
  let downInstances = 0;

  state.services.forEach((service) => {
    service.instances.forEach((instance) => {
      totalInstances++;
      if (instance.status === InstanceStatus.UP) {
        upInstances++;
      } else if (instance.status === InstanceStatus.DOWN) {
        downInstances++;
      }
    });
  });

  return {
    totalServices: state.services.size,
    totalInstances,
    upInstances,
    downInstances,
  };
};
