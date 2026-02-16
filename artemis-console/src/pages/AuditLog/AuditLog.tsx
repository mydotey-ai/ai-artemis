/**
 * Audit Log Page Component
 *
 * Features:
 * - Comprehensive audit log viewing with advanced filtering
 * - Event Type, Resource Type, Time Range, Operator, IP filtering
 * - Pagination with customizable page size (10/25/50/100)
 * - Log details dialog with JSON formatting
 * - Statistics visualization (Total Logs, Event Type Distribution, Top Operators)
 * - Export to CSV/JSON functionality
 * - Auto-refresh every 30 seconds
 * - Responsive design with loading and error states
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
  Grid,
  Collapse,
  Tooltip,
  type SelectChangeEvent,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Search as SearchIcon,
  Refresh as RefreshIcon,
  Download as DownloadIcon,
  Close as CloseIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
  ContentCopy as ContentCopyIcon,
  FilterList as FilterListIcon,
} from '@mui/icons-material';
import { format } from 'date-fns';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip as RechartsTooltip, Legend } from 'recharts';
import { queryLogs, type AuditLog as AuditLogType, type QueryLogsParams } from '@/api/audit';

/**
 * Event Type constants for filtering
 */
const EventType = {
  CREATE: 'CREATE',
  UPDATE: 'UPDATE',
  DELETE: 'DELETE',
  ACCESS: 'ACCESS',
  LOGIN: 'LOGIN',
  LOGOUT: 'LOGOUT',
} as const;

/**
 * Resource Type constants
 */
const ResourceType = {
  SERVICE: 'Service',
  INSTANCE: 'Instance',
  GROUP: 'Group',
  ROUTE_RULE: 'RouteRule',
  USER: 'User',
} as const;

/**
 * Time range preset options
 */
const TimeRangePreset = {
  LAST_HOUR: 'Last Hour',
  LAST_24_HOURS: 'Last 24 Hours',
  LAST_7_DAYS: 'Last 7 Days',
  LAST_30_DAYS: 'Last 30 Days',
  CUSTOM: 'Custom',
} as const;

type TimeRangePresetValue = typeof TimeRangePreset[keyof typeof TimeRangePreset];

/**
 * Active filter chip data
 */
interface FilterChip {
  key: string;
  label: string;
  value: string;
}

/**
 * Statistics data
 */
interface Statistics {
  totalLogs: number;
  eventTypeDistribution: { name: string; value: number }[];
  topOperators: { name: string; count: number }[];
}

/**
 * Event type color mapping
 */
const EVENT_TYPE_COLORS: Record<string, 'success' | 'info' | 'error' | 'default' | 'warning'> = {
  CREATE: 'success',
  UPDATE: 'info',
  DELETE: 'error',
  ACCESS: 'default',
  LOGIN: 'info',
  LOGOUT: 'default',
};

/**
 * Pie chart colors
 */
const CHART_COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8', '#82CA9D'];

/**
 * Format timestamp to local time
 */
const formatTimestamp = (timestamp: string): string => {
  try {
    return format(new Date(timestamp), 'yyyy-MM-dd HH:mm:ss');
  } catch (error) {
    return timestamp;
  }
};

/**
 * Get time range in ISO format
 */
const getTimeRange = (preset: TimeRangePresetValue): { startTime: string; endTime: string } | null => {
  const now = new Date();
  const endTime = now.toISOString();
  let startTime: string;

  switch (preset) {
    case 'Last Hour':
      startTime = new Date(now.getTime() - 60 * 60 * 1000).toISOString();
      break;
    case 'Last 24 Hours':
      startTime = new Date(now.getTime() - 24 * 60 * 60 * 1000).toISOString();
      break;
    case 'Last 7 Days':
      startTime = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
      break;
    case 'Last 30 Days':
      startTime = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000).toISOString();
      break;
    case 'Custom':
    default:
      return null;
  }

  return { startTime, endTime };
};

/**
 * AuditLog component
 */
const AuditLog: React.FC = () => {
  // ===== State Management =====
  const [logs, setLogs] = useState<AuditLogType[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdateTime, setLastUpdateTime] = useState<Date>(new Date());

  // Filters
  const [timeRangePreset, setTimeRangePreset] = useState<TimeRangePresetValue>(TimeRangePreset.LAST_24_HOURS);
  const [customStartTime, setCustomStartTime] = useState<string>('');
  const [customEndTime, setCustomEndTime] = useState<string>('');
  const [eventTypeFilter, setEventTypeFilter] = useState<string[]>([]);
  const [resourceTypeFilter, setResourceTypeFilter] = useState<string>('all');
  const [operatorFilter, setOperatorFilter] = useState<string>('');
  const [resourceIdFilter, setResourceIdFilter] = useState<string>('');
  const [ipAddressFilter, setIpAddressFilter] = useState<string>('');
  const [showFilters, setShowFilters] = useState<boolean>(true);

  // Pagination
  const [page, setPage] = useState<number>(0);
  const [rowsPerPage, setRowsPerPage] = useState<number>(25);

  // Detail dialog
  const [detailDialogOpen, setDetailDialogOpen] = useState<boolean>(false);
  const [selectedLog, setSelectedLog] = useState<AuditLogType | null>(null);

  // Statistics
  const [statistics, setStatistics] = useState<Statistics>({
    totalLogs: 0,
    eventTypeDistribution: [],
    topOperators: [],
  });

  // Expanded rows for inline details
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

  // Auto-refresh
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);

  // ===== Data Fetching =====

  /**
   * Fetch audit logs from API
   */
  const fetchLogs = useCallback(async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);

      // Build query parameters
      const params: QueryLogsParams = {
        limit: 1000, // Fetch more for client-side filtering
        offset: 0,
      };

      // Add time range filter
      const timeRange = getTimeRange(timeRangePreset);
      if (timeRange) {
        params.start_time = timeRange.startTime;
        params.end_time = timeRange.endTime;
      } else if (timeRangePreset === 'Custom') {
        if (customStartTime) params.start_time = new Date(customStartTime).toISOString();
        if (customEndTime) params.end_time = new Date(customEndTime).toISOString();
      }

      // Add other filters
      if (operatorFilter) params.operator_id = operatorFilter;
      if (resourceTypeFilter !== 'all') params.resource_type = resourceTypeFilter;

      const response = await queryLogs(params);

      if (response.success && response.data) {
        setLogs(response.data);
        setLastUpdateTime(new Date());

        // Calculate statistics
        calculateStatistics(response.data);
      } else {
        throw new Error(response.message || 'Failed to fetch audit logs');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
      console.error('Failed to fetch audit logs:', err);
    } finally {
      setLoading(false);
    }
  }, [timeRangePreset, customStartTime, customEndTime, operatorFilter, resourceTypeFilter]);

  /**
   * Calculate statistics from logs
   */
  const calculateStatistics = (logData: AuditLogType[]): void => {
    // Event type distribution
    const eventTypeCounts: Record<string, number> = {};
    logData.forEach((log) => {
      const eventType = log.operation_type;
      eventTypeCounts[eventType] = (eventTypeCounts[eventType] || 0) + 1;
    });

    const eventTypeDistribution = Object.entries(eventTypeCounts).map(([name, value]) => ({
      name,
      value,
    }));

    // Top 5 operators
    const operatorCounts: Record<string, number> = {};
    logData.forEach((log) => {
      const operator = log.operator_id;
      operatorCounts[operator] = (operatorCounts[operator] || 0) + 1;
    });

    const topOperators = Object.entries(operatorCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([name, count]) => ({ name, count }));

    setStatistics({
      totalLogs: logData.length,
      eventTypeDistribution,
      topOperators,
    });
  };

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  // Auto-refresh every 30 seconds
  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      fetchLogs();
    }, 30000);

    return () => clearInterval(interval);
  }, [autoRefresh, fetchLogs]);

  // ===== Filtering =====

  /**
   * Filtered logs based on all filters
   */
  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      // Event type filter
      if (eventTypeFilter.length > 0 && !eventTypeFilter.includes(log.operation_type)) {
        return false;
      }

      // Resource ID filter
      if (resourceIdFilter && !log.resource_id.toLowerCase().includes(resourceIdFilter.toLowerCase())) {
        return false;
      }

      // IP address filter (if available in details)
      if (ipAddressFilter && log.details) {
        const ipMatch = JSON.stringify(log.details).toLowerCase().includes(ipAddressFilter.toLowerCase());
        if (!ipMatch) return false;
      }

      return true;
    });
  }, [logs, eventTypeFilter, resourceIdFilter, ipAddressFilter]);

  /**
   * Paginated logs
   */
  const paginatedLogs = useMemo(() => {
    const startIndex = page * rowsPerPage;
    return filteredLogs.slice(startIndex, startIndex + rowsPerPage);
  }, [filteredLogs, page, rowsPerPage]);

  /**
   * Get active filter chips
   */
  const activeFilters = useMemo((): FilterChip[] => {
    const filters: FilterChip[] = [];

    if (timeRangePreset !== 'Last 24 Hours') {
      filters.push({
        key: 'timeRange',
        label: 'Time Range',
        value: timeRangePreset,
      });
    }

    if (eventTypeFilter.length > 0) {
      filters.push({
        key: 'eventType',
        label: 'Event Types',
        value: eventTypeFilter.join(', '),
      });
    }

    if (resourceTypeFilter !== 'all') {
      filters.push({
        key: 'resourceType',
        label: 'Resource Type',
        value: resourceTypeFilter,
      });
    }

    if (operatorFilter) {
      filters.push({
        key: 'operator',
        label: 'Operator',
        value: operatorFilter,
      });
    }

    if (resourceIdFilter) {
      filters.push({
        key: 'resourceId',
        label: 'Resource ID',
        value: resourceIdFilter,
      });
    }

    if (ipAddressFilter) {
      filters.push({
        key: 'ipAddress',
        label: 'IP Address',
        value: ipAddressFilter,
      });
    }

    return filters;
  }, [timeRangePreset, eventTypeFilter, resourceTypeFilter, operatorFilter, resourceIdFilter, ipAddressFilter]);

  // ===== Event Handlers =====

  /**
   * Handle time range preset change
   */
  const handleTimeRangeChange = (event: SelectChangeEvent<string>): void => {
    setTimeRangePreset(event.target.value as TimeRangePresetValue);
    setPage(0);
  };

  /**
   * Handle event type filter change
   */
  const handleEventTypeChange = (event: SelectChangeEvent<string[]>): void => {
    const value = event.target.value;
    setEventTypeFilter(typeof value === 'string' ? value.split(',') : value);
    setPage(0);
  };

  /**
   * Handle resource type filter change
   */
  const handleResourceTypeChange = (event: SelectChangeEvent<string>): void => {
    setResourceTypeFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle operator filter change (debounced)
   */
  const handleOperatorFilterChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    setOperatorFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle resource ID filter change
   */
  const handleResourceIdFilterChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    setResourceIdFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle IP address filter change
   */
  const handleIpAddressFilterChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    setIpAddressFilter(event.target.value);
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
    fetchLogs();
  };

  /**
   * Handle clear all filters
   */
  const handleClearAllFilters = (): void => {
    setTimeRangePreset('Last 24 Hours');
    setEventTypeFilter([]);
    setResourceTypeFilter('all');
    setOperatorFilter('');
    setResourceIdFilter('');
    setIpAddressFilter('');
    setCustomStartTime('');
    setCustomEndTime('');
    setPage(0);
  };

  /**
   * Handle remove single filter chip
   */
  const handleRemoveFilter = (filterKey: string): void => {
    switch (filterKey) {
      case 'timeRange':
        setTimeRangePreset('Last 24 Hours');
        break;
      case 'eventType':
        setEventTypeFilter([]);
        break;
      case 'resourceType':
        setResourceTypeFilter('all');
        break;
      case 'operator':
        setOperatorFilter('');
        break;
      case 'resourceId':
        setResourceIdFilter('');
        break;
      case 'ipAddress':
        setIpAddressFilter('');
        break;
    }
    setPage(0);
  };

  /**
   * Handle log row click to open details dialog
   */
  const handleLogClick = (log: AuditLogType): void => {
    setSelectedLog(log);
    setDetailDialogOpen(true);
  };

  /**
   * Handle close details dialog
   */
  const handleCloseDialog = (): void => {
    setDetailDialogOpen(false);
    setSelectedLog(null);
  };

  /**
   * Handle copy JSON to clipboard
   */
  const handleCopyJson = (): void => {
    if (selectedLog) {
      navigator.clipboard.writeText(JSON.stringify(selectedLog, null, 2));
    }
  };

  /**
   * Toggle row expansion
   */
  const handleToggleRow = (logId: string): void => {
    const newExpanded = new Set(expandedRows);
    if (newExpanded.has(logId)) {
      newExpanded.delete(logId);
    } else {
      newExpanded.add(logId);
    }
    setExpandedRows(newExpanded);
  };

  /**
   * Export to CSV
   */
  const handleExportCsv = (): void => {
    const csvHeaders = [
      'Timestamp',
      'Event Type',
      'Resource Type',
      'Resource ID',
      'Operator',
      'Action',
      'Result',
      'Error Message',
    ];

    const csvRows = filteredLogs.map((log) => [
      formatTimestamp(log.timestamp),
      log.operation_type,
      log.resource_type,
      log.resource_id,
      log.operator_id,
      log.action,
      log.result,
      log.error_message || '',
    ]);

    const csvContent = [
      csvHeaders.join(','),
      ...csvRows.map((row) => row.map((cell) => `"${cell}"`).join(',')),
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    const timeRange = getTimeRange(timeRangePreset);
    const startDate = timeRange ? format(new Date(timeRange.startTime), 'yyyy-MM-dd') : 'custom';
    const endDate = timeRange ? format(new Date(timeRange.endTime), 'yyyy-MM-dd') : 'custom';

    link.setAttribute('href', url);
    link.setAttribute('download', `audit-logs-${startDate}-${endDate}.csv`);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  /**
   * Export to JSON
   */
  const handleExportJson = (): void => {
    const jsonContent = JSON.stringify(filteredLogs, null, 2);
    const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    const timeRange = getTimeRange(timeRangePreset);
    const startDate = timeRange ? format(new Date(timeRange.startTime), 'yyyy-MM-dd') : 'custom';
    const endDate = timeRange ? format(new Date(timeRange.endTime), 'yyyy-MM-dd') : 'custom';

    link.setAttribute('href', url);
    link.setAttribute('download', `audit-logs-${startDate}-${endDate}.json`);
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
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  };

  const clickableTextSx: SxProps<Theme> = {
    cursor: 'pointer',
    color: 'primary.main',
    fontWeight: 500,
    '&:hover': {
      textDecoration: 'underline',
    },
  };

  // ===== Render Functions =====

  /**
   * Render statistics cards
   */
  const renderStatisticsCards = (): React.ReactElement => (
    <Grid container spacing={3} sx={{ marginBottom: 3 }}>
      {/* Total Logs */}
      <Grid size={{ xs: 12, md: 4 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Total Logs
            </Typography>
            <Typography variant="h3" fontWeight={600}>
              {statistics.totalLogs.toLocaleString()}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ marginTop: 1 }}>
              {timeRangePreset}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      {/* Event Type Distribution */}
      <Grid size={{ xs: 12, md: 4 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Event Type Distribution
            </Typography>
            {statistics.eventTypeDistribution.length > 0 ? (
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie
                    data={statistics.eventTypeDistribution}
                    dataKey="value"
                    nameKey="name"
                    cx="50%"
                    cy="50%"
                    outerRadius={60}
                    label
                  >
                    {statistics.eventTypeDistribution.map((_entry, index) => (
                      <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                    ))}
                  </Pie>
                  <RechartsTooltip />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <Typography variant="body2" color="text.secondary" sx={{ marginTop: 2 }}>
                No data available
              </Typography>
            )}
          </CardContent>
        </Card>
      </Grid>

      {/* Top 5 Operators */}
      <Grid size={{ xs: 12, md: 4 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Top 5 Operators
            </Typography>
            {statistics.topOperators.length > 0 ? (
              <Box sx={{ marginTop: 2 }}>
                {statistics.topOperators.map((op, index) => (
                  <Box
                    key={op.name}
                    sx={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      marginBottom: 1,
                      paddingY: 0.5,
                    }}
                  >
                    <Typography variant="body2">
                      {index + 1}. {op.name}
                    </Typography>
                    <Chip label={op.count} size="small" />
                  </Box>
                ))}
              </Box>
            ) : (
              <Typography variant="body2" color="text.secondary" sx={{ marginTop: 2 }}>
                No data available
              </Typography>
            )}
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );

  /**
   * Render active filters chips
   */
  const renderActiveFilters = (): React.ReactElement | null => {
    if (activeFilters.length === 0) return null;

    return (
      <Box sx={{ display: 'flex', gap: 1, marginBottom: 2, flexWrap: 'wrap', alignItems: 'center' }}>
        <Typography variant="body2" color="text.secondary">
          Active Filters:
        </Typography>
        {activeFilters.map((filter) => (
          <Chip
            key={filter.key}
            label={`${filter.label}: ${filter.value}`}
            onDelete={() => handleRemoveFilter(filter.key)}
            size="small"
          />
        ))}
        <Button size="small" onClick={handleClearAllFilters}>
          Clear All
        </Button>
      </Box>
    );
  };

  /**
   * Render loading skeleton
   */
  const renderLoadingSkeleton = (): React.ReactElement => (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Timestamp</TableCell>
            <TableCell>Event Type</TableCell>
            <TableCell>Resource Type</TableCell>
            <TableCell>Resource ID</TableCell>
            <TableCell>Operator</TableCell>
            <TableCell>Action</TableCell>
            <TableCell>Result</TableCell>
            <TableCell>Details</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {[1, 2, 3, 4, 5].map((index) => (
            <TableRow key={index}>
              <TableCell><Skeleton variant="text" width={150} /></TableCell>
              <TableCell><Skeleton variant="rounded" width={80} height={24} /></TableCell>
              <TableCell><Skeleton variant="text" width={100} /></TableCell>
              <TableCell><Skeleton variant="text" width={120} /></TableCell>
              <TableCell><Skeleton variant="text" width={80} /></TableCell>
              <TableCell><Skeleton variant="text" width={100} /></TableCell>
              <TableCell><Skeleton variant="rounded" width={80} height={24} /></TableCell>
              <TableCell><Skeleton variant="rectangular" width={40} height={24} /></TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  /**
   * Render log details dialog
   */
  const renderDetailsDialog = (): React.ReactElement => (
    <Dialog open={detailDialogOpen} onClose={handleCloseDialog} maxWidth="md" fullWidth>
      <DialogTitle>
        Audit Log Details
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
        {selectedLog && (
          <Box>
            <Typography variant="subtitle2" gutterBottom>
              Log ID
            </Typography>
            <Typography variant="body2" paragraph>
              {selectedLog.id}
            </Typography>

            <Typography variant="subtitle2" gutterBottom>
              Timestamp
            </Typography>
            <Typography variant="body2" paragraph>
              {formatTimestamp(selectedLog.timestamp)}
            </Typography>

            <Typography variant="subtitle2" gutterBottom>
              Event Type
            </Typography>
            <Box sx={{ marginBottom: 2 }}>
              <Chip
                label={selectedLog.operation_type}
                color={EVENT_TYPE_COLORS[selectedLog.operation_type] || 'default'}
                size="small"
              />
            </Box>

            <Typography variant="subtitle2" gutterBottom>
              Resource
            </Typography>
            <Typography variant="body2" paragraph>
              {selectedLog.resource_type}: {selectedLog.resource_id}
            </Typography>

            <Typography variant="subtitle2" gutterBottom>
              Operator
            </Typography>
            <Typography variant="body2" paragraph>
              {selectedLog.operator_id}
            </Typography>

            <Typography variant="subtitle2" gutterBottom>
              Action
            </Typography>
            <Typography variant="body2" paragraph>
              {selectedLog.action}
            </Typography>

            <Typography variant="subtitle2" gutterBottom>
              Result
            </Typography>
            <Box sx={{ marginBottom: 2 }}>
              <Chip
                label={selectedLog.result}
                color={selectedLog.result === 'SUCCESS' ? 'success' : 'error'}
                size="small"
              />
            </Box>

            {selectedLog.error_message && (
              <>
                <Typography variant="subtitle2" gutterBottom color="error">
                  Error Message
                </Typography>
                <Typography variant="body2" color="error" paragraph>
                  {selectedLog.error_message}
                </Typography>
              </>
            )}

            {selectedLog.details && (
              <>
                <Typography variant="subtitle2" gutterBottom>
                  Details
                  <Tooltip title="Copy JSON">
                    <IconButton size="small" onClick={handleCopyJson} sx={{ marginLeft: 1 }}>
                      <ContentCopyIcon fontSize="small" />
                    </IconButton>
                  </Tooltip>
                </Typography>
                <Paper
                  sx={{
                    padding: 2,
                    backgroundColor: 'grey.100',
                    fontFamily: 'monospace',
                    fontSize: 12,
                  }}
                >
                  <pre style={{ margin: 0, overflow: 'auto' }}>
                    {JSON.stringify(selectedLog.details, null, 2)}
                  </pre>
                </Paper>
              </>
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
          Audit Log
        </Typography>
        <Typography variant="body1" color="text.secondary">
          View system operation logs and audit trails
        </Typography>
        <Typography variant="caption" color="text.secondary">
          Last updated: {format(lastUpdateTime, 'yyyy-MM-dd HH:mm:ss')}
        </Typography>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Statistics Cards */}
      {renderStatisticsCards()}

      {/* Filters Card */}
      <Card sx={{ marginBottom: 3 }}>
        <CardContent>
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 2,
            }}
          >
            <Typography variant="h6">
              Filters
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Tooltip title={autoRefresh ? 'Disable auto-refresh' : 'Enable auto-refresh'}>
                <Button
                  size="small"
                  variant={autoRefresh ? 'contained' : 'outlined'}
                  onClick={() => setAutoRefresh(!autoRefresh)}
                >
                  Auto-refresh {autoRefresh ? 'ON' : 'OFF'}
                </Button>
              </Tooltip>
              <Tooltip title="Refresh now">
                <IconButton color="primary" onClick={handleRefresh} disabled={loading}>
                  <RefreshIcon />
                </IconButton>
              </Tooltip>
              <Tooltip title={showFilters ? 'Hide filters' : 'Show filters'}>
                <IconButton onClick={() => setShowFilters(!showFilters)}>
                  <FilterListIcon />
                </IconButton>
              </Tooltip>
            </Box>
          </Box>

          <Collapse in={showFilters}>
            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', marginBottom: 2 }}>
              {/* Time Range Preset */}
              <FormControl sx={{ minWidth: { xs: '100%', sm: 200 } }}>
                <InputLabel>Time Range</InputLabel>
                <Select value={timeRangePreset} onChange={handleTimeRangeChange} label="Time Range">
                  <MenuItem value={TimeRangePreset.LAST_HOUR}>Last Hour</MenuItem>
                  <MenuItem value={TimeRangePreset.LAST_24_HOURS}>Last 24 Hours</MenuItem>
                  <MenuItem value={TimeRangePreset.LAST_7_DAYS}>Last 7 Days</MenuItem>
                  <MenuItem value={TimeRangePreset.LAST_30_DAYS}>Last 30 Days</MenuItem>
                  <MenuItem value={TimeRangePreset.CUSTOM}>Custom</MenuItem>
                </Select>
              </FormControl>

              {/* Custom Time Range */}
              {timeRangePreset === 'Custom' && (
                <>
                  <TextField
                    label="Start Time"
                    type="datetime-local"
                    value={customStartTime}
                    onChange={(e) => setCustomStartTime(e.target.value)}
                    InputLabelProps={{ shrink: true }}
                    sx={{ minWidth: { xs: '100%', sm: 220 } }}
                  />
                  <TextField
                    label="End Time"
                    type="datetime-local"
                    value={customEndTime}
                    onChange={(e) => setCustomEndTime(e.target.value)}
                    InputLabelProps={{ shrink: true }}
                    sx={{ minWidth: { xs: '100%', sm: 220 } }}
                  />
                </>
              )}

              {/* Event Type Filter */}
              <FormControl sx={{ minWidth: { xs: '100%', sm: 200 } }}>
                <InputLabel>Event Types</InputLabel>
                <Select
                  multiple
                  value={eventTypeFilter}
                  onChange={handleEventTypeChange}
                  label="Event Types"
                  renderValue={(selected) => (selected as string[]).join(', ')}
                >
                  {Object.values(EventType).map((type) => (
                    <MenuItem key={type} value={type}>
                      {type}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>

              {/* Resource Type Filter */}
              <FormControl sx={{ minWidth: { xs: '100%', sm: 180 } }}>
                <InputLabel>Resource Type</InputLabel>
                <Select
                  value={resourceTypeFilter}
                  onChange={handleResourceTypeChange}
                  label="Resource Type"
                >
                  <MenuItem value="all">All Types</MenuItem>
                  {Object.values(ResourceType).map((type) => (
                    <MenuItem key={type} value={type}>
                      {type}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>

              {/* Operator Filter */}
              <TextField
                label="Operator"
                placeholder="Filter by operator..."
                value={operatorFilter}
                onChange={handleOperatorFilterChange}
                sx={{ minWidth: { xs: '100%', sm: 180 } }}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'action.active' }} />,
                }}
              />

              {/* Resource ID Filter */}
              <TextField
                label="Resource ID"
                placeholder="Filter by resource ID..."
                value={resourceIdFilter}
                onChange={handleResourceIdFilterChange}
                sx={{ minWidth: { xs: '100%', sm: 180 } }}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'action.active' }} />,
                }}
              />

              {/* IP Address Filter */}
              <TextField
                label="IP Address"
                placeholder="Filter by IP..."
                value={ipAddressFilter}
                onChange={handleIpAddressFilterChange}
                sx={{ minWidth: { xs: '100%', sm: 180 } }}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'action.active' }} />,
                }}
              />
            </Box>

            {/* Active Filters */}
            {renderActiveFilters()}
          </Collapse>
        </CardContent>
      </Card>

      {/* Logs Table */}
      <Card>
        <CardContent>
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 2,
            }}
          >
            <Typography variant="h6">
              Audit Logs ({filteredLogs.length})
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Button
                variant="outlined"
                size="small"
                startIcon={<DownloadIcon />}
                onClick={handleExportCsv}
                disabled={loading || filteredLogs.length === 0}
              >
                Export CSV
              </Button>
              <Button
                variant="outlined"
                size="small"
                startIcon={<DownloadIcon />}
                onClick={handleExportJson}
                disabled={loading || filteredLogs.length === 0}
              >
                Export JSON
              </Button>
            </Box>
          </Box>

          {loading ? (
            renderLoadingSkeleton()
          ) : filteredLogs.length === 0 ? (
            <Box sx={{ textAlign: 'center', padding: 4 }}>
              <Typography variant="body1" color="text.secondary">
                No audit logs found matching your filters
              </Typography>
            </Box>
          ) : (
            <>
              <TableContainer component={Paper} sx={{ marginTop: 2, overflowX: 'auto' }}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ width: 180 }}>Timestamp</TableCell>
                      <TableCell sx={{ width: 120 }}>Event Type</TableCell>
                      <TableCell sx={{ width: 120 }}>Resource Type</TableCell>
                      <TableCell>Resource ID</TableCell>
                      <TableCell>Operator</TableCell>
                      <TableCell>Action</TableCell>
                      <TableCell sx={{ width: 100 }}>Result</TableCell>
                      <TableCell sx={{ width: 80 }}>Details</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {paginatedLogs.map((log) => (
                      <React.Fragment key={log.id}>
                        <TableRow
                          hover
                          sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                        >
                          <TableCell>{formatTimestamp(log.timestamp)}</TableCell>
                          <TableCell>
                            <Chip
                              label={log.operation_type}
                              color={EVENT_TYPE_COLORS[log.operation_type] || 'default'}
                              size="small"
                            />
                          </TableCell>
                          <TableCell>{log.resource_type}</TableCell>
                          <TableCell>
                            <Typography
                              component="span"
                              sx={clickableTextSx}
                              onClick={() => handleLogClick(log)}
                            >
                              {log.resource_id}
                            </Typography>
                          </TableCell>
                          <TableCell>{log.operator_id}</TableCell>
                          <TableCell>{log.action}</TableCell>
                          <TableCell>
                            <Chip
                              label={log.result}
                              color={log.result === 'SUCCESS' ? 'success' : 'error'}
                              size="small"
                            />
                          </TableCell>
                          <TableCell>
                            <IconButton
                              size="small"
                              onClick={() => handleToggleRow(log.id as string)}
                            >
                              {expandedRows.has(log.id as string) ? (
                                <ExpandLessIcon />
                              ) : (
                                <ExpandMoreIcon />
                              )}
                            </IconButton>
                          </TableCell>
                        </TableRow>
                        {expandedRows.has(log.id as string) && (
                          <TableRow>
                            <TableCell colSpan={8}>
                              <Box sx={{ padding: 2, backgroundColor: 'grey.50' }}>
                                {log.error_message && (
                                  <>
                                    <Typography variant="subtitle2" color="error" gutterBottom>
                                      Error Message:
                                    </Typography>
                                    <Typography variant="body2" color="error" paragraph>
                                      {log.error_message}
                                    </Typography>
                                  </>
                                )}
                                {log.details && (
                                  <>
                                    <Typography variant="subtitle2" gutterBottom>
                                      Details:
                                    </Typography>
                                    <Paper
                                      sx={{
                                        padding: 1,
                                        backgroundColor: 'white',
                                        fontFamily: 'monospace',
                                        fontSize: 11,
                                      }}
                                    >
                                      <pre style={{ margin: 0, overflow: 'auto' }}>
                                        {JSON.stringify(log.details, null, 2)}
                                      </pre>
                                    </Paper>
                                  </>
                                )}
                              </Box>
                            </TableCell>
                          </TableRow>
                        )}
                      </React.Fragment>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>

              {/* Pagination */}
              <TablePagination
                rowsPerPageOptions={[10, 25, 50, 100]}
                component="div"
                count={filteredLogs.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onPageChange={handlePageChange}
                onRowsPerPageChange={handleRowsPerPageChange}
              />
            </>
          )}
        </CardContent>
      </Card>

      {/* Log Details Dialog */}
      {renderDetailsDialog()}
    </Box>
  );
};

AuditLog.displayName = 'AuditLog';

export default AuditLog;
