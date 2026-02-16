/**
 * 金丝雀发布 API
 *
 * 提供金丝雀发布（灰度发布）配置和管理功能
 */

import axios from 'axios';

const API_BASE = '/api/management/canary';

// ===== 请求/响应类型定义 =====

export interface SetCanaryConfigRequest {
  service_id: string;
  ip_whitelist: string[];
  description?: string;
}

export interface CanaryConfig {
  service_id: string;
  ip_whitelist: string[];
  enabled: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface EnableCanaryRequest {
  service_id: string;
  enabled: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== 金丝雀配置管理 API =====

/**
 * 设置金丝雀配置
 * POST /api/management/canary/config
 */
export async function setCanaryConfig(
  request: SetCanaryConfigRequest
): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.post(`${API_BASE}/config`, request);
  return response.data;
}

/**
 * 获取金丝雀配置
 * GET /api/management/canary/config/:service_id
 */
export async function getCanaryConfig(serviceId: string): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.get(`${API_BASE}/config/${serviceId}`);
  return response.data;
}

/**
 * 删除金丝雀配置
 * DELETE /api/management/canary/config/:service_id
 */
export async function deleteCanaryConfig(serviceId: string): Promise<ApiResponse<any>> {
  const response = await axios.delete(`${API_BASE}/config/${serviceId}`);
  return response.data;
}

/**
 * 列出所有金丝雀配置
 * GET /api/management/canary/configs
 */
export async function listCanaryConfigs(): Promise<ApiResponse<CanaryConfig[]>> {
  const response = await axios.get(`${API_BASE}/configs`);
  return response.data;
}

// ===== 金丝雀启用/禁用 API =====

/**
 * 启用金丝雀发布
 * POST /api/management/canary/enable
 */
export async function enableCanary(
  request: EnableCanaryRequest
): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.post(`${API_BASE}/enable`, request);
  return response.data;
}

/**
 * 禁用金丝雀发布
 * POST /api/management/canary/disable
 */
export async function disableCanary(serviceId: string): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.post(`${API_BASE}/disable`, { service_id: serviceId });
  return response.data;
}

/**
 * 检查金丝雀是否启用
 * GET /api/management/canary/enabled/:service_id
 */
export async function isCanaryEnabled(serviceId: string): Promise<ApiResponse<{ enabled: boolean }>> {
  const response = await axios.get(`${API_BASE}/enabled/${serviceId}`);
  return response.data;
}

// ===== IP 白名单管理 API =====

/**
 * 添加 IP 到白名单
 * POST /api/management/canary/:service_id/whitelist/add
 */
export async function addIpToWhitelist(
  serviceId: string,
  ips: string[]
): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.post(`${API_BASE}/${serviceId}/whitelist/add`, { ips });
  return response.data;
}

/**
 * 从白名单移除 IP
 * POST /api/management/canary/:service_id/whitelist/remove
 */
export async function removeIpFromWhitelist(
  serviceId: string,
  ips: string[]
): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.post(`${API_BASE}/${serviceId}/whitelist/remove`, { ips });
  return response.data;
}

/**
 * 获取白名单中的 IP
 * GET /api/management/canary/:service_id/whitelist
 */
export async function getWhitelistIps(serviceId: string): Promise<ApiResponse<string[]>> {
  const response = await axios.get(`${API_BASE}/${serviceId}/whitelist`);
  return response.data;
}

/**
 * 检查 IP 是否在白名单中
 * GET /api/management/canary/:service_id/whitelist/check/:ip
 */
export async function checkIpInWhitelist(
  serviceId: string,
  ip: string
): Promise<ApiResponse<{ in_whitelist: boolean }>> {
  const response = await axios.get(`${API_BASE}/${serviceId}/whitelist/check/${ip}`);
  return response.data;
}

/**
 * 清空白名单
 * DELETE /api/management/canary/:service_id/whitelist
 */
export async function clearWhitelist(serviceId: string): Promise<ApiResponse<CanaryConfig>> {
  const response = await axios.delete(`${API_BASE}/${serviceId}/whitelist`);
  return response.data;
}

// ===== 金丝雀统计 API =====

/**
 * 获取金丝雀发布统计
 * GET /api/management/canary/stats
 */
export async function getCanaryStats(): Promise<
  ApiResponse<{
    total_services: number;
    enabled_count: number;
    total_whitelisted_ips: number;
  }>
> {
  const response = await axios.get(`${API_BASE}/stats`);
  return response.data;
}

/**
 * 获取服务的金丝雀统计
 * GET /api/management/canary/:service_id/stats
 */
export async function getServiceCanaryStats(
  serviceId: string
): Promise<
  ApiResponse<{
    service_id: string;
    enabled: boolean;
    whitelist_count: number;
    last_update: string;
  }>
> {
  const response = await axios.get(`${API_BASE}/${serviceId}/stats`);
  return response.data;
}
