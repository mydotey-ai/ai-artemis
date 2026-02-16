/**
 * Dashboard Page Component
 *
 * Features:
 * - Welcome message
 * - Statistics cards (placeholder)
 * - Grid layout for organized display
 * - Responsive design
 */

import React from 'react';
import {
  Box,
  Typography,
  Grid2 as Grid,
  Card,
  CardContent,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Dns as DnsIcon,
  CloudQueue as CloudQueueIcon,
  Router as RouterIcon,
  Storage as StorageIcon,
} from '@mui/icons-material';

/**
 * Statistic card data type
 */
interface StatCardData {
  title: string;
  value: string | number;
  icon: React.ReactElement;
  color: string;
}

/**
 * Dashboard component
 *
 * @returns React component
 */
const Dashboard: React.FC = () => {
  // Mock statistics data
  const stats: StatCardData[] = [
    {
      title: 'Total Services',
      value: '--',
      icon: <DnsIcon sx={{ fontSize: 40 }} />,
      color: '#1976d2', // blue
    },
    {
      title: 'Total Instances',
      value: '--',
      icon: <CloudQueueIcon sx={{ fontSize: 40 }} />,
      color: '#2e7d32', // green
    },
    {
      title: 'Cluster Nodes',
      value: '--',
      icon: <RouterIcon sx={{ fontSize: 40 }} />,
      color: '#ed6c02', // orange
    },
    {
      title: 'Routing Rules',
      value: '--',
      icon: <StorageIcon sx={{ fontSize: 40 }} />,
      color: '#9c27b0', // purple
    },
  ];

  // ===== Styles =====

  /**
   * Page header styles
   */
  const headerBoxSx: SxProps<Theme> = {
    marginBottom: 4,
  };

  /**
   * Stat card styles
   */
  const statCardSx: SxProps<Theme> = {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    transition: 'transform 0.2s, box-shadow 0.2s',
    '&:hover': {
      transform: 'translateY(-4px)',
      boxShadow: 4,
    },
  };

  /**
   * Card content wrapper
   */
  const cardContentSx: SxProps<Theme> = {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    gap: 2,
  };

  /**
   * Icon container styles
   */
  const iconBoxSx = (color: string): SxProps<Theme> => ({
    padding: 2,
    borderRadius: 2,
    backgroundColor: `${color}20`, // 20% opacity
    color: color,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  });

  /**
   * Text content wrapper
   */
  const textBoxSx: SxProps<Theme> = {
    flex: 1,
  };

  return (
    <Box>
      {/* Page Header */}
      <Box sx={headerBoxSx}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Dashboard
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Welcome to Artemis Service Registry Console
        </Typography>
      </Box>

      {/* Statistics Cards */}
      <Grid container spacing={3}>
        {stats.map((stat, index) => (
          <Grid size={{ xs: 12, sm: 6, md: 3 }} key={index}>
            <Card sx={statCardSx}>
              <CardContent sx={cardContentSx}>
                {/* Icon */}
                <Box sx={iconBoxSx(stat.color)}>
                  {stat.icon}
                </Box>

                {/* Text Content */}
                <Box sx={textBoxSx}>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    {stat.title}
                  </Typography>
                  <Typography variant="h4" component="div" fontWeight={700}>
                    {stat.value}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* Coming Soon Section */}
      <Box sx={{ marginTop: 4 }}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Quick Actions
            </Typography>
            <Typography variant="body2" color="text.secondary">
              More features coming soon...
            </Typography>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Dashboard.displayName = 'Dashboard';

export default Dashboard;
