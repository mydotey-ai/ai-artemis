/**
 * 服务发现 API
 *
 * 提供服务查询和发现功能
 */

import apiClient from '@/api/client';
import type {
  GetServiceRequest,
  GetServiceResponse,
  GetServicesRequest,
  GetServicesResponse,
  DiscoveryConfig,
} from '@/api/types';

const API_BASE = '/api/discovery';

/**
 * 获取单个服务
 * POST /api/discovery/service.json
 *
 * @param serviceId - 服务 ID
 * @param config - 发现配置（包含 regionId, zoneId 等）
 * @returns 服务详细信息
 */
export async function getService(
  serviceId: string,
  config: DiscoveryConfig
): Promise<GetServiceResponse> {
  const request: GetServiceRequest = {
    discovery_config: {
      serviceId: serviceId,
      regionId: config.regionId,
      zoneId: config.zoneId,
      discovery_data: config.discovery_data,
    },
  };

  const response = await apiClient.post<GetServiceResponse>(
    `${API_BASE}/service.json`,
    request
  );
  return response.data;
}

/**
 * 获取所有服务
 * POST /api/discovery/services.json
 *
 * @param regionId - 区域 ID
 * @param zoneId - 可用区 ID
 * @returns 所有服务列表
 */
export async function getAllServices(
  regionId: string,
  zoneId: string
): Promise<GetServicesResponse> {
  const request: GetServicesRequest = {
    regionId: regionId,
    zoneId: zoneId,
  };

  const response = await apiClient.post<GetServicesResponse>(
    `${API_BASE}/services.json`,
    request
  );
  return response.data;
}
