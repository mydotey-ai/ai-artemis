/**
 * API 统一导出
 *
 * 集中管理所有 API 模块的导出，方便在应用中使用
 */

// ===== 现有 API 模块 =====
export * from './client';
export * from './types';
export * from './discovery';
export * from './management';

// ===== 新增 API 模块 =====

// 分组和路由规则 API
export * from './routing';

// 集群状态 API
export * from './cluster';

// 审计日志 API
export * from './audit';

// Zone 操作 API
export * from './zone';

// 金丝雀发布 API
export * from './canary';

// 认证 API
export * from './auth';

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
