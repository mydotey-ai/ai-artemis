/**
 * Axios HTTP Client Configuration
 *
 * Enhanced with authentication interceptors:
 * - Request interceptor: Automatically adds JWT token to headers
 * - Response interceptor: Handles 401/403 errors and redirects
 */

import axios, { type AxiosInstance, type AxiosResponse, type AxiosError } from 'axios';
import { getToken, removeToken } from '@/utils/token';

// ===== Axios Instance Configuration =====

/**
 * Create Axios instance with custom configuration
 */
const apiClient: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
  timeout: 30000, // 30 seconds
  headers: {
    'Content-Type': 'application/json',
  },
});

// ===== Request Interceptor =====

/**
 * Request interceptor: Add Authorization header
 *
 * Automatically attaches JWT token from storage to all outgoing requests
 */
apiClient.interceptors.request.use(
  (config) => {
    // Get token from utility (checks both localStorage and sessionStorage)
    const token = getToken();

    // Add Authorization header if token exists
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    return config;
  },
  (error) => {
    // Request setup error
    console.error('Request interceptor error:', error);
    return Promise.reject(error);
  }
);

// ===== Response Interceptor =====

/**
 * Response interceptor: Handle authentication errors
 *
 * Handles:
 * - 401 Unauthorized: Clear token and redirect to login
 * - 403 Forbidden: Show permission denied message
 */
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    // Success response - pass through
    return response;
  },
  (error: AxiosError) => {
    // Handle HTTP errors
    if (error.response) {
      const status = error.response.status;

      if (status === 401) {
        // 401 Unauthorized - Session expired or invalid token
        console.warn('Session expired - redirecting to login');

        // Clear token from storage
        removeToken();

        // Save current path for redirect after login
        const currentPath = window.location.pathname;
        const loginPath = `/login${currentPath !== '/login' ? `?redirect=${encodeURIComponent(currentPath)}` : ''}`;

        // Redirect to login page
        window.location.href = loginPath;
      } else if (status === 403) {
        // 403 Forbidden - Permission denied
        console.error('Permission denied - insufficient privileges');

        // Show error notification (if notification system is available)
        // For now, just log the error
        // TODO: Integrate with notification/toast system
      }
    }

    // Reject with error for caller to handle
    return Promise.reject(error);
  }
);

// ===== Export =====

export default apiClient;
