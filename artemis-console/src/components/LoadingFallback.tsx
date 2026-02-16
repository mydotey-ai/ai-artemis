/**
 * LoadingFallback component for Suspense lazy loading
 *
 * Provides a full-screen loading indicator while lazy-loaded components are being fetched
 */

import { Box, CircularProgress, Typography } from '@mui/material';

interface LoadingFallbackProps {
  message?: string;
}

/**
 * Full-screen loading fallback component
 *
 * Used as Suspense fallback for lazy-loaded route components
 */
export function LoadingFallback({ message = 'Loading...' }: LoadingFallbackProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '100vh',
        gap: 2,
      }}
    >
      <CircularProgress size={60} />
      <Typography variant="h6" color="text.secondary">
        {message}
      </Typography>
    </Box>
  );
}

/**
 * Minimal loading fallback for component-level suspense
 *
 * Used for smaller components or sections that don't need full-screen loading
 */
export function MinimalLoadingFallback() {
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: 200,
      }}
    >
      <CircularProgress />
    </Box>
  );
}

export default LoadingFallback;
