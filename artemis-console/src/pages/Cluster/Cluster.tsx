/**
 * Cluster Page Component
 *
 * Placeholder page for cluster management
 */

import React from 'react';
import { Box, Typography, Card, CardContent } from '@mui/material';

const Cluster: React.FC = () => {
  return (
    <Box>
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Cluster
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Monitor and manage cluster nodes
        </Typography>
      </Box>

      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Coming Soon
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Cluster management features will be available here.
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
};

Cluster.displayName = 'Cluster';

export default Cluster;
