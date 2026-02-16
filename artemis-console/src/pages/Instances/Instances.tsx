/**
 * Instances Page Component
 *
 * Placeholder page for instance management
 */

import React from 'react';
import { Box, Typography, Card, CardContent } from '@mui/material';

const Instances: React.FC = () => {
  return (
    <Box>
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Instances
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage service instances
        </Typography>
      </Box>

      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Coming Soon
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Instance management features will be available here.
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
};

Instances.displayName = 'Instances';

export default Instances;
