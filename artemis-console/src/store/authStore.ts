/**
 * 认证状态管理 (Zustand Store)
 *
 * 管理用户身份验证、授权和会话状态
 * - 用户信息、访问令牌、权限
 * - 登录/登出操作
 * - 权限检查
 * - 令牌存储在 localStorage
 */

import { create } from 'zustand';
import type { User, Permission, Role } from '../api/types';

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

const LOCAL_STORAGE_TOKEN_KEY = 'artemis_auth_token';
const LOCAL_STORAGE_USER_KEY = 'artemis_user';

// ===== Helper Functions =====

/**
 * 从 localStorage 恢复认证状态
 */
function restoreAuthState(): {
  token: string | null;
  user: User | null;
} {
  try {
    const token = localStorage.getItem(LOCAL_STORAGE_TOKEN_KEY);
    const userJson = localStorage.getItem(LOCAL_STORAGE_USER_KEY);
    const user = userJson ? JSON.parse(userJson) : null;
    return { token, user };
  } catch {
    return { token: null, user: null };
  }
}

/**
 * 保存认证状态到 localStorage
 */
function persistAuthState(token: string | null, user: User | null): void {
  if (token) {
    localStorage.setItem(LOCAL_STORAGE_TOKEN_KEY, token);
  } else {
    localStorage.removeItem(LOCAL_STORAGE_TOKEN_KEY);
  }

  if (user) {
    localStorage.setItem(LOCAL_STORAGE_USER_KEY, JSON.stringify(user));
  } else {
    localStorage.removeItem(LOCAL_STORAGE_USER_KEY);
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
    login: async (username: string, _password: string) => {
      set({ isLoading: true, error: null });
      try {
        // TODO: 调用实际的登录 API
        // const response = await loginAPI({ username, password });
        // const { token, user } = response.data;

        // 模拟登录 (用于演示)
        const mockToken = `token_${Date.now()}`;
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

        // 保存到 localStorage
        persistAuthState(mockToken, mockUser);

        // 更新状态
        set({
          user: mockUser,
          token: mockToken,
          permissions,
          roles: mockUser.roles,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Login failed';
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
        // TODO: 调用实际的登出 API
        // await logoutAPI();

        // 清空所有认证状态
        persistAuthState(null, null);
        set({
          user: null,
          token: null,
          permissions: [],
          roles: [],
          isAuthenticated: false,
          isLoading: false,
          error: null,
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Logout failed';
        set({
          error: errorMessage,
          isLoading: false,
        });
        throw err;
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
      persistAuthState(get().token, user);
    },

    // ===== 设置令牌 =====
    setToken: (token: string | null) => {
      set({ token });
      persistAuthState(token, get().user);
    },

    // ===== 清空认证状态 =====
    clearAuth: () => {
      persistAuthState(null, null);
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
      // TODO: 实现令牌刷新逻辑
      // 检查令牌是否过期，如果过期则刷新
      // const { token } = get();
      // if (isTokenExpired(token)) {
      //   const newToken = await refreshTokenAPI(token);
      //   get().setToken(newToken);
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
