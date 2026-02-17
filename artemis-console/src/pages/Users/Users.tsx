/**
 * Users Page Component
 *
 * Complete user management interface with:
 * - User CRUD operations
 * - Role and permission management
 * - Password reset functionality
 * - Status toggle (Active/Inactive)
 * - Login history tracking
 * - Audit log integration
 * - Permission matrix view
 * - Statistics dashboard
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import type { ChangeEvent } from 'react';
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
  Button,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Chip,
  IconButton,
  Alert,
  AlertTitle,
  Switch,
  Tabs,
  Tab,
  Tooltip,
  Skeleton,
  Checkbox,
  Grid,
  type SelectChangeEvent,
  type SxProps,
  type Theme,
} from '@mui/material';
import {
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Refresh as RefreshIcon,
  Search as SearchIcon,
  VpnKey as VpnKeyIcon,
  Download as DownloadIcon,
  Close as CloseIcon,
  Info as InfoIcon,
  AdminPanelSettings as AdminIcon,
  Engineering as OperatorIcon,
  Visibility as ViewerIcon,
} from '@mui/icons-material';
import { formatDistanceToNow } from 'date-fns';
import {
  getAllUsers,
  createUser,
  updateUser,
  deleteUser,
  resetUserPassword,
  changeUserStatus,
  getUserLoginHistory,
  UserRole,
  UserStatus,
  type UserDetails,
  type CreateUserRequest,
  type UpdateUserRequest,
  type LoginHistory,
} from '@/api/auth';
import { queryLogs, type AuditLog } from '@/api/audit';

/**
 * Tab panels
 */
interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel({ children, value, index }: TabPanelProps): React.ReactElement {
  return (
    <div role="tabpanel" hidden={value !== index}>
      {value === index && <Box sx={{ paddingTop: 3 }}>{children}</Box>}
    </div>
  );
}

/**
 * User form data
 */
interface UserFormData {
  username: string;
  email: string;
  password: string;
  confirmPassword: string;
  role: UserRole;
  description: string;
}

/**
 * Validation errors
 */
interface ValidationErrors {
  username?: string;
  email?: string;
  password?: string;
  confirmPassword?: string;
}

/**
 * Permission matrix data
 */
interface PermissionMatrixRow {
  feature: string;
  admin: { read: boolean; create: boolean; update: boolean; delete: boolean };
  operator: { read: boolean; create: boolean; update: boolean; delete: boolean };
  viewer: { read: boolean; create: boolean; update: boolean; delete: boolean };
}

/**
 * Statistics data
 */
interface Statistics {
  totalUsers: number;
  activeUsers: number;
  adminUsers: number;
  onlineNow: number;
}

/**
 * Role colors
 */
const ROLE_COLORS: Record<UserRole, 'primary' | 'success' | 'info' | 'warning' | 'error'> = {
  [UserRole.ADMIN]: 'error',
  [UserRole.OPERATOR]: 'primary',
  [UserRole.VIEWER]: 'info',
};

/**
 * Role icons
 */
const ROLE_ICONS: Record<UserRole, React.ReactElement> = {
  [UserRole.ADMIN]: <AdminIcon fontSize="small" />,
  [UserRole.OPERATOR]: <OperatorIcon fontSize="small" />,
  [UserRole.VIEWER]: <ViewerIcon fontSize="small" />,
};

/**
 * Role descriptions
 */
const ROLE_DESCRIPTIONS: Record<UserRole, string> = {
  [UserRole.ADMIN]: 'Full access to all features including user management',
  [UserRole.OPERATOR]: 'Can manage services, instances, routing, but cannot manage users',
  [UserRole.VIEWER]: 'Read-only access to all data',
};

/**
 * Permission matrix data
 */
const PERMISSION_MATRIX: PermissionMatrixRow[] = [
  {
    feature: 'Services',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: true, update: true, delete: true },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Instances',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: true, update: true, delete: true },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Cluster',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: false, update: false, delete: false },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Routing',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: true, update: true, delete: true },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Audit',
    admin: { read: true, create: false, update: false, delete: true },
    operator: { read: true, create: false, update: false, delete: false },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Zone',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: true, update: true, delete: true },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Canary',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: true, create: true, update: true, delete: true },
    viewer: { read: true, create: false, update: false, delete: false },
  },
  {
    feature: 'Users',
    admin: { read: true, create: true, update: true, delete: true },
    operator: { read: false, create: false, update: false, delete: false },
    viewer: { read: false, create: false, update: false, delete: false },
  },
];

/**
 * Validate email format
 */
const isValidEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

/**
 * Validate password strength
 */
const isValidPassword = (password: string): boolean => {
  // At least 8 characters, contains uppercase, lowercase, and number
  return password.length >= 8 && /[A-Z]/.test(password) && /[a-z]/.test(password) && /[0-9]/.test(password);
};

/**
 * Validate username
 */
const isValidUsername = (username: string): boolean => {
  // 3-20 characters, alphanumeric and underscore only
  return /^[a-zA-Z0-9_]{3,20}$/.test(username);
};

/**
 * Users Component
 */
const Users: React.FC = () => {
  // ===== State Management =====
  const [users, setUsers] = useState<UserDetails[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Tab state
  const [tabValue, setTabValue] = useState<number>(0);

  // Pagination
  const [page, setPage] = useState<number>(0);
  const [rowsPerPage, setRowsPerPage] = useState<number>(10);

  // Search and filters
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [roleFilter, setRoleFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [daysFilter, setDaysFilter] = useState<number>(0);

  // Dialogs
  const [addEditDialogOpen, setAddEditDialogOpen] = useState<boolean>(false);
  const [detailsDialogOpen, setDetailsDialogOpen] = useState<boolean>(false);
  const [resetPasswordDialogOpen, setResetPasswordDialogOpen] = useState<boolean>(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);

  // Selected user
  const [selectedUser, setSelectedUser] = useState<UserDetails | null>(null);
  const [editMode, setEditMode] = useState<boolean>(false);

  // Form state
  const [formData, setFormData] = useState<UserFormData>({
    username: '',
    email: '',
    password: '',
    confirmPassword: '',
    role: UserRole.VIEWER,
    description: '',
  });
  const [validationErrors, setValidationErrors] = useState<ValidationErrors>({});

  // Password reset
  const [newPassword, setNewPassword] = useState<string>('');
  const [confirmNewPassword, setConfirmNewPassword] = useState<string>('');

  // Details data
  const [loginHistory, setLoginHistory] = useState<LoginHistory[]>([]);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);

  // Statistics
  const [statistics, setStatistics] = useState<Statistics>({
    totalUsers: 0,
    activeUsers: 0,
    adminUsers: 0,
    onlineNow: 0,
  });

  // ===== Data Fetching =====

  /**
   * Fetch users from API
   */
  const fetchUsers = useCallback(async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);

      const response = await getAllUsers();

      if (response.success && response.data) {
        setUsers(response.data);

        // Calculate statistics
        const totalUsers = response.data.length;
        const activeUsers = response.data.filter((u) => u.status === UserStatus.ACTIVE).length;
        const adminUsers = response.data.filter((u) => u.role === UserRole.ADMIN).length;
        // Simulate online users (users who logged in within last hour)
        const oneHourAgo = Date.now() - 60 * 60 * 1000;
        const onlineNow = response.data.filter(
          (u) => u.last_login && new Date(u.last_login).getTime() > oneHourAgo
        ).length;

        setStatistics({ totalUsers, activeUsers, adminUsers, onlineNow });
      } else {
        throw new Error(response.message || 'Failed to fetch users');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
      console.error('Failed to fetch users:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  // ===== Filtering =====

  /**
   * Filtered users based on search and filters
   */
  const filteredUsers = useMemo(() => {
    return users.filter((user) => {
      // Search query
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matchesSearch =
          user.username.toLowerCase().includes(query) ||
          user.email?.toLowerCase().includes(query) ||
          user.id.toLowerCase().includes(query);
        if (!matchesSearch) return false;
      }

      // Role filter
      if (roleFilter !== 'all' && user.role !== roleFilter) {
        return false;
      }

      // Status filter
      if (statusFilter !== 'all' && user.status !== statusFilter) {
        return false;
      }

      // Days filter (created in last X days)
      if (daysFilter > 0 && user.created_at) {
        const createdDate = new Date(user.created_at);
        const daysAgo = (Date.now() - createdDate.getTime()) / (1000 * 60 * 60 * 24);
        if (daysAgo > daysFilter) return false;
      }

      return true;
    });
  }, [users, searchQuery, roleFilter, statusFilter, daysFilter]);

  /**
   * Paginated users
   */
  const paginatedUsers = useMemo(() => {
    const startIndex = page * rowsPerPage;
    return filteredUsers.slice(startIndex, startIndex + rowsPerPage);
  }, [filteredUsers, page, rowsPerPage]);

  // ===== Event Handlers =====

  /**
   * Handle tab change
   */
  const handleTabChange = (_event: React.SyntheticEvent, newValue: number): void => {
    setTabValue(newValue);
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
  const handleRowsPerPageChange = (event: ChangeEvent<HTMLInputElement>): void => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };

  /**
   * Handle search change
   */
  const handleSearchChange = (event: ChangeEvent<HTMLInputElement>): void => {
    setSearchQuery(event.target.value);
    setPage(0);
  };

  /**
   * Handle role filter change
   */
  const handleRoleFilterChange = (event: SelectChangeEvent<string>): void => {
    setRoleFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle status filter change
   */
  const handleStatusFilterChange = (event: SelectChangeEvent<string>): void => {
    setStatusFilter(event.target.value);
    setPage(0);
  };

  /**
   * Handle days filter change
   */
  const handleDaysFilterChange = (event: SelectChangeEvent<number>): void => {
    setDaysFilter(event.target.value as number);
    setPage(0);
  };

  /**
   * Handle refresh
   */
  const handleRefresh = (): void => {
    fetchUsers();
  };

  /**
   * Handle add user button
   */
  const handleAddUser = (): void => {
    setEditMode(false);
    setFormData({
      username: '',
      email: '',
      password: '',
      confirmPassword: '',
      role: UserRole.VIEWER,
      description: '',
    });
    setValidationErrors({});
    setAddEditDialogOpen(true);
  };

  /**
   * Handle edit user button
   */
  const handleEditUser = (user: UserDetails): void => {
    setEditMode(true);
    setSelectedUser(user);
    setFormData({
      username: user.username,
      email: user.email || '',
      password: '',
      confirmPassword: '',
      role: user.role,
      description: user.description || '',
    });
    setValidationErrors({});
    setAddEditDialogOpen(true);
  };

  /**
   * Handle delete user button
   */
  const handleDeleteUser = (user: UserDetails): void => {
    setSelectedUser(user);
    setDeleteDialogOpen(true);
  };

  /**
   * Handle reset password button
   */
  const handleResetPassword = (user: UserDetails): void => {
    setSelectedUser(user);
    setNewPassword('');
    setConfirmNewPassword('');
    setResetPasswordDialogOpen(true);
  };

  /**
   * Handle view details
   */
  const handleViewDetails = async (user: UserDetails): Promise<void> => {
    setSelectedUser(user);
    setDetailsDialogOpen(true);

    // Fetch login history
    try {
      const historyResponse = await getUserLoginHistory(user.id, 10);
      if (historyResponse.success && historyResponse.data) {
        setLoginHistory(historyResponse.data);
      }
    } catch (err) {
      console.error('Failed to fetch login history:', err);
    }

    // Fetch audit logs
    try {
      const logsResponse = await queryLogs({
        operator_id: user.username,
        limit: 20,
      });
      if (logsResponse.success && logsResponse.data) {
        setAuditLogs(logsResponse.data);
      }
    } catch (err) {
      console.error('Failed to fetch audit logs:', err);
    }
  };

  /**
   * Handle status toggle
   */
  const handleStatusToggle = async (user: UserDetails): Promise<void> => {
    try {
      const newStatus = user.status === UserStatus.ACTIVE ? UserStatus.INACTIVE : UserStatus.ACTIVE;
      const response = await changeUserStatus(user.id, newStatus);

      if (response.success) {
        fetchUsers();
      } else {
        setError(response.message || 'Failed to change user status');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change user status');
    }
  };

  /**
   * Validate form
   */
  const validateForm = (): boolean => {
    const errors: ValidationErrors = {};

    // Username validation (only for new users)
    if (!editMode) {
      if (!formData.username) {
        errors.username = 'Username is required';
      } else if (!isValidUsername(formData.username)) {
        errors.username = 'Username must be 3-20 characters, alphanumeric and underscore only';
      }
    }

    // Email validation
    if (!formData.email) {
      errors.email = 'Email is required';
    } else if (!isValidEmail(formData.email)) {
      errors.email = 'Invalid email format';
    }

    // Password validation (required for new users, optional for edit)
    if (!editMode) {
      if (!formData.password) {
        errors.password = 'Password is required';
      } else if (!isValidPassword(formData.password)) {
        errors.password = 'Password must be at least 8 characters with uppercase, lowercase, and numbers';
      }

      if (formData.password !== formData.confirmPassword) {
        errors.confirmPassword = 'Passwords do not match';
      }
    } else if (formData.password) {
      // If editing and password is provided, validate it
      if (!isValidPassword(formData.password)) {
        errors.password = 'Password must be at least 8 characters with uppercase, lowercase, and numbers';
      }

      if (formData.password !== formData.confirmPassword) {
        errors.confirmPassword = 'Passwords do not match';
      }
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  };

  /**
   * Handle save user
   */
  const handleSaveUser = async (): Promise<void> => {
    if (!validateForm()) return;

    try {
      if (editMode && selectedUser) {
        const updateRequest: UpdateUserRequest = {
          email: formData.email,
          role: formData.role,
          description: formData.description,
        };

        const response = await updateUser(selectedUser.id, updateRequest);
        if (response.success) {
          setAddEditDialogOpen(false);
          fetchUsers();
        } else {
          setError(response.message || 'Failed to update user');
        }
      } else {
        const createRequest: CreateUserRequest = {
          username: formData.username,
          email: formData.email,
          password: formData.password,
          role: formData.role,
          description: formData.description,
        };

        const response = await createUser(createRequest);
        if (response.success) {
          setAddEditDialogOpen(false);
          fetchUsers();
        } else {
          setError(response.message || 'Failed to create user');
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save user');
    }
  };

  /**
   * Handle confirm delete
   */
  const handleConfirmDelete = async (): Promise<void> => {
    if (!selectedUser) return;

    try {
      const response = await deleteUser(selectedUser.id);
      if (response.success) {
        setDeleteDialogOpen(false);
        fetchUsers();
      } else {
        setError(response.message || 'Failed to delete user');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete user');
    }
  };

  /**
   * Handle confirm reset password
   */
  const handleConfirmResetPassword = async (): Promise<void> => {
    if (!selectedUser) return;

    if (!isValidPassword(newPassword)) {
      setError('Password must be at least 8 characters with uppercase, lowercase, and numbers');
      return;
    }

    if (newPassword !== confirmNewPassword) {
      setError('Passwords do not match');
      return;
    }

    try {
      const response = await resetUserPassword(selectedUser.id, newPassword);
      if (response.success) {
        setResetPasswordDialogOpen(false);
        setNewPassword('');
        setConfirmNewPassword('');
      } else {
        setError(response.message || 'Failed to reset password');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset password');
    }
  };

  /**
   * Handle export CSV
   */
  const handleExportCsv = (): void => {
    const csvHeaders = ['User ID', 'Username', 'Email', 'Role', 'Status', 'Last Login', 'Created Time'];

    const csvRows = filteredUsers.map((user) => [
      user.id,
      user.username,
      user.email || '',
      user.role,
      user.status,
      user.last_login || '',
      user.created_at || '',
    ]);

    const csvContent = [
      csvHeaders.join(','),
      ...csvRows.map((row) => row.map((cell) => `"${cell}"`).join(',')),
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);

    link.setAttribute('href', url);
    link.setAttribute('download', `artemis-users-${new Date().toISOString().split('T')[0]}.csv`);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  /**
   * Format last login time
   */
  const formatLastLogin = (lastLogin?: string): string => {
    if (!lastLogin) return 'Never';
    try {
      return formatDistanceToNow(new Date(lastLogin), { addSuffix: true });
    } catch {
      return 'Unknown';
    }
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
      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Total Users
            </Typography>
            <Typography variant="h3" fontWeight={600}>
              {statistics.totalUsers}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Active Users
            </Typography>
            <Typography variant="h3" fontWeight={600} color="success.main">
              {statistics.activeUsers}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Admin Users
            </Typography>
            <Typography variant="h3" fontWeight={600} color="error.main">
              {statistics.adminUsers}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <Card sx={statsCardSx}>
          <CardContent>
            <Typography variant="h6" color="text.secondary" gutterBottom>
              Online Now
            </Typography>
            <Typography variant="h3" fontWeight={600} color="primary.main">
              {statistics.onlineNow}
            </Typography>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );

  /**
   * Render filters
   */
  const renderFilters = (): React.ReactElement => (
    <Box sx={{ display: 'flex', gap: 2, marginBottom: 3, flexWrap: 'wrap' }}>
      <TextField
        label="Search"
        placeholder="Username or Email..."
        value={searchQuery}
        onChange={handleSearchChange}
        sx={{ minWidth: { xs: '100%', sm: 250 } }}
        InputProps={{
          startAdornment: <SearchIcon sx={{ marginRight: 1, color: 'action.active' }} />,
        }}
      />

      <FormControl sx={{ minWidth: { xs: '100%', sm: 150 } }}>
        <InputLabel>Role</InputLabel>
        <Select value={roleFilter} onChange={handleRoleFilterChange} label="Role">
          <MenuItem value="all">All Roles</MenuItem>
          <MenuItem value={UserRole.ADMIN}>Admin</MenuItem>
          <MenuItem value={UserRole.OPERATOR}>Operator</MenuItem>
          <MenuItem value={UserRole.VIEWER}>Viewer</MenuItem>
        </Select>
      </FormControl>

      <FormControl sx={{ minWidth: { xs: '100%', sm: 150 } }}>
        <InputLabel>Status</InputLabel>
        <Select value={statusFilter} onChange={handleStatusFilterChange} label="Status">
          <MenuItem value="all">All Status</MenuItem>
          <MenuItem value={UserStatus.ACTIVE}>Active</MenuItem>
          <MenuItem value={UserStatus.INACTIVE}>Inactive</MenuItem>
        </Select>
      </FormControl>

      <FormControl sx={{ minWidth: { xs: '100%', sm: 180 } }}>
        <InputLabel>Created In</InputLabel>
        <Select value={daysFilter} onChange={handleDaysFilterChange} label="Created In">
          <MenuItem value={0}>All Time</MenuItem>
          <MenuItem value={7}>Last 7 Days</MenuItem>
          <MenuItem value={30}>Last 30 Days</MenuItem>
          <MenuItem value={90}>Last 90 Days</MenuItem>
        </Select>
      </FormControl>

      <Box sx={{ flexGrow: 1 }} />

      <Tooltip title="Refresh">
        <span>
          <IconButton color="primary" onClick={handleRefresh} disabled={loading}>
            <RefreshIcon />
          </IconButton>
        </span>
      </Tooltip>

      <Button variant="outlined" startIcon={<DownloadIcon />} onClick={handleExportCsv}>
        Export CSV
      </Button>

      <Button variant="contained" startIcon={<AddIcon />} onClick={handleAddUser}>
        Add User
      </Button>
    </Box>
  );

  /**
   * Render loading skeleton
   */
  const renderLoadingSkeleton = (): React.ReactElement => (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>User ID</TableCell>
            <TableCell>Username</TableCell>
            <TableCell>Email</TableCell>
            <TableCell>Role</TableCell>
            <TableCell>Status</TableCell>
            <TableCell>Last Login</TableCell>
            <TableCell>Created Time</TableCell>
            <TableCell>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {[1, 2, 3, 4, 5].map((index) => (
            <TableRow key={index}>
              <TableCell>
                <Skeleton variant="text" width={80} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={150} />
              </TableCell>
              <TableCell>
                <Skeleton variant="rounded" width={80} height={24} />
              </TableCell>
              <TableCell>
                <Skeleton variant="rectangular" width={40} height={24} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={100} />
              </TableCell>
              <TableCell>
                <Skeleton variant="text" width={120} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  /**
   * Render users table
   */
  const renderUsersTable = (): React.ReactElement => (
    <>
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>User ID</TableCell>
              <TableCell>Username</TableCell>
              <TableCell>Email</TableCell>
              <TableCell>Role</TableCell>
              <TableCell>Status</TableCell>
              <TableCell>Last Login</TableCell>
              <TableCell>Created Time</TableCell>
              <TableCell>Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {paginatedUsers.map((user) => (
              <TableRow key={user.id} hover>
                <TableCell>{user.id}</TableCell>
                <TableCell>
                  <Typography component="span" sx={clickableTextSx} onClick={() => handleViewDetails(user)}>
                    {user.username}
                  </Typography>
                </TableCell>
                <TableCell>{user.email || '-'}</TableCell>
                <TableCell>
                  <Chip
                    icon={ROLE_ICONS[user.role]}
                    label={user.role.toUpperCase()}
                    color={ROLE_COLORS[user.role]}
                    size="small"
                  />
                </TableCell>
                <TableCell>
                  <Switch
                    checked={user.status === UserStatus.ACTIVE}
                    onChange={() => handleStatusToggle(user)}
                    color={user.status === UserStatus.ACTIVE ? 'success' : 'default'}
                    size="small"
                  />
                </TableCell>
                <TableCell>{formatLastLogin(user.last_login)}</TableCell>
                <TableCell>
                  {user.created_at ? new Date(user.created_at).toLocaleDateString() : '-'}
                </TableCell>
                <TableCell>
                  <Tooltip title="Edit">
                    <IconButton size="small" onClick={() => handleEditUser(user)}>
                      <EditIcon fontSize="small" />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="Reset Password">
                    <IconButton size="small" onClick={() => handleResetPassword(user)}>
                      <VpnKeyIcon fontSize="small" />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="Delete">
                    <IconButton size="small" onClick={() => handleDeleteUser(user)} color="error">
                      <DeleteIcon fontSize="small" />
                    </IconButton>
                  </Tooltip>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>

      <TablePagination
        rowsPerPageOptions={[10, 25, 50]}
        component="div"
        count={filteredUsers.length}
        rowsPerPage={rowsPerPage}
        page={page}
        onPageChange={handlePageChange}
        onRowsPerPageChange={handleRowsPerPageChange}
      />
    </>
  );

  /**
   * Render permission matrix
   */
  const renderPermissionMatrix = (): React.ReactElement => (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Feature</TableCell>
            <TableCell align="center" colSpan={4}>
              Admin
            </TableCell>
            <TableCell align="center" colSpan={4}>
              Operator
            </TableCell>
            <TableCell align="center" colSpan={4}>
              Viewer
            </TableCell>
          </TableRow>
          <TableRow>
            <TableCell />
            {['Read', 'Create', 'Update', 'Delete'].map((action) => (
              <TableCell key={`admin-${action}`} align="center">
                {action}
              </TableCell>
            ))}
            {['Read', 'Create', 'Update', 'Delete'].map((action) => (
              <TableCell key={`operator-${action}`} align="center">
                {action}
              </TableCell>
            ))}
            {['Read', 'Create', 'Update', 'Delete'].map((action) => (
              <TableCell key={`viewer-${action}`} align="center">
                {action}
              </TableCell>
            ))}
          </TableRow>
        </TableHead>
        <TableBody>
          {PERMISSION_MATRIX.map((row) => (
            <TableRow key={row.feature}>
              <TableCell>{row.feature}</TableCell>
              <TableCell align="center">
                <Checkbox checked={row.admin.read} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.admin.create} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.admin.update} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.admin.delete} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.operator.read} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.operator.create} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.operator.update} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.operator.delete} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.viewer.read} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.viewer.create} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.viewer.update} disabled size="small" />
              </TableCell>
              <TableCell align="center">
                <Checkbox checked={row.viewer.delete} disabled size="small" />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  /**
   * Render add/edit user dialog
   */
  const renderAddEditDialog = (): React.ReactElement => (
    <Dialog open={addEditDialogOpen} onClose={() => setAddEditDialogOpen(false)} maxWidth="sm" fullWidth>
      <DialogTitle>
        {editMode ? 'Edit User' : 'Add User'}
        <IconButton
          aria-label="close"
          onClick={() => setAddEditDialogOpen(false)}
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
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <TextField
            label="Username"
            value={formData.username}
            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
            disabled={editMode}
            required
            error={!!validationErrors.username}
            helperText={validationErrors.username}
            fullWidth
          />

          <TextField
            label="Email"
            type="email"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            required
            error={!!validationErrors.email}
            helperText={validationErrors.email}
            fullWidth
          />

          {!editMode && (
            <>
              <TextField
                label="Password"
                type="password"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                required
                error={!!validationErrors.password}
                helperText={validationErrors.password || 'At least 8 characters with uppercase, lowercase, and numbers'}
                fullWidth
              />

              <TextField
                label="Confirm Password"
                type="password"
                value={formData.confirmPassword}
                onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
                required
                error={!!validationErrors.confirmPassword}
                helperText={validationErrors.confirmPassword}
                fullWidth
              />
            </>
          )}

          <FormControl fullWidth>
            <InputLabel>Role</InputLabel>
            <Select
              value={formData.role}
              onChange={(e) => setFormData({ ...formData, role: e.target.value as UserRole })}
              label="Role"
            >
              <MenuItem value={UserRole.ADMIN}>Admin</MenuItem>
              <MenuItem value={UserRole.OPERATOR}>Operator</MenuItem>
              <MenuItem value={UserRole.VIEWER}>Viewer</MenuItem>
            </Select>
          </FormControl>

          <Alert severity="info" icon={<InfoIcon />}>
            <AlertTitle>{formData.role.toUpperCase()}</AlertTitle>
            {ROLE_DESCRIPTIONS[formData.role]}
          </Alert>

          <TextField
            label="Description"
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            multiline
            rows={3}
            fullWidth
          />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setAddEditDialogOpen(false)}>Cancel</Button>
        <Button onClick={handleSaveUser} variant="contained">
          {editMode ? 'Update' : 'Create'}
        </Button>
      </DialogActions>
    </Dialog>
  );

  /**
   * Render details dialog
   */
  const renderDetailsDialog = (): React.ReactElement => (
    <Dialog open={detailsDialogOpen} onClose={() => setDetailsDialogOpen(false)} maxWidth="md" fullWidth>
      <DialogTitle>
        User Details
        <IconButton
          aria-label="close"
          onClick={() => setDetailsDialogOpen(false)}
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
        {selectedUser && (
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
            {/* Basic Info */}
            <Box>
              <Typography variant="h6" gutterBottom>
                Basic Information
              </Typography>
              <Grid container spacing={2}>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    User ID
                  </Typography>
                  <Typography variant="body1">{selectedUser.id}</Typography>
                </Grid>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    Username
                  </Typography>
                  <Typography variant="body1">{selectedUser.username}</Typography>
                </Grid>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    Email
                  </Typography>
                  <Typography variant="body1">{selectedUser.email || '-'}</Typography>
                </Grid>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    Role
                  </Typography>
                  <Chip
                    icon={ROLE_ICONS[selectedUser.role]}
                    label={selectedUser.role.toUpperCase()}
                    color={ROLE_COLORS[selectedUser.role]}
                    size="small"
                  />
                </Grid>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    Status
                  </Typography>
                  <Chip
                    label={selectedUser.status.toUpperCase()}
                    color={selectedUser.status === UserStatus.ACTIVE ? 'success' : 'default'}
                    size="small"
                  />
                </Grid>
                <Grid size={{ xs: 6 }}>
                  <Typography variant="body2" color="text.secondary">
                    Last Login
                  </Typography>
                  <Typography variant="body1">{formatLastLogin(selectedUser.last_login)}</Typography>
                </Grid>
              </Grid>
            </Box>

            {/* Permissions */}
            <Box>
              <Typography variant="h6" gutterBottom>
                Permissions
              </Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                {selectedUser.permissions.map((permission) => (
                  <Chip key={permission} label={permission} size="small" variant="outlined" />
                ))}
              </Box>
            </Box>

            {/* Login History */}
            <Box>
              <Typography variant="h6" gutterBottom>
                Recent Login History
              </Typography>
              {loginHistory.length > 0 ? (
                <TableContainer component={Paper} variant="outlined">
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>Time</TableCell>
                        <TableCell>IP Address</TableCell>
                        <TableCell>Status</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {loginHistory.map((history) => (
                        <TableRow key={history.id}>
                          <TableCell>{formatLastLogin(history.login_time)}</TableCell>
                          <TableCell>{history.ip_address}</TableCell>
                          <TableCell>
                            <Chip
                              label={history.status.toUpperCase()}
                              color={history.status === 'success' ? 'success' : 'error'}
                              size="small"
                            />
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No login history available
                </Typography>
              )}
            </Box>

            {/* Audit Logs */}
            <Box>
              <Typography variant="h6" gutterBottom>
                Recent Operations
              </Typography>
              {auditLogs.length > 0 ? (
                <TableContainer component={Paper} variant="outlined">
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>Time</TableCell>
                        <TableCell>Operation</TableCell>
                        <TableCell>Resource</TableCell>
                        <TableCell>Result</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {auditLogs.slice(0, 10).map((log) => (
                        <TableRow key={log.id}>
                          <TableCell>{formatLastLogin(log.timestamp)}</TableCell>
                          <TableCell>{log.operation_type}</TableCell>
                          <TableCell>
                            {log.resource_type}: {log.resource_id}
                          </TableCell>
                          <TableCell>
                            <Chip
                              label={log.result}
                              color={log.result === 'SUCCESS' ? 'success' : 'error'}
                              size="small"
                            />
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No operations found
                </Typography>
              )}
            </Box>
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setDetailsDialogOpen(false)}>Close</Button>
      </DialogActions>
    </Dialog>
  );

  /**
   * Render reset password dialog
   */
  const renderResetPasswordDialog = (): React.ReactElement => (
    <Dialog open={resetPasswordDialogOpen} onClose={() => setResetPasswordDialogOpen(false)} maxWidth="xs" fullWidth>
      <DialogTitle>
        Reset Password
        <IconButton
          aria-label="close"
          onClick={() => setResetPasswordDialogOpen(false)}
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
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <Typography variant="body2" color="text.secondary">
            Reset password for user: <strong>{selectedUser?.username}</strong>
          </Typography>

          <TextField
            label="New Password"
            type="password"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            helperText="At least 8 characters with uppercase, lowercase, and numbers"
            fullWidth
          />

          <TextField
            label="Confirm New Password"
            type="password"
            value={confirmNewPassword}
            onChange={(e) => setConfirmNewPassword(e.target.value)}
            fullWidth
          />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setResetPasswordDialogOpen(false)}>Cancel</Button>
        <Button onClick={handleConfirmResetPassword} variant="contained" color="primary">
          Reset Password
        </Button>
      </DialogActions>
    </Dialog>
  );

  /**
   * Render delete confirmation dialog
   */
  const renderDeleteDialog = (): React.ReactElement => (
    <Dialog open={deleteDialogOpen} onClose={() => setDeleteDialogOpen(false)} maxWidth="xs" fullWidth>
      <DialogTitle>Confirm Delete</DialogTitle>
      <DialogContent>
        <Typography variant="body1">
          Are you sure you want to delete user <strong>{selectedUser?.username}</strong>?
        </Typography>
        <Alert severity="warning" sx={{ marginTop: 2 }}>
          This action cannot be undone.
        </Alert>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
        <Button onClick={handleConfirmDelete} variant="contained" color="error">
          Delete
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
          Users
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage user accounts and permissions
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

      {/* Tabs */}
      <Card>
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={tabValue} onChange={handleTabChange}>
            <Tab label="Users List" />
            <Tab label="Permissions Matrix" />
          </Tabs>
        </Box>

        <CardContent>
          {/* Users List Tab */}
          <TabPanel value={tabValue} index={0}>
            {renderFilters()}

            {loading ? (
              renderLoadingSkeleton()
            ) : filteredUsers.length === 0 ? (
              <Box sx={{ textAlign: 'center', padding: 4 }}>
                <Typography variant="body1" color="text.secondary">
                  No users found
                </Typography>
              </Box>
            ) : (
              renderUsersTable()
            )}
          </TabPanel>

          {/* Permissions Matrix Tab */}
          <TabPanel value={tabValue} index={1}>
            <Typography variant="body1" color="text.secondary" sx={{ marginBottom: 2 }}>
              Permission matrix showing access rights for each role across different features.
            </Typography>
            {renderPermissionMatrix()}
          </TabPanel>
        </CardContent>
      </Card>

      {/* Dialogs */}
      {renderAddEditDialog()}
      {renderDetailsDialog()}
      {renderResetPasswordDialog()}
      {renderDeleteDialog()}
    </Box>
  );
};

Users.displayName = 'Users';

export default Users;
