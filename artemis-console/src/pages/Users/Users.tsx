/**
 * Users Page Component
 *
 * Placeholder page for user management
 */

import React from 'react';
import { Box, Typography, Card, CardContent } from '@mui/material';

const Users: React.FC = () => {
  return (
    <Box>
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Users
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage user accounts and permissions
        </Typography>
      </Box>

      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Coming Soon
          </Typography>
          <Typography variant="body2" color="text.secondary">
            User management features will be available here.
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
};

Users.displayName = 'Users';

export default Users;
