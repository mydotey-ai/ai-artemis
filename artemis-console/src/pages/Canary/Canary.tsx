/**
 * Canary Page Component
 *
 * Features:
 * - Canary configuration list with status management
 * - Create/Edit canary configurations with IP whitelist
 * - Enable/Disable canary deployment per service
 * - IP whitelist management (add/remove IPs)
 * - Search and filtering by service ID and status
 * - Statistics cards showing active configs and services
 * - Pagination and CSV export
 * - Responsive design with confirmation dialogs
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
  Paper,
  Chip,
  Button,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  Tooltip,
  Alert,
  Switch,
  Grid,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Divider,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  Skeleton,
  type SelectChangeEvent,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Refresh as RefreshIcon,
  Download as DownloadIcon,
  Search as SearchIcon,
  Close as CloseIcon,
  List as ListIcon,
} from '@mui/icons-material';
import {
  listCanaryConfigs,
  setCanaryConfig,
  deleteCanaryConfig,
  enableCanary,
  disableCanary,
  getCanaryStats,
  addIpToWhitelist,
  removeIpFromWhitelist,
  type CanaryConfig,
  type SetCanaryConfigRequest,
} from '@/api/canary';
import { getAllServices } from '@/api/discovery';
import type { Service } from '@/api/types';

// ===== Type Definitions =====

/**
 * Canary configuration display data
 */
interface CanaryConfigDisplay extends CanaryConfig {
  stats?: {
    whitelist_count: number;
  };
}

/**
 * Statistics data
 */
interface Statistics {
  activeConfigs: number;
  totalConfigs: number;
  servicesWithCanary: number;
}

/**
 * Canary form data
 */
interface CanaryFormData {
  service_id: string;
  ip_whitelist: string;
  description: string;
}

/**
 * Delete dialog state
 */
interface DeleteDialogState {
  open: boolean;
  serviceId: string | null;
}

/**
 * IP dialog state
 */
interface IpDialogState {
  open: boolean;
  serviceId: string | null;
  currentIps: string[];
}

// ===== Main Component =====

/**
 * Canary component
 *
 * @returns React component
 */
const Canary: React.FC = () => {
  // ===== State Management =====

  // Data state
  const [configs, setConfigs] = useState<CanaryConfigDisplay[]>([]);
  const [services, setServices] = useState<Service[]>([]);
  const [statistics, setStatistics] = useState<Statistics>({
    activeConfigs: 0,
    totalConfigs: 0,
    servicesWithCanary: 0,
  });

  // Loading and error state
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<boolean>(false);

  // Filter state
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [statusFilter, setStatusFilter] = useState<string>('all');

  // Pagination state
  const [page, setPage] = useState<number>(0);
  const [rowsPerPage, setRowsPerPage] = useState<number>(10);

  // Dialog state
  const [dialogOpen, setDialogOpen] = useState<boolean>(false);
  const [editMode, setEditMode] = useState<boolean>(false);
  const [deleteDialog, setDeleteDialog] = useState<DeleteDialogState>({
    open: false,
    serviceId: null,
  });
  const [ipDialog, setIpDialog] = useState<IpDialogState>({
    open: false,
    serviceId: null,
    currentIps: [],
  });

  // Form state
  const [formData, setFormData] = useState<CanaryFormData>({
    service_id: '',
    ip_whitelist: '',
    description: '',
  });
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  // IP management state
  const [newIp, setNewIp] = useState<string>('');
  const [ipError, setIpError] = useState<string>('');

  // ===== Data Fetching =====

  /**
   * Fetch all data
   */
  const fetchAllData = useCallback(async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);

      // Fetch canary configs
      const configsResponse = await listCanaryConfigs();
      if (!configsResponse.success || !configsResponse.data) {
        throw new Error(configsResponse.message || 'Failed to fetch canary configs');
      }

      // Fetch services
      const servicesResponse = await getAllServices('default-region', 'default-zone');
      if (servicesResponse.response_status.error_code !== 'success') {
        throw new Error(
          servicesResponse.response_status.error_message || 'Failed to fetch services'
        );
      }

      // Fetch stats
      const statsResponse = await getCanaryStats();
      if (statsResponse.success && statsResponse.data) {
        setStatistics({
          activeConfigs: statsResponse.data.enabled_count,
          totalConfigs: statsResponse.data.total_services,
          servicesWithCanary: statsResponse.data.total_services,
        });
      }

      setConfigs(configsResponse.data);
      setServices(servicesResponse.services);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAllData();
  }, [fetchAllData]);

  // ===== Filtering and Sorting =====

  /**
   * Filtered configs based on search and filters
   */
  const filteredConfigs = useMemo(() => {
    return configs.filter((config) => {
      // Search filter
      if (
        searchQuery &&
        !config.service_id.toLowerCase().includes(searchQuery.toLowerCase())
      ) {
        return false;
      }

      // Status filter
      if (statusFilter === 'active' && !config.enabled) {
        return false;
      }
      if (statusFilter === 'inactive' && config.enabled) {
        return false;
      }

      return true;
    });
  }, [configs, searchQuery, statusFilter]);

  /**
   * Paginated configs
   */
  const paginatedConfigs = useMemo(() => {
    const startIndex = page * rowsPerPage;
    return filteredConfigs.slice(startIndex, startIndex + rowsPerPage);
  }, [filteredConfigs, page, rowsPerPage]);

  /**
   * Available services for selection (not already configured)
   */
  const availableServices = useMemo(() => {
    if (editMode) {
      return services;
    }
    const configuredServiceIds = new Set(configs.map((c) => c.service_id));
    return services.filter((s) => !configuredServiceIds.has(s.service_id));
  }, [services, configs, editMode]);

  // ===== Event Handlers =====

  /**
   * Handle search input change
   */
  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    setSearchQuery(event.target.value);
    setPage(0);
  };

  /**
   * Handle status filter change
   */
  const handleStatusChange = (event: SelectChangeEvent<string>): void => {
    setStatusFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle page change
   */
  const handlePageChange = (_event: unknown, newPage: number): void => {
    setPage(newPage);
  };

  /**
   * Handle rows per page change
   */
  const handleRowsPerPageChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };

  /**
   * Handle refresh button click
   */
  const handleRefresh = (): void => {
    fetchAllData();
  };

  /**
   * Handle create button click
   */
  const handleCreate = (): void => {
    setEditMode(false);
    setFormData({
      service_id: '',
      ip_whitelist: '',
      description: '',
    });
    setFormErrors({});
    setDialogOpen(true);
  };

  /**
   * Handle edit button click
   */
  const handleEdit = (config: CanaryConfig): void => {
    setEditMode(true);
    setFormData({
      service_id: config.service_id,
      ip_whitelist: config.ip_whitelist.join('\n'),
      description: '',
    });
    setFormErrors({});
    setDialogOpen(true);
  };

  /**
   * Handle delete button click
   */
  const handleDelete = (serviceId: string): void => {
    setDeleteDialog({ open: true, serviceId });
  };

  /**
   * Handle confirm delete
   */
  const handleConfirmDelete = async (): Promise<void> => {
    if (!deleteDialog.serviceId) return;

    try {
      setActionLoading(true);
      const response = await deleteCanaryConfig(deleteDialog.serviceId);
      if (!response.success) {
        throw new Error(response.message || 'Failed to delete canary config');
      }

      await fetchAllData();
      setDeleteDialog({ open: false, serviceId: null });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete config');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle status toggle
   */
  const handleStatusToggle = async (
    serviceId: string,
    currentEnabled: boolean
  ): Promise<void> => {
    try {
      setActionLoading(true);
      const response = currentEnabled
        ? await disableCanary(serviceId)
        : await enableCanary({ service_id: serviceId, enabled: true });

      if (!response.success) {
        throw new Error(response.message || 'Failed to update canary status');
      }

      await fetchAllData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update status');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle manage IPs button click
   */
  const handleManageIps = (config: CanaryConfig): void => {
    setIpDialog({
      open: true,
      serviceId: config.service_id,
      currentIps: [...config.ip_whitelist],
    });
    setNewIp('');
    setIpError('');
  };

  /**
   * Validate IP address (supports CIDR notation)
   */
  const validateIp = (ip: string): boolean => {
    // Simple IP validation (IPv4 and CIDR)
    const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}(\/\d{1,2})?$/;
    if (!ipv4Regex.test(ip)) {
      return false;
    }

    const parts = ip.split('/');
    const ipParts = parts[0].split('.');
    if (ipParts.some((part) => parseInt(part, 10) > 255)) {
      return false;
    }

    if (parts.length === 2) {
      const cidr = parseInt(parts[1], 10);
      if (cidr < 0 || cidr > 32) {
        return false;
      }
    }

    return true;
  };

  /**
   * Handle add IP to whitelist
   */
  const handleAddIp = async (): Promise<void> => {
    if (!ipDialog.serviceId) return;

    const trimmedIp = newIp.trim();
    if (!trimmedIp) {
      setIpError('IP address is required');
      return;
    }

    if (!validateIp(trimmedIp)) {
      setIpError('Invalid IP address format (e.g., 192.168.1.1 or 192.168.1.0/24)');
      return;
    }

    if (ipDialog.currentIps.includes(trimmedIp)) {
      setIpError('IP address already exists in whitelist');
      return;
    }

    try {
      setActionLoading(true);
      const response = await addIpToWhitelist(ipDialog.serviceId, [trimmedIp]);
      if (!response.success || !response.data) {
        throw new Error(response.message || 'Failed to add IP');
      }

      setIpDialog({
        ...ipDialog,
        currentIps: response.data.ip_whitelist,
      });
      setNewIp('');
      setIpError('');
      await fetchAllData();
    } catch (err) {
      setIpError(err instanceof Error ? err.message : 'Failed to add IP');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle remove IP from whitelist
   */
  const handleRemoveIp = async (ip: string): Promise<void> => {
    if (!ipDialog.serviceId) return;

    try {
      setActionLoading(true);
      const response = await removeIpFromWhitelist(ipDialog.serviceId, [ip]);
      if (!response.success || !response.data) {
        throw new Error(response.message || 'Failed to remove IP');
      }

      setIpDialog({
        ...ipDialog,
        currentIps: response.data.ip_whitelist,
      });
      await fetchAllData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove IP');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Validate form data
   */
  const validateForm = (): boolean => {
    const errors: Record<string, string> = {};

    if (!formData.service_id.trim()) {
      errors.service_id = 'Service ID is required';
    }

    // Validate IPs
    const ips = formData.ip_whitelist
      .split('\n')
      .map((ip) => ip.trim())
      .filter((ip) => ip.length > 0);

    if (ips.length === 0) {
      errors.ip_whitelist = 'At least one IP address is required';
    } else {
      const invalidIps = ips.filter((ip) => !validateIp(ip));
      if (invalidIps.length > 0) {
        errors.ip_whitelist = `Invalid IP addresses: ${invalidIps.join(', ')}`;
      }
    }

    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  /**
   * Handle form submit
   */
  const handleSubmit = async (): Promise<void> => {
    if (!validateForm()) {
      return;
    }

    try {
      setActionLoading(true);

      const ips = formData.ip_whitelist
        .split('\n')
        .map((ip) => ip.trim())
        .filter((ip) => ip.length > 0);

      const request: SetCanaryConfigRequest = {
        service_id: formData.service_id,
        ip_whitelist: ips,
        description: formData.description || undefined,
      };

      const response = await setCanaryConfig(request);
      if (!response.success) {
        throw new Error(response.message || 'Failed to save canary config');
      }

      await fetchAllData();
      setDialogOpen(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save config');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle form input change
   */
  const handleFormChange = (field: keyof CanaryFormData, value: string): void => {
    setFormData({ ...formData, [field]: value });
    if (formErrors[field]) {
      setFormErrors({ ...formErrors, [field]: '' });
    }
  };

  /**
   * Export configs to CSV
   */
  const handleExport = (): void => {
    const csvHeaders = ['Service ID', 'Status', 'Whitelist IPs', 'Created At', 'Updated At'];

    const csvRows = filteredConfigs.map((config) => [
      config.service_id,
      config.enabled ? 'ACTIVE' : 'INACTIVE',
      config.ip_whitelist.join('; '),
      config.created_at || 'N/A',
      config.updated_at || 'N/A',
    ]);

    const csvContent = [csvHeaders.join(','), ...csvRows.map((row) => row.join(','))].join(
      '\n'
    );

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    link.setAttribute('href', url);
    link.setAttribute(
      'download',
      `artemis-canary-configs-${new Date().toISOString().split('T')[0]}.csv`
    );
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  // ===== Styles =====

  const headerBoxSx: SxProps<Theme> = {
    marginBottom: 3,
  };

  const statsCardSx: SxProps<Theme> = {
    textAlign: 'center',
    padding: 2,
  };

  const filtersBoxSx: SxProps<Theme> = {
    display: 'flex',
    gap: 2,
    marginBottom: 3,
    flexWrap: 'wrap',
    alignItems: 'center',
  };

  const searchFieldSx: SxProps<Theme> = {
    minWidth: { xs: '100%', sm: 300 },
  };

  const filterFormControlSx: SxProps<Theme> = {
    minWidth: { xs: '100%', sm: 150 },
  };

  const actionsBoxSx: SxProps<Theme> = {
    display: 'flex',
    gap: 1,
    marginLeft: 'auto',
  };

  const tableContainerSx: SxProps<Theme> = {
    marginTop: 2,
    overflowX: 'auto',
  };

  // ===== Render Functions =====

  /**
   * Render loading skeleton
   */
  const renderLoadingSkeleton = (): React.ReactElement => (
    <TableContainer component={Paper} sx={tableContainerSx}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Service ID</TableCell>
            <TableCell align="center">Status</TableCell>
            <TableCell>Whitelist IPs</TableCell>
            <TableCell>Created At</TableCell>
            <TableCell align="center">Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {[1, 2, 3, 4, 5].map((index) => (
            <TableRow key={index}>
              <TableCell>
                <Skeleton variant="text" width={150} />
              </TableCell>
              <TableCell align="center">
                <Skeleton variant="rectangular" width={60} height={24} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={200} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell align="center">
                <Skeleton variant="rectangular" width={200} height={36} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  /**
   * Render statistics cards
   */
  const renderStatisticsCards = (): React.ReactElement => (
    <Grid container spacing={3} sx={{ marginBottom: 3 }}>
      <Grid size={{ xs: 12, sm: 4 }}>
        <Card>
          <CardContent sx={statsCardSx}>
            <Typography variant="h4" color="primary" fontWeight={600}>
              {statistics.activeConfigs}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Active Canary Configs
            </Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid size={{ xs: 12, sm: 4 }}>
        <Card>
          <CardContent sx={statsCardSx}>
            <Typography variant="h4" color="secondary" fontWeight={600}>
              {statistics.totalConfigs}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Total Configs
            </Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid size={{ xs: 12, sm: 4 }}>
        <Card>
          <CardContent sx={statsCardSx}>
            <Typography variant="h4" color="success.main" fontWeight={600}>
              {statistics.servicesWithCanary}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Services with Canary
            </Typography>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );

  /**
   * Render create/edit dialog
   */
  const renderDialog = (): React.ReactElement => (
    <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} maxWidth="md" fullWidth>
      <DialogTitle>
        {editMode ? 'Edit Canary Config' : 'Create Canary Config'}
        <IconButton
          aria-label="close"
          onClick={() => setDialogOpen(false)}
          sx={{
            position: 'absolute',
            right: 8,
            top: 8,
            color: (theme) => theme.palette.grey[500],
          }}
        >
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          {/* Service ID */}
          <FormControl fullWidth error={Boolean(formErrors.service_id)}>
            <InputLabel>Service ID *</InputLabel>
            <Select
              value={formData.service_id}
              onChange={(e) => handleFormChange('service_id', e.target.value)}
              label="Service ID *"
              disabled={editMode}
            >
              {availableServices.map((service) => (
                <MenuItem key={service.service_id} value={service.service_id}>
                  {service.service_id}
                </MenuItem>
              ))}
            </Select>
            {formErrors.service_id && (
              <Typography variant="caption" color="error">
                {formErrors.service_id}
              </Typography>
            )}
          </FormControl>

          {/* IP Whitelist */}
          <TextField
            fullWidth
            multiline
            rows={6}
            label="Whitelist IPs *"
            placeholder="Enter one IP per line (e.g., 192.168.1.1 or 192.168.1.0/24)"
            value={formData.ip_whitelist}
            onChange={(e) => handleFormChange('ip_whitelist', e.target.value)}
            error={Boolean(formErrors.ip_whitelist)}
            helperText={
              formErrors.ip_whitelist ||
              'Enter IP addresses (one per line). Supports CIDR notation (e.g., 192.168.1.0/24)'
            }
          />

          {/* Description */}
          <TextField
            fullWidth
            multiline
            rows={3}
            label="Description"
            placeholder="Optional description for this canary configuration"
            value={formData.description}
            onChange={(e) => handleFormChange('description', e.target.value)}
          />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setDialogOpen(false)} disabled={actionLoading}>
          Cancel
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={actionLoading}
          startIcon={actionLoading ? <Skeleton width={20} height={20} /> : undefined}
        >
          {editMode ? 'Update' : 'Create'}
        </Button>
      </DialogActions>
    </Dialog>
  );

  /**
   * Render delete confirmation dialog
   */
  const renderDeleteDialog = (): React.ReactElement => (
    <Dialog open={deleteDialog.open} onClose={() => setDeleteDialog({ open: false, serviceId: null })}>
      <DialogTitle>Delete Canary Config</DialogTitle>
      <DialogContent>
        <Typography>
          Are you sure you want to delete the canary configuration for service{' '}
          <strong>{deleteDialog.serviceId}</strong>?
        </Typography>
        <Alert severity="warning" sx={{ marginTop: 2 }}>
          This will revert all traffic to the stable version.
        </Alert>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setDeleteDialog({ open: false, serviceId: null })} disabled={actionLoading}>
          Cancel
        </Button>
        <Button
          onClick={handleConfirmDelete}
          variant="contained"
          color="error"
          disabled={actionLoading}
        >
          Delete
        </Button>
      </DialogActions>
    </Dialog>
  );

  /**
   * Render IP management dialog
   */
  const renderIpDialog = (): React.ReactElement => (
    <Dialog
      open={ipDialog.open}
      onClose={() => setIpDialog({ open: false, serviceId: null, currentIps: [] })}
      maxWidth="sm"
      fullWidth
    >
      <DialogTitle>
        Manage Whitelist IPs
        <IconButton
          aria-label="close"
          onClick={() => setIpDialog({ open: false, serviceId: null, currentIps: [] })}
          sx={{
            position: 'absolute',
            right: 8,
            top: 8,
            color: (theme) => theme.palette.grey[500],
          }}
        >
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers>
        <Box sx={{ marginBottom: 3 }}>
          <Typography variant="subtitle2" gutterBottom>
            Service: <strong>{ipDialog.serviceId}</strong>
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Current whitelist: {ipDialog.currentIps.length} IP(s)
          </Typography>
        </Box>

        {/* Add IP */}
        <Box sx={{ display: 'flex', gap: 1, marginBottom: 3 }}>
          <TextField
            fullWidth
            size="small"
            label="Add IP"
            placeholder="192.168.1.1 or 192.168.1.0/24"
            value={newIp}
            onChange={(e) => {
              setNewIp(e.target.value);
              if (ipError) setIpError('');
            }}
            error={Boolean(ipError)}
            helperText={ipError}
          />
          <Button
            variant="contained"
            onClick={handleAddIp}
            disabled={actionLoading || !newIp.trim()}
            sx={{ minWidth: 100 }}
          >
            Add
          </Button>
        </Box>

        <Divider sx={{ marginBottom: 2 }} />

        {/* IP List */}
        <Typography variant="subtitle2" gutterBottom>
          Current Whitelist
        </Typography>
        {ipDialog.currentIps.length === 0 ? (
          <Typography variant="body2" color="text.secondary" sx={{ textAlign: 'center', padding: 2 }}>
            No IPs in whitelist
          </Typography>
        ) : (
          <List dense>
            {ipDialog.currentIps.map((ip) => (
              <ListItem key={ip} divider>
                <ListItemText primary={ip} />
                <ListItemSecondaryAction>
                  <IconButton
                    edge="end"
                    size="small"
                    onClick={() => handleRemoveIp(ip)}
                    disabled={actionLoading}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setIpDialog({ open: false, serviceId: null, currentIps: [] })}>
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );

  // ===== Main Render =====

  return (
    <Box>
      {/* Page Header */}
      <Box sx={headerBoxSx}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Canary Deployment
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage canary deployment configurations and IP whitelists
        </Typography>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Statistics Cards */}
      {!loading && renderStatisticsCards()}

      {/* Filters and Actions */}
      <Box sx={filtersBoxSx}>
        {/* Search */}
        <TextField
          sx={searchFieldSx}
          placeholder="Search by Service ID..."
          value={searchQuery}
          onChange={handleSearchChange}
          InputProps={{
            startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'action.active' }} />,
          }}
        />

        {/* Status Filter */}
        <FormControl sx={filterFormControlSx}>
          <InputLabel>Status</InputLabel>
          <Select value={statusFilter} onChange={handleStatusChange} label="Status">
            <MenuItem value="all">All Status</MenuItem>
            <MenuItem value="active">Active</MenuItem>
            <MenuItem value="inactive">Inactive</MenuItem>
          </Select>
        </FormControl>

        {/* Actions */}
        <Box sx={actionsBoxSx}>
          <Tooltip title="Refresh">
            <span>
              <IconButton color="primary" onClick={handleRefresh} disabled={loading}>
                <RefreshIcon />
              </IconButton>
            </span>
          </Tooltip>
          <Button
            variant="outlined"
            startIcon={<DownloadIcon />}
            onClick={handleExport}
            disabled={loading || filteredConfigs.length === 0}
          >
            Export
          </Button>
          <Button variant="contained" startIcon={<AddIcon />} onClick={handleCreate}>
            Create Config
          </Button>
        </Box>
      </Box>

      {/* Configs Table */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Canary Configurations ({filteredConfigs.length})
          </Typography>

          {loading ? (
            renderLoadingSkeleton()
          ) : filteredConfigs.length === 0 ? (
            <Box sx={{ textAlign: 'center', padding: 4 }}>
              <Typography variant="body1" color="text.secondary">
                No canary configurations found
              </Typography>
            </Box>
          ) : (
            <>
              <TableContainer component={Paper} sx={tableContainerSx}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Service ID</TableCell>
                      <TableCell align="center">Status</TableCell>
                      <TableCell>Whitelist IPs</TableCell>
                      <TableCell>Created At</TableCell>
                      <TableCell align="center">Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {paginatedConfigs.map((config) => (
                      <TableRow
                        key={config.service_id}
                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                      >
                        <TableCell>
                          <Typography variant="body2" fontWeight={500}>
                            {config.service_id}
                          </Typography>
                        </TableCell>
                        <TableCell align="center">
                          <Tooltip title={config.enabled ? 'Disable' : 'Enable'}>
                            <Switch
                              checked={config.enabled}
                              onChange={() => handleStatusToggle(config.service_id, config.enabled)}
                              color="success"
                            />
                          </Tooltip>
                        </TableCell>
                        <TableCell>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Chip
                              label={`${config.ip_whitelist.length} IP(s)`}
                              size="small"
                              color="primary"
                              variant="outlined"
                            />
                            <IconButton
                              size="small"
                              onClick={() => handleManageIps(config)}
                              title="Manage IPs"
                            >
                              <ListIcon fontSize="small" />
                            </IconButton>
                          </Box>
                        </TableCell>
                        <TableCell>
                          <Typography variant="body2">
                            {config.created_at
                              ? new Date(config.created_at).toLocaleString()
                              : 'N/A'}
                          </Typography>
                        </TableCell>
                        <TableCell align="center">
                          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'center' }}>
                            <Tooltip title="Edit">
                              <IconButton size="small" onClick={() => handleEdit(config)}>
                                <EditIcon fontSize="small" />
                              </IconButton>
                            </Tooltip>
                            <Tooltip title="Delete">
                              <IconButton
                                size="small"
                                color="error"
                                onClick={() => handleDelete(config.service_id)}
                              >
                                <DeleteIcon fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          </Box>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>

              {/* Pagination */}
              <TablePagination
                rowsPerPageOptions={[10, 25, 50]}
                component="div"
                count={filteredConfigs.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onPageChange={handlePageChange}
                onRowsPerPageChange={handleRowsPerPageChange}
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Dialogs */}
      {renderDialog()}
      {renderDeleteDialog()}
      {renderIpDialog()}
    </Box>
  );
};

/**
 * Display name for debugging
 */
Canary.displayName = 'Canary';

export default Canary;
