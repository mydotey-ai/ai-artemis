/**
 * PrivateRoute Component
 *
 * Route guard for protected pages that require authentication.
 * - Checks if user is authenticated
 * - Redirects to login page if not authenticated
 * - Preserves the original URL for redirect after login
 * - Renders the protected content if authenticated
 */

import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '@/store/authStore';
import type { ReactNode } from 'react';

// ===== Type Definitions =====

/**
 * PrivateRoute component props
 */
export interface PrivateRouteProps {
  /** Child components to render if authenticated */
  children: ReactNode;
}

// ===== Main Component =====

/**
 * PrivateRoute Component
 *
 * Protects routes by checking authentication status.
 * If not authenticated, redirects to login with the original path saved.
 *
 * @param children - Protected content to render
 * @returns Protected content or redirect to login
 */
export const PrivateRoute: React.FC<PrivateRouteProps> = ({ children }) => {
  const location = useLocation();
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  // Check authentication status
  if (!isAuthenticated) {
    // Save the current path to redirect back after login
    const redirectPath = location.pathname + location.search + location.hash;

    // Redirect to login with original path
    return (
      <Navigate
        to={`/login?redirect=${encodeURIComponent(redirectPath)}`}
        replace
      />
    );
  }

  // User is authenticated, render protected content
  return <>{children}</>;
};

/**
 * Display name for debugging
 */
PrivateRoute.displayName = 'PrivateRoute';

/**
 * Export default for convenience
 */
export default PrivateRoute;
