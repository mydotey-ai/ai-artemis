/**
 * Login Page Component
 *
 * Features:
 * - Material-UI Card layout with centered design
 * - Username and password input fields
 * - Login button (UI only, no API integration yet)
 * - Gradient background
 * - Responsive design
 */

import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
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
  type SxProps,
  type Theme,
} from '@mui/material';
import { Visibility, VisibilityOff, AccountCircle, Lock } from '@mui/icons-material';

/**
 * Login component
 *
 * @returns React component
 */
const Login: React.FC = () => {
  const navigate = useNavigate();

  // Form state
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  /**
   * Handle login button click
   * TODO: Integrate with authentication API
   */
  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();

    // Temporary: just navigate to dashboard
    // TODO: Replace with actual authentication logic
    console.log('Login attempt:', { username, password });
    navigate('/dashboard');
  };

  /**
   * Toggle password visibility
   */
  const handleTogglePasswordVisibility = () => {
    setShowPassword(!showPassword);
  };

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

            {/* Login Form */}
            <Box component="form" onSubmit={handleLogin} sx={formSx}>
              {/* Username Field */}
              <TextField
                fullWidth
                label="Username"
                variant="outlined"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
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
                onChange={(e) => setPassword(e.target.value)}
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
                      >
                        {showPassword ? <VisibilityOff /> : <Visibility />}
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
              />

              {/* Login Button */}
              <Button
                type="submit"
                fullWidth
                variant="contained"
                size="large"
                sx={loginButtonSx}
              >
                Sign In
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
