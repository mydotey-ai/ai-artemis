/**
 * Header Component for Artemis Console
 *
 * Material-UI based header with:
 * - Menu button for sidebar toggle
 * - Logo and application title
 * - Theme switcher (light/dark mode)
 * - User menu with dropdown actions (profile, logout)
 *
 * Features:
 * - Integrates with useUIStore for sidebar and theme state
 * - Integrates with useAuthStore for user information
 * - Responsive design with proper spacing
 * - Professional UI using Material-UI components
 *
 * Layout:
 * ┌─────────────┬──────────────────────────┬──────────────────┐
 * │ Menu | Logo │    "Artemis Console"     │ Theme | User    │
 * │         (A) │                          │  Avatar & Menu   │
 * └─────────────┴──────────────────────────┴──────────────────┘
 */

import { useMemo, useState } from 'react';
import {
  Box,
  AppBar,
  Toolbar,
  IconButton,
  Typography,
  useTheme,
  useMediaQuery,
  SxProps,
  Theme,
  Tooltip,
  Menu,
  MenuItem,
  Avatar,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Brightness4 as DarkModeIcon,
  Brightness7 as LightModeIcon,
  Close as CloseIcon,
  Logout as LogoutIcon,
  AccountCircle as AccountIcon,
  Settings as SettingsIcon,
} from '@mui/icons-material';
import { useUIStore } from '@/store/uiStore';
import { useAuthStore } from '@/store/authStore';

// ===== Type Definitions =====

/**
 * Header component props
 */
export interface HeaderProps {
  /** Optional custom title text (default: 'Artemis Console') */
  title?: string;
  /** Optional callback when sidebar toggle is clicked */
  onMenuClick?: () => void;
  /** Optional custom sx styles */
  sx?: SxProps<Theme>;
}

/**
 * Avatar color type for user avatars
 */
type AvatarColor = string;

// ===== Constants =====

/**
 * Default header title
 */
const DEFAULT_TITLE = 'Artemis Console';

/**
 * Header height in pixels (should match MainLayout HEADER_HEIGHT)
 */
const HEADER_HEIGHT = 64;

// ===== Helper Functions =====

/**
 * Generate avatar color based on username
 * Uses a hash function to consistently generate colors
 */
function getAvatarColor(username: string): AvatarColor {
  const colors: AvatarColor[] = [
    '#FF6B6B', // Red
    '#4ECDC4', // Teal
    '#45B7D1', // Blue
    '#FFA07A', // Light Salmon
    '#98D8C8', // Mint
    '#F7DC6F', // Yellow
    '#BB8FCE', // Purple
    '#85C1E2', // Sky Blue
  ];

  let hash = 0;
  for (let i = 0; i < username.length; i += 1) {
    hash = username.charCodeAt(i) + ((hash << 5) - hash);
  }

  const colorIndex = Math.abs(hash) % colors.length;
  return colors[colorIndex];
}

/**
 * Get avatar initials from username
 * Takes first letter of first and last name, or first two letters
 */
function getAvatarInitials(username: string): string {
  const parts = username.split(' ');
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }
  return username.substring(0, 2).toUpperCase();
}

// ===== Main Component =====

/**
 * Header Component
 *
 * Fixed navigation bar at the top of the application
 * Combines branding, controls, theme toggle, and user menu in a responsive layout
 *
 * Features:
 * - Menu button for sidebar toggle with tooltip
 * - Logo icon (A) and "Artemis Console" title
 * - Theme toggle button (Light/Dark mode)
 * - User avatar with dropdown menu containing:
 *   - User information (username + email)
 *   - Profile navigation
 *   - Settings navigation
 *   - Logout action
 * - Responsive design for all screen sizes
 * - Integrates with Zustand stores (useUIStore, useAuthStore)
 *
 * @param title - Optional custom header title
 * @param onMenuClick - Optional callback for menu button clicks
 * @param sx - Optional additional Material-UI sx prop
 * @returns React component
 */
export const Header: React.FC<HeaderProps> = ({
  title = DEFAULT_TITLE,
  onMenuClick,
  sx = {},
}) => {
  const theme = useTheme();

  // Check if we're on desktop (lg breakpoint and above)
  const isDesktop = useMediaQuery(theme.breakpoints.up('lg'));

  // Get theme state from Zustand store
  const currentTheme = useUIStore((state) => state.theme);
  const toggleTheme = useUIStore((state) => state.toggleTheme);

  // Get sidebar state from Zustand store
  const sidebarOpen = useUIStore((state) => state.sidebarOpen);
  const toggleSidebar = useUIStore((state) => state.toggleSidebar);

  // Get user state from auth store
  const user = useAuthStore((state) => state.user);
  const logout = useAuthStore((state) => state.logout);
  const isAuthLoading = useAuthStore((state) => state.isLoading);

  // Local state for user menu
  const [userMenuAnchor, setUserMenuAnchor] = useState<null | HTMLElement>(
    null
  );

  /**
   * Memoize the theme icon to avoid unnecessary re-renders
   */
  const themeIcon = useMemo(() => {
    return currentTheme === 'light' ? <DarkModeIcon /> : <LightModeIcon />;
  }, [currentTheme]);

  /**
   * Memoize the menu icon based on sidebar state
   * Shows MenuIcon when closed, CloseIcon when open
   */
  const menuIcon = useMemo(() => {
    return sidebarOpen ? <CloseIcon /> : <MenuIcon />;
  }, [sidebarOpen]);

  /**
   * Handle sidebar/menu toggle
   * Calls custom handler if provided, otherwise uses store action
   */
  const handleMenuToggle = (): void => {
    if (onMenuClick) {
      onMenuClick();
    } else {
      toggleSidebar();
    }
  };

  /**
   * Handle theme toggle
   */
  const handleThemeToggle = (): void => {
    toggleTheme();
  };

  /**
   * Handle user menu open
   */
  const handleUserMenuOpen = (
    event: React.MouseEvent<HTMLElement>
  ): void => {
    setUserMenuAnchor(event.currentTarget);
  };

  /**
   * Handle user menu close
   */
  const handleUserMenuClose = (): void => {
    setUserMenuAnchor(null);
  };

  /**
   * Handle profile click
   */
  const handleProfileClick = (): void => {
    handleUserMenuClose();
    // TODO: Navigate to profile page
    console.log('Navigate to profile page');
  };

  /**
   * Handle settings click
   */
  const handleSettingsClick = (): void => {
    handleUserMenuClose();
    // TODO: Navigate to settings page
    console.log('Navigate to settings page');
  };

  /**
   * Handle logout
   */
  const handleLogout = async (): Promise<void> => {
    handleUserMenuClose();
    try {
      await logout();
      // Navigation to login page is handled by router guard
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  // ===== Derived State =====
  const isUserMenuOpen = Boolean(userMenuAnchor);
  const avatarColor = user ? getAvatarColor(user.username) : '#9C27B0';
  const avatarInitials = user ? getAvatarInitials(user.username) : 'U';

  // ===== Styles =====

  /**
   * AppBar styles with custom theme awareness
   */
  const appBarSx: SxProps<Theme> = {
    height: `${HEADER_HEIGHT}px`,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    boxShadow: theme.shadows[2],
    borderBottom: `1px solid ${theme.palette.divider}`,
    ...sx,
  };

  /**
   * Toolbar styles - overrides default padding to match design
   */
  const toolbarSx: SxProps<Theme> = {
    height: HEADER_HEIGHT,
    px: { xs: 1, sm: 2, md: 3 },
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  };

  /**
   * Logo/Branding area styles
   */
  const brandingBoxSx: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'center',
    gap: 1,
    flex: 1,
  };

  /**
   * Logo/Brand icon styles
   */
  const logoBoxSx: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 40,
    height: 40,
    borderRadius: 1,
    backgroundColor: theme.palette.primary.main,
    color: 'white',
    fontWeight: 'bold',
    fontSize: '1.25rem',
    flexShrink: 0,
  };

  /**
   * Title text styles
   */
  const titleSx: SxProps<Theme> = {
    fontWeight: 700,
    fontSize: { xs: '1rem', sm: '1.25rem' },
    color: theme.palette.text.primary,
    letterSpacing: 0.5,
  };

  /**
   * Controls section (theme toggle, menu) styles
   */
  const controlsBoxSx: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'center',
    gap: 0.5,
  };

  /**
   * Icon button base styles
   */
  const iconButtonSx: SxProps<Theme> = {
    color: theme.palette.text.primary,
    transition: theme.transitions.create(['color', 'background-color'], {
      duration: theme.transitions.duration.standard,
    }),
    '&:hover': {
      backgroundColor: theme.palette.action.hover,
    },
    '&:active': {
      backgroundColor: theme.palette.action.selected,
    },
  };

  // ===== Render =====

  return (
    <AppBar position="static" sx={appBarSx} elevation={0}>
      <Toolbar sx={toolbarSx}>
        {/* ===== Logo/Branding Section ===== */}
        <Box sx={brandingBoxSx}>
          {/* Menu toggle button (mobile only) */}
          {!isDesktop && (
            <Tooltip title={sidebarOpen ? 'Close menu' : 'Open menu'}>
              <IconButton
                edge="start"
                color="inherit"
                aria-label="toggle menu"
                onClick={handleMenuToggle}
                sx={iconButtonSx}
              >
                {menuIcon}
              </IconButton>
            </Tooltip>
          )}

          {/* Logo box */}
          <Box sx={logoBoxSx}>A</Box>

          {/* Title text */}
          <Typography variant="h6" sx={titleSx} noWrap>
            {title}
          </Typography>
        </Box>

        {/* ===== Spacer ===== */}
        <Box sx={{ flex: 1 }} />

        {/* ===== Controls Section ===== */}
        <Box sx={controlsBoxSx}>
          {/* Theme toggle button */}
          <Tooltip
            title={
              currentTheme === 'light'
                ? 'Switch to dark mode'
                : 'Switch to light mode'
            }
          >
            <IconButton
              color="inherit"
              aria-label="toggle theme"
              onClick={handleThemeToggle}
              sx={iconButtonSx}
            >
              {themeIcon}
            </IconButton>
          </Tooltip>

          {/* User Menu Section */}
          {user ? (
            <>
              {/* User Avatar Button */}
              <Tooltip title={user.username} arrow>
                <IconButton
                  onClick={handleUserMenuOpen}
                  size="small"
                  aria-controls={
                    isUserMenuOpen ? 'user-menu' : undefined
                  }
                  aria-haspopup="true"
                  aria-expanded={isUserMenuOpen ? 'true' : undefined}
                  sx={{
                    p: 0,
                    '&:hover': {
                      backgroundColor: theme.palette.action.hover,
                    },
                  }}
                >
                  <Avatar
                    sx={{
                      backgroundColor: avatarColor,
                      width: 36,
                      height: 36,
                      fontSize: '0.875rem',
                      fontWeight: 600,
                      cursor: 'pointer',
                    }}
                  >
                    {avatarInitials}
                  </Avatar>
                </IconButton>
              </Tooltip>

              {/* User Dropdown Menu */}
              <Menu
                id="user-menu"
                anchorEl={userMenuAnchor}
                open={isUserMenuOpen}
                onClose={handleUserMenuClose}
                anchorOrigin={{
                  vertical: 'bottom',
                  horizontal: 'right',
                }}
                transformOrigin={{
                  vertical: 'top',
                  horizontal: 'right',
                }}
                slotProps={{
                  paper: {
                    elevation: 3,
                    sx: {
                      minWidth: 220,
                      mt: 1,
                    },
                  },
                }}
              >
                {/* User Info Header */}
                <MenuItem
                  disabled
                  sx={{
                    py: 1.5,
                    px: 2,
                  }}
                >
                  <Box sx={{ display: 'flex', flexDirection: 'column' }}>
                    <Typography
                      variant="subtitle2"
                      sx={{
                        fontWeight: 600,
                        color: theme.palette.text.primary,
                      }}
                    >
                      {user.username}
                    </Typography>
                    <Typography
                      variant="caption"
                      sx={{
                        color: theme.palette.text.secondary,
                        mt: 0.5,
                      }}
                    >
                      {user.email || 'No email'}
                    </Typography>
                  </Box>
                </MenuItem>

                {/* Divider */}
                <Box
                  sx={{
                    my: 0.5,
                    borderBottom: 1,
                    borderColor: theme.palette.divider,
                  }}
                />

                {/* Profile Menu Item */}
                <MenuItem
                  onClick={handleProfileClick}
                  sx={{
                    '&:hover': {
                      backgroundColor: theme.palette.action.hover,
                    },
                  }}
                >
                  <AccountIcon
                    sx={{
                      mr: 1.5,
                      fontSize: '1.25rem',
                      color: theme.palette.text.secondary,
                    }}
                  />
                  <Typography variant="body2">Profile</Typography>
                </MenuItem>

                {/* Settings Menu Item */}
                <MenuItem
                  onClick={handleSettingsClick}
                  sx={{
                    '&:hover': {
                      backgroundColor: theme.palette.action.hover,
                    },
                  }}
                >
                  <SettingsIcon
                    sx={{
                      mr: 1.5,
                      fontSize: '1.25rem',
                      color: theme.palette.text.secondary,
                    }}
                  />
                  <Typography variant="body2">Settings</Typography>
                </MenuItem>

                {/* Divider before Logout */}
                <Box
                  sx={{
                    my: 0.5,
                    borderBottom: 1,
                    borderColor: theme.palette.divider,
                  }}
                />

                {/* Logout Menu Item */}
                <MenuItem
                  onClick={handleLogout}
                  disabled={isAuthLoading}
                  sx={{
                    color: theme.palette.error.main,
                    '&:hover': {
                      backgroundColor: 'rgba(211, 47, 47, 0.04)',
                    },
                    '&.Mui-disabled': {
                      color: theme.palette.action.disabled,
                    },
                  }}
                >
                  <LogoutIcon
                    sx={{
                      mr: 1.5,
                      fontSize: '1.25rem',
                    }}
                  />
                  <Typography variant="body2">
                    {isAuthLoading ? 'Logging out...' : 'Logout'}
                  </Typography>
                </MenuItem>
              </Menu>
            </>
          ) : (
            /* Not logged in - show disabled avatar */
            <Tooltip title="Not logged in" arrow>
              <IconButton
                disabled
                size="small"
                sx={{
                  p: 0,
                  '&.Mui-disabled': {
                    cursor: 'default',
                  },
                }}
              >
                <Avatar
                  sx={{
                    backgroundColor: theme.palette.action.disabled,
                    width: 36,
                    height: 36,
                    fontSize: '0.875rem',
                    color: theme.palette.text.disabled,
                  }}
                >
                  ?
                </Avatar>
              </IconButton>
            </Tooltip>
          )}
        </Box>
      </Toolbar>
    </AppBar>
  );
};

/**
 * Display name for debugging
 */
Header.displayName = 'Header';

/**
 * Export default for convenience
 */
export default Header;
