/**
 * Services Page Component
 *
 * Features:
 * - Services list view with real API data
 * - Search and filtering by Service ID, Region, Zone, and Status
 * - Pagination with customizable page size
 * - Instance count and health status indicators
 * - Export to CSV functionality
 * - View instances navigation
 * - Service details dialog
 * - Responsive design with loading and error states
 */

import React, { useState, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
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
  Tooltip,
  type SelectChangeEvent,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Search as SearchIcon,
  Refresh as RefreshIcon,
  Download as DownloadIcon,
  Visibility as VisibilityIcon,
  Close as CloseIcon,
} from '@mui/icons-material';
import { getAllServices } from '@/api/discovery';
import type { Service, InstanceStatus, ErrorCode } from '@/api/types';

/**
 * Health status type
 */
type HealthStatus = 'healthy' | 'degraded' | 'down';

/**
 * Service row data for table display
 */
interface ServiceRow {
  serviceId: string;
  regionId: string;
  zoneId: string;
  instanceCount: number;
  healthyCount: number;
  unhealthyCount: number;
  healthStatus: HealthStatus;
  service: Service;
}

/**
 * Services component
 *
 * @returns React component
 */
const Services: React.FC = () => {
  const navigate = useNavigate();

  // ===== State Management =====

  const [services, setServices] = useState<ServiceRow[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [regionFilter, setRegionFilter] = useState<string>('all');
  const [zoneFilter, setZoneFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');

  // Pagination
  const [page, setPage] = useState<number>(0);
  const [rowsPerPage, setRowsPerPage] = useState<number>(10);

  // Detail dialog
  const [detailDialogOpen, setDetailDialogOpen] = useState<boolean>(false);
  const [selectedService, setSelectedService] = useState<Service | null>(null);

  // ===== Data Fetching =====

  /**
   * Fetch services from API
   */
  const fetchServices = async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);

      // TODO: These should come from user context or config
      const regionId = 'default-region';
      const zoneId = 'default-zone';

      const response = await getAllServices(regionId, zoneId);

      if (response.response_status.error_code !== ('success' as ErrorCode)) {
        throw new Error(
          response.response_status.error_message || 'Failed to fetch services'
        );
      }

      // Transform services into table rows
      const rows: ServiceRow[] = response.services.map((service) => {
        const instances = service.instances || [];
        const healthyCount = instances.filter(
          (inst) => inst.status === ('up' as InstanceStatus)
        ).length;
        const unhealthyCount = instances.length - healthyCount;

        let healthStatus: HealthStatus;
        if (instances.length === 0) {
          healthStatus = 'down';
        } else if (healthyCount === instances.length) {
          healthStatus = 'healthy';
        } else if (healthyCount > 0) {
          healthStatus = 'degraded';
        } else {
          healthStatus = 'down';
        }

        // Extract region and zone from first instance
        const firstInstance = instances[0];

        return {
          serviceId: service.service_id,
          regionId: firstInstance?.region_id || 'unknown',
          zoneId: firstInstance?.zone_id || 'unknown',
          instanceCount: instances.length,
          healthyCount,
          unhealthyCount,
          healthStatus,
          service,
        };
      });

      setServices(rows);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchServices();
  }, []);

  // ===== Filtering and Sorting =====

  /**
   * Get unique regions from services
   */
  const regions = useMemo(() => {
    const uniqueRegions = new Set(services.map((s) => s.regionId));
    return Array.from(uniqueRegions).sort();
  }, [services]);

  /**
   * Get unique zones from services
   */
  const zones = useMemo(() => {
    const uniqueZones = new Set(services.map((s) => s.zoneId));
    return Array.from(uniqueZones).sort();
  }, [services]);

  /**
   * Filtered services based on search and filters
   */
  const filteredServices = useMemo(() => {
    return services.filter((service) => {
      // Search filter
      if (
        searchQuery &&
        !service.serviceId.toLowerCase().includes(searchQuery.toLowerCase())
      ) {
        return false;
      }

      // Region filter
      if (regionFilter !== 'all' && service.regionId !== regionFilter) {
        return false;
      }

      // Zone filter
      if (zoneFilter !== 'all' && service.zoneId !== zoneFilter) {
        return false;
      }

      // Status filter
      if (statusFilter !== 'all' && service.healthStatus !== statusFilter) {
        return false;
      }

      return true;
    });
  }, [services, searchQuery, regionFilter, zoneFilter, statusFilter]);

  /**
   * Paginated services
   */
  const paginatedServices = useMemo(() => {
    const startIndex = page * rowsPerPage;
    return filteredServices.slice(startIndex, startIndex + rowsPerPage);
  }, [filteredServices, page, rowsPerPage]);

  // ===== Event Handlers =====

  /**
   * Handle search input change
   */
  const handleSearchChange = (
    event: React.ChangeEvent<HTMLInputElement>
  ): void => {
    setSearchQuery(event.target.value);
    setPage(0); // Reset to first page
  };

  /**
   * Handle region filter change
   */
  const handleRegionChange = (event: SelectChangeEvent<string>): void => {
    setRegionFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle zone filter change
   */
  const handleZoneChange = (event: SelectChangeEvent<string>): void => {
    setZoneFilter(event.target.value);
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
  const handleRowsPerPageChange = (
    event: React.ChangeEvent<HTMLInputElement>
  ): void => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };

  /**
   * Handle refresh button click
   */
  const handleRefresh = (): void => {
    fetchServices();
  };

  /**
   * Handle view instances button click
   */
  const handleViewInstances = (serviceId: string): void => {
    navigate(`/instances?serviceId=${encodeURIComponent(serviceId)}`);
  };

  /**
   * Handle service ID click to open details dialog
   */
  const handleServiceClick = (service: Service): void => {
    setSelectedService(service);
    setDetailDialogOpen(true);
  };

  /**
   * Handle close details dialog
   */
  const handleCloseDialog = (): void => {
    setDetailDialogOpen(false);
    setSelectedService(null);
  };

  /**
   * Export services to CSV
   */
  const handleExport = (): void => {
    const csvHeaders = [
      'Service ID',
      'Region',
      'Zone',
      'Instance Count',
      'Healthy',
      'Unhealthy',
      'Status',
    ];

    const csvRows = filteredServices.map((service) => [
      service.serviceId,
      service.regionId,
      service.zoneId,
      service.instanceCount.toString(),
      service.healthyCount.toString(),
      service.unhealthyCount.toString(),
      service.healthStatus,
    ]);

    const csvContent = [
      csvHeaders.join(','),
      ...csvRows.map((row) => row.join(',')),
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    link.setAttribute('href', url);
    link.setAttribute(
      'download',
      `artemis-services-${new Date().toISOString().split('T')[0]}.csv`
    );
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  // ===== Helper Functions =====

  /**
   * Get health status chip color
   */
  const getHealthStatusColor = (
    status: HealthStatus
  ): 'success' | 'warning' | 'error' => {
    switch (status) {
      case 'healthy':
        return 'success';
      case 'degraded':
        return 'warning';
      case 'down':
        return 'error';
    }
  };

  /**
   * Get health status label
   */
  const getHealthStatusLabel = (status: HealthStatus): string => {
    switch (status) {
      case 'healthy':
        return 'Healthy';
      case 'degraded':
        return 'Degraded';
      case 'down':
        return 'Down';
    }
  };

  // ===== Styles =====

  const headerBoxSx: SxProps<Theme> = {
    marginBottom: 3,
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

  const clickableServiceIdSx: SxProps<Theme> = {
    cursor: 'pointer',
    color: 'primary.main',
    fontWeight: 500,
    '&:hover': {
      textDecoration: 'underline',
    },
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
            <TableCell>Region</TableCell>
            <TableCell>Zone</TableCell>
            <TableCell align="center">Instances</TableCell>
            <TableCell align="center">Status</TableCell>
            <TableCell align="center">Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {[1, 2, 3, 4, 5].map((index) => (
            <TableRow key={index}>
              <TableCell>
                <Skeleton variant="text" width={150} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell align="center">
                <Skeleton variant="text" width={60} />
              </TableCell>
              <TableCell align="center">
                <Skeleton variant="rounded" width={80} height={24} />
              </TableCell>
              <TableCell align="center">
                <Skeleton variant="rounded" width={120} height={36} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  /**
   * Render service details dialog
   */
  const renderDetailsDialog = (): React.ReactElement => (
    <Dialog
      open={detailDialogOpen}
      onClose={handleCloseDialog}
      maxWidth="md"
      fullWidth
    >
      <DialogTitle>
        Service Details
        <IconButton
          aria-label="close"
          onClick={handleCloseDialog}
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
        {selectedService && (
          <Box>
            <Typography variant="h6" gutterBottom>
              {selectedService.service_id}
            </Typography>

            {/* Metadata */}
            {selectedService.metadata && (
              <Box sx={{ marginBottom: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  Metadata:
                </Typography>
                <Paper sx={{ padding: 2, backgroundColor: 'grey.50' }}>
                  {Object.entries(selectedService.metadata).map(
                    ([key, value]) => (
                      <Typography key={key} variant="body2">
                        <strong>{key}:</strong> {value}
                      </Typography>
                    )
                  )}
                </Paper>
              </Box>
            )}

            {/* Instances */}
            <Box sx={{ marginBottom: 3 }}>
              <Typography variant="subtitle2" gutterBottom>
                Instances ({selectedService.instances.length}):
              </Typography>
              <TableContainer component={Paper}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Instance ID</TableCell>
                      <TableCell>IP:Port</TableCell>
                      <TableCell>Status</TableCell>
                      <TableCell>Region/Zone</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {selectedService.instances.map((instance) => (
                      <TableRow key={instance.instance_id}>
                        <TableCell>{instance.instance_id}</TableCell>
                        <TableCell>
                          {instance.ip}:{instance.port}
                        </TableCell>
                        <TableCell>
                          <Chip
                            label={instance.status}
                            size="small"
                            color={
                              instance.status === ('up' as InstanceStatus)
                                ? 'success'
                                : 'error'
                            }
                          />
                        </TableCell>
                        <TableCell>
                          {instance.region_id}/{instance.zone_id}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>

            {/* Route Rules */}
            {selectedService.route_rules &&
              selectedService.route_rules.length > 0 && (
                <Box>
                  <Typography variant="subtitle2" gutterBottom>
                    Route Rules ({selectedService.route_rules.length}):
                  </Typography>
                  <Paper sx={{ padding: 2, backgroundColor: 'grey.50' }}>
                    {selectedService.route_rules.map((rule, index) => (
                      <Typography key={index} variant="body2">
                        <strong>{rule.name}</strong> - {rule.strategy} (
                        {rule.status})
                      </Typography>
                    ))}
                  </Paper>
                </Box>
              )}
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleCloseDialog}>Close</Button>
      </DialogActions>
    </Dialog>
  );

  // ===== Main Render =====

  return (
    <Box>
      {/* Page Header */}
      <Box sx={headerBoxSx}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Services
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage and monitor registered services
        </Typography>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

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

        {/* Region Filter */}
        <FormControl sx={filterFormControlSx}>
          <InputLabel>Region</InputLabel>
          <Select value={regionFilter} onChange={handleRegionChange} label="Region">
            <MenuItem value="all">All Regions</MenuItem>
            {regions.map((region) => (
              <MenuItem key={region} value={region}>
                {region}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        {/* Zone Filter */}
        <FormControl sx={filterFormControlSx}>
          <InputLabel>Zone</InputLabel>
          <Select value={zoneFilter} onChange={handleZoneChange} label="Zone">
            <MenuItem value="all">All Zones</MenuItem>
            {zones.map((zone) => (
              <MenuItem key={zone} value={zone}>
                {zone}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        {/* Status Filter */}
        <FormControl sx={filterFormControlSx}>
          <InputLabel>Status</InputLabel>
          <Select value={statusFilter} onChange={handleStatusChange} label="Status">
            <MenuItem value="all">All Status</MenuItem>
            <MenuItem value="healthy">Healthy</MenuItem>
            <MenuItem value="degraded">Degraded</MenuItem>
            <MenuItem value="down">Down</MenuItem>
          </Select>
        </FormControl>

        {/* Actions */}
        <Box sx={actionsBoxSx}>
          <Tooltip title="Refresh">
            <span>
              <IconButton
                color="primary"
                onClick={handleRefresh}
                disabled={loading}
              >
                <RefreshIcon />
              </IconButton>
            </span>
          </Tooltip>
          <Button
            variant="outlined"
            startIcon={<DownloadIcon />}
            onClick={handleExport}
            disabled={loading || filteredServices.length === 0}
          >
            Export
          </Button>
        </Box>
      </Box>

      {/* Services Table */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Registered Services ({filteredServices.length})
          </Typography>

          {loading ? (
            renderLoadingSkeleton()
          ) : filteredServices.length === 0 ? (
            <Box sx={{ textAlign: 'center', padding: 4 }}>
              <Typography variant="body1" color="text.secondary">
                No services found matching your filters
              </Typography>
            </Box>
          ) : (
            <>
              <TableContainer component={Paper} sx={tableContainerSx}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Service ID</TableCell>
                      <TableCell>Region</TableCell>
                      <TableCell>Zone</TableCell>
                      <TableCell align="center">Instances</TableCell>
                      <TableCell align="center">Status</TableCell>
                      <TableCell align="center">Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {paginatedServices.map((service) => (
                      <TableRow
                        key={service.serviceId}
                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                      >
                        <TableCell>
                          <Typography
                            component="span"
                            sx={clickableServiceIdSx}
                            onClick={() => handleServiceClick(service.service)}
                          >
                            {service.serviceId}
                          </Typography>
                        </TableCell>
                        <TableCell>{service.regionId}</TableCell>
                        <TableCell>{service.zoneId}</TableCell>
                        <TableCell align="center">
                          <Tooltip
                            title={`${service.healthyCount} healthy, ${service.unhealthyCount} unhealthy`}
                          >
                            <span>{service.instanceCount}</span>
                          </Tooltip>
                        </TableCell>
                        <TableCell align="center">
                          <Chip
                            label={getHealthStatusLabel(service.healthStatus)}
                            color={getHealthStatusColor(service.healthStatus)}
                            size="small"
                          />
                        </TableCell>
                        <TableCell align="center">
                          <Button
                            variant="outlined"
                            size="small"
                            startIcon={<VisibilityIcon />}
                            onClick={() => handleViewInstances(service.serviceId)}
                          >
                            View Instances
                          </Button>
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
                count={filteredServices.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onPageChange={handlePageChange}
                onRowsPerPageChange={handleRowsPerPageChange}
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Service Details Dialog */}
      {renderDetailsDialog()}
    </Box>
  );
};

/**
 * Display name for debugging
 */
Services.displayName = 'Services';

export default Services;
