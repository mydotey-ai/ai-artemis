/**
 * Services Page Component
 *
 * Features:
 * - Services list view
 * - Table placeholder for service data
 * - Search and filter capabilities (coming soon)
 */

import React from 'react';
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
  Paper,
  Chip,
  type SxProps,
  type Theme,
} from '@mui/material';

/**
 * Services component
 *
 * @returns React component
 */
const Services: React.FC = () => {
  // Mock data for demonstration
  const mockServices = [
    { id: 'service-1', name: 'user-service', instances: 5, status: 'UP' },
    { id: 'service-2', name: 'order-service', instances: 3, status: 'UP' },
    { id: 'service-3', name: 'payment-service', instances: 2, status: 'DOWN' },
  ];

  // ===== Styles =====

  /**
   * Page header styles
   */
  const headerBoxSx: SxProps<Theme> = {
    marginBottom: 3,
  };

  /**
   * Table container styles
   */
  const tableContainerSx: SxProps<Theme> = {
    marginTop: 2,
  };

  /**
   * Get status chip color based on status
   */
  const getStatusColor = (status: string): 'success' | 'error' | 'default' => {
    switch (status) {
      case 'UP':
        return 'success';
      case 'DOWN':
        return 'error';
      default:
        return 'default';
    }
  };

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

      {/* Services Table */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Registered Services
          </Typography>

          <TableContainer component={Paper} sx={tableContainerSx}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Service ID</TableCell>
                  <TableCell>Service Name</TableCell>
                  <TableCell align="center">Instances</TableCell>
                  <TableCell align="center">Status</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {mockServices.map((service) => (
                  <TableRow
                    key={service.id}
                    sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                  >
                    <TableCell component="th" scope="row">
                      {service.id}
                    </TableCell>
                    <TableCell>{service.name}</TableCell>
                    <TableCell align="center">{service.instances}</TableCell>
                    <TableCell align="center">
                      <Chip
                        label={service.status}
                        color={getStatusColor(service.status)}
                        size="small"
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>

          <Box sx={{ marginTop: 2, textAlign: 'center' }}>
            <Typography variant="caption" color="text.secondary">
              API integration coming soon...
            </Typography>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};

/**
 * Display name for debugging
 */
Services.displayName = 'Services';

export default Services;
