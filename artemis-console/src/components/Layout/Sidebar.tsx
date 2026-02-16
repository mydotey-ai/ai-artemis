/**
 * Sidebar Navigation Component
 *
 * Material-UI Drawer-based responsive sidebar with navigation links
 * Features:
 * - Permanent drawer on desktop (breakpoint: lg and above)
 * - Temporary drawer on mobile (breakpoint: md and below)
 * - Navigation menu items with icons
 * - Active route highlighting
 * - Responsive design using Material-UI breakpoints
 * - Theme-aware styling
 * - Zustand state management for drawer open/close
 */

import React, { useMemo } from 'react';
import { useLocation, Link } from 'react-router-dom';
import {
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemButton,
  ListItemText,
  Box,
  useTheme,
  useMediaQuery,
  Divider,
  Typography,
} from '@mui/material';
import {
  Dashboard as DashboardIcon,
  Cloud as CloudIcon,
  Devices as DevicesIcon,
  AccountTree as AccountTreeIcon,
  Route as RouteIcon,
  History as HistoryIcon,
  Public as PublicIcon,
  Science as ScienceIcon,
  People as PeopleIcon,
} from '@mui/icons-material';
import { useUIStore } from '@/store/uiStore';

// ===== Type Definitions =====

/**
 * Navigation menu item definition
 */
interface NavMenuItem {
  /** Unique identifier for the menu item */
  id: string;
  /** Display label */
  label: string;
  /** Route path for navigation */
  path: string;
  /** Material-UI icon component */
  icon: React.ComponentType<{ className?: string }>;
  /** Optional description for tooltips */
  description?: string;
}

/**
 * Sidebar component props
 */
interface SidebarProps {
  /** Custom width for the drawer (default: 280px) */
  drawerWidth?: number;
  /** Optional CSS class name */
  className?: string;
}

// ===== Constants =====

/**
 * Default drawer width in pixels
 */
const DEFAULT_DRAWER_WIDTH = 280;

/**
 * Navigation menu items
 * Each item maps to a feature area in the console
 */
const NAVIGATION_ITEMS: NavMenuItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    path: '/dashboard',
    icon: DashboardIcon,
    description: 'Overview and key metrics',
  },
  {
    id: 'services',
    label: 'Services',
    path: '/services',
    icon: CloudIcon,
    description: 'Service registry and discovery',
  },
  {
    id: 'instances',
    label: 'Instances',
    path: '/instances',
    icon: DevicesIcon,
    description: 'Service instances management',
  },
  {
    id: 'cluster',
    label: 'Cluster',
    path: '/cluster',
    icon: AccountTreeIcon,
    description: 'Cluster nodes and health',
  },
  {
    id: 'routing',
    label: 'Routing',
    path: '/routing',
    icon: RouteIcon,
    description: 'Routing rules and configuration',
  },
  {
    id: 'audit-log',
    label: 'Audit Log',
    path: '/audit-log',
    icon: HistoryIcon,
    description: 'Operation history and logs',
  },
  {
    id: 'zone-ops',
    label: 'Zone Ops',
    path: '/zone-ops',
    icon: PublicIcon,
    description: 'Zone-based operations',
  },
  {
    id: 'canary',
    label: 'Canary',
    path: '/canary',
    icon: ScienceIcon,
    description: 'Canary deployment management',
  },
  {
    id: 'users',
    label: 'Users',
    path: '/users',
    icon: PeopleIcon,
    description: 'User management and permissions',
  },
];

// ===== Helper Functions =====

/**
 * Check if a path is the current active route
 * Supports partial path matching for nested routes
 *
 * @param currentPath - The current location pathname
 * @param itemPath - The navigation item path
 * @returns true if the item represents the current active route
 */
function isActiveRoute(currentPath: string, itemPath: string): boolean {
  if (itemPath === '/dashboard' && currentPath === '/') {
    return true;
  }
  return currentPath.startsWith(itemPath);
}

/**
 * Navigation List Item Component
 * Handles rendering of individual menu items with proper styling and routing
 */
interface NavItemProps {
  item: NavMenuItem;
  isActive: boolean;
  onItemClick?: () => void;
}

const NavItem: React.FC<NavItemProps> = ({ item, isActive, onItemClick }) => {
  const theme = useTheme();
  const IconComponent = item.icon;

  return (
    <ListItem
      disablePadding
      key={item.id}
      sx={{
        transition: theme.transitions.create(['background-color'], {
          duration: theme.transitions.duration.standard,
        }),
      }}
    >
      <ListItemButton
        component={Link}
        to={item.path}
        onClick={onItemClick}
        selected={isActive}
        sx={{
          pl: 2,
          pr: 2,
          py: 1.5,
          '&.Mui-selected': {
            backgroundColor: theme.palette.action.selected,
            borderLeft: `4px solid ${theme.palette.primary.main}`,
            pl: 'calc(2rem - 4px)',
            '&:hover': {
              backgroundColor: theme.palette.action.selected,
            },
          },
          '&:hover': {
            backgroundColor: theme.palette.action.hover,
          },
        }}
      >
        <ListItemIcon
          sx={{
            minWidth: 40,
            color: isActive ? theme.palette.primary.main : 'inherit',
            transition: theme.transitions.create(['color'], {
              duration: theme.transitions.duration.standard,
            }),
          }}
        >
          <IconComponent />
        </ListItemIcon>
        <ListItemText
          primary={item.label}
          primaryTypographyProps={{
            fontWeight: isActive ? 600 : 500,
            fontSize: '0.95rem',
          }}
          sx={{
            '& .MuiTypography-root': {
              transition: theme.transitions.create(['font-weight', 'color'], {
                duration: theme.transitions.duration.standard,
              }),
            },
          }}
        />
      </ListItemButton>
    </ListItem>
  );
};

// ===== Main Component =====

/**
 * Sidebar Component
 *
 * Responsive navigation sidebar with Material-UI Drawer
 * - Desktop: Permanent drawer (lg breakpoint and above)
 * - Mobile: Temporary drawer that opens/closes (below lg)
 * - Features active route highlighting and smooth transitions
 *
 * @param drawerWidth - Optional custom drawer width (default: 280px)
 * @param className - Optional CSS class name
 * @returns React component
 */
export const Sidebar: React.FC<SidebarProps> = ({
  drawerWidth = DEFAULT_DRAWER_WIDTH,
  className,
}) => {
  const theme = useTheme();
  const location = useLocation();

  // Check if we're on desktop (lg breakpoint and above)
  // On desktop, drawer is permanent; on mobile, it's temporary
  const isDesktop = useMediaQuery(theme.breakpoints.up('lg'));

  // Get sidebar state from Zustand store
  const sidebarOpen = useUIStore((state) => state.sidebarOpen);
  const setSidebarOpen = useUIStore((state) => state.setSidebarOpen);

  // Memoize active items calculation to avoid unnecessary re-renders
  const activeItems = useMemo(() => {
    return new Set(
      NAVIGATION_ITEMS.map((item) =>
        isActiveRoute(location.pathname, item.path) ? item.id : null
      ).filter((id) => id !== null)
    );
  }, [location.pathname]);

  /**
   * Handle drawer toggle on mobile
   */
  const handleDrawerToggle = (): void => {
    setSidebarOpen(!sidebarOpen);
  };

  /**
   * Handle navigation item click
   * On mobile, close the drawer after navigation
   */
  const handleNavItemClick = (): void => {
    if (!isDesktop) {
      setSidebarOpen(false);
    }
  };

  /**
   * Render the drawer content
   */
  const drawerContent = (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
      }}
    >
      {/* Header Section */}
      <Box
        sx={{
          p: 2,
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          borderBottom: `1px solid ${theme.palette.divider}`,
        }}
      >
        <Box
          sx={{
            width: 40,
            height: 40,
            borderRadius: 1,
            backgroundColor: theme.palette.primary.main,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontWeight: 'bold',
          }}
        >
          A
        </Box>
        <Box>
          <Typography
            variant="subtitle1"
            sx={{
              fontWeight: 700,
              lineHeight: 1,
            }}
          >
            Artemis
          </Typography>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{
              lineHeight: 1,
            }}
          >
            Registry
          </Typography>
        </Box>
      </Box>

      {/* Navigation List */}
      <List
        component="nav"
        sx={{
          flex: 1,
          overflow: 'auto',
          py: 1,
        }}
      >
        {NAVIGATION_ITEMS.map((item) => (
          <NavItem
            key={item.id}
            item={item}
            isActive={activeItems.has(item.id)}
            onItemClick={handleNavItemClick}
          />
        ))}
      </List>

      {/* Footer Section */}
      <Box
        sx={{
          p: 2,
          borderTop: `1px solid ${theme.palette.divider}`,
        }}
      >
        <Divider sx={{ mb: 1 }} />
        <Typography
          variant="caption"
          color="text.secondary"
          display="block"
          sx={{
            textAlign: 'center',
            py: 1,
          }}
        >
          Artemis v1.0
        </Typography>
      </Box>
    </Box>
  );

  // Desktop: Permanent drawer
  if (isDesktop) {
    return (
      <Drawer
        variant="permanent"
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
            borderRight: `1px solid ${theme.palette.divider}`,
          },
        }}
        className={className}
      >
        {drawerContent}
      </Drawer>
    );
  }

  // Mobile: Temporary drawer
  return (
    <Drawer
      variant="temporary"
      anchor="left"
      open={sidebarOpen}
      onClose={handleDrawerToggle}
      ModalProps={{
        keepMounted: true, // Better mobile performance
      }}
      sx={{
        display: { xs: 'block', lg: 'none' },
        '& .MuiDrawer-paper': {
          width: drawerWidth,
          boxSizing: 'border-box',
        },
      }}
      className={className}
    >
      {drawerContent}
    </Drawer>
  );
};

// Export for backward compatibility
Sidebar.displayName = 'Sidebar';

export default Sidebar;
