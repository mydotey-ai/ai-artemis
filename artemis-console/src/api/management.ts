/**
 * Artemis Management API
 *
 * 实例管理和服务器管理的 API 封装
 * - 实例拉入/拉出操作
 * - 服务器批量操作
 * - 操作历史查询
 */

import apiClient from '@/api/client';
import type { ResponseStatus } from '@/api/types';

/**
 * 实例键 (唯一标识一个实例)
 * 与后端 artemis-common/src/model/instance.rs 中 InstanceKey 保持一致
 */
export interface InstanceKey {
  regionId: string;
  zoneId: string;
  serviceId: string;
  groupId: string;
  instanceId: string;
}

/**
 * 实例操作类型
 */
export const InstanceOperationType = {
  PullIn: 'pullin',
  PullOut: 'pullout',
} as const;

export type InstanceOperationType = typeof InstanceOperationType[keyof typeof InstanceOperationType];

/**
 * 服务器操作类型
 */
export const ServerOperationType = {
  PullIn: 'pullin',
  PullOut: 'pullout',
} as const;

export type ServerOperationType = typeof ServerOperationType[keyof typeof ServerOperationType];

/**
 * 实例操作记录
 */
export interface InstanceOperationRecord {
  instanceKey: InstanceKey;
  operation: InstanceOperationType;
  operationComplete: boolean;
  operatorId: string;
  token?: string;
}

/**
 * 服务器操作记录 (用于查询返回)
 */
export interface ServerOperationInfo {
  serverId: string;
  regionId: string;
  operation: ServerOperationType;
}

// ========== 实例管理 API ==========

/**
 * 操作实例请求
 */
export interface OperateInstanceRequest {
  instanceKey: InstanceKey;
  operation: InstanceOperationType;
  operationComplete?: boolean;
  operatorId: string;
  token?: string;
}

/**
 * 操作实例响应
 */
export interface OperateInstanceResponse {
  status: ResponseStatus;
}

/**
 * 拉出/拉入实例
 *
 * POST /api/management/instance/operate-instance.json
 *
 * @param instance_key - 实例键
 * @param operation - 操作类型 (pullout/pullin)
 * @param operatorId - 操作人 ID
 * @param operation_complete - 操作是否完成 (默认 false)
 * @returns 操作结果
 *
 * @example
 * // 拉出实例
 * await operateInstance(instanceKey, InstanceOperationType.PullOut, 'admin');
 *
 * // 拉入实例
 * await operateInstance(instanceKey, InstanceOperationType.PullIn, 'admin');
 */
export async function operateInstance(
  instanceKey: InstanceKey,
  operation: InstanceOperationType,
  operatorId: string,
  operationComplete: boolean = false
): Promise<OperateInstanceResponse> {
  const request: OperateInstanceRequest = {
    instanceKey,
    operation,
    operatorId,
    operationComplete,
  };

  try {
    const response = await apiClient.post(
      '/api/management/instance/operate-instance.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to operate instance:', error);
    throw error;
  }
}

/**
 * 查询实例操作历史请求
 */
export interface GetInstanceOperationsRequest {
  instanceKey: InstanceKey;
}

/**
 * 查询实例操作历史响应
 */
export interface GetInstanceOperationsResponse {
  status: ResponseStatus;
  operations: InstanceOperationType[];
}

/**
 * 查询实例操作历史
 *
 * POST /api/management/instance/get-instance-operations.json
 *
 * 获取一个实例的所有操作记录
 *
 * @param instance_key - 实例键
 * @returns 操作历史列表
 *
 * @example
 * const history = await getInstanceOperations(instanceKey);
 * console.log(history.operations); // ['pullout', 'pullin', ...]
 */
export async function getInstanceOperations(
  instanceKey: InstanceKey
): Promise<GetInstanceOperationsResponse> {
  const request: GetInstanceOperationsRequest = {
    instanceKey,
  };

  try {
    const response = await apiClient.post(
      '/api/management/instance/get-instance-operations.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to get instance operations:', error);
    throw error;
  }
}

/**
 * 查询实例是否被拉出请求
 */
export interface IsInstanceDownRequest {
  instanceKey: InstanceKey;
}

/**
 * 查询实例是否被拉出响应
 */
export interface IsInstanceDownResponse {
  status: ResponseStatus;
  isDown: boolean;
}

/**
 * 查询实例是否被拉出
 *
 * POST /api/management/instance/is-instance-down.json
 *
 * 检查一个实例是否被拉出或所在服务器是否被拉出
 *
 * @param instance_key - 实例键
 * @returns 是否被拉出的状态
 *
 * @example
 * const result = await isInstanceDown(instanceKey);
 * if (result.is_down) {
 *   console.log('实例已被拉出');
 * }
 */
export async function isInstanceDown(
  instanceKey: InstanceKey
): Promise<IsInstanceDownResponse> {
  const request: IsInstanceDownRequest = {
    instanceKey,
  };

  try {
    const response = await apiClient.post(
      '/api/management/instance/is-instance-down.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to check if instance is down:', error);
    throw error;
  }
}

// ========== 服务器管理 API ==========

/**
 * 操作服务器请求
 */
export interface OperateServerRequest {
  serverId: string;
  regionId: string;
  operation: ServerOperationType;
  operationComplete?: boolean;
  operatorId: string;
  token?: string;
}

/**
 * 操作服务器响应
 */
export interface OperateServerResponse {
  status: ResponseStatus;
}

/**
 * 拉出/拉入服务器
 *
 * POST /api/management/server/operate-server.json
 *
 * 批量操作服务器上的所有实例
 *
 * @param serverId - 服务器 ID (IP 地址)
 * @param regionId - Region ID
 * @param operation - 操作类型 (pullout/pullin)
 * @param operatorId - 操作人 ID
 * @param operation_complete - 操作是否完成 (默认 false)
 * @returns 操作结果
 *
 * @example
 * // 拉出整个服务器
 * await operateServer('192.168.1.100', 'us-east', ServerOperationType.PullOut, 'admin');
 *
 * // 拉入整个服务器
 * await operateServer('192.168.1.100', 'us-east', ServerOperationType.PullIn, 'admin');
 */
export async function operateServer(
  serverId: string,
  regionId: string,
  operation: ServerOperationType,
  operatorId: string,
  operationComplete: boolean = false
): Promise<OperateServerResponse> {
  const request: OperateServerRequest = {
    serverId,
    regionId,
    operation,
    operatorId,
    operationComplete,
  };

  try {
    const response = await apiClient.post(
      '/api/management/server/operate-server.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to operate server:', error);
    throw error;
  }
}

/**
 * 查询服务器是否被拉出请求
 */
export interface IsServerDownRequest {
  serverId: string;
  regionId: string;
}

/**
 * 查询服务器是否被拉出响应
 */
export interface IsServerDownResponse {
  status: ResponseStatus;
  isDown: boolean;
}

/**
 * 查询服务器是否被拉出
 *
 * POST /api/management/server/is-server-down.json
 *
 * 检查一个服务器是否被拉出
 *
 * @param serverId - 服务器 ID (IP 地址)
 * @param regionId - Region ID
 * @returns 是否被拉出的状态
 *
 * @example
 * const result = await isServerDown('192.168.1.100', 'us-east');
 * if (result.is_down) {
 *   console.log('服务器已被拉出');
 * }
 */
export async function isServerDown(
  serverId: string,
  regionId: string
): Promise<IsServerDownResponse> {
  const request: IsServerDownRequest = {
    serverId,
    regionId,
  };

  try {
    const response = await apiClient.post(
      '/api/management/server/is-server-down.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to check if server is down:', error);
    throw error;
  }
}

// ========== 批量查询 API (Phase 25) ==========

/**
 * 查询所有实例操作请求
 */
export interface GetAllInstanceOperationsRequest {
  regionId?: string;
}

/**
 * 查询所有实例操作响应
 */
export interface GetAllInstanceOperationsResponse {
  status: ResponseStatus;
  instanceOperationRecords: InstanceOperationRecord[];
}

/**
 * 查询所有实例操作 (POST 版本)
 *
 * POST /api/management/all-instance-operations.json
 *
 * 查询所有实例的操作记录,可按 Region 过滤
 *
 * @param regionId - 可选的 Region ID 过滤
 * @returns 所有实例操作记录
 *
 * @example
 * const result = await getAllInstanceOperationsPost('us-east');
 * result.instanceOperationRecords.forEach(record => {
 *   console.log(`${record.instanceKey.instanceId}: ${record.operation}`);
 * });
 */
export async function getAllInstanceOperationsPost(
  regionId?: string
): Promise<GetAllInstanceOperationsResponse> {
  const request: GetAllInstanceOperationsRequest = {
    regionId,
  };

  try {
    const response = await apiClient.post(
      '/api/management/all-instance-operations.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to get all instance operations:', error);
    throw error;
  }
}

/**
 * 查询所有实例操作
 *
 * GET /api/management/all-instance-operations.json?regionId=X
 *
 * 使用 query parameter 查询所有实例操作
 *
 * @param regionId - 可选的 Region ID 过滤
 * @returns 所有实例操作记录
 *
 * @example
 * const result = await getAllInstanceOperations('us-east');
 */
export async function getAllInstanceOperations(
  regionId?: string
): Promise<GetAllInstanceOperationsResponse> {
  try {
    const params = new URLSearchParams();
    if (regionId) {
      params.append('regionId', regionId);
    }

    const response = await apiClient.get(
      '/api/management/all-instance-operations.json',
      { params }
    );
    return response.data;
  } catch (error) {
    console.error('Failed to get all instance operations:', error);
    throw error;
  }
}

/**
 * 查询所有服务器操作请求
 */
export interface GetAllServerOperationsRequest {
  regionId?: string;
}

/**
 * 查询所有服务器操作响应
 */
export interface GetAllServerOperationsResponse {
  status: ResponseStatus;
  serverOperationRecords: ServerOperationInfo[];
}

/**
 * 查询所有服务器操作 (POST 版本)
 *
 * POST /api/management/all-server-operations.json
 *
 * 查询所有服务器的操作记录,可按 Region 过滤
 *
 * @param regionId - 可选的 Region ID 过滤
 * @returns 所有服务器操作记录
 *
 * @example
 * const result = await getAllServerOperationsPost('us-east');
 * result.server_operation_records.forEach(record => {
 *   console.log(`${record.serverId}: ${record.operation}`);
 * });
 */
export async function getAllServerOperationsPost(
  regionId?: string
): Promise<GetAllServerOperationsResponse> {
  const request: GetAllServerOperationsRequest = {
    regionId,
  };

  try {
    const response = await apiClient.post(
      '/api/management/all-server-operations.json',
      request
    );
    return response.data;
  } catch (error) {
    console.error('Failed to get all server operations:', error);
    throw error;
  }
}

/**
 * 查询所有服务器操作
 *
 * GET /api/management/all-server-operations.json?regionId=X
 *
 * 使用 query parameter 查询所有服务器操作
 *
 * @param regionId - 可选的 Region ID 过滤
 * @returns 所有服务器操作记录
 *
 * @example
 * const result = await getAllServerOperations('us-east');
 */
export async function getAllServerOperations(
  regionId?: string
): Promise<GetAllServerOperationsResponse> {
  try {
    const params = new URLSearchParams();
    if (regionId) {
      params.append('regionId', regionId);
    }

    const response = await apiClient.get(
      '/api/management/all-server-operations.json',
      { params }
    );
    return response.data;
  } catch (error) {
    console.error('Failed to get all server operations:', error);
    throw error;
  }
}
