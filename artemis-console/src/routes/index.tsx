/**
 * Main routing configuration for Artemis Console
 *
 * Uses react-router-dom v6's createBrowserRouter API
 * Route structure:
 * - / → Redirect to /dashboard
 * - /login → Login page (no layout)
 * - Protected routes (with MainLayout):
 *   - /dashboard → Dashboard
 *   - /services → Services
 *   - /instances → Instances
 *   - /cluster → Cluster
 *   - /routing → Routing
 *   - /audit-log → AuditLog
 *   - /zone-ops → ZoneOps
 *   - /canary → Canary
 *   - /users → Users
 */

import { createBrowserRouter, Navigate } from 'react-router-dom';
import { MainLayout } from '@/components/Layout/MainLayout';

// Page imports
import Login from '@/pages/Login/Login';
import Dashboard from '@/pages/Dashboard/Dashboard';
import Services from '@/pages/Services/Services';
import Instances from '@/pages/Instances/Instances';
import Cluster from '@/pages/Cluster/Cluster';
import Routing from '@/pages/Routing/Routing';
import AuditLog from '@/pages/AuditLog/AuditLog';
import ZoneOps from '@/pages/ZoneOps/ZoneOps';
import Canary from '@/pages/Canary/Canary';
import Users from '@/pages/Users/Users';

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

  // Protected routes - with MainLayout
  {
    element: <MainLayout />,
    children: [
      {
        path: '/dashboard',
        element: <Dashboard />,
      },
      {
        path: '/services',
        element: <Services />,
      },
      {
        path: '/instances',
        element: <Instances />,
      },
      {
        path: '/cluster',
        element: <Cluster />,
      },
      {
        path: '/routing',
        element: <Routing />,
      },
      {
        path: '/audit-log',
        element: <AuditLog />,
      },
      {
        path: '/zone-ops',
        element: <ZoneOps />,
      },
      {
        path: '/canary',
        element: <Canary />,
      },
      {
        path: '/users',
        element: <Users />,
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
