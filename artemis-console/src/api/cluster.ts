/**
 * 集群状态 API
 *
 * 提供集群节点状态、健康检查和集群统计信息的查询
 */

import apiClient from '@/api/client';

const API_BASE = '/api/status';

// ===== 请求/响应类型定义 =====

export interface ClusterNodeStatus {
  node_id: string;
  host: string;
  port: number;
  status: 'ACTIVE' | 'INACTIVE' | 'SUSPECTED';
  last_heartbeat: string;
  region_id: string;
  zone_id: string;
}

export interface ClusterStatus {
  cluster_id: string;
  total_nodes: number;
  active_nodes: number;
  suspected_nodes: number;
  inactive_nodes: number;
  total_instances: number;
  total_services: number;
  timestamp: string;
}

export interface ConfigStatus {
  total_groups: number;
  total_rules: number;
  total_zones: number;
  last_update: string;
}

export interface DeploymentStatus {
  version: string;
  build_time: string;
  deployment_date: string;
}

export interface LeasesStatus {
  total_leases: number;
  active_leases: number;
  expired_leases: number;
  expiring_soon: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== 集群节点状态 API =====

/**
 * 获取集群节点状态 (POST)
 * POST /api/status/node.json
 */
export async function getClusterNodeStatusPost(): Promise<ApiResponse<ClusterNodeStatus[]>> {
  const response = await apiClient.post(`${API_BASE}/node.json`);
  return response.data;
}

/**
 * 获取集群节点状态 (GET)
 * GET /api/status/node.json
 */
export async function getClusterNodeStatusGet(): Promise<ApiResponse<ClusterNodeStatus[]>> {
  const response = await apiClient.get(`${API_BASE}/node.json`);
  return response.data;
}

/**
 * 获取集群节点状态 (统一方法)
 */
export async function getClusterNodeStatus(): Promise<ApiResponse<ClusterNodeStatus[]>> {
  return getClusterNodeStatusGet();
}

// ===== 集群整体状态 API =====

/**
 * 获取集群状态 (POST)
 * POST /api/status/cluster.json
 */
export async function getClusterStatusPost(): Promise<ApiResponse<ClusterStatus>> {
  const response = await apiClient.post(`${API_BASE}/cluster.json`);
  return response.data;
}

/**
 * 获取集群状态 (GET)
 * GET /api/status/cluster.json
 */
export async function getClusterStatusGet(): Promise<ApiResponse<ClusterStatus>> {
  const response = await apiClient.get(`${API_BASE}/cluster.json`);
  return response.data;
}

/**
 * 获取集群状态 (统一方法)
 */
export async function getClusterStatus(): Promise<ApiResponse<ClusterStatus>> {
  return getClusterStatusGet();
}

// ===== 配置状态 API =====

/**
 * 获取配置状态 (POST)
 * POST /api/status/config.json
 */
export async function getConfigStatusPost(): Promise<ApiResponse<ConfigStatus>> {
  const response = await apiClient.post(`${API_BASE}/config.json`);
  return response.data;
}

/**
 * 获取配置状态 (GET)
 * GET /api/status/config.json
 */
export async function getConfigStatusGet(): Promise<ApiResponse<ConfigStatus>> {
  const response = await apiClient.get(`${API_BASE}/config.json`);
  return response.data;
}

/**
 * 获取配置状态 (统一方法)
 */
export async function getConfigStatus(): Promise<ApiResponse<ConfigStatus>> {
  return getConfigStatusGet();
}

// ===== 部署状态 API =====

/**
 * 获取部署状态 (POST)
 * POST /api/status/deployment.json
 */
export async function getDeploymentStatusPost(): Promise<ApiResponse<DeploymentStatus>> {
  const response = await apiClient.post(`${API_BASE}/deployment.json`);
  return response.data;
}

/**
 * 获取部署状态 (GET)
 * GET /api/status/deployment.json
 */
export async function getDeploymentStatusGet(): Promise<ApiResponse<DeploymentStatus>> {
  const response = await apiClient.get(`${API_BASE}/deployment.json`);
  return response.data;
}

/**
 * 获取部署状态 (统一方法)
 */
export async function getDeploymentStatus(): Promise<ApiResponse<DeploymentStatus>> {
  return getDeploymentStatusGet();
}

// ===== 租约状态 API =====

/**
 * 获取租约状态 (POST)
 * POST /api/status/leases.json
 */
export async function getLeasesStatusPost(): Promise<ApiResponse<LeasesStatus>> {
  const response = await apiClient.post(`${API_BASE}/leases.json`);
  return response.data;
}

/**
 * 获取租约状态 (GET)
 * GET /api/status/leases.json
 */
export async function getLeasesStatusGet(): Promise<ApiResponse<LeasesStatus>> {
  const response = await apiClient.get(`${API_BASE}/leases.json`);
  return response.data;
}

/**
 * 获取租约状态 (统一方法)
 */
export async function getLeasesStatus(): Promise<ApiResponse<LeasesStatus>> {
  return getLeasesStatusGet();
}

// ===== 健康检查 API =====

/**
 * 健康检查
 * GET /health
 */
export async function healthCheck(): Promise<ApiResponse<{ status: string }>> {
  const response = await apiClient.get('/health');
  return response.data;
}

/**
 * 就绪检查
 * GET /ready
 */
export async function readyCheck(): Promise<ApiResponse<{ ready: boolean }>> {
  const response = await apiClient.get('/ready');
  return response.data;
}
