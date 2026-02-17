/**
 * Routing Page Component
 *
 * Features:
 * - Dual-tab layout (Groups and Route Rules)
 * - Groups management: create, edit, delete, view instances
 * - Route Rules management: create, edit, delete, enable/disable
 * - Weight configuration with real-time validation
 * - Search and filtering capabilities
 * - Pagination and export functionality
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
  Tabs,
  Tab,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Switch,
  Grid,
  LinearProgress,
  Autocomplete,
  Divider,
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
  CheckCircle as CheckCircleIcon,
  Group as GroupIcon,
  Router as RouterIcon,
  Close as CloseIcon,
} from '@mui/icons-material';
import {
  createGroup,
  listGroups,
  updateGroup,
  deleteGroup,
  createRule,
  listRules,
  updateRule,
  deleteRule,
  addRuleGroup,
  listRuleGroups,
  updateRuleGroup,
  removeRuleGroup,
} from '@/api/routing';
import { getAllServices } from '@/api/discovery';
import type { Service } from '@/api/types';

// ===== Type Definitions =====

/**
 * Group display data
 */
interface GroupDisplay {
  group_id: string;
  name: string;
  description?: string;
  service_id: string;
  region_id: string;
  zone_id: string;
  status: 'ACTIVE' | 'INACTIVE';
  instance_count: number;
  created_at?: number;
}

/**
 * Route Rule display data
 */
interface RouteRuleDisplay {
  route_id: string;
  name: string;
  description?: string;
  service_id: string;
  strategy: 'WEIGHT_ROUND_ROBIN' | 'CONSISTENT_HASH';
  status: 'ACTIVE' | 'INACTIVE';
  groups: GroupWeightDisplay[];
  created_at?: number;
}

/**
 * Group weight display
 */
interface GroupWeightDisplay {
  group_id: string;
  group_name: string;
  weight: number;
}

/**
 * Statistics data
 */
interface Statistics {
  totalGroups: number;
  totalRules: number;
  activeRules: number;
}

/**
 * Group form data
 */
interface GroupFormData {
  group_id: string;
  name: string;
  description: string;
  service_id: string;
  region_id: string;
  zone_id: string;
}

/**
 * Route Rule form data
 */
interface RouteRuleFormData {
  route_id: string;
  name: string;
  description: string;
  service_id: string;
  strategy: 'WEIGHT_ROUND_ROBIN' | 'CONSISTENT_HASH';
  selectedGroups: string[];
  weights: Record<string, number>;
  zonePreference?: string;
}

// ===== Main Component =====

/**
 * Routing component
 *
 * @returns React component
 */
const Routing: React.FC = () => {
  // ===== State Management =====

  // Tab state
  const [currentTab, setCurrentTab] = useState<number>(0);

  // Data state
  const [groups, setGroups] = useState<GroupDisplay[]>([]);
  const [rules, setRules] = useState<RouteRuleDisplay[]>([]);
  const [services, setServices] = useState<Service[]>([]);
  const [statistics, setStatistics] = useState<Statistics>({
    totalGroups: 0,
    totalRules: 0,
    activeRules: 0,
  });

  // Loading and error state
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<boolean>(false);

  // Filter state
  const [groupSearchQuery, setGroupSearchQuery] = useState<string>('');
  const [ruleSearchQuery, setRuleSearchQuery] = useState<string>('');
  const [ruleStatusFilter, setRuleStatusFilter] = useState<string>('all');

  // Pagination state
  const [groupPage, setGroupPage] = useState<number>(0);
  const [groupRowsPerPage, setGroupRowsPerPage] = useState<number>(10);
  const [rulePage, setRulePage] = useState<number>(0);
  const [ruleRowsPerPage, setRuleRowsPerPage] = useState<number>(10);

  // Dialog state
  const [groupDialogOpen, setGroupDialogOpen] = useState<boolean>(false);
  const [ruleDialogOpen, setRuleDialogOpen] = useState<boolean>(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);
  const [editMode, setEditMode] = useState<boolean>(false);

  // Form state
  const [groupFormData, setGroupFormData] = useState<GroupFormData>({
    group_id: '',
    name: '',
    description: '',
    service_id: '',
    region_id: 'default-region',
    zone_id: 'default-zone',
  });

  const [ruleFormData, setRuleFormData] = useState<RouteRuleFormData>({
    route_id: '',
    name: '',
    description: '',
    service_id: '',
    strategy: 'WEIGHT_ROUND_ROBIN',
    selectedGroups: [],
    weights: {},
    zonePreference: '',
  });

  // Delete confirmation state
  const [deleteTarget, setDeleteTarget] = useState<{
    type: 'group' | 'rule';
    id: string;
    name: string;
  } | null>(null);

  // ===== Data Fetching =====

  /**
   * Fetch all data
   */
  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const [groupsResponse, rulesResponse, servicesResponse] = await Promise.all([
        listGroups(),
        listRules(),
        getAllServices('default-region', 'default-zone'),
      ]);

      // Process groups
      const groupsData: GroupDisplay[] = groupsResponse.data?.map((g: any) => ({
        group_id: g.group_id || g.id,
        name: g.name,
        description: g.description,
        service_id: g.service_id,
        region_id: g.region_id,
        zone_id: g.zone_id,
        status: g.status?.status || 'ACTIVE',
        instance_count: g.instance_count || 0,
        created_at: g.created_at,
      })) || [];

      // Process rules
      const rulesData: RouteRuleDisplay[] = [];
      if (rulesResponse.data) {
        for (const r of rulesResponse.data) {
          const ruleGroupsResponse = await listRuleGroups(r.route_id);
          const ruleGroups: GroupWeightDisplay[] = ruleGroupsResponse.data?.map((rg: any) => ({
            group_id: rg.group_id,
            group_name: groupsData.find(g => g.group_id === rg.group_id)?.name || rg.group_id,
            weight: rg.weight,
          })) || [];

          rulesData.push({
            route_id: r.route_id,
            name: r.name,
            description: r.description,
            service_id: r.service_id,
            strategy: r.strategy?.strategy || 'WEIGHT_ROUND_ROBIN',
            status: r.status?.status || 'ACTIVE',
            groups: ruleGroups,
            created_at: r.created_at,
          });
        }
      }

      setGroups(groupsData);
      setRules(rulesData);
      setServices(servicesResponse.services || []);

      // Calculate statistics
      setStatistics({
        totalGroups: groupsData.length,
        totalRules: rulesData.length,
        activeRules: rulesData.filter(r => r.status === 'ACTIVE').length,
      });
    } catch (err: any) {
      console.error('Failed to fetch routing data:', err);
      setError(err.message || 'Failed to load routing data');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // ===== Event Handlers =====

  /**
   * Handle tab change
   */
  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setCurrentTab(newValue);
  };

  /**
   * Open group dialog for create/edit
   */
  const openGroupDialog = (group?: GroupDisplay) => {
    if (group) {
      setEditMode(true);
      setGroupFormData({
        group_id: group.group_id,
        name: group.name,
        description: group.description || '',
        service_id: group.service_id,
        region_id: group.region_id,
        zone_id: group.zone_id,
      });
    } else {
      setEditMode(false);
      setGroupFormData({
        group_id: '',
        name: '',
        description: '',
        service_id: '',
        region_id: 'default-region',
        zone_id: 'default-zone',
      });
    }
    setGroupDialogOpen(true);
  };

  /**
   * Open rule dialog for create/edit
   */
  const openRuleDialog = (rule?: RouteRuleDisplay) => {
    if (rule) {
      setEditMode(true);
      const weights: Record<string, number> = {};
      rule.groups.forEach(g => {
        weights[g.group_id] = g.weight;
      });
      setRuleFormData({
        route_id: rule.route_id,
        name: rule.name,
        description: rule.description || '',
        service_id: rule.service_id,
        strategy: rule.strategy,
        selectedGroups: rule.groups.map(g => g.group_id),
        weights,
        zonePreference: '',
      });
    } else {
      setEditMode(false);
      setRuleFormData({
        route_id: '',
        name: '',
        description: '',
        service_id: '',
        strategy: 'WEIGHT_ROUND_ROBIN',
        selectedGroups: [],
        weights: {},
        zonePreference: '',
      });
    }
    setRuleDialogOpen(true);
  };

  /**
   * Handle group form submission
   */
  const handleGroupSubmit = async () => {
    try {
      setActionLoading(true);
      setError(null);

      if (editMode) {
        await updateGroup(groupFormData.group_id, {
          description: groupFormData.description,
        });
      } else {
        await createGroup({
          service_id: groupFormData.service_id,
          region_id: groupFormData.region_id,
          zone_id: groupFormData.zone_id,
          name: groupFormData.name,
          group_type: { type: 'WEIGHT' },
          description: groupFormData.description,
        });
      }

      setGroupDialogOpen(false);
      await fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to save group');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle rule form submission
   */
  const handleRuleSubmit = async () => {
    try {
      setActionLoading(true);
      setError(null);

      // Validate weights for weighted round robin
      if (ruleFormData.strategy === 'WEIGHT_ROUND_ROBIN') {
        const totalWeight = Object.values(ruleFormData.weights).reduce((sum, w) => sum + w, 0);
        if (Math.abs(totalWeight - 100) > 0.01) {
          setError('Total weight must equal 100%');
          return;
        }
      }

      if (editMode) {
        // Update rule metadata
        await updateRule(ruleFormData.route_id, {
          name: ruleFormData.name,
          description: ruleFormData.description,
          strategy: { strategy: ruleFormData.strategy },
        });

        // Update rule groups and weights
        const existingRule = rules.find(r => r.route_id === ruleFormData.route_id);
        if (existingRule) {
          // Remove old groups not in new selection
          for (const g of existingRule.groups) {
            if (!ruleFormData.selectedGroups.includes(g.group_id)) {
              await removeRuleGroup(ruleFormData.route_id, g.group_id);
            }
          }

          // Add or update groups
          for (const groupId of ruleFormData.selectedGroups) {
            const weight = ruleFormData.weights[groupId] || 0;
            const existingGroup = existingRule.groups.find(g => g.group_id === groupId);
            if (existingGroup) {
              await updateRuleGroup(ruleFormData.route_id, groupId, { weight });
            } else {
              await addRuleGroup(ruleFormData.route_id, {
                group_id: groupId,
                weight,
              });
            }
          }
        }
      } else {
        // Create new rule
        await createRule({
          route_id: ruleFormData.route_id,
          service_id: ruleFormData.service_id,
          name: ruleFormData.name,
          description: ruleFormData.description,
          strategy: { strategy: ruleFormData.strategy },
        });

        // Add groups to rule
        for (const groupId of ruleFormData.selectedGroups) {
          await addRuleGroup(ruleFormData.route_id, {
            group_id: groupId,
            weight: ruleFormData.weights[groupId] || 0,
          });
        }
      }

      setRuleDialogOpen(false);
      await fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to save route rule');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Handle delete confirmation
   */
  const confirmDelete = (type: 'group' | 'rule', id: string, name: string) => {
    setDeleteTarget({ type, id, name });
    setDeleteDialogOpen(true);
  };

  /**
   * Handle delete action
   */
  const handleDelete = async () => {
    if (!deleteTarget) return;

    try {
      setActionLoading(true);
      setError(null);

      if (deleteTarget.type === 'group') {
        await deleteGroup(deleteTarget.id);
      } else {
        await deleteRule(deleteTarget.id);
      }

      setDeleteDialogOpen(false);
      setDeleteTarget(null);
      await fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to delete item');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Toggle rule status
   */
  const toggleRuleStatus = async (ruleId: string, _currentStatus: 'ACTIVE' | 'INACTIVE') => {
    try {
      setActionLoading(true);
      // Note: API doesn't have explicit enable/disable endpoints, using update
      await updateRule(ruleId, {
        // Placeholder - actual implementation may differ
      });
      await fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to toggle rule status');
    } finally {
      setActionLoading(false);
    }
  };

  /**
   * Auto-distribute weights evenly
   */
  const autoDistributeWeights = () => {
    const groupCount = ruleFormData.selectedGroups.length;
    if (groupCount === 0) return;

    const baseWeight = Math.floor(100 / groupCount);
    const remainder = 100 - baseWeight * groupCount;

    const newWeights: Record<string, number> = {};
    ruleFormData.selectedGroups.forEach((groupId, index) => {
      newWeights[groupId] = baseWeight + (index < remainder ? 1 : 0);
    });

    setRuleFormData(prev => ({ ...prev, weights: newWeights }));
  };

  /**
   * Export to CSV
   */
  const exportToCSV = (type: 'groups' | 'rules') => {
    let csvContent = '';
    let filename = '';

    if (type === 'groups') {
      csvContent = 'Group ID,Name,Service ID,Status,Instance Count,Description\n';
      filteredGroups.forEach(g => {
        csvContent += `"${g.group_id}","${g.name}","${g.service_id}","${g.status}",${g.instance_count},"${g.description || ''}"\n`;
      });
      filename = 'groups.csv';
    } else {
      csvContent = 'Rule ID,Name,Service ID,Strategy,Status,Groups,Description\n';
      filteredRules.forEach(r => {
        const groupsStr = r.groups.map(g => `${g.group_name}:${g.weight}%`).join('; ');
        csvContent += `"${r.route_id}","${r.name}","${r.service_id}","${r.strategy}","${r.status}","${groupsStr}","${r.description || ''}"\n`;
      });
      filename = 'route-rules.csv';
    }

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = filename;
    link.click();
  };

  // ===== Filtering and Pagination =====

  /**
   * Filtered groups
   */
  const filteredGroups = useMemo(() => {
    return groups.filter(g => {
      const matchesSearch = g.group_id.toLowerCase().includes(groupSearchQuery.toLowerCase()) ||
        g.name.toLowerCase().includes(groupSearchQuery.toLowerCase());
      return matchesSearch;
    });
  }, [groups, groupSearchQuery]);

  /**
   * Filtered rules
   */
  const filteredRules = useMemo(() => {
    return rules.filter(r => {
      const matchesSearch = r.route_id.toLowerCase().includes(ruleSearchQuery.toLowerCase()) ||
        r.name.toLowerCase().includes(ruleSearchQuery.toLowerCase());
      const matchesStatus = ruleStatusFilter === 'all' || r.status === ruleStatusFilter.toUpperCase();
      return matchesSearch && matchesStatus;
    });
  }, [rules, ruleSearchQuery, ruleStatusFilter]);

  /**
   * Paginated groups
   */
  const paginatedGroups = useMemo(() => {
    const start = groupPage * groupRowsPerPage;
    return filteredGroups.slice(start, start + groupRowsPerPage);
  }, [filteredGroups, groupPage, groupRowsPerPage]);

  /**
   * Paginated rules
   */
  const paginatedRules = useMemo(() => {
    const start = rulePage * ruleRowsPerPage;
    return filteredRules.slice(start, start + ruleRowsPerPage);
  }, [filteredRules, rulePage, ruleRowsPerPage]);

  /**
   * Available groups for rule form (filtered by service)
   */
  const availableGroups = useMemo(() => {
    if (!ruleFormData.service_id) return [];
    return groups.filter(g => g.service_id === ruleFormData.service_id);
  }, [groups, ruleFormData.service_id]);

  /**
   * Calculate total weight
   */
  const totalWeight = useMemo(() => {
    return Object.values(ruleFormData.weights).reduce((sum, w) => sum + w, 0);
  }, [ruleFormData.weights]);

  /**
   * Weight validation status
   */
  const weightValid = Math.abs(totalWeight - 100) < 0.01;

  // ===== Styles =====

  const statCardSx: SxProps<Theme> = {
    transition: 'transform 0.2s, box-shadow 0.2s',
    '&:hover': {
      transform: 'translateY(-4px)',
      boxShadow: 4,
    },
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

  // ===== Render =====

  return (
    <Box>
      {/* Page Header */}
      <Box sx={{ marginBottom: 3, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Box>
          <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
            Routing
          </Typography>
          <Typography variant="body1" color="text.secondary">
            Configure routing rules and service groups
          </Typography>
        </Box>
        <Tooltip title="Refresh">
          <span>
            <IconButton onClick={fetchData} disabled={loading}>
              <RefreshIcon />
            </IconButton>
          </span>
        </Tooltip>
      </Box>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ marginBottom: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Statistics Cards */}
      <Grid container spacing={3} sx={{ marginBottom: 3 }}>
        <Grid size={{ xs: 12, sm: 4 }}>
          <Card sx={statCardSx}>
            <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Box sx={iconBoxSx('#1976d2')}>
                <GroupIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box>
                <Typography variant="body2" color="text.secondary">
                  Total Groups
                </Typography>
                <Typography variant="h4" fontWeight={700}>
                  {statistics.totalGroups}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, sm: 4 }}>
          <Card sx={statCardSx}>
            <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Box sx={iconBoxSx('#2e7d32')}>
                <RouterIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box>
                <Typography variant="body2" color="text.secondary">
                  Total Route Rules
                </Typography>
                <Typography variant="h4" fontWeight={700}>
                  {statistics.totalRules}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, sm: 4 }}>
          <Card sx={statCardSx}>
            <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Box sx={iconBoxSx('#ed6c02')}>
                <CheckCircleIcon sx={{ fontSize: 40 }} />
              </Box>
              <Box>
                <Typography variant="body2" color="text.secondary">
                  Active Rules
                </Typography>
                <Typography variant="h4" fontWeight={700}>
                  {statistics.activeRules}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Tabs */}
      <Card>
        <Tabs value={currentTab} onChange={handleTabChange} sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tab label="Groups" />
          <Tab label="Route Rules" />
        </Tabs>

        {/* Tab Panel 1: Groups */}
        {currentTab === 0 && (
          <CardContent>
            {/* Toolbar */}
            <Box sx={{ display: 'flex', gap: 2, marginBottom: 2, flexWrap: 'wrap' }}>
              <TextField
                placeholder="Search by Group ID or Name..."
                size="small"
                value={groupSearchQuery}
                onChange={(e) => setGroupSearchQuery(e.target.value)}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'text.secondary' }} />,
                }}
                sx={{ flexGrow: 1, minWidth: '200px' }}
              />
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={() => openGroupDialog()}
              >
                Add Group
              </Button>
              <Button
                variant="outlined"
                startIcon={<DownloadIcon />}
                onClick={() => exportToCSV('groups')}
              >
                Export CSV
              </Button>
            </Box>

            {/* Groups Table */}
            <TableContainer component={Paper} variant="outlined">
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Group ID</TableCell>
                    <TableCell>Name</TableCell>
                    <TableCell>Description</TableCell>
                    <TableCell>Service ID</TableCell>
                    <TableCell align="center">Instance Count</TableCell>
                    <TableCell>Created Time</TableCell>
                    <TableCell align="center">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {loading ? (
                    <TableRow>
                      <TableCell colSpan={7} align="center">
                        <Typography variant="body2" color="text.secondary">
                          Loading...
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ) : paginatedGroups.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={7} align="center">
                        <Typography variant="body2" color="text.secondary">
                          No groups found
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ) : (
                    paginatedGroups.map((group) => (
                      <TableRow key={group.group_id}>
                        <TableCell>{group.group_id}</TableCell>
                        <TableCell>{group.name}</TableCell>
                        <TableCell>{group.description || '-'}</TableCell>
                        <TableCell>{group.service_id}</TableCell>
                        <TableCell align="center">{group.instance_count}</TableCell>
                        <TableCell>
                          {group.created_at ? new Date(group.created_at * 1000).toLocaleString() : '-'}
                        </TableCell>
                        <TableCell align="center">
                          <Tooltip title="Edit">
                            <IconButton size="small" onClick={() => openGroupDialog(group)}>
                              <EditIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          <Tooltip title="Delete">
                            <IconButton
                              size="small"
                              color="error"
                              onClick={() => confirmDelete('group', group.group_id, group.name)}
                            >
                              <DeleteIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </TableContainer>

            {/* Pagination */}
            <TablePagination
              component="div"
              count={filteredGroups.length}
              page={groupPage}
              onPageChange={(_, newPage) => setGroupPage(newPage)}
              rowsPerPage={groupRowsPerPage}
              onRowsPerPageChange={(e) => {
                setGroupRowsPerPage(parseInt(e.target.value, 10));
                setGroupPage(0);
              }}
              rowsPerPageOptions={[10, 25, 50]}
            />
          </CardContent>
        )}

        {/* Tab Panel 2: Route Rules */}
        {currentTab === 1 && (
          <CardContent>
            {/* Toolbar */}
            <Box sx={{ display: 'flex', gap: 2, marginBottom: 2, flexWrap: 'wrap' }}>
              <TextField
                placeholder="Search by Rule ID or Name..."
                size="small"
                value={ruleSearchQuery}
                onChange={(e) => setRuleSearchQuery(e.target.value)}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'text.secondary' }} />,
                }}
                sx={{ flexGrow: 1, minWidth: '200px' }}
              />
              <FormControl size="small" sx={{ minWidth: 120 }}>
                <InputLabel>Status</InputLabel>
                <Select
                  value={ruleStatusFilter}
                  label="Status"
                  onChange={(e: SelectChangeEvent) => setRuleStatusFilter(e.target.value)}
                >
                  <MenuItem value="all">All</MenuItem>
                  <MenuItem value="active">Active</MenuItem>
                  <MenuItem value="inactive">Inactive</MenuItem>
                </Select>
              </FormControl>
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={() => openRuleDialog()}
              >
                Add Rule
              </Button>
              <Button
                variant="outlined"
                startIcon={<DownloadIcon />}
                onClick={() => exportToCSV('rules')}
              >
                Export CSV
              </Button>
            </Box>

            {/* Rules Table */}
            <TableContainer component={Paper} variant="outlined">
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Rule ID</TableCell>
                    <TableCell>Name</TableCell>
                    <TableCell>Strategy</TableCell>
                    <TableCell>Weights</TableCell>
                    <TableCell>Target Groups</TableCell>
                    <TableCell align="center">Status</TableCell>
                    <TableCell align="center">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {loading ? (
                    <TableRow>
                      <TableCell colSpan={7} align="center">
                        <Typography variant="body2" color="text.secondary">
                          Loading...
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ) : paginatedRules.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={7} align="center">
                        <Typography variant="body2" color="text.secondary">
                          No route rules found
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ) : (
                    paginatedRules.map((rule) => (
                      <TableRow key={rule.route_id}>
                        <TableCell>{rule.route_id}</TableCell>
                        <TableCell>{rule.name}</TableCell>
                        <TableCell>
                          {rule.strategy === 'WEIGHT_ROUND_ROBIN' ? 'Weighted Round Robin' : 'Consistent Hash'}
                        </TableCell>
                        <TableCell>
                          {rule.groups.map(g => `${g.weight}%`).join(', ')}
                        </TableCell>
                        <TableCell>
                          {rule.groups.map(g => g.group_name).join(', ')}
                        </TableCell>
                        <TableCell align="center">
                          <Switch
                            checked={rule.status === 'ACTIVE'}
                            onChange={() => toggleRuleStatus(rule.route_id, rule.status)}
                            size="small"
                            disabled={actionLoading}
                          />
                          <Chip
                            label={rule.status}
                            color={rule.status === 'ACTIVE' ? 'success' : 'default'}
                            size="small"
                            sx={{ marginLeft: 1 }}
                          />
                        </TableCell>
                        <TableCell align="center">
                          <Tooltip title="Edit">
                            <IconButton size="small" onClick={() => openRuleDialog(rule)}>
                              <EditIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          <Tooltip title="Delete">
                            <IconButton
                              size="small"
                              color="error"
                              onClick={() => confirmDelete('rule', rule.route_id, rule.name)}
                            >
                              <DeleteIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </TableContainer>

            {/* Pagination */}
            <TablePagination
              component="div"
              count={filteredRules.length}
              page={rulePage}
              onPageChange={(_, newPage) => setRulePage(newPage)}
              rowsPerPage={ruleRowsPerPage}
              onRowsPerPageChange={(e) => {
                setRuleRowsPerPage(parseInt(e.target.value, 10));
                setRulePage(0);
              }}
              rowsPerPageOptions={[10, 25, 50]}
            />
          </CardContent>
        )}
      </Card>

      {/* Group Dialog */}
      <Dialog
        open={groupDialogOpen}
        onClose={() => setGroupDialogOpen(false)}
        maxWidth="sm"
        fullWidth
        fullScreen={false}
      >
        <DialogTitle>
          {editMode ? 'Edit Group' : 'Add Group'}
          <IconButton
            sx={{ position: 'absolute', right: 8, top: 8 }}
            onClick={() => setGroupDialogOpen(false)}
          >
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, paddingTop: 1 }}>
            <TextField
              label="Group ID"
              value={groupFormData.group_id}
              onChange={(e) => setGroupFormData(prev => ({ ...prev, group_id: e.target.value }))}
              disabled={editMode}
              required
              fullWidth
            />
            <TextField
              label="Group Name"
              value={groupFormData.name}
              onChange={(e) => setGroupFormData(prev => ({ ...prev, name: e.target.value }))}
              required
              fullWidth
            />
            <TextField
              label="Description"
              value={groupFormData.description}
              onChange={(e) => setGroupFormData(prev => ({ ...prev, description: e.target.value }))}
              multiline
              rows={3}
              fullWidth
            />
            <Autocomplete
              options={services}
              getOptionLabel={(option) => option.service_id}
              value={services.find(s => s.service_id === groupFormData.service_id) || null}
              onChange={(_, newValue) => {
                setGroupFormData(prev => ({
                  ...prev,
                  service_id: newValue?.service_id || '',
                }));
              }}
              disabled={editMode}
              renderInput={(params) => (
                <TextField {...params} label="Service ID" required />
              )}
              fullWidth
            />
            <TextField
              label="Region ID"
              value={groupFormData.region_id}
              onChange={(e) => setGroupFormData(prev => ({ ...prev, region_id: e.target.value }))}
              disabled={editMode}
              required
              fullWidth
            />
            <TextField
              label="Zone ID"
              value={groupFormData.zone_id}
              onChange={(e) => setGroupFormData(prev => ({ ...prev, zone_id: e.target.value }))}
              disabled={editMode}
              required
              fullWidth
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setGroupDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleGroupSubmit}
            variant="contained"
            disabled={actionLoading || !groupFormData.group_id || !groupFormData.name || !groupFormData.service_id}
          >
            {actionLoading ? 'Saving...' : editMode ? 'Update' : 'Create'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Route Rule Dialog */}
      <Dialog
        open={ruleDialogOpen}
        onClose={() => setRuleDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          {editMode ? 'Edit Route Rule' : 'Add Route Rule'}
          <IconButton
            sx={{ position: 'absolute', right: 8, top: 8 }}
            onClick={() => setRuleDialogOpen(false)}
          >
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, paddingTop: 1 }}>
            <TextField
              label="Rule ID"
              value={ruleFormData.route_id}
              onChange={(e) => setRuleFormData(prev => ({ ...prev, route_id: e.target.value }))}
              disabled={editMode}
              required
              fullWidth
            />
            <TextField
              label="Rule Name"
              value={ruleFormData.name}
              onChange={(e) => setRuleFormData(prev => ({ ...prev, name: e.target.value }))}
              required
              fullWidth
            />
            <TextField
              label="Description"
              value={ruleFormData.description}
              onChange={(e) => setRuleFormData(prev => ({ ...prev, description: e.target.value }))}
              multiline
              rows={2}
              fullWidth
            />
            <Autocomplete
              options={services}
              getOptionLabel={(option) => option.service_id}
              value={services.find(s => s.service_id === ruleFormData.service_id) || null}
              onChange={(_, newValue) => {
                setRuleFormData(prev => ({
                  ...prev,
                  service_id: newValue?.service_id || '',
                  selectedGroups: [],
                  weights: {},
                }));
              }}
              disabled={editMode}
              renderInput={(params) => (
                <TextField {...params} label="Service ID" required />
              )}
              fullWidth
            />
            <FormControl fullWidth>
              <InputLabel>Strategy</InputLabel>
              <Select
                value={ruleFormData.strategy}
                label="Strategy"
                onChange={(e: SelectChangeEvent) => {
                  setRuleFormData(prev => ({
                    ...prev,
                    strategy: e.target.value as 'WEIGHT_ROUND_ROBIN' | 'CONSISTENT_HASH',
                  }));
                }}
              >
                <MenuItem value="WEIGHT_ROUND_ROBIN">Weighted Round Robin</MenuItem>
                <MenuItem value="CONSISTENT_HASH">Consistent Hash</MenuItem>
              </Select>
            </FormControl>

            <Autocomplete
              multiple
              options={availableGroups}
              getOptionLabel={(option) => `${option.name} (${option.group_id})`}
              value={availableGroups.filter(g => ruleFormData.selectedGroups.includes(g.group_id))}
              onChange={(_, newValue) => {
                const selectedIds = newValue.map(g => g.group_id);
                const newWeights = { ...ruleFormData.weights };
                // Remove weights for unselected groups
                Object.keys(newWeights).forEach(id => {
                  if (!selectedIds.includes(id)) {
                    delete newWeights[id];
                  }
                });
                // Initialize weights for new groups
                selectedIds.forEach(id => {
                  if (!(id in newWeights)) {
                    newWeights[id] = 0;
                  }
                });
                setRuleFormData(prev => ({
                  ...prev,
                  selectedGroups: selectedIds,
                  weights: newWeights,
                }));
              }}
              renderInput={(params) => (
                <TextField {...params} label="Target Groups" required />
              )}
              fullWidth
            />

            {/* Weight Configuration (only for Weighted Round Robin) */}
            {ruleFormData.strategy === 'WEIGHT_ROUND_ROBIN' && ruleFormData.selectedGroups.length > 0 && (
              <Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 1 }}>
                  <Typography variant="subtitle2">Weight Configuration</Typography>
                  <Button size="small" onClick={autoDistributeWeights}>
                    Auto Distribute
                  </Button>
                </Box>
                <Divider sx={{ marginBottom: 2 }} />
                {ruleFormData.selectedGroups.map(groupId => {
                  const group = availableGroups.find(g => g.group_id === groupId);
                  return (
                    <TextField
                      key={groupId}
                      label={`${group?.name || groupId} Weight (%)`}
                      type="number"
                      value={ruleFormData.weights[groupId] || 0}
                      onChange={(e) => {
                        const value = parseFloat(e.target.value) || 0;
                        setRuleFormData(prev => ({
                          ...prev,
                          weights: {
                            ...prev.weights,
                            [groupId]: Math.max(0, Math.min(100, value)),
                          },
                        }));
                      }}
                      fullWidth
                      sx={{ marginBottom: 2 }}
                      inputProps={{ min: 0, max: 100, step: 0.1 }}
                    />
                  );
                })}
                <Box sx={{ marginTop: 1 }}>
                  <Typography variant="body2" gutterBottom>
                    Total Weight: {totalWeight.toFixed(1)}%
                  </Typography>
                  <LinearProgress
                    variant="determinate"
                    value={Math.min(totalWeight, 100)}
                    color={weightValid ? 'success' : 'error'}
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                  {!weightValid && (
                    <Typography variant="caption" color="error" sx={{ marginTop: 0.5, display: 'block' }}>
                      Total weight must equal 100%
                    </Typography>
                  )}
                </Box>
              </Box>
            )}
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRuleDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleRuleSubmit}
            variant="contained"
            disabled={
              actionLoading ||
              !ruleFormData.route_id ||
              !ruleFormData.name ||
              !ruleFormData.service_id ||
              ruleFormData.selectedGroups.length === 0 ||
              (ruleFormData.strategy === 'WEIGHT_ROUND_ROBIN' && !weightValid)
            }
          >
            {actionLoading ? 'Saving...' : editMode ? 'Update' : 'Create'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>Confirm Delete</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete {deleteTarget?.type} <strong>{deleteTarget?.name}</strong>?
          </Typography>
          {deleteTarget?.type === 'group' && (
            <Alert severity="warning" sx={{ marginTop: 2 }}>
              Deleting this group may affect associated route rules.
            </Alert>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={handleDelete}
            variant="contained"
            color="error"
            disabled={actionLoading}
          >
            {actionLoading ? 'Deleting...' : 'Delete'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Routing.displayName = 'Routing';

export default Routing;
