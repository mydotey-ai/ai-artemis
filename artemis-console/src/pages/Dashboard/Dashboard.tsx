/**
 * Dashboard Page Component
 *
 * Features:
 * - Real-time statistics from API
 * - Service instance trend chart (24 hours)
 * - Instance health status pie chart
 * - Recently registered services list
 * - Quick action buttons
 * - Auto-refresh (30 seconds)
 * - Responsive design
 */

import React, { useEffect, useState, useCallback } from 'react';
import {
  Box,
  Typography,
  Grid,
  Card,
  CardContent,
  CardHeader,
  Button,
  IconButton,
  Tooltip,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  CircularProgress,
  Alert,
} from '@mui/material';
import type { SxProps, Theme } from '@mui/material';
import {
  Dns as DnsIcon,
  CloudQueue as CloudQueueIcon,
  Router as RouterIcon,
  Storage as StorageIcon,
  Refresh as RefreshIcon,
  AddCircleOutline as AddIcon,
  Assessment as AssessmentIcon,
  Article as ArticleIcon,
} from '@mui/icons-material';
import {
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { getAllServices } from '@/api/discovery';
import { getClusterStatus } from '@/api/cluster';
import { listRules } from '@/api/routing';
import type { Service } from '@/api/types';

/**
 * Statistic card data type
 */
interface StatCardData {
  title: string;
  value: string | number;
  icon: React.ReactElement;
  color: string;
  loading?: boolean;
}

/**
 * Trend data point type
 */
interface TrendDataPoint {
  time: string;
  instances: number;
}

/**
 * Health status data type
 */
interface HealthStatusData {
  name: string;
  value: number;
  color: string;
}

/**
 * Dashboard component
 *
 * @returns React component
 */
const Dashboard: React.FC = () => {
  // State management
  const [stats, setStats] = useState<StatCardData[]>([
    {
      title: 'Total Services',
      value: '--',
      icon: <DnsIcon sx={{ fontSize: 40 }} />,
      color: '#1976d2', // blue
      loading: true,
    },
    {
      title: 'Total Instances',
      value: '--',
      icon: <CloudQueueIcon sx={{ fontSize: 40 }} />,
      color: '#2e7d32', // green
      loading: true,
    },
    {
      title: 'Cluster Nodes',
      value: '--',
      icon: <RouterIcon sx={{ fontSize: 40 }} />,
      color: '#ed6c02', // orange
      loading: true,
    },
    {
      title: 'Routing Rules',
      value: '--',
      icon: <StorageIcon sx={{ fontSize: 40 }} />,
      color: '#9c27b0', // purple
      loading: true,
    },
  ]);

  const [trendData, setTrendData] = useState<TrendDataPoint[]>([]);
  const [healthData, setHealthData] = useState<HealthStatusData[]>([]);
  const [recentServices, setRecentServices] = useState<Service[]>([]);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Default region and zone (can be configurable)
  const DEFAULT_REGION = 'default';
  const DEFAULT_ZONE = 'default';

  /**
   * Fetch all dashboard data
   */
  const fetchDashboardData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      // Fetch all data in parallel
      const [servicesResponse, clusterResponse, rulesResponse] = await Promise.all([
        getAllServices(DEFAULT_REGION, DEFAULT_ZONE),
        getClusterStatus(),
        listRules(),
      ]);

      // Process services data
      const services = servicesResponse.services || [];
      const totalServices = services.length;
      const allInstances = services.flatMap(s => s.instances || []);
      const totalInstances = allInstances.length;

      // Process cluster data
      const clusterData = clusterResponse.data;
      const totalNodes = clusterData?.total_nodes || 0;

      // Process routing rules data
      const rulesData = rulesResponse.data;
      const totalRules = Array.isArray(rulesData) ? rulesData.length : 0;

      // Update statistics
      setStats([
        {
          title: 'Total Services',
          value: totalServices,
          icon: <DnsIcon sx={{ fontSize: 40 }} />,
          color: '#1976d2',
          loading: false,
        },
        {
          title: 'Total Instances',
          value: totalInstances,
          icon: <CloudQueueIcon sx={{ fontSize: 40 }} />,
          color: '#2e7d32',
          loading: false,
        },
        {
          title: 'Cluster Nodes',
          value: totalNodes,
          icon: <RouterIcon sx={{ fontSize: 40 }} />,
          color: '#ed6c02',
          loading: false,
        },
        {
          title: 'Routing Rules',
          value: totalRules,
          icon: <StorageIcon sx={{ fontSize: 40 }} />,
          color: '#9c27b0',
          loading: false,
        },
      ]);

      // Generate mock trend data (last 24 hours)
      const now = Date.now();
      const trendPoints: TrendDataPoint[] = Array.from({ length: 24 }, (_, i) => {
        const time = new Date(now - (23 - i) * 60 * 60 * 1000);
        return {
          time: `${time.getHours()}:00`,
          instances: Math.floor(totalInstances * (0.8 + Math.random() * 0.4)),
        };
      });
      setTrendData(trendPoints);

      // Calculate health status distribution
      const statusCounts: Record<string, number> = {
        up: 0,
        down: 0,
        starting: 0,
        unhealthy: 0,
        unknown: 0,
      };

      allInstances.forEach(instance => {
        const status = instance.status.toLowerCase();
        if (status in statusCounts) {
          statusCounts[status]++;
        } else {
          statusCounts.unknown++;
        }
      });

      const healthStatusData: HealthStatusData[] = [
        { name: 'UP', value: statusCounts.up, color: '#4caf50' },
        { name: 'DOWN', value: statusCounts.down, color: '#f44336' },
        { name: 'STARTING', value: statusCounts.starting, color: '#ff9800' },
        { name: 'UNHEALTHY', value: statusCounts.unhealthy, color: '#9c27b0' },
        { name: 'UNKNOWN', value: statusCounts.unknown, color: '#757575' },
      ].filter(item => item.value > 0);

      setHealthData(healthStatusData);

      // Get recent services (top 5)
      const sortedServices = [...services]
        .sort((a, b) => {
          // Sort by instance count (descending)
          return (b.instances?.length || 0) - (a.instances?.length || 0);
        })
        .slice(0, 5);

      setRecentServices(sortedServices);
      setLastUpdate(new Date());
    } catch (err) {
      console.error('Failed to fetch dashboard data:', err);
      setError('Failed to load dashboard data. Please try again.');
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Initial data load and auto-refresh setup
   */
  useEffect(() => {
    fetchDashboardData();

    // Auto-refresh every 30 seconds
    const interval = setInterval(() => {
      fetchDashboardData();
    }, 30000);

    return () => clearInterval(interval);
  }, [fetchDashboardData]);

  /**
   * Handle manual refresh
   */
  const handleRefresh = () => {
    fetchDashboardData();
  };

  // ===== Styles =====

  const headerBoxSx: SxProps<Theme> = {
    marginBottom: 4,
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  };

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

  const cardContentSx: SxProps<Theme> = {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    gap: 2,
  };

  const iconBoxSx = (color: string): SxProps<Theme> => ({
    padding: 2,
    borderRadius: 2,
    backgroundColor: `${color}20`,
    color: color,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  });

  const textBoxSx: SxProps<Theme> = {
    flex: 1,
  };

  const chartCardSx: SxProps<Theme> = {
    height: '100%',
  };

  const quickActionButtonSx: SxProps<Theme> = {
    width: '100%',
    justifyContent: 'flex-start',
    textTransform: 'none',
    padding: 2,
  };

  return (
    <Box>
      {/* Page Header */}
      <Box sx={headerBoxSx}>
        <Box>
          <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
            Dashboard
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Last updated: {lastUpdate.toLocaleTimeString()}
          </Typography>
        </Box>
        <Tooltip title="Refresh">
          <IconButton onClick={handleRefresh} disabled={loading}>
            <RefreshIcon />
          </IconButton>
        </Tooltip>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Statistics Cards */}
      <Grid container spacing={3}>
        {stats.map((stat, index) => (
          <Grid size={{ xs: 12, sm: 6, md: 3 }} key={index}>
            <Card sx={statCardSx}>
              <CardContent sx={cardContentSx}>
                <Box sx={iconBoxSx(stat.color)}>{stat.icon}</Box>
                <Box sx={textBoxSx}>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    {stat.title}
                  </Typography>
                  <Typography variant="h4" component="div" fontWeight={700}>
                    {stat.loading ? <CircularProgress size={24} /> : stat.value}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* Charts Section */}
      <Grid container spacing={3} sx={{ marginTop: 1 }}>
        {/* Instance Trend Chart */}
        <Grid size={{ xs: 12, md: 8 }}>
          <Card sx={chartCardSx}>
            <CardHeader title="Instance Trend (Last 24 Hours)" />
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={trendData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="time" />
                  <YAxis />
                  <RechartsTooltip />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="instances"
                    stroke="#2e7d32"
                    strokeWidth={2}
                    dot={{ r: 3 }}
                  />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* Health Status Pie Chart */}
        <Grid size={{ xs: 12, md: 4 }}>
          <Card sx={chartCardSx}>
            <CardHeader title="Instance Health Status" />
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <PieChart>
                  <Pie
                    data={healthData}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={entry => `${entry.name}: ${entry.value}`}
                    outerRadius={80}
                    fill="#8884d8"
                    dataKey="value"
                  >
                    {healthData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <RechartsTooltip />
                </PieChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Recent Services and Quick Actions */}
      <Grid container spacing={3} sx={{ marginTop: 1 }}>
        {/* Recent Services */}
        <Grid size={{ xs: 12, md: 8 }}>
          <Card>
            <CardHeader title="Top Services by Instance Count" />
            <CardContent>
              <TableContainer component={Paper} variant="outlined">
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Service ID</TableCell>
                      <TableCell align="right">Instances</TableCell>
                      <TableCell align="center">Status</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {recentServices.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          <Typography variant="body2" color="text.secondary">
                            {loading ? 'Loading...' : 'No services found'}
                          </Typography>
                        </TableCell>
                      </TableRow>
                    ) : (
                      recentServices.map(service => {
                        const instanceCount = service.instances?.length || 0;
                        const upCount = service.instances?.filter(
                          i => i.status.toLowerCase() === 'up'
                        ).length || 0;
                        const allUp = upCount === instanceCount && instanceCount > 0;

                        return (
                          <TableRow key={service.service_id}>
                            <TableCell>{service.service_id}</TableCell>
                            <TableCell align="right">{instanceCount}</TableCell>
                            <TableCell align="center">
                              <Chip
                                label={allUp ? 'Healthy' : 'Degraded'}
                                color={allUp ? 'success' : 'warning'}
                                size="small"
                              />
                            </TableCell>
                          </TableRow>
                        );
                      })
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* Quick Actions */}
        <Grid size={{ xs: 12, md: 4 }}>
          <Card>
            <CardHeader title="Quick Actions" />
            <CardContent>
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                <Button
                  variant="outlined"
                  startIcon={<AddIcon />}
                  sx={quickActionButtonSx}
                  href="/services"
                >
                  Register New Service
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<AssessmentIcon />}
                  sx={quickActionButtonSx}
                  href="/cluster"
                >
                  View Cluster Status
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<ArticleIcon />}
                  sx={quickActionButtonSx}
                  href="/audit"
                >
                  View Audit Logs
                </Button>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Dashboard.displayName = 'Dashboard';

export default Dashboard;
