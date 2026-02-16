import axios, { type AxiosInstance, type AxiosResponse, AxiosError } from 'axios';

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

/**
 * Request interceptor: Add Authorization header from localStorage
 */
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

/**
 * Response interceptor: Handle 401 errors
 */
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    return response;
  },
  (error: AxiosError) => {
    // Handle 401 Unauthorized error
    if (error.response?.status === 401) {
      // Clear token from localStorage
      localStorage.removeItem('token');

      // Redirect to login page
      window.location.href = '/login';
    }

    return Promise.reject(error);
  }
);

export default apiClient;
