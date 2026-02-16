/**
 * 审计日志 API
 *
 * 提供操作日志的查询和管理功能
 */

import axios from 'axios';

const API_BASE = '/api/management/audit';

// ===== 请求/响应类型定义 =====

export interface AuditLog {
  id: string;
  timestamp: string;
  operation_type: string;
  operator_id: string;
  resource_type: string;
  resource_id: string;
  action: string;
  details: Record<string, any>;
  result: 'SUCCESS' | 'FAILURE';
  error_message?: string;
}

export interface QueryLogsParams {
  operation_type?: string;
  operator_id?: string;
  resource_type?: string;
  start_time?: string;
  end_time?: string;
  limit?: number;
  offset?: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== 审计日志查询 API =====

/**
 * 查询审计日志
 * GET /api/management/audit/logs
 */
export async function queryLogs(params?: QueryLogsParams): Promise<ApiResponse<AuditLog[]>> {
  const response = await axios.get(`${API_BASE}/logs`, { params });
  return response.data;
}

/**
 * 按操作类型查询
 * GET /api/management/audit/logs?operation_type=...
 */
export async function queryLogsByOperation(
  operationType: string,
  limit?: number
): Promise<ApiResponse<AuditLog[]>> {
  return queryLogs({ operation_type: operationType, limit });
}

/**
 * 按操作员查询
 * GET /api/management/audit/logs?operator_id=...
 */
export async function queryLogsByOperator(
  operatorId: string,
  limit?: number
): Promise<ApiResponse<AuditLog[]>> {
  return queryLogs({ operator_id: operatorId, limit });
}

/**
 * 按资源类型查询
 * GET /api/management/audit/logs?resource_type=...
 */
export async function queryLogsByResourceType(
  resourceType: string,
  limit?: number
): Promise<ApiResponse<AuditLog[]>> {
  return queryLogs({ resource_type: resourceType, limit });
}

/**
 * 按时间范围查询
 * GET /api/management/audit/logs?start_time=...&end_time=...
 */
export async function queryLogsByTimeRange(
  startTime: string,
  endTime: string,
  limit?: number
): Promise<ApiResponse<AuditLog[]>> {
  return queryLogs({ start_time: startTime, end_time: endTime, limit });
}

/**
 * 获取日志详情
 * GET /api/management/audit/logs/:log_id
 */
export async function getLogDetail(logId: string): Promise<ApiResponse<AuditLog>> {
  const response = await axios.get(`${API_BASE}/logs/${logId}`);
  return response.data;
}

/**
 * 导出审计日志
 * GET /api/management/audit/logs/export
 */
export async function exportLogs(params?: QueryLogsParams): Promise<Blob> {
  const response = await axios.get(`${API_BASE}/logs/export`, {
    params,
    responseType: 'blob',
  });
  return response.data;
}

/**
 * 清理老日志
 * DELETE /api/management/audit/logs/cleanup
 */
export async function cleanupOldLogs(daysToKeep: number): Promise<ApiResponse<{ deleted_count: number }>> {
  const response = await axios.delete(`${API_BASE}/logs/cleanup`, {
    params: { days_to_keep: daysToKeep },
  });
  return response.data;
}

/**
 * 获取日志统计信息
 * GET /api/management/audit/logs/stats
 */
export async function getLogStats(
  startTime?: string,
  endTime?: string
): Promise<ApiResponse<{ total: number; by_operation: Record<string, number> }>> {
  const response = await axios.get(`${API_BASE}/logs/stats`, {
    params: {
      start_time: startTime,
      end_time: endTime,
    },
  });
  return response.data;
}
