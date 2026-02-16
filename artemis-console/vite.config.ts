import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { visualizer } from 'rollup-plugin-visualizer';

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const plugins = [react()];

  // Add bundle analyzer in analyze mode
  if (mode === 'analyze') {
    plugins.push(
      visualizer({
        open: true,
        filename: 'dist/stats.html',
        gzipSize: true,
        brotliSize: true,
      }) as never,
    );
  }

  return {
    plugins,
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
    server: {
      port: 3000,
      proxy: {
        '/api': {
          target: 'http://localhost:8080',
          changeOrigin: true,
        },
      },
    },
    build: {
      outDir: 'dist',
      sourcemap: mode !== 'production',
      // Chunk size warnings
      chunkSizeWarningLimit: 600,
      rollupOptions: {
        output: {
          // Optimized manual chunks strategy
          manualChunks: (id: string) => {
            // React core libraries
            if (id.includes('node_modules/react/') || id.includes('node_modules/react-dom/')) {
              return 'react-vendor';
            }

            // React Router
            if (id.includes('node_modules/react-router-dom/')) {
              return 'react-vendor';
            }

            // MUI core components
            if (id.includes('node_modules/@mui/material/')) {
              return 'mui-core';
            }

            // MUI icons
            if (id.includes('node_modules/@mui/icons-material/')) {
              return 'mui-icons';
            }

            // Emotion (MUI dependency)
            if (id.includes('node_modules/@emotion/')) {
              return 'mui-core';
            }

            // Charts library
            if (id.includes('node_modules/recharts/')) {
              return 'charts';
            }

            // API modules (split by domain)
            if (id.includes('src/api/')) {
              if (id.includes('src/api/auth.ts') || id.includes('src/api/client.ts')) {
                return 'api-core';
              }
              if (id.includes('src/api/discovery.ts') || id.includes('src/api/cluster.ts')) {
                return 'api-registry';
              }
              if (id.includes('src/api/management.ts') || id.includes('src/api/routing.ts')) {
                return 'api-management';
              }
              if (
                id.includes('src/api/zone.ts') ||
                id.includes('src/api/canary.ts') ||
                id.includes('src/api/audit.ts')
              ) {
                return 'api-operations';
              }
            }

            // Page components (each page as separate chunk)
            if (id.includes('src/pages/Dashboard/')) {
              return 'page-dashboard';
            }
            if (id.includes('src/pages/Services/')) {
              return 'page-services';
            }
            if (id.includes('src/pages/Instances/')) {
              return 'page-instances';
            }
            if (id.includes('src/pages/Cluster/')) {
              return 'page-cluster';
            }
            if (id.includes('src/pages/Routing/')) {
              return 'page-routing';
            }
            if (id.includes('src/pages/AuditLog/')) {
              return 'page-audit';
            }
            if (id.includes('src/pages/ZoneOps/')) {
              return 'page-zone';
            }
            if (id.includes('src/pages/Canary/')) {
              return 'page-canary';
            }
            if (id.includes('src/pages/Users/')) {
              return 'page-users';
            }

            // Other node_modules go into vendor chunk
            if (id.includes('node_modules/')) {
              return 'vendor';
            }

            // Default behavior for app code
            return undefined;
          },
        },
      },
    },
  };
});
