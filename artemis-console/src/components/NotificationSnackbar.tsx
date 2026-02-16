/**
 * Notification Snackbar Component
 *
 * Displays stacked notifications using Material-UI Snackbar
 * Integrates with uiStore notification system
 *
 * Features:
 * - Auto-stacking multiple notifications
 * - Auto-dismiss after duration
 * - Manual dismiss with close button
 * - Action button support
 * - Different severity levels (success, error, warning, info)
 */

import React from 'react';
import {
  Snackbar,
  Alert,
  Button,
  IconButton,
  type AlertColor,
} from '@mui/material';
import { Close as CloseIcon } from '@mui/icons-material';
import { useUIStore } from '@/store/uiStore';
import type { Notification } from '@/store/uiStore';

/**
 * Map notification type to MUI AlertColor
 */
function mapNotificationTypeToAlertColor(type: Notification['type']): AlertColor {
  const mapping: Record<Notification['type'], AlertColor> = {
    success: 'success',
    error: 'error',
    warning: 'warning',
    info: 'info',
  };
  return mapping[type];
}

/**
 * Notification Snackbar Component
 *
 * Automatically displays notifications from uiStore
 */
export const NotificationSnackbar: React.FC = () => {
  const notifications = useUIStore((state) => state.notifications);
  const hideNotification = useUIStore((state) => state.hideNotification);

  // Only show the first notification (oldest)
  const currentNotification = notifications[0];

  if (!currentNotification) {
    return null;
  }

  const handleClose = (_event?: React.SyntheticEvent | Event, reason?: string) => {
    // Don't close on clickaway
    if (reason === 'clickaway') {
      return;
    }

    hideNotification(currentNotification.id);
  };

  const handleActionClick = () => {
    if (currentNotification.action?.onClick) {
      currentNotification.action.onClick();
    }
    hideNotification(currentNotification.id);
  };

  return (
    <Snackbar
      open
      autoHideDuration={currentNotification.duration || null}
      onClose={handleClose}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
    >
      <Alert
        severity={mapNotificationTypeToAlertColor(currentNotification.type)}
        variant="filled"
        onClose={handleClose}
        action={
          currentNotification.action ? (
            <>
              <Button
                color="inherit"
                size="small"
                onClick={handleActionClick}
              >
                {currentNotification.action.label}
              </Button>
              <IconButton
                size="small"
                aria-label="close"
                color="inherit"
                onClick={handleClose}
              >
                <CloseIcon fontSize="small" />
              </IconButton>
            </>
          ) : undefined
        }
        sx={{
          width: '100%',
          minWidth: 300,
          maxWidth: 600,
        }}
      >
        {currentNotification.title && (
          <strong>{currentNotification.title}: </strong>
        )}
        {currentNotification.message}
      </Alert>
    </Snackbar>
  );
};

/**
 * Display name for debugging
 */
NotificationSnackbar.displayName = 'NotificationSnackbar';

export default NotificationSnackbar;
