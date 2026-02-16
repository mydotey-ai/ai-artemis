/**
 * Routing Page Component
 *
 * Placeholder page for routing rules management
 */

import React from 'react';
import { Box, Typography, Card, CardContent } from '@mui/material';

const Routing: React.FC = () => {
  return (
    <Box>
      <Box sx={{ marginBottom: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Routing
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Configure routing rules and strategies
        </Typography>
      </Box>

      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Coming Soon
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Routing configuration features will be available here.
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
};

Routing.displayName = 'Routing';

export default Routing;
