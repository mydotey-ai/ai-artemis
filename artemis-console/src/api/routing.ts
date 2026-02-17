/**
 * 分组和路由规则 API
 *
 * 提供分组管理和路由规则配置的 HTTP API
 */

import apiClient from '@/api/client';

const API_BASE = '/api/routing';

// ===== 请求/响应类型定义 =====

export interface GroupType {
  type: 'WEIGHT' | 'OTHER';
}

export interface GroupStatus {
  status: 'ACTIVE' | 'INACTIVE';
}

export interface RouteStrategy {
  strategy: 'WEIGHT_ROUND_ROBIN' | 'CONSISTENT_HASH';
}

export interface RouteRuleStatus {
  status: 'ACTIVE' | 'INACTIVE';
}

export interface CreateGroupRequest {
  service_id: string;
  region_id: string;
  zone_id: string;
  name: string;
  group_type: GroupType;
  description?: string;
}

export interface CreateRuleRequest {
  route_id: string;
  service_id: string;
  name: string;
  description?: string;
  strategy: RouteStrategy;
}

export interface AddRuleGroupRequest {
  group_id: string;
  weight: number;
  region_id?: string;
  zone_id?: string;
}

export interface UpdateGroupRequest {
  description?: string;
  status?: GroupStatus;
}

export interface UpdateRuleRequest {
  name?: string;
  description?: string;
  strategy?: RouteStrategy;
}

export interface UpdateRuleGroupRequest {
  weight: number;
}

export interface ListGroupsQuery {
  service_id?: string;
  region_id?: string;
}

export interface GetGroupInstancesQuery {
  region_id?: string;
  zone_id?: string;
}

export interface GroupTag {
  key: string;
  value: string;
}

export interface AddGroupTagsRequest {
  tags: GroupTag[];
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== 分组管理 API =====

/**
 * 创建分组
 * POST /api/management/routing/groups
 */
export async function createGroup(request: CreateGroupRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/groups`, request);
  return response.data;
}

/**
 * 列出分组
 * GET /api/management/routing/groups
 */
export async function listGroups(query?: ListGroupsQuery): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/groups`, { params: query });
  return response.data;
}

/**
 * 获取分组详情
 * GET /api/management/routing/groups/:group_id
 */
export async function getGroup(groupId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/groups/${groupId}`);
  return response.data;
}

/**
 * 更新分组
 * PUT /api/management/routing/groups/:group_id
 */
export async function updateGroup(groupId: string, request: UpdateGroupRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.put(`${API_BASE}/groups/${groupId}`, request);
  return response.data;
}

/**
 * 删除分组
 * DELETE /api/management/routing/groups/:group_id
 */
export async function deleteGroup(groupId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.delete(`${API_BASE}/groups/${groupId}`);
  return response.data;
}

/**
 * 获取分组下的实例
 * GET /api/management/routing/groups/:group_id/instances
 */
export async function getGroupInstances(
  groupId: string,
  query?: GetGroupInstancesQuery
): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/groups/${groupId}/instances`, {
    params: query,
  });
  return response.data;
}

/**
 * 添加分组标签
 * POST /api/management/routing/groups/:group_id/tags
 */
export async function addGroupTags(
  groupId: string,
  request: AddGroupTagsRequest
): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/groups/${groupId}/tags`, request);
  return response.data;
}

/**
 * 移除分组标签
 * DELETE /api/management/routing/groups/:group_id/tags/:tag_key
 */
export async function removeGroupTag(groupId: string, tagKey: string): Promise<ApiResponse<any>> {
  const response = await apiClient.delete(`${API_BASE}/groups/${groupId}/tags/${tagKey}`);
  return response.data;
}

// ===== 路由规则管理 API =====

/**
 * 创建路由规则
 * POST /api/management/routing/rules
 */
export async function createRule(request: CreateRuleRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/rules`, request);
  return response.data;
}

/**
 * 列出路由规则
 * GET /api/management/routing/rules
 */
export async function listRules(serviceId?: string): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/rules`, {
    params: { service_id: serviceId },
  });
  return response.data;
}

/**
 * 获取路由规则详情
 * GET /api/management/routing/rules/:rule_id
 */
export async function getRule(ruleId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/rules/${ruleId}`);
  return response.data;
}

/**
 * 更新路由规则
 * PUT /api/management/routing/rules/:rule_id
 */
export async function updateRule(ruleId: string, request: UpdateRuleRequest): Promise<ApiResponse<any>> {
  const response = await apiClient.put(`${API_BASE}/rules/${ruleId}`, request);
  return response.data;
}

/**
 * 删除路由规则
 * DELETE /api/management/routing/rules/:rule_id
 */
export async function deleteRule(ruleId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.delete(`${API_BASE}/rules/${ruleId}`);
  return response.data;
}

// ===== 规则分组管理 API =====

/**
 * 添加分组到规则
 * POST /api/management/routing/rules/:rule_id/groups
 */
export async function addRuleGroup(
  ruleId: string,
  request: AddRuleGroupRequest
): Promise<ApiResponse<any>> {
  const response = await apiClient.post(`${API_BASE}/rules/${ruleId}/groups`, request);
  return response.data;
}

/**
 * 列出规则的分组
 * GET /api/management/routing/rules/:rule_id/groups
 */
export async function listRuleGroups(ruleId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.get(`${API_BASE}/rules/${ruleId}/groups`);
  return response.data;
}

/**
 * 更新规则分组权重
 * PUT /api/management/routing/rules/:rule_id/groups/:group_id
 */
export async function updateRuleGroup(
  ruleId: string,
  groupId: string,
  request: UpdateRuleGroupRequest
): Promise<ApiResponse<any>> {
  const response = await apiClient.put(
    `${API_BASE}/rules/${ruleId}/groups/${groupId}`,
    request
  );
  return response.data;
}

/**
 * 从规则中移除分组
 * DELETE /api/management/routing/rules/:rule_id/groups/:group_id
 */
export async function removeRuleGroup(ruleId: string, groupId: string): Promise<ApiResponse<any>> {
  const response = await apiClient.delete(`${API_BASE}/rules/${ruleId}/groups/${groupId}`);
  return response.data;
}
