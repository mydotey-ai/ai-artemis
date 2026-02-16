/**
 * Canary Page Component
 *
 * Placeholder page for canary deployment management
 */

import React from 'react';
import { Box, Typography, Card, CardContent } from '@mui/material';

const Canary: React.FC = () => {
  return (
    <Box>
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Canary Deployment
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage canary deployment configurations
        </Typography>
      </Box>

      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Coming Soon
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Canary deployment features will be available here.
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
};

Canary.displayName = 'Canary';

export default Canary;
