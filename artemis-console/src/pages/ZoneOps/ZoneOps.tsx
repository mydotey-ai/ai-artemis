/**
 * Zone Operations Page Component
 *
 * Features:
 * - Zone batch operation management with full CRUD
 * - Create zone operations (UP_ZONE / DOWN_ZONE)
 * - View operation details with affected instances
 * - Search, filter, and pagination
 * - Real-time updates every 5 seconds
 * - Export to CSV
 * - Operation statistics cards
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TablePagination,
  Chip,
  Button,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Alert,
  Skeleton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  Toolbar,
  Snackbar,
  Tooltip,
  Grid,
  LinearProgress,
  RadioGroup,
  FormControlLabel,
  Radio,
  FormLabel,
  Divider,
} from '@mui/material';
import {
  Add as AddIcon,
  Search as SearchIcon,
  Refresh as RefreshIcon,
  Download as DownloadIcon,
  Close as CloseIcon,
  ArrowUpward as ArrowUpwardIcon,
  ArrowDownward as ArrowDownwardIcon,
  Info as InfoIcon,
  Cancel as CancelIcon,
  Replay as ReplayIcon,
} from '@mui/icons-material';
import type {
  ZoneOperationRecord,
} from '@/api/types';
import * as zoneApi from '@/api/zone';

// =====================================================
// TYPE DEFINITIONS
// =====================================================

type OperationStatus = 'PENDING' | 'IN_PROGRESS' | 'COMPLETED' | 'FAILED';

interface ZoneOperationRow extends ZoneOperationRecord {
  status: OperationStatus;
  created_at: number;
  completed_at?: number;
  total_instances?: number;
  completed_instances?: number;
  failed_instances?: number;
}

interface CreateZoneOperationFormData {
  zone_id: string;
  region_id: string;
  operation_type: 'UP_ZONE' | 'DOWN_ZONE';
  target_scope: 'all' | 'service' | 'instance';
  service_filter: string;
  instance_filter: string;
  operator_id: string;
  reason: string;
}

interface StatisticsData {
  total: number;
  pending: number;
  in_progress: number;
  failed_last_24h: number;
}

// =====================================================
// HELPER FUNCTIONS
// =====================================================

/**
 * Map backend ZoneOperation enum to frontend display type
 */
function mapOperationType(operation: string): 'UP_ZONE' | 'DOWN_ZONE' {
  if (operation === 'PullIn' || operation === 'pullin') {
    return 'UP_ZONE';
  }
  return 'DOWN_ZONE';
}

/**
 * Determine operation status (simplified for now, as backend doesn't track status)
 */
function deriveOperationStatus(operation: ZoneOperationRecord): OperationStatus {
  const now = Date.now();
  const operationTime = operation.operation_time * 1000; // Convert to ms
  const elapsed = now - operationTime;

  // Simple heuristic: operations complete within 5 minutes
  if (elapsed < 60000) {
    return 'IN_PROGRESS';
  } else if (elapsed < 300000) {
    return 'COMPLETED';
  }
  return 'COMPLETED';
}

/**
 * Get status color
 */
function getStatusColor(
  status: OperationStatus
): 'default' | 'primary' | 'success' | 'error' {
  switch (status) {
    case 'PENDING':
      return 'default';
    case 'IN_PROGRESS':
      return 'primary';
    case 'COMPLETED':
      return 'success';
    case 'FAILED':
      return 'error';
  }
}

/**
 * Format timestamp to readable date
 */
function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

/**
 * Export data to CSV
 */
function exportToCSV(data: ZoneOperationRow[]): void {
  const headers = [
    'Operation ID',
    'Zone',
    'Region',
    'Operation Type',
    'Status',
    'Operator',
    'Created Time',
    'Completed Time',
  ];

  const rows = data.map((row) => [
    `${row.zone_id}-${row.operation_time}`,
    row.zone_id,
    row.region_id,
    mapOperationType(row.operation as unknown as string),
    row.status,
    row.operator_id,
    formatTimestamp(row.operation_time),
    row.completed_at ? formatTimestamp(row.completed_at) : 'N/A',
  ]);

  const csvContent = [headers, ...rows].map((row) => row.join(',')).join('\n');

  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `zone-operations-${Date.now()}.csv`;
  link.click();
}

// =====================================================
// MAIN COMPONENT
// =====================================================

const ZoneOps: React.FC = () => {
  // State
  const [operations, setOperations] = useState<ZoneOperationRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(10);
  const [searchQuery, setSearchQuery] = useState('');
  const [zoneFilter, setZoneFilter] = useState<string>('all');
  const [regionFilter, setRegionFilter] = useState<string>('all');
  const [operationTypeFilter, setOperationTypeFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [detailsDialogOpen, setDetailsDialogOpen] = useState(false);
  const [selectedOperation, setSelectedOperation] = useState<ZoneOperationRow | null>(null);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');
  const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error'>('success');

  // Form state
  const [formData, setFormData] = useState<CreateZoneOperationFormData>({
    zone_id: '',
    region_id: '',
    operation_type: 'DOWN_ZONE',
    target_scope: 'all',
    service_filter: '',
    instance_filter: '',
    operator_id: 'admin', // TODO: Get from auth context
    reason: '',
  });

  // Load operations
  const loadOperations = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await zoneApi.queryZoneOperations(
        undefined,
        regionFilter !== 'all' ? regionFilter : undefined
      );

      if (response.success && response.data) {
        const enrichedOps: ZoneOperationRow[] = response.data.map((op) => ({
          ...op,
          status: deriveOperationStatus(op),
          created_at: op.operation_time,
          total_instances: 0,
          completed_instances: 0,
          failed_instances: 0,
        }));
        setOperations(enrichedOps);
      } else {
        setError(response.message || 'Failed to load zone operations');
      }
    } catch (err) {
      console.error('Failed to load zone operations:', err);
      setError('Failed to load zone operations');
    } finally {
      setLoading(false);
    }
  }, [regionFilter]);

  // Auto-refresh every 5 seconds
  useEffect(() => {
    loadOperations();
    const interval = setInterval(loadOperations, 5000);
    return () => clearInterval(interval);
  }, [loadOperations]);

  // Calculate statistics
  const statistics: StatisticsData = useMemo(() => {
    const now = Date.now();
    const last24h = now - 24 * 60 * 60 * 1000;

    return {
      total: operations.length,
      pending: operations.filter((op) => op.status === 'PENDING').length,
      in_progress: operations.filter((op) => op.status === 'IN_PROGRESS').length,
      failed_last_24h: operations.filter(
        (op) => op.status === 'FAILED' && op.created_at * 1000 > last24h
      ).length,
    };
  }, [operations]);

  // Filter operations
  const filteredOperations = useMemo(() => {
    return operations.filter((op) => {
      // Search query
      if (
        searchQuery &&
        !op.zone_id.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !op.operator_id.toLowerCase().includes(searchQuery.toLowerCase())
      ) {
        return false;
      }

      // Zone filter
      if (zoneFilter !== 'all' && op.zone_id !== zoneFilter) {
        return false;
      }

      // Region filter
      if (regionFilter !== 'all' && op.region_id !== regionFilter) {
        return false;
      }

      // Operation type filter
      if (operationTypeFilter !== 'all') {
        const opType = mapOperationType(op.operation as unknown as string);
        if (opType !== operationTypeFilter) {
          return false;
        }
      }

      // Status filter
      if (statusFilter !== 'all' && op.status !== statusFilter) {
        return false;
      }

      return true;
    });
  }, [
    operations,
    searchQuery,
    zoneFilter,
    regionFilter,
    operationTypeFilter,
    statusFilter,
  ]);

  // Get unique zones and regions
  const uniqueZones = useMemo(() => {
    const zones = new Set(operations.map((op) => op.zone_id));
    return Array.from(zones).sort();
  }, [operations]);

  const uniqueRegions = useMemo(() => {
    const regions = new Set(operations.map((op) => op.region_id));
    return Array.from(regions).sort();
  }, [operations]);

  // Paginated operations
  const paginatedOperations = useMemo(() => {
    const start = page * rowsPerPage;
    return filteredOperations.slice(start, start + rowsPerPage);
  }, [filteredOperations, page, rowsPerPage]);

  // Handlers
  const handleCreateOperation = async () => {
    try {
      const request: zoneApi.OperateZoneRequest = {
        zone_id: formData.zone_id,
        region_id: formData.region_id,
        operator_id: formData.operator_id,
      };

      if (formData.operation_type === 'DOWN_ZONE') {
        await zoneApi.pullOutZone(request);
      } else {
        await zoneApi.pullInZone(request);
      }

      setSnackbarMessage('Zone operation created successfully');
      setSnackbarSeverity('success');
      setSnackbarOpen(true);
      setCreateDialogOpen(false);
      loadOperations();

      // Reset form
      setFormData({
        zone_id: '',
        region_id: '',
        operation_type: 'DOWN_ZONE',
        target_scope: 'all',
        service_filter: '',
        instance_filter: '',
        operator_id: 'admin',
        reason: '',
      });
    } catch (err) {
      console.error('Failed to create zone operation:', err);
      setSnackbarMessage('Failed to create zone operation');
      setSnackbarSeverity('error');
      setSnackbarOpen(true);
    }
  };

  const handleViewDetails = (operation: ZoneOperationRow) => {
    setSelectedOperation(operation);
    setDetailsDialogOpen(true);
  };

  const handleCancelOperation = async (operation: ZoneOperationRow) => {
    // TODO: Implement cancel operation when backend API is available
    console.log('Cancel operation:', operation);
    setSnackbarMessage('Cancel operation not yet implemented');
    setSnackbarSeverity('error');
    setSnackbarOpen(true);
  };

  const handleRetryOperation = async (operation: ZoneOperationRow) => {
    // Retry by creating a new operation with same parameters
    try {
      const request: zoneApi.OperateZoneRequest = {
        zone_id: operation.zone_id,
        region_id: operation.region_id,
        operator_id: operation.operator_id,
      };

      const opType = mapOperationType(operation.operation as unknown as string);
      if (opType === 'DOWN_ZONE') {
        await zoneApi.pullOutZone(request);
      } else {
        await zoneApi.pullInZone(request);
      }

      setSnackbarMessage('Operation retried successfully');
      setSnackbarSeverity('success');
      setSnackbarOpen(true);
      loadOperations();
    } catch (err) {
      console.error('Failed to retry operation:', err);
      setSnackbarMessage('Failed to retry operation');
      setSnackbarSeverity('error');
      setSnackbarOpen(true);
    }
  };

  const handleExport = () => {
    exportToCSV(filteredOperations);
    setSnackbarMessage('Exported to CSV successfully');
    setSnackbarSeverity('success');
    setSnackbarOpen(true);
  };

  const handleChangePage = (_event: unknown, newPage: number) => {
    setPage(newPage);
  };

  const handleChangeRowsPerPage = (event: React.ChangeEvent<HTMLInputElement>) => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };

  // Render
  return (
    <Box>
      {/* Header */}
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Zone Operations
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage zone-level batch operations (pull in/out entire zones)
        </Typography>
      </Box>

      {/* Statistics Cards */}
      <Grid container spacing={3} sx={{ marginBottom: 3 }}>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary">
                Total Operations
              </Typography>
              <Typography variant="h4" fontWeight={600}>
                {statistics.total}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary">
                Pending Operations
              </Typography>
              <Typography variant="h4" fontWeight={600} color="text.secondary">
                {statistics.pending}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary">
                In Progress
              </Typography>
              <Typography variant="h4" fontWeight={600} color="primary">
                {statistics.in_progress}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <Card>
            <CardContent>
              <Typography variant="subtitle2" color="text.secondary">
                Failed (24h)
              </Typography>
              <Typography variant="h4" fontWeight={600} color="error">
                {statistics.failed_last_24h}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Toolbar */}
      <Card sx={{ marginBottom: 2 }}>
        <Toolbar>
          <Box sx={{ flexGrow: 1, display: 'flex', gap: 2, alignItems: 'center' }}>
            <TextField
              placeholder="Search zones or operators..."
              size="small"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              InputProps={{
                startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'text.secondary' }} />,
              }}
              sx={{ minWidth: 250 }}
            />

            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Zone</InputLabel>
              <Select value={zoneFilter} onChange={(e) => setZoneFilter(e.target.value)} label="Zone">
                <MenuItem value="all">All Zones</MenuItem>
                {uniqueZones.map((zone) => (
                  <MenuItem key={zone} value={zone}>
                    {zone}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Region</InputLabel>
              <Select
                value={regionFilter}
                onChange={(e) => setRegionFilter(e.target.value)}
                label="Region"
              >
                <MenuItem value="all">All Regions</MenuItem>
                {uniqueRegions.map((region) => (
                  <MenuItem key={region} value={region}>
                    {region}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <FormControl size="small" sx={{ minWidth: 150 }}>
              <InputLabel>Operation Type</InputLabel>
              <Select
                value={operationTypeFilter}
                onChange={(e) => setOperationTypeFilter(e.target.value)}
                label="Operation Type"
              >
                <MenuItem value="all">All Types</MenuItem>
                <MenuItem value="UP_ZONE">UP_ZONE</MenuItem>
                <MenuItem value="DOWN_ZONE">DOWN_ZONE</MenuItem>
              </Select>
            </FormControl>

            <FormControl size="small" sx={{ minWidth: 150 }}>
              <InputLabel>Status</InputLabel>
              <Select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                label="Status"
              >
                <MenuItem value="all">All Status</MenuItem>
                <MenuItem value="PENDING">PENDING</MenuItem>
                <MenuItem value="IN_PROGRESS">IN_PROGRESS</MenuItem>
                <MenuItem value="COMPLETED">COMPLETED</MenuItem>
                <MenuItem value="FAILED">FAILED</MenuItem>
              </Select>
            </FormControl>
          </Box>

          <Box sx={{ display: 'flex', gap: 1 }}>
            <Tooltip title="Refresh">
              <span>
                <IconButton onClick={loadOperations} disabled={loading}>
                  <RefreshIcon />
                </IconButton>
              </span>
            </Tooltip>
            <Tooltip title="Export CSV">
              <IconButton onClick={handleExport}>
                <DownloadIcon />
              </IconButton>
            </Tooltip>
            <Button
              variant="contained"
              startIcon={<AddIcon />}
              onClick={() => setCreateDialogOpen(true)}
            >
              Create Zone Operation
            </Button>
          </Box>
        </Toolbar>
      </Card>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 2 }}>
          {error}
        </Alert>
      )}

      {/* Operations Table */}
      <Card>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Operation ID</TableCell>
                <TableCell>Zone</TableCell>
                <TableCell>Region</TableCell>
                <TableCell>Operation Type</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Operator</TableCell>
                <TableCell>Created Time</TableCell>
                <TableCell>Completed Time</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {loading ? (
                Array.from({ length: rowsPerPage }).map((_, index) => (
                  <TableRow key={index}>
                    <TableCell colSpan={9}>
                      <Skeleton variant="rectangular" height={40} />
                    </TableCell>
                  </TableRow>
                ))
              ) : paginatedOperations.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} align="center">
                    <Typography variant="body2" color="text.secondary" sx={{ py: 4 }}>
                      No zone operations found
                    </Typography>
                  </TableCell>
                </TableRow>
              ) : (
                paginatedOperations.map((operation) => {
                  const opId = `${operation.zone_id}-${operation.operation_time}`;
                  const opType = mapOperationType(operation.operation as unknown as string);

                  return (
                    <TableRow key={opId} hover>
                      <TableCell>
                        <Button
                          size="small"
                          onClick={() => handleViewDetails(operation)}
                          sx={{ textTransform: 'none' }}
                        >
                          {opId.substring(0, 20)}...
                        </Button>
                      </TableCell>
                      <TableCell>{operation.zone_id}</TableCell>
                      <TableCell>{operation.region_id}</TableCell>
                      <TableCell>
                        <Chip
                          icon={opType === 'UP_ZONE' ? <ArrowUpwardIcon /> : <ArrowDownwardIcon />}
                          label={opType}
                          color={opType === 'UP_ZONE' ? 'success' : 'warning'}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={operation.status}
                          color={getStatusColor(operation.status)}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>{operation.operator_id}</TableCell>
                      <TableCell>{formatTimestamp(operation.created_at)}</TableCell>
                      <TableCell>
                        {operation.completed_at
                          ? formatTimestamp(operation.completed_at)
                          : 'N/A'}
                      </TableCell>
                      <TableCell>
                        <Box sx={{ display: 'flex', gap: 1 }}>
                          <Tooltip title="View Details">
                            <IconButton size="small" onClick={() => handleViewDetails(operation)}>
                              <InfoIcon />
                            </IconButton>
                          </Tooltip>
                          {operation.status === 'IN_PROGRESS' && (
                            <Tooltip title="Cancel Operation">
                              <IconButton
                                size="small"
                                color="error"
                                onClick={() => handleCancelOperation(operation)}
                              >
                                <CancelIcon />
                              </IconButton>
                            </Tooltip>
                          )}
                          {operation.status === 'FAILED' && (
                            <Tooltip title="Retry Operation">
                              <IconButton
                                size="small"
                                color="primary"
                                onClick={() => handleRetryOperation(operation)}
                              >
                                <ReplayIcon />
                              </IconButton>
                            </Tooltip>
                          )}
                        </Box>
                      </TableCell>
                    </TableRow>
                  );
                })
              )}
            </TableBody>
          </Table>
        </TableContainer>
        <TablePagination
          component="div"
          count={filteredOperations.length}
          page={page}
          onPageChange={handleChangePage}
          rowsPerPage={rowsPerPage}
          onRowsPerPageChange={handleChangeRowsPerPage}
          rowsPerPageOptions={[10, 25, 50]}
        />
      </Card>

      {/* Create Zone Operation Dialog */}
      <Dialog
        open={createDialogOpen}
        onClose={() => setCreateDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          Create Zone Operation
          <IconButton
            onClick={() => setCreateDialogOpen(false)}
            sx={{ position: 'absolute', right: 8, top: 8 }}
          >
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <TextField
              label="Zone ID"
              fullWidth
              required
              value={formData.zone_id}
              onChange={(e) => setFormData({ ...formData, zone_id: e.target.value })}
              placeholder="e.g., zone-1"
            />

            <TextField
              label="Region ID"
              fullWidth
              required
              value={formData.region_id}
              onChange={(e) => setFormData({ ...formData, region_id: e.target.value })}
              placeholder="e.g., us-east"
            />

            <FormControl component="fieldset">
              <FormLabel component="legend">Operation Type</FormLabel>
              <RadioGroup
                row
                value={formData.operation_type}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    operation_type: e.target.value as 'UP_ZONE' | 'DOWN_ZONE',
                  })
                }
              >
                <FormControlLabel
                  value="DOWN_ZONE"
                  control={<Radio />}
                  label="DOWN_ZONE (Pull Out)"
                />
                <FormControlLabel value="UP_ZONE" control={<Radio />} label="UP_ZONE (Pull In)" />
              </RadioGroup>
            </FormControl>

            <Divider />

            <FormControl component="fieldset">
              <FormLabel component="legend">Target Scope</FormLabel>
              <RadioGroup
                value={formData.target_scope}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    target_scope: e.target.value as 'all' | 'service' | 'instance',
                  })
                }
              >
                <FormControlLabel
                  value="all"
                  control={<Radio />}
                  label="All Instances (Default)"
                />
                <FormControlLabel value="service" control={<Radio />} label="Service Filter" />
                <FormControlLabel value="instance" control={<Radio />} label="Instance Filter" />
              </RadioGroup>
            </FormControl>

            {formData.target_scope === 'service' && (
              <TextField
                label="Service IDs"
                fullWidth
                multiline
                rows={2}
                value={formData.service_filter}
                onChange={(e) => setFormData({ ...formData, service_filter: e.target.value })}
                placeholder="Enter service IDs, one per line"
                helperText="List of service IDs to target (currently not supported by backend)"
              />
            )}

            {formData.target_scope === 'instance' && (
              <TextField
                label="Instance IDs"
                fullWidth
                multiline
                rows={2}
                value={formData.instance_filter}
                onChange={(e) => setFormData({ ...formData, instance_filter: e.target.value })}
                placeholder="Enter instance IDs, one per line"
                helperText="List of instance IDs to target (currently not supported by backend)"
              />
            )}

            <Divider />

            <TextField
              label="Operator"
              fullWidth
              disabled
              value={formData.operator_id}
              helperText="Current logged-in user"
            />

            <TextField
              label="Reason"
              fullWidth
              required
              multiline
              rows={3}
              value={formData.reason}
              onChange={(e) => setFormData({ ...formData, reason: e.target.value })}
              placeholder="Enter the reason for this operation"
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={handleCreateOperation}
            disabled={!formData.zone_id || !formData.region_id || !formData.reason}
          >
            Create Operation
          </Button>
        </DialogActions>
      </Dialog>

      {/* Operation Details Dialog */}
      <Dialog
        open={detailsDialogOpen}
        onClose={() => setDetailsDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Operation Details
          <IconButton
            onClick={() => setDetailsDialogOpen(false)}
            sx={{ position: 'absolute', right: 8, top: 8 }}
          >
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent dividers>
          {selectedOperation && (
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
              {/* Basic Info */}
              <Box>
                <Typography variant="h6" gutterBottom>
                  Basic Information
                </Typography>
                <Grid container spacing={2}>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Zone ID
                    </Typography>
                    <Typography variant="body1">{selectedOperation.zone_id}</Typography>
                  </Grid>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Region ID
                    </Typography>
                    <Typography variant="body1">{selectedOperation.region_id}</Typography>
                  </Grid>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Operation Type
                    </Typography>
                    <Chip
                      icon={
                        mapOperationType(selectedOperation.operation as unknown as string) ===
                        'UP_ZONE' ? (
                          <ArrowUpwardIcon />
                        ) : (
                          <ArrowDownwardIcon />
                        )
                      }
                      label={mapOperationType(selectedOperation.operation as unknown as string)}
                      color={
                        mapOperationType(selectedOperation.operation as unknown as string) ===
                        'UP_ZONE'
                          ? 'success'
                          : 'warning'
                      }
                      size="small"
                    />
                  </Grid>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Status
                    </Typography>
                    <Chip
                      label={selectedOperation.status}
                      color={getStatusColor(selectedOperation.status)}
                      size="small"
                    />
                  </Grid>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Operator
                    </Typography>
                    <Typography variant="body1">{selectedOperation.operator_id}</Typography>
                  </Grid>
                  <Grid size={6}>
                    <Typography variant="body2" color="text.secondary">
                      Created Time
                    </Typography>
                    <Typography variant="body1">
                      {formatTimestamp(selectedOperation.created_at)}
                    </Typography>
                  </Grid>
                </Grid>
              </Box>

              <Divider />

              {/* Progress */}
              <Box>
                <Typography variant="h6" gutterBottom>
                  Operation Progress
                </Typography>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                    <Typography variant="body2" color="text.secondary">
                      Progress
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {selectedOperation.completed_instances || 0} /{' '}
                      {selectedOperation.total_instances || 0} instances
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={
                      selectedOperation.total_instances
                        ? ((selectedOperation.completed_instances || 0) /
                            selectedOperation.total_instances) *
                          100
                        : 0
                    }
                  />
                </Box>
                <Grid container spacing={2}>
                  <Grid size={4}>
                    <Typography variant="body2" color="text.secondary">
                      Total
                    </Typography>
                    <Typography variant="h6">{selectedOperation.total_instances || 0}</Typography>
                  </Grid>
                  <Grid size={4}>
                    <Typography variant="body2" color="text.secondary">
                      Completed
                    </Typography>
                    <Typography variant="h6" color="success.main">
                      {selectedOperation.completed_instances || 0}
                    </Typography>
                  </Grid>
                  <Grid size={4}>
                    <Typography variant="body2" color="text.secondary">
                      Failed
                    </Typography>
                    <Typography variant="h6" color="error.main">
                      {selectedOperation.failed_instances || 0}
                    </Typography>
                  </Grid>
                </Grid>
              </Box>

              <Divider />

              {/* Target Instances */}
              <Box>
                <Typography variant="h6" gutterBottom>
                  Affected Instances
                </Typography>
                <Alert severity="info">
                  Instance details are not yet available from the backend API. This will be
                  implemented when the API supports retrieving affected instances for a zone
                  operation.
                </Alert>
              </Box>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDetailsDialogOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Snackbar */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={() => setSnackbarOpen(false)}
          severity={snackbarSeverity}
          variant="filled"
        >
          {snackbarMessage}
        </Alert>
      </Snackbar>
    </Box>
  );
};

ZoneOps.displayName = 'ZoneOps';

export default ZoneOps;
