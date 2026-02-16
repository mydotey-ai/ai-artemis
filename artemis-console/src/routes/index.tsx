/**
 * Main routing configuration for Artemis Console
 *
 * Uses react-router-dom v6's createBrowserRouter API with lazy loading
 * Route structure:
 * - / → Redirect to /dashboard
 * - /login → Login page (no layout, public)
 * - Protected routes (with MainLayout and PrivateRoute guard):
 *   - /dashboard → Dashboard
 *   - /services → Services
 *   - /instances → Instances
 *   - /cluster → Cluster
 *   - /routing → Routing
 *   - /audit-log → AuditLog
 *   - /zone-ops → ZoneOps
 *   - /canary → Canary
 *   - /users → Users
 *
 * Performance optimizations:
 * - All page components are lazy-loaded using React.lazy()
 * - Suspense boundary provides loading fallback
 * - Code splitting reduces initial bundle size
 *
 * Security:
 * - All protected routes wrapped with PrivateRoute guard
 * - Redirects to login if not authenticated
 */

import { lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import { MainLayout } from '@/components/Layout/MainLayout';
import { LoadingFallback } from '@/components/LoadingFallback';
import { PrivateRoute } from '@/routes/PrivateRoute';

// Lazy-loaded page components
// Login page is kept non-lazy as it's the first page users see
import Login from '@/pages/Login/Login';

// All other pages are lazy-loaded for better performance
const Dashboard = lazy(() => import('@/pages/Dashboard/Dashboard'));
const Services = lazy(() => import('@/pages/Services/Services'));
const Instances = lazy(() => import('@/pages/Instances/Instances'));
const Cluster = lazy(() => import('@/pages/Cluster/Cluster'));
const Routing = lazy(() => import('@/pages/Routing/Routing'));
const AuditLog = lazy(() => import('@/pages/AuditLog/AuditLog'));
const ZoneOps = lazy(() => import('@/pages/ZoneOps/ZoneOps'));
const Canary = lazy(() => import('@/pages/Canary/Canary'));
const Users = lazy(() => import('@/pages/Users/Users'));

/**
 * Wrapper component that adds Suspense boundary to lazy-loaded components
 */
function LazyComponent({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<LoadingFallback />}>{children}</Suspense>;
}

/**
 * Main application router
 *
 * Created using createBrowserRouter for better data loading and error handling
 */
export const router = createBrowserRouter([
  // Root route - redirect to dashboard
  {
    path: '/',
    element: <Navigate to="/dashboard" replace />,
  },

  // Login route - no layout
  {
    path: '/login',
    element: <Login />,
  },

  // Protected routes - with MainLayout and authentication guard
  {
    element: (
      <PrivateRoute>
        <MainLayout />
      </PrivateRoute>
    ),
    children: [
      {
        path: '/dashboard',
        element: (
          <LazyComponent>
            <Dashboard />
          </LazyComponent>
        ),
      },
      {
        path: '/services',
        element: (
          <LazyComponent>
            <Services />
          </LazyComponent>
        ),
      },
      {
        path: '/instances',
        element: (
          <LazyComponent>
            <Instances />
          </LazyComponent>
        ),
      },
      {
        path: '/cluster',
        element: (
          <LazyComponent>
            <Cluster />
          </LazyComponent>
        ),
      },
      {
        path: '/routing',
        element: (
          <LazyComponent>
            <Routing />
          </LazyComponent>
        ),
      },
      {
        path: '/audit-log',
        element: (
          <LazyComponent>
            <AuditLog />
          </LazyComponent>
        ),
      },
      {
        path: '/zone-ops',
        element: (
          <LazyComponent>
            <ZoneOps />
          </LazyComponent>
        ),
      },
      {
        path: '/canary',
        element: (
          <LazyComponent>
            <Canary />
          </LazyComponent>
        ),
      },
      {
        path: '/users',
        element: (
          <LazyComponent>
            <Users />
          </LazyComponent>
        ),
      },
    ],
  },

  // Catch-all route - redirect to dashboard
  {
    path: '*',
    element: <Navigate to="/dashboard" replace />,
  },
]);

export default router;
