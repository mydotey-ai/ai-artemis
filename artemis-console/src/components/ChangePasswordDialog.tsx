/**
 * Change Password Dialog Component
 *
 * Features:
 * - Material-UI Dialog with form layout
 * - Current password verification
 * - New password with strength validation
 * - Confirm password with match validation
 * - Real-time validation feedback
 * - Submit button with loading state
 * - Success/error handling
 * - Forces re-login after successful change
 */

import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  Box,
  Alert,
  CircularProgress,
  InputAdornment,
  IconButton,
  LinearProgress,
  Typography,
  type SxProps,
  type Theme,
} from '@mui/material';
import { Visibility, VisibilityOff } from '@mui/icons-material';
import { changePassword } from '@/api/auth';
import { useAuthStore } from '@/store/authStore';

// ===== Type Definitions =====

/**
 * ChangePasswordDialog component props
 */
export interface ChangePasswordDialogProps {
  /** Whether the dialog is open */
  open: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
}

/**
 * Password strength level
 */
type PasswordStrength = 'weak' | 'medium' | 'strong' | 'very-strong';

// ===== Validation Constants =====

const PASSWORD_MIN_LENGTH = 8;
const PASSWORD_REGEX = {
  hasLowerCase: /[a-z]/,
  hasUpperCase: /[A-Z]/,
  hasNumber: /[0-9]/,
  hasSpecial: /[!@#$%^&*(),.?":{}|<>]/,
};

// ===== Helper Functions =====

/**
 * Calculate password strength
 */
function getPasswordStrength(password: string): PasswordStrength {
  if (password.length < PASSWORD_MIN_LENGTH) {
    return 'weak';
  }

  let score = 0;

  // Length bonus
  if (password.length >= 12) score += 1;
  if (password.length >= 16) score += 1;

  // Character type bonuses
  if (PASSWORD_REGEX.hasLowerCase.test(password)) score += 1;
  if (PASSWORD_REGEX.hasUpperCase.test(password)) score += 1;
  if (PASSWORD_REGEX.hasNumber.test(password)) score += 1;
  if (PASSWORD_REGEX.hasSpecial.test(password)) score += 1;

  if (score <= 2) return 'weak';
  if (score <= 4) return 'medium';
  if (score <= 5) return 'strong';
  return 'very-strong';
}

/**
 * Get strength color
 */
function getStrengthColor(strength: PasswordStrength): string {
  switch (strength) {
    case 'weak':
      return '#d32f2f';
    case 'medium':
      return '#ff9800';
    case 'strong':
      return '#2e7d32';
    case 'very-strong':
      return '#1976d2';
  }
}

/**
 * Get strength progress value (0-100)
 */
function getStrengthProgress(strength: PasswordStrength): number {
  switch (strength) {
    case 'weak':
      return 25;
    case 'medium':
      return 50;
    case 'strong':
      return 75;
    case 'very-strong':
      return 100;
  }
}

/**
 * Validate new password
 */
function validateNewPassword(password: string): string | null {
  if (!password) {
    return 'New password is required';
  }
  if (password.length < PASSWORD_MIN_LENGTH) {
    return `Password must be at least ${PASSWORD_MIN_LENGTH} characters`;
  }
  if (!PASSWORD_REGEX.hasLowerCase.test(password)) {
    return 'Password must contain at least one lowercase letter';
  }
  if (!PASSWORD_REGEX.hasUpperCase.test(password)) {
    return 'Password must contain at least one uppercase letter';
  }
  if (!PASSWORD_REGEX.hasNumber.test(password)) {
    return 'Password must contain at least one number';
  }
  return null;
}

// ===== Main Component =====

/**
 * ChangePasswordDialog Component
 *
 * @param open - Whether dialog is open
 * @param onClose - Callback when dialog should close
 * @returns React component
 */
export const ChangePasswordDialog: React.FC<ChangePasswordDialogProps> = ({
  open,
  onClose,
}) => {
  // Auth store
  const logout = useAuthStore((state) => state.logout);

  // Form state
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  // Visibility state
  const [showCurrentPassword, setShowCurrentPassword] = useState(false);
  const [showNewPassword, setShowNewPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  // Validation state
  const [currentPasswordError, setCurrentPasswordError] = useState<string | null>(null);
  const [newPasswordError, setNewPasswordError] = useState<string | null>(null);
  const [confirmPasswordError, setConfirmPasswordError] = useState<string | null>(null);
  const [touched, setTouched] = useState({
    currentPassword: false,
    newPassword: false,
    confirmPassword: false,
  });

  // UI state
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  // Password strength
  const passwordStrength = getPasswordStrength(newPassword);

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (!open) {
      // Reset form after animation completes
      setTimeout(() => {
        setCurrentPassword('');
        setNewPassword('');
        setConfirmPassword('');
        setCurrentPasswordError(null);
        setNewPasswordError(null);
        setConfirmPasswordError(null);
        setTouched({
          currentPassword: false,
          newPassword: false,
          confirmPassword: false,
        });
        setError(null);
        setSuccess(false);
      }, 300);
    }
  }, [open]);

  /**
   * Handle current password change
   */
  const handleCurrentPasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setCurrentPassword(value);
    if (error) setError(null);

    if (touched.currentPassword) {
      setCurrentPasswordError(value ? null : 'Current password is required');
    }
  };

  /**
   * Handle new password change
   */
  const handleNewPasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setNewPassword(value);
    if (error) setError(null);

    if (touched.newPassword) {
      setNewPasswordError(validateNewPassword(value));
    }

    // Revalidate confirm password if it's been touched
    if (touched.confirmPassword && confirmPassword) {
      setConfirmPasswordError(
        value === confirmPassword ? null : 'Passwords do not match'
      );
    }
  };

  /**
   * Handle confirm password change
   */
  const handleConfirmPasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setConfirmPassword(value);
    if (error) setError(null);

    if (touched.confirmPassword) {
      setConfirmPasswordError(
        value === newPassword ? null : 'Passwords do not match'
      );
    }
  };

  /**
   * Handle field blur
   */
  const handleBlur = (field: keyof typeof touched) => {
    setTouched((prev) => ({ ...prev, [field]: true }));

    switch (field) {
      case 'currentPassword':
        setCurrentPasswordError(currentPassword ? null : 'Current password is required');
        break;
      case 'newPassword':
        setNewPasswordError(validateNewPassword(newPassword));
        break;
      case 'confirmPassword':
        setConfirmPasswordError(
          confirmPassword === newPassword ? null : 'Passwords do not match'
        );
        break;
    }
  };

  /**
   * Handle form submission
   */
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Mark all fields as touched
    setTouched({
      currentPassword: true,
      newPassword: true,
      confirmPassword: true,
    });

    // Validate all fields
    const currentErr = currentPassword ? null : 'Current password is required';
    const newErr = validateNewPassword(newPassword);
    const confirmErr = confirmPassword === newPassword ? null : 'Passwords do not match';

    setCurrentPasswordError(currentErr);
    setNewPasswordError(newErr);
    setConfirmPasswordError(confirmErr);

    // Stop if validation fails
    if (currentErr || newErr || confirmErr) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Call change password API
      const response = await changePassword(currentPassword, newPassword);

      if (!response.success) {
        throw new Error(response.message || 'Failed to change password');
      }

      // Show success message
      setSuccess(true);

      // Force logout after 2 seconds
      setTimeout(async () => {
        await logout();
        onClose();
      }, 2000);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Failed to change password';
      setError(errorMessage);
      setIsLoading(false);
    }
  };

  // Check if form is valid
  const isFormValid =
    !currentPasswordError &&
    !newPasswordError &&
    !confirmPasswordError &&
    currentPassword &&
    newPassword &&
    confirmPassword;

  // ===== Styles =====

  const dialogContentSx: SxProps<Theme> = {
    display: 'flex',
    flexDirection: 'column',
    gap: 2,
    paddingTop: 2,
    minWidth: { xs: 300, sm: 400 },
  };

  return (
    <Dialog open={open} onClose={isLoading ? undefined : onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Change Password</DialogTitle>
      <DialogContent sx={dialogContentSx}>
        {/* Error Alert */}
        {error && !success && (
          <Alert severity="error" onClose={() => setError(null)}>
            {error}
          </Alert>
        )}

        {/* Success Alert */}
        {success && (
          <Alert severity="success">
            Password changed successfully! You will be logged out in a moment...
          </Alert>
        )}

        {/* Form */}
        <Box component="form" onSubmit={handleSubmit}>
          {/* Current Password */}
          <TextField
            fullWidth
            label="Current Password"
            type={showCurrentPassword ? 'text' : 'password'}
            value={currentPassword}
            onChange={handleCurrentPasswordChange}
            onBlur={() => handleBlur('currentPassword')}
            error={touched.currentPassword && !!currentPasswordError}
            helperText={touched.currentPassword ? currentPasswordError : ''}
            disabled={isLoading || success}
            required
            autoFocus
            sx={{ marginBottom: 2 }}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    onClick={() => setShowCurrentPassword(!showCurrentPassword)}
                    edge="end"
                    disabled={isLoading || success}
                  >
                    {showCurrentPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />

          {/* New Password */}
          <TextField
            fullWidth
            label="New Password"
            type={showNewPassword ? 'text' : 'password'}
            value={newPassword}
            onChange={handleNewPasswordChange}
            onBlur={() => handleBlur('newPassword')}
            error={touched.newPassword && !!newPasswordError}
            helperText={touched.newPassword ? newPasswordError : ''}
            disabled={isLoading || success}
            required
            sx={{ marginBottom: 1 }}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    onClick={() => setShowNewPassword(!showNewPassword)}
                    edge="end"
                    disabled={isLoading || success}
                  >
                    {showNewPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />

          {/* Password Strength Indicator */}
          {newPassword && (
            <Box sx={{ marginBottom: 2 }}>
              <Box
                sx={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  marginBottom: 0.5,
                }}
              >
                <Typography variant="caption" color="text.secondary">
                  Password Strength:
                </Typography>
                <Typography
                  variant="caption"
                  sx={{
                    color: getStrengthColor(passwordStrength),
                    fontWeight: 600,
                    textTransform: 'capitalize',
                  }}
                >
                  {passwordStrength.replace('-', ' ')}
                </Typography>
              </Box>
              <LinearProgress
                variant="determinate"
                value={getStrengthProgress(passwordStrength)}
                sx={{
                  height: 6,
                  borderRadius: 1,
                  backgroundColor: 'grey.200',
                  '& .MuiLinearProgress-bar': {
                    backgroundColor: getStrengthColor(passwordStrength),
                  },
                }}
              />
            </Box>
          )}

          {/* Confirm Password */}
          <TextField
            fullWidth
            label="Confirm New Password"
            type={showConfirmPassword ? 'text' : 'password'}
            value={confirmPassword}
            onChange={handleConfirmPasswordChange}
            onBlur={() => handleBlur('confirmPassword')}
            error={touched.confirmPassword && !!confirmPasswordError}
            helperText={touched.confirmPassword ? confirmPasswordError : ''}
            disabled={isLoading || success}
            required
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    edge="end"
                    disabled={isLoading || success}
                  >
                    {showConfirmPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
        </Box>
      </DialogContent>

      <DialogActions sx={{ padding: 2 }}>
        <Button onClick={onClose} disabled={isLoading || success}>
          Cancel
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={!isFormValid || isLoading || success}
        >
          {isLoading ? (
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <CircularProgress size={16} color="inherit" />
              <span>Changing...</span>
            </Box>
          ) : (
            'Change Password'
          )}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

/**
 * Display name for debugging
 */
ChangePasswordDialog.displayName = 'ChangePasswordDialog';

/**
 * Export default for convenience
 */
export default ChangePasswordDialog;
