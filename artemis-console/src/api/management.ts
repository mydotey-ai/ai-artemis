/**
 * Artemis Management API
 *
 * 实例管理和服务器管理的 API 封装
 * - 实例拉入/拉出操作
 * - 服务器批量操作
 * - 操作历史查询
 */

import apiClient from '@/api/client';

/**
 * 响应状态
 */
export interface ResponseStatus {
  code: string;
  message: string;
}

/**
 * 实例键 (唯一标识一个实例)
 */
export interface InstanceKey {
  service_id: string;
  instance_id: string;
  app_id?: string;
  group_id?: string;
  ip: string;
  port: number;
  region_id: string;
  zone_id?: string;
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
  instance_key: InstanceKey;
  operation: InstanceOperationType;
  operation_complete: boolean;
  operator_id: string;
  token?: string;
}

/**
 * 服务器操作记录 (用于查询返回)
 */
export interface ServerOperationInfo {
  server_id: string;
  region_id: string;
  operation: ServerOperationType;
}

// ========== 实例管理 API ==========

/**
 * 操作实例请求
 */
export interface OperateInstanceRequest {
  instance_key: InstanceKey;
  operation: InstanceOperationType;
  operation_complete?: boolean;
  operator_id: string;
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
 * @param operator_id - 操作人 ID
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
  instance_key: InstanceKey,
  operation: InstanceOperationType,
  operator_id: string,
  operation_complete: boolean = false
): Promise<OperateInstanceResponse> {
  const request: OperateInstanceRequest = {
    instance_key,
    operation,
    operator_id,
    operation_complete,
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
  instance_key: InstanceKey;
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
  instance_key: InstanceKey
): Promise<GetInstanceOperationsResponse> {
  const request: GetInstanceOperationsRequest = {
    instance_key,
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
  instance_key: InstanceKey;
}

/**
 * 查询实例是否被拉出响应
 */
export interface IsInstanceDownResponse {
  status: ResponseStatus;
  is_down: boolean;
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
  instance_key: InstanceKey
): Promise<IsInstanceDownResponse> {
  const request: IsInstanceDownRequest = {
    instance_key,
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
  server_id: string;
  region_id: string;
  operation: ServerOperationType;
  operation_complete?: boolean;
  operator_id: string;
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
 * @param server_id - 服务器 ID (IP 地址)
 * @param region_id - Region ID
 * @param operation - 操作类型 (pullout/pullin)
 * @param operator_id - 操作人 ID
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
  server_id: string,
  region_id: string,
  operation: ServerOperationType,
  operator_id: string,
  operation_complete: boolean = false
): Promise<OperateServerResponse> {
  const request: OperateServerRequest = {
    server_id,
    region_id,
    operation,
    operator_id,
    operation_complete,
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
  server_id: string;
  region_id: string;
}

/**
 * 查询服务器是否被拉出响应
 */
export interface IsServerDownResponse {
  status: ResponseStatus;
  is_down: boolean;
}

/**
 * 查询服务器是否被拉出
 *
 * POST /api/management/server/is-server-down.json
 *
 * 检查一个服务器是否被拉出
 *
 * @param server_id - 服务器 ID (IP 地址)
 * @param region_id - Region ID
 * @returns 是否被拉出的状态
 *
 * @example
 * const result = await isServerDown('192.168.1.100', 'us-east');
 * if (result.is_down) {
 *   console.log('服务器已被拉出');
 * }
 */
export async function isServerDown(
  server_id: string,
  region_id: string
): Promise<IsServerDownResponse> {
  const request: IsServerDownRequest = {
    server_id,
    region_id,
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
  region_id?: string;
}

/**
 * 查询所有实例操作响应
 */
export interface GetAllInstanceOperationsResponse {
  status: ResponseStatus;
  instance_operation_records: InstanceOperationRecord[];
}

/**
 * 查询所有实例操作 (POST 版本)
 *
 * POST /api/management/all-instance-operations.json
 *
 * 查询所有实例的操作记录,可按 Region 过滤
 *
 * @param region_id - 可选的 Region ID 过滤
 * @returns 所有实例操作记录
 *
 * @example
 * const result = await getAllInstanceOperations('us-east');
 * result.instance_operation_records.forEach(record => {
 *   console.log(`${record.instance_key.instance_id}: ${record.operation}`);
 * });
 */
export async function getAllInstanceOperations(
  region_id?: string
): Promise<GetAllInstanceOperationsResponse> {
  const request: GetAllInstanceOperationsRequest = {
    region_id,
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
 * 查询所有实例操作 (GET 版本)
 *
 * GET /api/management/all-instance-operations.json?regionId=X
 *
 * 使用 query parameter 查询所有实例操作
 *
 * @param region_id - 可选的 Region ID 过滤
 * @returns 所有实例操作记录
 *
 * @example
 * const result = await getAllInstanceOperationsGet('us-east');
 */
export async function getAllInstanceOperationsGet(
  region_id?: string
): Promise<GetAllInstanceOperationsResponse> {
  try {
    const params = new URLSearchParams();
    if (region_id) {
      params.append('regionId', region_id);
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
  region_id?: string;
}

/**
 * 查询所有服务器操作响应
 */
export interface GetAllServerOperationsResponse {
  status: ResponseStatus;
  server_operation_records: ServerOperationInfo[];
}

/**
 * 查询所有服务器操作 (POST 版本)
 *
 * POST /api/management/all-server-operations.json
 *
 * 查询所有服务器的操作记录,可按 Region 过滤
 *
 * @param region_id - 可选的 Region ID 过滤
 * @returns 所有服务器操作记录
 *
 * @example
 * const result = await getAllServerOperations('us-east');
 * result.server_operation_records.forEach(record => {
 *   console.log(`${record.server_id}: ${record.operation}`);
 * });
 */
export async function getAllServerOperations(
  region_id?: string
): Promise<GetAllServerOperationsResponse> {
  const request: GetAllServerOperationsRequest = {
    region_id,
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
 * 查询所有服务器操作 (GET 版本)
 *
 * GET /api/management/all-server-operations.json?regionId=X
 *
 * 使用 query parameter 查询所有服务器操作
 *
 * @param region_id - 可选的 Region ID 过滤
 * @returns 所有服务器操作记录
 *
 * @example
 * const result = await getAllServerOperationsGet('us-east');
 */
export async function getAllServerOperationsGet(
  region_id?: string
): Promise<GetAllServerOperationsResponse> {
  try {
    const params = new URLSearchParams();
    if (region_id) {
      params.append('regionId', region_id);
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
