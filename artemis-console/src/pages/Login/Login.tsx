/**
 * Login Page Component
 *
 * Features:
 * - Material-UI Card layout with centered design
 * - Username and password input fields with validation
 * - Remember Me checkbox
 * - Login button with loading state
 * - Error handling and display
 * - Form validation (real-time and on submit)
 * - Gradient background
 * - Responsive design
 * - Integration with authStore for authentication
 */

import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import {
  Box,
  Card,
  CardContent,
  TextField,
  Button,
  Typography,
  Container,
  InputAdornment,
  IconButton,
  Checkbox,
  FormControlLabel,
  Alert,
  CircularProgress,
  type SxProps,
  type Theme,
} from '@mui/material';
import { Visibility, VisibilityOff, AccountCircle, Lock } from '@mui/icons-material';
import { useAuthStore } from '@/store/authStore';
import { saveToken } from '@/utils/token';

// ===== Validation Constants =====

const USERNAME_MIN_LENGTH = 3;
const USERNAME_MAX_LENGTH = 20;
const PASSWORD_MIN_LENGTH = 6;

// ===== Validation Functions =====

/**
 * Validate username
 */
function validateUsername(username: string): string | null {
  if (!username) {
    return 'Username is required';
  }
  if (username.length < USERNAME_MIN_LENGTH) {
    return `Username must be at least ${USERNAME_MIN_LENGTH} characters`;
  }
  if (username.length > USERNAME_MAX_LENGTH) {
    return `Username must not exceed ${USERNAME_MAX_LENGTH} characters`;
  }
  // Only allow alphanumeric and underscore
  if (!/^[a-zA-Z0-9_]+$/.test(username)) {
    return 'Username can only contain letters, numbers, and underscores';
  }
  return null;
}

/**
 * Validate password
 */
function validatePassword(password: string): string | null {
  if (!password) {
    return 'Password is required';
  }
  if (password.length < PASSWORD_MIN_LENGTH) {
    return `Password must be at least ${PASSWORD_MIN_LENGTH} characters`;
  }
  return null;
}

// ===== Main Component =====

/**
 * Login component
 *
 * @returns React component
 */
const Login: React.FC = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Get auth store actions and state
  const login = useAuthStore((state) => state.login);
  const isAuthLoading = useAuthStore((state) => state.isLoading);
  const authError = useAuthStore((state) => state.error);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  // Form state
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberMe, setRememberMe] = useState(false);

  // Validation state
  const [usernameError, setUsernameError] = useState<string | null>(null);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [touched, setTouched] = useState({ username: false, password: false });

  // Error display state
  const [displayError, setDisplayError] = useState<string | null>(null);

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      const redirectPath = searchParams.get('redirect') || '/dashboard';
      navigate(redirectPath, { replace: true });
    }
  }, [isAuthenticated, navigate, searchParams]);

  // Update display error when authError changes
  useEffect(() => {
    if (authError) {
      setDisplayError(authError);
    }
  }, [authError]);

  /**
   * Handle username change with validation
   */
  const handleUsernameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setUsername(value);

    // Clear display error when user types
    if (displayError) {
      setDisplayError(null);
    }

    // Validate if field has been touched
    if (touched.username) {
      setUsernameError(validateUsername(value));
    }
  };

  /**
   * Handle password change with validation
   */
  const handlePasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setPassword(value);

    // Clear display error when user types
    if (displayError) {
      setDisplayError(null);
    }

    // Validate if field has been touched
    if (touched.password) {
      setPasswordError(validatePassword(value));
    }
  };

  /**
   * Handle username blur - mark as touched and validate
   */
  const handleUsernameBlur = () => {
    setTouched((prev) => ({ ...prev, username: true }));
    setUsernameError(validateUsername(username));
  };

  /**
   * Handle password blur - mark as touched and validate
   */
  const handlePasswordBlur = () => {
    setTouched((prev) => ({ ...prev, password: true }));
    setPasswordError(validatePassword(password));
  };

  /**
   * Handle login form submission
   */
  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();

    // Mark all fields as touched
    setTouched({ username: true, password: true });

    // Validate all fields
    const usernameValidationError = validateUsername(username);
    const passwordValidationError = validatePassword(password);

    setUsernameError(usernameValidationError);
    setPasswordError(passwordValidationError);

    // Stop if validation fails
    if (usernameValidationError || passwordValidationError) {
      return;
    }

    try {
      // Call login action
      await login(username, password);

      // Save token with remember me preference
      // Token is already saved in authStore, but we update the storage type
      const token = useAuthStore.getState().token;
      if (token) {
        saveToken(token, rememberMe);
      }

      // Navigation is handled by useEffect when isAuthenticated changes
    } catch (error) {
      // Error is handled by authStore and displayed via displayError
      console.error('Login failed:', error);

      // Clear password field on error
      setPassword('');
      setPasswordError(null);
    }
  };

  /**
   * Toggle password visibility
   */
  const handleTogglePasswordVisibility = () => {
    setShowPassword(!showPassword);
  };

  /**
   * Handle remember me checkbox
   */
  const handleRememberMeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setRememberMe(e.target.checked);
  };

  // Check if form is valid
  const isFormValid = !usernameError && !passwordError && username && password;

  // ===== Styles =====

  /**
   * Root container with gradient background
   */
  const rootBoxSx: SxProps<Theme> = {
    minHeight: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    padding: 2,
  };

  /**
   * Login card container
   */
  const cardSx: SxProps<Theme> = {
    maxWidth: 450,
    width: '100%',
    borderRadius: 2,
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
  };

  /**
   * Card content wrapper
   */
  const cardContentSx: SxProps<Theme> = {
    padding: 4,
    '&:last-child': {
      paddingBottom: 4,
    },
  };

  /**
   * Logo/title container
   */
  const logoBoxSx: SxProps<Theme> = {
    textAlign: 'center',
    marginBottom: 4,
  };

  /**
   * Form wrapper
   */
  const formSx: SxProps<Theme> = {
    display: 'flex',
    flexDirection: 'column',
    gap: 2,
  };

  /**
   * Login button styles
   */
  const loginButtonSx: SxProps<Theme> = {
    marginTop: 2,
    padding: 1.5,
    fontSize: '1rem',
    fontWeight: 600,
    textTransform: 'none',
    borderRadius: 2,
  };

  return (
    <Box sx={rootBoxSx}>
      <Container maxWidth="sm">
        <Card sx={cardSx}>
          <CardContent sx={cardContentSx}>
            {/* Logo and Title */}
            <Box sx={logoBoxSx}>
              <Typography
                variant="h4"
                component="h1"
                gutterBottom
                sx={{ fontWeight: 700, color: 'primary.main' }}
              >
                Artemis Console
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Service Registry Management
              </Typography>
            </Box>

            {/* Error Alert */}
            {displayError && (
              <Alert severity="error" sx={{ marginBottom: 2 }}>
                {displayError}
              </Alert>
            )}

            {/* Login Form */}
            <Box component="form" onSubmit={handleLogin} sx={formSx}>
              {/* Username Field */}
              <TextField
                fullWidth
                label="Username"
                variant="outlined"
                value={username}
                onChange={handleUsernameChange}
                onBlur={handleUsernameBlur}
                error={touched.username && !!usernameError}
                helperText={touched.username ? usernameError : ''}
                disabled={isAuthLoading}
                required
                autoFocus
                autoComplete="username"
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <AccountCircle color="action" />
                    </InputAdornment>
                  ),
                }}
              />

              {/* Password Field */}
              <TextField
                fullWidth
                label="Password"
                type={showPassword ? 'text' : 'password'}
                variant="outlined"
                value={password}
                onChange={handlePasswordChange}
                onBlur={handlePasswordBlur}
                error={touched.password && !!passwordError}
                helperText={touched.password ? passwordError : ''}
                disabled={isAuthLoading}
                required
                autoComplete="current-password"
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <Lock color="action" />
                    </InputAdornment>
                  ),
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton
                        aria-label="toggle password visibility"
                        onClick={handleTogglePasswordVisibility}
                        edge="end"
                        disabled={isAuthLoading}
                      >
                        {showPassword ? <VisibilityOff /> : <Visibility />}
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
              />

              {/* Remember Me Checkbox */}
              <FormControlLabel
                control={
                  <Checkbox
                    checked={rememberMe}
                    onChange={handleRememberMeChange}
                    disabled={isAuthLoading}
                    color="primary"
                  />
                }
                label={
                  <Typography variant="body2" color="text.secondary">
                    Remember me
                  </Typography>
                }
                sx={{ marginTop: -1 }}
              />

              {/* Login Button */}
              <Button
                type="submit"
                fullWidth
                variant="contained"
                size="large"
                disabled={!isFormValid || isAuthLoading}
                sx={loginButtonSx}
              >
                {isAuthLoading ? (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <CircularProgress size={20} color="inherit" />
                    <span>Signing in...</span>
                  </Box>
                ) : (
                  'Sign In'
                )}
              </Button>
            </Box>

            {/* Footer Text */}
            <Box sx={{ textAlign: 'center', marginTop: 3 }}>
              <Typography variant="caption" color="text.secondary">
                Artemis v1.0.0 - Powered by Rust
              </Typography>
            </Box>
          </CardContent>
        </Card>
      </Container>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Login.displayName = 'Login';

export default Login;
