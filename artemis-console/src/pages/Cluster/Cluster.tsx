/**
 * Cluster Page Component
 *
 * Features:
 * - Cluster topology visualization (SVG-based)
 * - Cluster statistics cards
 * - Cluster node list with status and details
 * - Node details dialog
 * - Real-time auto-refresh (every 5 seconds)
 * - Responsive design
 */

import React, { useEffect, useState, useCallback, useMemo } from 'react';
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
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Snackbar,
  Divider,
  List,
  ListItem,
  ListItemText,
} from '@mui/material';
import type { SxProps, Theme } from '@mui/material';
import {
  Refresh as RefreshIcon,
  Info as InfoIcon,
  Router as RouterIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
  Cloud as CloudIcon,
  Storage as StorageIcon,
} from '@mui/icons-material';
import { getClusterStatus, getClusterNodeStatus } from '@/api/cluster';
import type { ClusterNodeStatus } from '@/api/cluster';

/**
 * Cluster statistics data type
 */
interface ClusterStats {
  totalNodes: number;
  healthyNodes: number;
  totalInstances: number;
  totalServices: number;
}

/**
 * Cluster component
 */
const Cluster: React.FC = () => {
  // State management
  const [nodes, setNodes] = useState<ClusterNodeStatus[]>([]);
  const [stats, setStats] = useState<ClusterStats>({
    totalNodes: 0,
    healthyNodes: 0,
    totalInstances: 0,
    totalServices: 0,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [selectedNode, setSelectedNode] = useState<ClusterNodeStatus | null>(null);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [snackbar, setSnackbar] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info';
  }>({
    open: false,
    message: '',
    severity: 'info',
  });

  /**
   * Fetch cluster data
   */
  const fetchClusterData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      // Fetch cluster node status and overall status
      const [nodeStatusResponse, clusterStatusResponse] = await Promise.all([
        getClusterNodeStatus(),
        getClusterStatus(),
      ]);

      // Process node status data
      const nodeStatusData = nodeStatusResponse.data || [];
      setNodes(nodeStatusData);

      // Process cluster statistics
      const clusterData = clusterStatusResponse.data;
      const totalNodes = nodeStatusData.length;
      const healthyNodes = nodeStatusData.filter(
        (node) => node.status === 'ACTIVE'
      ).length;

      setStats({
        totalNodes,
        healthyNodes,
        totalInstances: clusterData?.total_instances || 0,
        totalServices: clusterData?.total_services || 0,
      });

      setLastUpdate(new Date());
    } catch (err) {
      console.error('Failed to fetch cluster data:', err);
      setError('Failed to load cluster data. Please try again.');
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Initial data load and auto-refresh setup
   */
  useEffect(() => {
    fetchClusterData();

    // Auto-refresh every 5 seconds
    const interval = setInterval(() => {
      fetchClusterData();
    }, 5000);

    return () => clearInterval(interval);
  }, [fetchClusterData]);

  /**
   * Handle manual refresh
   */
  const handleRefresh = () => {
    fetchClusterData();
  };

  /**
   * Handle node details view
   */
  const handleViewDetails = (node: ClusterNodeStatus) => {
    setSelectedNode(node);
    setDetailsOpen(true);
  };

  /**
   * Handle close details dialog
   */
  const handleCloseDetails = () => {
    setDetailsOpen(false);
    setSelectedNode(null);
  };

  /**
   * Handle close snackbar
   */
  const handleCloseSnackbar = () => {
    setSnackbar({ ...snackbar, open: false });
  };

  /**
   * Get status color
   */
  const getStatusColor = (status: string): 'success' | 'error' | 'warning' | 'default' => {
    switch (status.toUpperCase()) {
      case 'ACTIVE':
        return 'success';
      case 'INACTIVE':
        return 'error';
      case 'SUSPECTED':
        return 'warning';
      default:
        return 'default';
    }
  };

  /**
   * Get status icon
   */
  const getStatusIcon = (status: string) => {
    switch (status.toUpperCase()) {
      case 'ACTIVE':
        return <CheckCircleIcon fontSize="small" />;
      case 'INACTIVE':
        return <ErrorIcon fontSize="small" />;
      case 'SUSPECTED':
        return <WarningIcon fontSize="small" />;
      default:
        return <InfoIcon fontSize="small" />;
    }
  };

  /**
   * Format relative time
   */
  const formatRelativeTime = (timestamp: string): string => {
    const now = Date.now();
    const time = new Date(timestamp).getTime();
    const diff = now - time;

    if (diff < 60000) {
      return 'Just now';
    } else if (diff < 3600000) {
      const minutes = Math.floor(diff / 60000);
      return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    } else if (diff < 86400000) {
      const hours = Math.floor(diff / 3600000);
      return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    } else {
      const days = Math.floor(diff / 86400000);
      return `${days} day${days > 1 ? 's' : ''} ago`;
    }
  };

  /**
   * Cluster topology visualization
   */
  const ClusterTopology: React.FC = useMemo(
    () =>
      function ClusterTopologyComponent() {
        const width = 800;
        const height = 400;
        const centerX = width / 2;
        const centerY = height / 2;
        const radius = 150;

        // Calculate node positions in a circle
        const nodePositions = nodes.map((_, index) => {
          const angle = (index / nodes.length) * 2 * Math.PI - Math.PI / 2;
          return {
            x: centerX + radius * Math.cos(angle),
            y: centerY + radius * Math.sin(angle),
          };
        });

        return (
          <Card sx={{ marginBottom: 3 }}>
            <CardHeader
              title="Cluster Topology"
              subheader="Visual representation of cluster nodes and their connections"
            />
            <CardContent>
              <Box
                sx={{
                  display: 'flex',
                  justifyContent: 'center',
                  alignItems: 'center',
                  width: '100%',
                  overflow: 'auto',
                }}
              >
                <svg
                  width="100%"
                  height={height}
                  viewBox={`0 0 ${width} ${height}`}
                  style={{ maxWidth: width }}
                >
                  {/* Draw connections between all nodes */}
                  {nodePositions.map((pos1, i) =>
                    nodePositions.slice(i + 1).map((pos2, j) => (
                      <line
                        key={`line-${i}-${j}`}
                        x1={pos1.x}
                        y1={pos1.y}
                        x2={pos2.x}
                        y2={pos2.y}
                        stroke="#e0e0e0"
                        strokeWidth="2"
                        strokeDasharray="5,5"
                      />
                    ))
                  )}

                  {/* Draw nodes */}
                  {nodes.map((node, index) => {
                    const pos = nodePositions[index];
                    const statusColor =
                      node.status === 'ACTIVE'
                        ? '#4caf50'
                        : node.status === 'SUSPECTED'
                          ? '#ff9800'
                          : '#f44336';

                    return (
                      <g key={node.node_id}>
                        {/* Node circle */}
                        <circle
                          cx={pos.x}
                          cy={pos.y}
                          r="40"
                          fill={statusColor}
                          stroke="#fff"
                          strokeWidth="3"
                          style={{ cursor: 'pointer' }}
                          onClick={() => handleViewDetails(node)}
                        />

                        {/* Node ID text */}
                        <text
                          x={pos.x}
                          y={pos.y}
                          textAnchor="middle"
                          dominantBaseline="middle"
                          fill="#fff"
                          fontSize="12"
                          fontWeight="bold"
                          style={{ cursor: 'pointer', pointerEvents: 'none' }}
                        >
                          {node.node_id.substring(0, 8)}
                        </text>

                        {/* Node label */}
                        <text
                          x={pos.x}
                          y={pos.y + 60}
                          textAnchor="middle"
                          fill="#666"
                          fontSize="10"
                        >
                          {node.host}:{node.port}
                        </text>
                      </g>
                    );
                  })}

                  {/* No nodes message */}
                  {nodes.length === 0 && (
                    <text
                      x={centerX}
                      y={centerY}
                      textAnchor="middle"
                      fill="#999"
                      fontSize="14"
                    >
                      No cluster nodes available
                    </text>
                  )}
                </svg>
              </Box>

              {/* Legend */}
              <Box sx={{ marginTop: 2, display: 'flex', justifyContent: 'center', gap: 3 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Box
                    sx={{
                      width: 16,
                      height: 16,
                      borderRadius: '50%',
                      backgroundColor: '#4caf50',
                    }}
                  />
                  <Typography variant="body2">Active</Typography>
                </Box>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Box
                    sx={{
                      width: 16,
                      height: 16,
                      borderRadius: '50%',
                      backgroundColor: '#ff9800',
                    }}
                  />
                  <Typography variant="body2">Suspected</Typography>
                </Box>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Box
                    sx={{
                      width: 16,
                      height: 16,
                      borderRadius: '50%',
                      backgroundColor: '#f44336',
                    }}
                  />
                  <Typography variant="body2">Inactive</Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        );
      },
    [nodes]
  );

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

  return (
    <Box>
      {/* Page Header */}
      <Box sx={headerBoxSx}>
        <Box>
          <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
            Cluster
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Last updated: {lastUpdate.toLocaleTimeString()}
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Refresh">
            <IconButton onClick={handleRefresh} disabled={loading}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Statistics Cards */}
      <Grid container spacing={3} sx={{ marginBottom: 3 }}>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card sx={statCardSx}>
            <CardContent sx={cardContentSx}>
              <Box sx={iconBoxSx('#1976d2')}>
                <RouterIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box sx={textBoxSx}>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Total Nodes
                </Typography>
                <Typography variant="h4" component="div" fontWeight={700}>
                  {loading ? <CircularProgress size={24} /> : stats.totalNodes}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card sx={statCardSx}>
            <CardContent sx={cardContentSx}>
              <Box sx={iconBoxSx('#2e7d32')}>
                <CheckCircleIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box sx={textBoxSx}>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Healthy Nodes
                </Typography>
                <Typography variant="h4" component="div" fontWeight={700}>
                  {loading ? <CircularProgress size={24} /> : stats.healthyNodes}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card sx={statCardSx}>
            <CardContent sx={cardContentSx}>
              <Box sx={iconBoxSx('#ed6c02')}>
                <CloudIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box sx={textBoxSx}>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Total Instances
                </Typography>
                <Typography variant="h4" component="div" fontWeight={700}>
                  {loading ? <CircularProgress size={24} /> : stats.totalInstances}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card sx={statCardSx}>
            <CardContent sx={cardContentSx}>
              <Box sx={iconBoxSx('#9c27b0')}>
                <StorageIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box sx={textBoxSx}>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Total Services
                </Typography>
                <Typography variant="h4" component="div" fontWeight={700}>
                  {loading ? <CircularProgress size={24} /> : stats.totalServices}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Cluster Topology */}
      <ClusterTopology />

      {/* Cluster Node List */}
      <Card>
        <CardHeader title="Cluster Nodes" subheader="List of all nodes in the cluster" />
        <CardContent>
          <TableContainer component={Paper} variant="outlined">
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Node ID</TableCell>
                  <TableCell>Host</TableCell>
                  <TableCell>Port</TableCell>
                  <TableCell align="center">Status</TableCell>
                  <TableCell>Region / Zone</TableCell>
                  <TableCell>Last Heartbeat</TableCell>
                  <TableCell align="center">Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {loading && nodes.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} align="center">
                      <CircularProgress size={24} />
                    </TableCell>
                  </TableRow>
                ) : nodes.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} align="center">
                      <Typography variant="body2" color="text.secondary">
                        No cluster nodes found
                      </Typography>
                    </TableCell>
                  </TableRow>
                ) : (
                  nodes.map((node) => (
                    <TableRow key={node.node_id} hover>
                      <TableCell>
                        <Typography variant="body2" fontFamily="monospace">
                          {node.node_id}
                        </Typography>
                      </TableCell>
                      <TableCell>{node.host}</TableCell>
                      <TableCell>{node.port}</TableCell>
                      <TableCell align="center">
                        <Chip
                          icon={getStatusIcon(node.status)}
                          label={node.status}
                          color={getStatusColor(node.status)}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>
                        {node.region_id} / {node.zone_id}
                      </TableCell>
                      <TableCell>
                        <Tooltip title={new Date(node.last_heartbeat).toLocaleString()}>
                          <Typography variant="body2">
                            {formatRelativeTime(node.last_heartbeat)}
                          </Typography>
                        </Tooltip>
                      </TableCell>
                      <TableCell align="center">
                        <Tooltip title="View Details">
                          <IconButton size="small" onClick={() => handleViewDetails(node)}>
                            <InfoIcon />
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>

      {/* Node Details Dialog */}
      <Dialog
        open={detailsOpen}
        onClose={handleCloseDetails}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Node Details
          {selectedNode && (
            <Typography variant="body2" color="text.secondary">
              {selectedNode.node_id}
            </Typography>
          )}
        </DialogTitle>
        <DialogContent dividers>
          {selectedNode && (
            <Box>
              <Typography variant="h6" gutterBottom>
                Basic Information
              </Typography>
              <List dense>
                <ListItem>
                  <ListItemText
                    primary="Node ID"
                    secondary={
                      <Typography variant="body2" fontFamily="monospace">
                        {selectedNode.node_id}
                      </Typography>
                    }
                  />
                </ListItem>
                <ListItem>
                  <ListItemText primary="Host" secondary={selectedNode.host} />
                </ListItem>
                <ListItem>
                  <ListItemText primary="Port" secondary={selectedNode.port} />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="Status"
                    secondary={
                      <Chip
                        icon={getStatusIcon(selectedNode.status)}
                        label={selectedNode.status}
                        color={getStatusColor(selectedNode.status)}
                        size="small"
                      />
                    }
                  />
                </ListItem>
                <ListItem>
                  <ListItemText primary="Region ID" secondary={selectedNode.region_id} />
                </ListItem>
                <ListItem>
                  <ListItemText primary="Zone ID" secondary={selectedNode.zone_id} />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="Last Heartbeat"
                    secondary={
                      <>
                        {new Date(selectedNode.last_heartbeat).toLocaleString()}
                        <br />
                        <Typography variant="caption" color="text.secondary">
                          ({formatRelativeTime(selectedNode.last_heartbeat)})
                        </Typography>
                      </>
                    }
                  />
                </ListItem>
              </List>

              <Divider sx={{ my: 2 }} />

              <Typography variant="h6" gutterBottom>
                Node URL
              </Typography>
              <Box
                sx={{
                  padding: 2,
                  backgroundColor: 'background.default',
                  borderRadius: 1,
                  fontFamily: 'monospace',
                  fontSize: '0.875rem',
                  wordBreak: 'break-all',
                }}
              >
                http://{selectedNode.host}:{selectedNode.port}
              </Box>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCloseDetails}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Snackbar for notifications */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={handleCloseSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={handleCloseSnackbar}
          severity={snackbar.severity}
          sx={{ width: '100%' }}
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Cluster.displayName = 'Cluster';

export default Cluster;
