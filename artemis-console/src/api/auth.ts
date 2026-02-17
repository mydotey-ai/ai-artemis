/**
 * 认证 API
 *
 * 提供用户认证、授权和会话管理功能
 * 已集成后端认证服务，支持 JWT 认证
 */

import apiClient from './client';

// ===== 请求/响应类型定义 =====

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface User {
  user_id: string;
  username: string;
  email?: string;
  role: string;
  status: string;
  created_at: number;
  updated_at: number;
}

export interface RefreshTokenRequest {
  token: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// ===== 用户认证 API =====

/**
 * 用户登录
 * POST /api/auth/login
 */
export async function login(request: LoginRequest): Promise<ApiResponse<LoginResponse>> {
  const response = await apiClient.post('/api/auth/login', request);
  return response.data;
}

/**
 * 用户登出
 * POST /api/auth/logout
 */
export async function logout(): Promise<ApiResponse<string>> {
  const response = await apiClient.post('/api/auth/logout');
  return response.data;
}

/**
 * 刷新令牌
 * POST /api/auth/refresh
 */
export async function refreshToken(request: RefreshTokenRequest): Promise<ApiResponse<LoginResponse>> {
  const response = await apiClient.post('/api/auth/refresh', request);
  return response.data;
}

// ===== 当前用户 API =====

/**
 * 获取当前登录用户信息
 * GET /api/auth/user
 */
export async function getCurrentUser(): Promise<ApiResponse<User>> {
  const response = await apiClient.get('/api/auth/user');
  return response.data;
}

/**
 * 获取用户权限
 * GET /api/auth/permissions
 */
export async function getUserPermissions(): Promise<ApiResponse<string[]>> {
  const response = await apiClient.get('/api/auth/permissions');
  return response.data;
}

// ===== 密码管理 API =====

export interface ChangePasswordRequest {
  old_password: string;
  new_password: string;
}

/**
 * 修改密码
 * POST /api/auth/password/change
 */
export async function changePassword(
  oldPassword: string,
  newPassword: string
): Promise<ApiResponse<string>> {
  const response = await apiClient.post('/api/auth/password/change', {
    old_password: oldPassword,
    new_password: newPassword,
  });
  return response.data;
}

/**
 * 重置密码 (管理员操作)
 * POST /api/auth/password/reset/:user_id
 */
export async function resetUserPassword(
  userId: string,
  newPassword: string
): Promise<ApiResponse<string>> {
  const response = await apiClient.post(`/api/auth/password/reset/${userId}`, {
    new_password: newPassword,
  });
  return response.data;
}

// ===== 会话管理 API =====

export interface Session {
  session_id: string;
  user_id: string;
  ip_address?: string;
  user_agent?: string;
  created_at: number;
  expires_at: number;
  last_activity: number;
}

/**
 * 列出所有活跃会话
 * GET /api/auth/sessions
 */
export async function listActiveSessions(): Promise<ApiResponse<Session[]>> {
  const response = await apiClient.get('/api/auth/sessions');
  return response.data;
}

/**
 * 撤销会话
 * DELETE /api/auth/sessions/:session_id
 */
export async function revokeSession(sessionId: string): Promise<ApiResponse<string>> {
  const response = await apiClient.delete(`/api/auth/sessions/${sessionId}`);
  return response.data;
}

// ===== RBAC（基于角色的访问控制）API =====

/**
 * 获取所有角色
 * GET /api/auth/roles
 */
export async function listRoles(): Promise<ApiResponse<string[]>> {
  const response = await apiClient.get('/api/auth/roles');
  return response.data;
}

export interface CheckPermissionRequest {
  resource: string;
  action: string;
}

export interface CheckPermissionResponse {
  allowed: boolean;
}

/**
 * 检查权限
 * POST /api/auth/check-permission
 */
export async function checkPermission(
  resource: string,
  action: string
): Promise<ApiResponse<CheckPermissionResponse>> {
  const response = await apiClient.post('/api/auth/check-permission', {
    resource,
    action,
  });
  return response.data;
}

// ===== 用户管理 API =====

/**
 * User Status Enum
 */
export const UserStatus = {
  ACTIVE: 'active',
  INACTIVE: 'inactive',
} as const;

export type UserStatus = (typeof UserStatus)[keyof typeof UserStatus];

/**
 * User Role Enum
 */
export const UserRole = {
  ADMIN: 'admin',
  OPERATOR: 'operator',
  VIEWER: 'viewer',
} as const;

export type UserRole = (typeof UserRole)[keyof typeof UserRole];

/**
 * Complete User Details
 */
export interface UserDetails {
  user_id: string;
  username: string;
  email?: string;
  description?: string;
  role: string;
  status: string;
  created_at: number;
  updated_at: number;
}

/**
 * Login History Record
 */
export interface LoginHistory {
  id: number;
  user_id: string;
  login_time: number;
  ip_address: string;
  user_agent: string;
  status: 'success' | 'failed';
}

/**
 * Create User Request
 */
export interface CreateUserRequest {
  username: string;
  email?: string;
  description?: string;
  password: string;
  role: string;
}

/**
 * Update User Request
 */
export interface UpdateUserRequest {
  email?: string;
  description?: string;
  role?: string;
}

/**
 * Update User Status Request
 */
export interface UpdateUserStatusRequest {
  status: string;
}

/**
 * 获取所有用户
 * GET /api/auth/users
 */
export async function getAllUsers(): Promise<ApiResponse<UserDetails[]>> {
  const response = await apiClient.get('/api/auth/users');
  return response.data;
}

/**
 * 获取用户详情
 * GET /api/auth/users/:user_id
 */
export async function getUser(userId: string): Promise<ApiResponse<UserDetails>> {
  const response = await apiClient.get(`/api/auth/users/${userId}`);
  return response.data;
}

/**
 * 创建用户
 * POST /api/auth/users
 */
export async function createUser(request: CreateUserRequest): Promise<ApiResponse<UserDetails>> {
  const response = await apiClient.post('/api/auth/users', request);
  return response.data;
}

/**
 * 更新用户
 * PUT /api/auth/users/:user_id
 */
export async function updateUser(
  userId: string,
  request: UpdateUserRequest
): Promise<ApiResponse<UserDetails>> {
  const response = await apiClient.put(`/api/auth/users/${userId}`, request);
  return response.data;
}

/**
 * 删除用户
 * DELETE /api/auth/users/:user_id
 */
export async function deleteUser(userId: string): Promise<ApiResponse<string>> {
  const response = await apiClient.delete(`/api/auth/users/${userId}`);
  return response.data;
}

/**
 * 更改用户状态
 * PATCH /api/auth/users/:user_id/status
 */
export async function changeUserStatus(
  userId: string,
  status: string
): Promise<ApiResponse<UserDetails>> {
  const response = await apiClient.patch(`/api/auth/users/${userId}/status`, { status });
  return response.data;
}

/**
 * 获取用户登录历史
 * GET /api/auth/users/:user_id/login-history
 */
export async function getUserLoginHistory(userId: string): Promise<ApiResponse<LoginHistory[]>> {
  const response = await apiClient.get(`/api/auth/users/${userId}/login-history`);
  return response.data;
}
