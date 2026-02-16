/**
 * API 统一导出
 *
 * 集中管理所有 API 模块的导出，方便在应用中使用
 */

// ===== 现有 API 模块 =====
export * from './client';
export * from './types';
export * from './discovery';

// Export only functions from management, types are already exported from ./types
export {
  operateInstance,
  getInstanceOperations,
  isInstanceDown,
  operateServer,
  isServerDown,
  getAllInstanceOperations,
  getAllInstanceOperationsGet,
  getAllServerOperations,
  getAllServerOperationsGet,
} from './management';

// ===== 新增 API 模块 =====

// 分组和路由规则 API - export only functions
export {
  createGroup,
  listGroups,
  getGroup,
  updateGroup,
  deleteGroup,
  addGroupTags,
  createRule,
  listRules,
  getRule,
  updateRule,
  deleteRule,
  addRuleGroup,
  removeRuleGroup,
  updateRuleGroup,
  getGroupInstances,
} from './routing';

// 集群状态 API - export only functions
export {
  getClusterStatus,
  getClusterNodeStatus,
  getConfigStatus,
  getDeploymentStatus,
  getLeasesStatus,
} from './cluster';

// 审计日志 API - export only functions
export {
  queryLogs,
  queryLogsByOperation,
  queryLogsByOperator,
  queryLogsByResourceType,
  queryLogsByTimeRange,
  getLogDetail,
  exportLogs,
  cleanupOldLogs,
  getLogStats,
} from './audit';

// Zone 操作 API - export only functions
export {
  pullOutZone,
  pullInZone,
  queryZoneOperations,
  getZoneInfo,
  listZones,
  isZoneDown,
  getZoneInstances,
  updateZoneStatus,
  batchPullOutZones,
  batchPullInZones,
} from './zone';

// 金丝雀发布 API - export only functions
export {
  listCanaryConfigs,
  getCanaryConfig,
  setCanaryConfig,
  deleteCanaryConfig,
  enableCanary,
  disableCanary,
  addIpToWhitelist,
  removeIpFromWhitelist,
  getCanaryStats,
} from './canary';

// 认证 API - export only functions
export {
  login,
  logout,
  refreshToken,
  getCurrentUser,
  getUserPermissions,
  changePassword,
  resetPassword,
  listActiveSessions,
  revokeSession,
  listRoles,
  checkPermission,
  getAllUsers,
  getUser,
  createUser,
  updateUser,
  deleteUser,
  resetUserPassword,
  changeUserStatus,
  getUserLoginHistory,
  UserStatus,
  UserRole,
} from './auth';

// ===== API 基础配置 =====

/**
 * API 基础 URL 配置
 * 可根据环境变量调整
 */
export const API_CONFIG = {
  BASE_URL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
  TIMEOUT: import.meta.env.VITE_API_TIMEOUT || 30000,
  // 其他配置项...
};

/**
 * HTTP 状态码定义
 */
export const HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  NO_CONTENT: 204,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  CONFLICT: 409,
  INTERNAL_SERVER_ERROR: 500,
  SERVICE_UNAVAILABLE: 503,
} as const;

/**
 * API 错误码定义
 */
export const ERROR_CODES = {
  INVALID_REQUEST: 'INVALID_REQUEST',
  NOT_FOUND: 'NOT_FOUND',
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  INTERNAL_ERROR: 'INTERNAL_ERROR',
  SERVICE_UNAVAILABLE: 'SERVICE_UNAVAILABLE',
  TIMEOUT: 'TIMEOUT',
} as const;

/**
 * 通用 API 响应类型
 */
export interface BaseApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  code?: string;
  timestamp?: string;
}

/**
 * 分页响应类型
 */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

/**
 * API 请求拦截器配置
 * 用于添加全局请求头、令牌等
 */
export function setupApiInterceptors() {
  // TODO: 在主应用中调用此函数以设置 axios 拦截器
  // 用于添加认证令牌、日志、错误处理等
}
