/**
 * 认证状态管理 (Zustand Store)
 *
 * 管理用户身份验证、授权和会话状态
 * - 用户信息、访问令牌、权限
 * - 登录/登出操作
 * - 权限检查
 * - 令牌存储在 localStorage/sessionStorage
 * - JWT token 管理集成
 */

import { create } from 'zustand';
import type { User, Role } from '../api/types';
import { saveToken, getToken, removeToken, isTokenValid } from '@/utils/token';
import * as authApi from '@/api/auth';

// ===== Store State Interface =====

interface AuthStoreState {
  // 状态
  user: User | null;
  token: string | null;
  permissions: string[];
  roles: Role[];
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // 操作
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  checkPermission: (permission: string) => boolean;
  hasRole: (roleId: string) => boolean;
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  clearAuth: () => void;
  refreshTokenIfNeeded: () => Promise<void>;
  setError: (error: string | null) => void;
}

// ===== Constants =====

const LOCAL_STORAGE_USER_KEY = 'artemis_user';

// ===== Helper Functions =====

/**
 * 从 storage 恢复认证状态
 * 使用 token utility 获取 token，从 localStorage 获取 user
 */
function restoreAuthState(): {
  token: string | null;
  user: User | null;
} {
  try {
    // Get token from utility (handles both localStorage and sessionStorage)
    const token = getToken();

    // Validate token before restoring state
    if (token && !isTokenValid(token)) {
      // Token expired, clear everything
      removeToken();
      localStorage.removeItem(LOCAL_STORAGE_USER_KEY);
      return { token: null, user: null };
    }

    // Get user from localStorage
    const userJson = localStorage.getItem(LOCAL_STORAGE_USER_KEY);
    const user = userJson ? JSON.parse(userJson) : null;

    return { token, user };
  } catch (error) {
    console.error('Failed to restore auth state:', error);
    return { token: null, user: null };
  }
}

/**
 * 保存用户信息到 localStorage
 * Token 使用 token utility 管理
 */
function persistUserState(user: User | null): void {
  try {
    if (user) {
      localStorage.setItem(LOCAL_STORAGE_USER_KEY, JSON.stringify(user));
    } else {
      localStorage.removeItem(LOCAL_STORAGE_USER_KEY);
    }
  } catch (error) {
    console.error('Failed to persist user state:', error);
  }
}

/**
 * 提取所有权限 ID (从用户的角色和权限)
 */
function extractPermissionIds(user: User): string[] {
  const permissionSet = new Set<string>();

  // 从用户的直接权限
  user.permissions?.forEach((p) => {
    permissionSet.add(p.permission_id);
  });

  // 从用户的角色权限
  user.roles?.forEach((role) => {
    role.permissions?.forEach((p) => {
      permissionSet.add(p.permission_id);
    });
  });

  return Array.from(permissionSet);
}

// ===== Zustand Store =====

export const useAuthStore = create<AuthStoreState>((set, get) => {
  // 初始化状态
  const { token: initialToken, user: initialUser } = restoreAuthState();

  return {
    // 初始状态
    user: initialUser || null,
    token: initialToken || null,
    permissions: initialUser ? extractPermissionIds(initialUser) : [],
    roles: initialUser?.roles || [],
    isAuthenticated: !!initialToken && !!initialUser,
    isLoading: false,
    error: null,

    // ===== 登录 =====
    login: async (username: string, password: string) => {
      set({ isLoading: true, error: null });
      try {
        // Call login API (currently mock implementation in auth.ts)
        const response = await authApi.login({ username, password });

        if (!response.success || !response.data) {
          throw new Error(response.message || 'Login failed');
        }

        const { access_token } = response.data;

        // For now, create mock user until backend is implemented
        // TODO: Replace with actual user data from API response
        const mockUser: User = {
          user_id: `user_${username}`,
          username,
          email: `${username}@example.com`,
          roles: [
            {
              role_id: 'admin',
              name: 'Administrator',
              description: 'Admin role',
              permissions: [],
            },
          ],
          permissions: [
            {
              permission_id: 'service:read',
              name: 'Read Services',
              resource: 'service',
              action: 'read',
            },
            {
              permission_id: 'service:write',
              name: 'Write Services',
              resource: 'service',
              action: 'write',
            },
          ],
        };

        const permissions = extractPermissionIds(mockUser);

        // Save token using utility (will use appropriate storage)
        // Note: Remember me is handled in Login component
        // Token is already saved there, but we ensure it's set
        const currentToken = getToken();
        if (!currentToken) {
          saveToken(access_token, false);
        }

        // Save user to localStorage
        persistUserState(mockUser);

        // Update state
        set({
          user: mockUser,
          token: access_token,
          permissions,
          roles: mockUser.roles,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Login failed';
        console.error('Login error:', errorMessage);

        // Clear any partial state
        removeToken();
        persistUserState(null);

        set({
          error: errorMessage,
          isLoading: false,
          user: null,
          token: null,
          isAuthenticated: false,
        });
        throw err;
      }
    },

    // ===== 登出 =====
    logout: async () => {
      set({ isLoading: true, error: null });
      try {
        // Call logout API (currently mock implementation in auth.ts)
        await authApi.logout();

        // Clear all authentication state
        removeToken();
        persistUserState(null);

        set({
          user: null,
          token: null,
          permissions: [],
          roles: [],
          isAuthenticated: false,
          isLoading: false,
          error: null,
        });

        // Redirect to login page
        window.location.href = '/login';
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Logout failed';
        console.error('Logout error:', errorMessage);

        // Even if API fails, clear local state
        removeToken();
        persistUserState(null);

        set({
          user: null,
          token: null,
          permissions: [],
          roles: [],
          isAuthenticated: false,
          error: errorMessage,
          isLoading: false,
        });

        // Still redirect on error
        window.location.href = '/login';
      }
    },

    // ===== 权限检查 =====
    checkPermission: (permission: string): boolean => {
      const { permissions } = get();
      return permissions.includes(permission);
    },

    // ===== 角色检查 =====
    hasRole: (roleId: string): boolean => {
      const { roles } = get();
      return roles.some((role) => role.role_id === roleId);
    },

    // ===== 设置用户 =====
    setUser: (user: User | null) => {
      const permissions = user ? extractPermissionIds(user) : [];
      set({
        user,
        permissions,
        roles: user?.roles || [],
      });
      persistUserState(user);
    },

    // ===== 设置令牌 =====
    setToken: (token: string | null) => {
      set({ token });
      if (token) {
        saveToken(token, false); // Default to sessionStorage
      } else {
        removeToken();
      }
    },

    // ===== 清空认证状态 =====
    clearAuth: () => {
      removeToken();
      persistUserState(null);
      set({
        user: null,
        token: null,
        permissions: [],
        roles: [],
        isAuthenticated: false,
        error: null,
      });
    },

    // ===== 刷新令牌 =====
    refreshTokenIfNeeded: async () => {
      const { token } = get();

      // Check if token needs refresh
      if (!token || isTokenValid(token)) {
        return; // Token is valid or doesn't exist
      }

      // Token is expired or invalid, clear auth
      get().clearAuth();

      // TODO: Implement token refresh when backend supports it
      // try {
      //   const response = await authApi.refreshToken({ refresh_token: token });
      //   if (response.success && response.data) {
      //     get().setToken(response.data.access_token);
      //   }
      // } catch (error) {
      //   console.error('Token refresh failed:', error);
      //   get().clearAuth();
      // }
    },

    // ===== 设置错误 =====
    setError: (error: string | null) => {
      set({ error });
    },
  };
});

// ===== Store Selectors (性能优化) =====

/**
 * 选择器：只获取认证状态
 */
export const selectIsAuthenticated = (state: AuthStoreState) =>
  state.isAuthenticated;

/**
 * 选择器：只获取当前用户
 */
export const selectUser = (state: AuthStoreState) => state.user;

/**
 * 选择器：只获取权限
 */
export const selectPermissions = (state: AuthStoreState) =>
  state.permissions;

/**
 * 选择器：只获取加载状态
 */
export const selectIsLoading = (state: AuthStoreState) => state.isLoading;

/**
 * 选择器：只获取错误信息
 */
export const selectError = (state: AuthStoreState) => state.error;
