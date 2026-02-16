/**
 * UI 状态管理 (Zustand Store)
 *
 * 管理应用级别的 UI 状态
 * - 侧边栏开/关状态
 * - 主题切换 (亮/暗)
 * - 通知显示/隐藏
 * - 本地存储保持用户偏好
 */

import { create } from 'zustand';

// ===== Type Definitions =====

export type Theme = 'light' | 'dark';

export type NotificationType = 'success' | 'error' | 'warning' | 'info'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface Notification {
  id: string;
  type: NotificationType;
  message: string;
  title?: string;
  duration?: number; // 毫秒，undefined 表示不自动关闭
  action?: {
    label: string;
    onClick: () => void;
  };
}

// ===== Store State Interface =====

interface UIStoreState {
  // 状态
  sidebarOpen: boolean;
  theme: Theme;
  notifications: Notification[];

  // 侧边栏操作
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;

  // 主题操作
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;

  // 通知操作
  showNotification: (notification: Omit<Notification, 'id'>) => string;
  hideNotification: (id: string) => void;
  clearAllNotifications: () => void;
  updateNotification: (id: string, notification: Partial<Notification>) => void;
}

// ===== Constants =====

const LOCAL_STORAGE_THEME_KEY = 'artemis_theme';
const LOCAL_STORAGE_SIDEBAR_KEY = 'artemis_sidebar_open';

// ===== Helper Functions =====

/**
 * 从 localStorage 恢复主题偏好
 */
function restoreTheme(): Theme {
  try {
    const savedTheme = localStorage.getItem(LOCAL_STORAGE_THEME_KEY);
    if (savedTheme === 'light' || savedTheme === 'dark') {
      return savedTheme;
    }
  } catch {
    // 忽略存储错误
  }

  // 默认使用系统偏好
  if (typeof window !== 'undefined') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  }

  return 'light';
}

/**
 * 从 localStorage 恢复侧边栏状态
 */
function restoreSidebarState(): boolean {
  try {
    const saved = localStorage.getItem(LOCAL_STORAGE_SIDEBAR_KEY);
    return saved ? JSON.parse(saved) : true; // 默认打开
  } catch {
    return true;
  }
}

/**
 * 保存主题到 localStorage
 */
function persistTheme(theme: Theme): void {
  try {
    localStorage.setItem(LOCAL_STORAGE_THEME_KEY, theme);
  } catch {
    // 忽略存储错误
  }
}

/**
 * 保存侧边栏状态到 localStorage
 */
function persistSidebarState(open: boolean): void {
  try {
    localStorage.setItem(LOCAL_STORAGE_SIDEBAR_KEY, JSON.stringify(open));
  } catch {
    // 忽略存储错误
  }
}

/**
 * 生成唯一的通知 ID
 */
function generateNotificationId(): string {
  return `notification_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * 应用主题到文档
 */
function applyTheme(theme: Theme): void {
  if (typeof document === 'undefined') return;

  const root = document.documentElement;
  if (theme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

// ===== Zustand Store =====

export const useUIStore = create<UIStoreState>((set, get) => {
  // 初始化
  const initialTheme = restoreTheme();
  const initialSidebarOpen = restoreSidebarState();
  applyTheme(initialTheme);

  return {
    // 初始状态
    sidebarOpen: initialSidebarOpen,
    theme: initialTheme,
    notifications: [],

    // ===== 侧边栏操作 =====
    toggleSidebar: () => {
      const newState = !get().sidebarOpen;
      persistSidebarState(newState);
      set({ sidebarOpen: newState });
    },

    setSidebarOpen: (open: boolean) => {
      persistSidebarState(open);
      set({ sidebarOpen: open });
    },

    // ===== 主题操作 =====
    toggleTheme: () => {
      const newTheme = get().theme === 'light' ? 'dark' : 'light';
      persistTheme(newTheme);
      applyTheme(newTheme);
      set({ theme: newTheme });
    },

    setTheme: (theme: Theme) => {
      persistTheme(theme);
      applyTheme(theme);
      set({ theme });
    },

    // ===== 通知操作 =====
    showNotification: (notification: Omit<Notification, 'id'>) => {
      const id = generateNotificationId();
      const newNotification: Notification = {
        ...notification,
        id,
        duration: notification.duration ?? 5000, // 默认 5 秒
      };

      set((state) => ({
        notifications: [...state.notifications, newNotification],
      }));

      // 自动关闭通知
      if (newNotification.duration && newNotification.duration > 0) {
        setTimeout(() => {
          get().hideNotification(id);
        }, newNotification.duration);
      }

      return id;
    },

    hideNotification: (id: string) => {
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
      }));
    },

    clearAllNotifications: () => {
      set({ notifications: [] });
    },

    updateNotification: (id: string, notification: Partial<Notification>) => {
      set((state) => ({
        notifications: state.notifications.map((n) =>
          n.id === id ? { ...n, ...notification } : n
        ),
      }));
    },
  };
});

// ===== Store Selectors (性能优化) =====

/**
 * 选择器：获取侧边栏状态
 */
export const selectSidebarOpen = (state: UIStoreState): boolean =>
  state.sidebarOpen;

/**
 * 选择器：获取主题
 */
export const selectTheme = (state: UIStoreState): Theme => state.theme;

/**
 * 选择器：获取通知列表
 */
export const selectNotifications = (state: UIStoreState): Notification[] =>
  state.notifications;

/**
 * 选择器：获取通知数量
 */
export const selectNotificationCount = (state: UIStoreState): number =>
  state.notifications.length;

/**
 * 选择器：检查是否有错误通知
 */
export const selectHasErrors = (state: UIStoreState): boolean =>
  state.notifications.some((n) => n.type === 'error');

// ===== Hooks for Common Patterns =====

/**
 * 显示成功通知的便捷方法
 */
export const showSuccess = (message: string, title?: string): string => {
  return useUIStore.getState().showNotification({
    type: 'success',
    message,
    title,
  });
};

/**
 * 显示错误通知的便捷方法
 */
export const showError = (message: string, title?: string): string => {
  return useUIStore.getState().showNotification({
    type: 'error',
    message,
    title,
    duration: 0, // 错误消息不自动关闭
  });
};

/**
 * 显示警告通知的便捷方法
 */
export const showWarning = (message: string, title?: string): string => {
  return useUIStore.getState().showNotification({
    type: 'warning',
    message,
    title,
  });
};

/**
 * 显示信息通知的便捷方法
 */
export const showInfo = (message: string, title?: string): string => {
  return useUIStore.getState().showNotification({
    type: 'info',
    message,
    title,
  });
};

/**
 * 显示操作通知的便捷方法（带操作按钮）
 */
export const showActionNotification = (
  message: string,
  actionLabel: string,
  onAction: () => void,
  type: NotificationType = 'info'
): string => {
  return useUIStore.getState().showNotification({
    type,
    message,
    action: {
      label: actionLabel,
      onClick: onAction,
    },
    duration: 0, // 有操作的通知不自动关闭
  });
};
