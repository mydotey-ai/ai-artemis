/**
 * MainLayout Component
 *
 * Master layout component for the Artemis Console application
 * Features:
 * - Fixed header at the top with z-index: 1100
 * - Responsive sidebar (permanent on desktop, collapsible on mobile)
 * - Main content area with proper spacing and padding
 * - Outlet for react-router-dom child routes
 * - Smooth transitions and responsive design
 * - Theme-aware styling with Material-UI
 *
 * Layout Structure:
 * ┌─────────────────────────────────┐
 * │           Header (z: 1100)       │ (64px height)
 * ├─────────────────────────────────┤
 * │ │                               │
 * │ │      Main Content Area        │
 * │ │      (Sidebar: 0-280px)       │
 * │ │      (Outlet)                 │
 * │ │                               │
 * └─────────────────────────────────┘
 */

import React, { type ReactNode } from 'react';
import { Outlet } from 'react-router-dom';
import {
  Box,
  Container,
  useTheme,
  useMediaQuery,
  type SxProps,
  type Theme,
} from '@mui/material';
import { Sidebar } from './Sidebar';
import { Header } from './Header';
import { NotificationSnackbar } from '@/components/NotificationSnackbar';

// ===== Type Definitions =====

/**
 * MainLayout component props
 */
interface MainLayoutProps {
  /** Optional children to render (alternative to using Outlet) */
  children?: ReactNode;
  /** Optional custom container max width (default: 'lg') */
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | false;
  /** Optional additional sx styles for the main container */
  containerSx?: SxProps<Theme>;
  /** Custom drawer width in pixels (default: 240) */
  drawerWidth?: number;
}

// ===== Constants =====

/**
 * Default drawer width for sidebar
 */
const DEFAULT_DRAWER_WIDTH = 240;

/**
 * Header height in pixels
 */
const HEADER_HEIGHT = 64;

/**
 * Z-index for header (Material-UI AppBar uses 1100 by default)
 */
const HEADER_Z_INDEX = 1100;

/**
 * Z-index for sidebar (should be below header but above main content)
 */
const SIDEBAR_Z_INDEX = 1000;

/**
 * Transition duration for smooth animations (milliseconds)
 */
const TRANSITION_DURATION = 225;

// ===== Main Component =====

/**
 * MainLayout Component
 *
 * Primary layout wrapper for all authenticated pages
 * Combines Header and Sidebar in a responsive box layout
 * Uses Material-UI Box for layout and Container for content wrapping
 *
 * Responsive Behavior:
 * - Desktop (lg+): Permanent sidebar (240px width)
 * - Tablet/Mobile (below lg): Temporary sidebar (drawer opens from left)
 *
 * @param children - Optional React components to render as main content
 * @param maxWidth - Container max width (default: 'lg')
 * @param containerSx - Additional Material-UI sx prop for the container
 * @param drawerWidth - Custom sidebar width (default: 240px)
 * @returns React component
 */
export const MainLayout: React.FC<MainLayoutProps> = ({
  children,
  maxWidth = 'lg',
  containerSx = {},
  drawerWidth = DEFAULT_DRAWER_WIDTH,
}) => {
  const theme = useTheme();

  // Check if we're on desktop (lg breakpoint and above)
  const isDesktop = useMediaQuery(theme.breakpoints.up('lg'));

  /**
   * Main content area margin-left accounts for sidebar width
   * Used for desktop layout where sidebar is permanent
   */
  const contentMarginLeft = isDesktop ? drawerWidth : 0;

  // ===== Styles =====

  /**
   * Root container that wraps everything
   * Uses flex layout to stack header above main content
   */
  const rootBoxSx: SxProps<Theme> = {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    overflow: 'hidden',
    backgroundColor: theme.palette.background.default,
  };

  /**
   * Header wrapper
   * Fixed at top with proper z-index
   */
  const headerBoxSx: SxProps<Theme> = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    height: `${HEADER_HEIGHT}px`,
    zIndex: HEADER_Z_INDEX,
    backgroundColor: theme.palette.background.paper,
    borderBottom: `1px solid ${theme.palette.divider}`,
    display: 'flex',
    alignItems: 'center',
    paddingLeft: isDesktop ? `${drawerWidth}px` : 0,
    transition: theme.transitions.create(['padding-left'], {
      duration: TRANSITION_DURATION,
      easing: theme.transitions.easing.easeInOut,
    }),
  };

  /**
   * Main content area wrapper
   * Positioned below header, accounts for sidebar width on desktop
   */
  const mainBoxSx: SxProps<Theme> = {
    display: 'flex',
    flex: 1,
    marginTop: `${HEADER_HEIGHT}px`,
    overflow: 'hidden',
    position: 'relative',
  };

  /**
   * Sidebar wrapper (desktop permanent)
   * Fixed on desktop, hidden on mobile
   */
  const sidebarWrapperSx: SxProps<Theme> = {
    position: 'fixed',
    left: 0,
    top: HEADER_HEIGHT,
    width: drawerWidth,
    height: `calc(100vh - ${HEADER_HEIGHT}px)`,
    zIndex: SIDEBAR_Z_INDEX,
    display: { xs: 'none', lg: 'block' },
    backgroundColor: theme.palette.background.paper,
    borderRight: `1px solid ${theme.palette.divider}`,
    overflow: 'hidden',
  };

  /**
   * Content area that takes up remaining space
   * Adjusts margin based on sidebar width on desktop
   */
  const contentWrapperSx: SxProps<Theme> = {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    marginLeft: isDesktop ? `${contentMarginLeft}px` : 0,
    overflow: 'auto',
    transition: theme.transitions.create(['margin-left'], {
      duration: TRANSITION_DURATION,
      easing: theme.transitions.easing.easeInOut,
    }),
  };

  /**
   * Container wrapper for main content
   * Provides padding and max-width constraint
   */
  const containerWrapperSx: SxProps<Theme> = {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    py: 3,
    px: { xs: 2, sm: 3, md: 4 },
    ...containerSx,
  };

  // ===== Render =====

  return (
    <Box sx={rootBoxSx}>
      {/* ===== Header ===== */}
      <Box sx={headerBoxSx}>
        <Header />
      </Box>

      {/* ===== Main Content Area ===== */}
      <Box sx={mainBoxSx}>
        {/* ===== Sidebar (Desktop Only - Permanent Drawer) ===== */}
        <Box sx={sidebarWrapperSx}>
          <Sidebar drawerWidth={drawerWidth} />
        </Box>

        {/* ===== Sidebar (Mobile - Temporary Drawer) ===== */}
        {/* Note: Sidebar component handles mobile drawer internally */}

        {/* ===== Content Wrapper ===== */}
        <Box sx={contentWrapperSx}>
          <Container maxWidth={maxWidth} sx={containerWrapperSx}>
            {/* Render children if provided, otherwise use Outlet for routing */}
            {children || <Outlet />}
          </Container>
        </Box>
      </Box>

      {/* ===== Notification Snackbar ===== */}
      <NotificationSnackbar />
    </Box>
  );
};

/**
 * Display name for debugging
 */
MainLayout.displayName = 'MainLayout';

/**
 * Export default for convenience
 */
export default MainLayout;
