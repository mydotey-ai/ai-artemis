/**
 * Login Page Component - Modern Glassmorphism Design
 *
 * Features:
 * - Glassmorphism card design
 * - Animated gradient background
 * - Micro-interactions and hover effects
 * - Neon accent lighting
 * - Responsive design with split layout
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
  InputAdornment,
  IconButton,
  Checkbox,
  FormControlLabel,
  Alert,
  CircularProgress,
  useMediaQuery,
  useTheme,
  keyframes,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Visibility,
  VisibilityOff,
  AccountCircle,
  Lock,
  Hub,
  Bolt,
  Timer,
  Memory,
} from '@mui/icons-material';
import { useAuthStore } from '@/store/authStore';
import { saveToken } from '@/utils/token';

// ===== Validation Constants =====

const USERNAME_MIN_LENGTH = 3;
const USERNAME_MAX_LENGTH = 20;
const PASSWORD_MIN_LENGTH = 6;

// ===== Animation Keyframes =====

const float = keyframes`
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
`;

const pulse = keyframes`
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
`;

const gradientShift = keyframes`
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
`;

const slideUp = keyframes`
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
`;

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
  const theme = useTheme();
  const isDesktop = useMediaQuery(theme.breakpoints.up('md'));
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

    if (displayError) {
      setDisplayError(null);
    }

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

    if (displayError) {
      setDisplayError(null);
    }

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

    setTouched({ username: true, password: true });

    const usernameValidationError = validateUsername(username);
    const passwordValidationError = validatePassword(password);

    setUsernameError(usernameValidationError);
    setPasswordError(passwordValidationError);

    if (usernameValidationError || passwordValidationError) {
      return;
    }

    try {
      await login(username, password);

      const token = useAuthStore.getState().token;
      if (token) {
        saveToken(token, rememberMe);
      }
    } catch (error) {
      console.error('Login failed:', error);
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

  const isFormValid = !usernameError && !passwordError && username && password;

  // ===== Styles =====

  /**
   * Root container with animated gradient background
   */
  const rootBoxSx: SxProps<Theme> = {
    width: '100vw',
    height: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    background: 'linear-gradient(-45deg, #0a0a0f, #1a1a2e, #16213e, #0f3460, #1a1a2e, #0a0a0f)',
    backgroundSize: '400% 400%',
    animation: `${gradientShift} 15s ease infinite`,
    overflow: 'hidden',
    position: 'relative',
    '&::before': {
      content: '""',
      position: 'absolute',
      width: '600px',
      height: '600px',
      background: 'radial-gradient(circle, rgba(102, 126, 234, 0.15) 0%, transparent 70%)',
      borderRadius: '50%',
      top: '-100px',
      left: '-100px',
      animation: `${pulse} 8s ease-in-out infinite`,
    },
    '&::after': {
      content: '""',
      position: 'absolute',
      width: '500px',
      height: '500px',
      background: 'radial-gradient(circle, rgba(118, 75, 162, 0.12) 0%, transparent 70%)',
      borderRadius: '50%',
      bottom: '-150px',
      right: '-100px',
      animation: `${pulse} 10s ease-in-out infinite 2s`,
    },
  };

  /**
   * Main container - glassmorphism design
   */
  const mainContainerSx: SxProps<Theme> = {
    width: isDesktop ? '85vw' : '90vw',
    maxWidth: isDesktop ? 1200 : 450,
    height: isDesktop ? '78vh' : undefined,
    minHeight: isDesktop ? 600 : undefined,
    maxHeight: 800,
    display: 'flex',
    flexDirection: isDesktop ? 'row' : 'column',
    borderRadius: '24px',
    overflow: 'hidden',
    backdropFilter: 'blur(20px)',
    background: 'rgba(255, 255, 255, 0.03)',
    border: '1px solid rgba(255, 255, 255, 0.08)',
    boxShadow: '0 25px 80px rgba(0, 0, 0, 0.4)',
    position: 'relative',
    zIndex: 1,
    animation: `${slideUp} 0.6s ease-out`,
  };

  /**
   * Left panel - brand showcase with neon accents
   */
  const leftPanelSx: SxProps<Theme> = {
    flex: '0 0 48%',
    background: 'linear-gradient(145deg, rgba(102, 126, 234, 0.85) 0%, rgba(118, 75, 162, 0.9) 100%)',
    display: isDesktop ? 'flex' : 'none',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 6,
    color: 'white',
    position: 'relative',
    overflow: 'hidden',
    '&::before': {
      content: '""',
      position: 'absolute',
      width: '200%',
      height: '200%',
      background: 'radial-gradient(circle at 30% 30%, rgba(255,255,255,0.1) 0%, transparent 50%), radial-gradient(circle at 70% 70%, rgba(255,255,255,0.08) 0%, transparent 50%)',
      animation: `${float} 6s ease-in-out infinite`,
    },
  };

  /**
   * Right panel - glassmorphism login form
   */
  const rightPanelSx: SxProps<Theme> = {
    flex: 1,
    minWidth: isDesktop ? 480 : undefined,
    bgcolor: 'transparent',
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'center',
    position: 'relative',
  };

  /**
   * Card container (transparent for glass effect)
   */
  const cardSx: SxProps<Theme> = {
    width: '100%',
    height: '100%',
    borderRadius: 0,
    boxShadow: 'none',
    background: 'transparent',
  };

  /**
   * Card content wrapper
   */
  const cardContentSx: SxProps<Theme> = {
    padding: isDesktop ? '64px 72px' : '32px 24px',
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'center',
    '&:last-child': {
      paddingBottom: isDesktop ? '64px' : '32px',
    },
    maxWidth: isDesktop ? 480 : undefined,
    margin: isDesktop ? '0 auto' : undefined,
    width: '100%',
  };

  /**
   * Logo/title container
   */
  const logoBoxSx: SxProps<Theme> = {
    textAlign: 'center',
    marginBottom: 4,
    animation: `${slideUp} 0.6s ease-out 0.1s both`,
  };

  /**
   * Form wrapper
   */
  const formSx: SxProps<Theme> = {
    display: 'flex',
    flexDirection: 'column',
    gap: 2.5,
    animation: `${slideUp} 0.6s ease-out 0.2s both`,
  };

  /**
   * Custom text field styling
   */
  const textFieldSx: SxProps<Theme> = {
    '& .MuiOutlinedInput-root': {
      background: 'rgba(255, 255, 255, 0.03)',
      backdropFilter: 'blur(10px)',
      borderRadius: '12px',
      transition: 'all 0.3s ease',
      '&:hover': {
        background: 'rgba(255, 255, 255, 0.05)',
      },
      '&.Mui-focused': {
        background: 'rgba(255, 255, 255, 0.06)',
      },
    },
    '& .MuiOutlinedInput-notchedOutline': {
      borderColor: 'rgba(255, 255, 255, 0.08)',
      transition: 'all 0.3s ease',
    },
    '&:hover .MuiOutlinedInput-notchedOutline': {
      borderColor: 'rgba(102, 126, 234, 0.3)',
    },
    '& .MuiOutlinedInput-root.Mui-focused .MuiOutlinedInput-notchedOutline': {
      borderColor: 'rgba(102, 126, 234, 0.8)',
      borderWidth: '2px',
    },
    '& .MuiInputLabel-root': {
      color: 'rgba(255, 255, 255, 0.5)',
    },
    '& .MuiInputLabel-root.Mui-focused': {
      color: 'rgba(102, 126, 234, 0.9)',
    },
    '& .MuiInputBase-input': {
      color: 'rgba(255, 255, 255, 0.9)',
    },
  };

  /**
   * Login button with neon glow effect
   */
  const loginButtonSx: SxProps<Theme> = {
    marginTop: 2,
    padding: '16px 32px',
    fontSize: '1rem',
    fontWeight: 600,
    textTransform: 'none',
    borderRadius: '12px',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    boxShadow: '0 8px 24px rgba(102, 126, 234, 0.4)',
    transition: 'all 0.3s ease',
    '&:hover': {
      transform: 'translateY(-2px)',
      boxShadow: '0 12px 32px rgba(102, 126, 234, 0.5)',
    },
    '&:active': {
      transform: 'translateY(0)',
    },
    '&.Mui-disabled': {
      background: 'rgba(255, 255, 255, 0.1)',
      boxShadow: 'none',
    },
  };

  /**
   * Feature item style for left panel
   */
  const featureItemSx: SxProps<Theme> = {
    display: 'flex',
    alignItems: 'flex-start',
    gap: 2,
    marginBottom: 3,
    position: 'relative',
    zIndex: 1,
  };

  return (
    <Box sx={rootBoxSx}>
      <Box sx={mainContainerSx}>
        {/* Left Panel - Brand Showcase (Desktop) */}
        <Box sx={leftPanelSx}>
          <Box sx={{ textAlign: 'center', marginBottom: 6, position: 'relative', zIndex: 1 }}>
            <Hub sx={{ fontSize: 90, marginBottom: 2, opacity: 0.95, animation: `${float} 4s ease-in-out infinite` }} />
            <Typography
              variant="h2"
              component="h1"
              sx={{
                fontWeight: 700,
                marginBottom: 1,
                letterSpacing: '-0.5px',
                textShadow: '0 4px 20px rgba(0, 0, 0, 0.2)',
              }}
            >
              Artemis
            </Typography>
            <Typography
              variant="h6"
              sx={{
                fontWeight: 400,
                opacity: 0.9,
                letterSpacing: '0.5px',
              }}
            >
              Service Registry Center
            </Typography>
          </Box>

          {/* Features */}
          <Box sx={{ width: '100%', maxWidth: 380, position: 'relative', zIndex: 1 }}>
            <Box sx={featureItemSx}>
              <Bolt sx={{ fontSize: 36, opacity: 0.95 }} />
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600, marginBottom: 0.5 }}>
                  Ultra Low Latency
                </Typography>
                <Typography variant="body2" sx={{ opacity: 0.85, lineHeight: 1.5 }}>
                  P99 &lt; 0.5ms, 100x faster than Java version
                </Typography>
              </Box>
            </Box>

            <Box sx={featureItemSx}>
              <Timer sx={{ fontSize: 36, opacity: 0.95 }} />
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600, marginBottom: 0.5 }}>
                  Zero GC Pauses
                </Typography>
                <Typography variant="body2" sx={{ opacity: 0.85, lineHeight: 1.5 }}>
                  Built with Rust, no Stop-The-World interruptions
                </Typography>
              </Box>
            </Box>

            <Box sx={featureItemSx}>
              <Memory sx={{ fontSize: 36, opacity: 0.95 }} />
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600, marginBottom: 0.5 }}>
                  High Scalability
                </Typography>
                <Typography variant="body2" sx={{ opacity: 0.85, lineHeight: 1.5 }}>
                  Supports 100,000+ service instances with ease
                </Typography>
              </Box>
            </Box>
          </Box>
        </Box>

        {/* Right Panel - Login Form */}
        <Box sx={rightPanelSx}>
          <Card sx={cardSx}>
            <CardContent sx={cardContentSx}>
              {/* Logo and Title */}
              <Box sx={logoBoxSx}>
                {!isDesktop && (
                  <Hub sx={{ fontSize: 56, color: '#667eea', marginBottom: 1.5 }} />
                )}
                <Typography
                  variant={isDesktop ? "h4" : "h5"}
                  component="h1"
                  gutterBottom
                  sx={{
                    fontWeight: 700,
                    color: 'rgba(255, 255, 255, 0.95)',
                    letterSpacing: '-0.5px',
                  }}
                >
                  {isDesktop ? "Welcome Back" : "Artemis Console"}
                </Typography>
                <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.5)' }}>
                  {isDesktop ? "Sign in to access the console" : "Service Registry Management"}
                </Typography>
              </Box>

              {/* Error Alert */}
              {displayError && (
                <Alert
                  severity="error"
                  sx={{
                    marginBottom: 2,
                    background: 'rgba(211, 47, 47, 0.1)',
                    border: '1px solid rgba(211, 47, 47, 0.3)',
                    color: 'rgba(255, 255, 255, 0.9)',
                    borderRadius: '12px',
                    '& .MuiAlert-icon': {
                      color: 'rgba(244, 67, 54, 0.9)',
                    },
                  }}
                >
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
                  size="medium"
                  sx={textFieldSx}
                  InputProps={{
                    startAdornment: (
                      <InputAdornment position="start">
                        <AccountCircle sx={{ color: 'rgba(255, 255, 255, 0.5)' }} />
                      </InputAdornment>
                    ),
                  }}
                  FormHelperTextProps={{
                    sx: {
                      color: touched.username && usernameError ? 'rgba(244, 67, 54, 0.8)' : 'rgba(255, 255, 255, 0.4)',
                    },
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
                  size="medium"
                  sx={textFieldSx}
                  InputProps={{
                    startAdornment: (
                      <InputAdornment position="start">
                        <Lock sx={{ color: 'rgba(255, 255, 255, 0.5)' }} />
                      </InputAdornment>
                    ),
                    endAdornment: (
                      <InputAdornment position="end">
                        <IconButton
                          aria-label="toggle password visibility"
                          onClick={handleTogglePasswordVisibility}
                          edge="end"
                          disabled={isAuthLoading}
                          sx={{ color: 'rgba(255, 255, 255, 0.5)' }}
                        >
                          {showPassword ? <VisibilityOff /> : <Visibility />}
                        </IconButton>
                      </InputAdornment>
                    ),
                  }}
                  FormHelperTextProps={{
                    sx: {
                      color: touched.password && passwordError ? 'rgba(244, 67, 54, 0.8)' : 'rgba(255, 255, 255, 0.4)',
                    },
                  }}
                />

                {/* Remember Me Checkbox */}
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={rememberMe}
                      onChange={handleRememberMeChange}
                      disabled={isAuthLoading}
                      sx={{
                        color: 'rgba(255, 255, 255, 0.5)',
                        '&.Mui-checked': {
                          color: '#667eea',
                        },
                      }}
                    />
                  }
                  label={
                    <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.6)' }}>
                      Remember me
                    </Typography>
                  }
                  sx={{ marginTop: -0.5 }}
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
              <Box sx={{ textAlign: 'center', marginTop: 4, animation: `${slideUp} 0.6s ease-out 0.3s both` }}>
                <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.4)' }}>
                  Artemis v1.0.0 — Powered by Rust
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Box>
      </Box>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Login.displayName = 'Login';

export default Login;
