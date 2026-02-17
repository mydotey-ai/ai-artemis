/**
 * Zone 操作 API
 *
 * 提供 Zone 级别的批量操作和管理功能
 */

import apiClient from '@/api/client';

const API_BASE = '/api/management/zone';

// ===== 请求/响应类型定义 =====

export interface OperateZoneRequest {
  zone_id: string;
  region_id: string;
  operator_id?: string;
}

export interface ZoneInfo {
  zone_id: string;
  region_id: string;
  total_instances: number;
  active_instances: number;
  status: 'ACTIVE' | 'INACTIVE';
  last_update: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== Zone 批量操作 API =====

/**
 * 拉出整个 Zone (将 Zone 中所有实例拉出)
 * POST /api/management/zone/pull-out
 */
export async function pullOutZone(request: OperateZoneRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/pull-out`, request);
  return response.data;
}

/**
 * 拉入整个 Zone (将 Zone 中所有实例拉入)
 * POST /api/management/zone/pull-in
 */
export async function pullInZone(request: OperateZoneRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/pull-in`, request);
  return response.data;
}

/**
 * 查询 Zone 操作历史
 * GET /api/management/zone/operations
 */
export async function queryZoneOperations(
  zoneId?: string,
  regionId?: string,
  limit?: number
): Promise<ApiResponse<any[]>> {
  const response = await apiClient.get(`${API_BASE}/operations`, {
    params: {
      zone_id: zoneId,
      region_id: regionId,
      limit,
    },
  });
  return response.data;
}

/**
 * 获取 Zone 信息
 * GET /api/management/zone/:zone_id
 */
export async function getZoneInfo(zoneId: string, regionId: string): Promise<ApiResponse<ZoneInfo>> {
  const response = await apiClient.get(`${API_BASE}/${zoneId}`, {
    params: { region_id: regionId },
  });
  return response.data;
}

/**
 * 列出所有 Zone
 * GET /api/management/zone
 */
export async function listZones(regionId?: string): Promise<ApiResponse<ZoneInfo[]>> {
  const response = await apiClient.get(API_BASE, {
    params: { region_id: regionId },
  });
  return response.data;
}

/**
 * 检查 Zone 中的实例是否全部拉出
 * GET /api/management/zone/:zone_id/is-down
 */
export async function isZoneDown(zoneId: string, regionId: string): Promise<ApiResponse<{ is_down: boolean }>> {
  const response = await apiClient.get(`${API_BASE}/${zoneId}/is-down`, {
    params: { region_id: regionId },
  });
  return response.data;
}

/**
 * 获取 Zone 中的实例列表
 * GET /api/management/zone/:zone_id/instances
 */
export async function getZoneInstances(
  zoneId: string,
  regionId: string,
  status?: string
): Promise<ApiResponse<any[]>> {
  const response = await apiClient.get(`${API_BASE}/${zoneId}/instances`, {
    params: {
      region_id: regionId,
      status,
    },
  });
  return response.data;
}

/**
 * 更新 Zone 状态
 * PUT /api/management/zone/:zone_id/status
 */
export async function updateZoneStatus(
  zoneId: string,
  regionId: string,
  status: 'ACTIVE' | 'INACTIVE'
): Promise<ApiResponse<any>> {
  const response = await apiClient.put(`${API_BASE}/${zoneId}/status`, {
    zone_id: zoneId,
    region_id: regionId,
    status,
  });
  return response.data;
}

/**
 * 批量拉出多个 Zone
 * POST /api/management/zone/batch-pull-out
 */
export async function batchPullOutZones(
  zones: OperateZoneRequest[]
): Promise<ApiResponse<{ failed: any[] }>> {
  const response = await apiClient.post(`${API_BASE}/batch-pull-out`, { zones });
  return response.data;
}

/**
 * 批量拉入多个 Zone
 * POST /api/management/zone/batch-pull-in
 */
export async function batchPullInZones(
  zones: OperateZoneRequest[]
): Promise<ApiResponse<{ failed: any[] }>> {
  const response = await apiClient.post(`${API_BASE}/batch-pull-in`, { zones });
  return response.data;
}
