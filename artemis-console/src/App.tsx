import { ThemeProvider, CssBaseline } from '@mui/material';
import { RouterProvider } from 'react-router-dom';
import { useUIStore } from '@/store/uiStore';
import { lightTheme, darkTheme } from '@/theme';
import { router } from '@/routes';
import { WebSocketProvider } from '@/components/WebSocketProvider';

function App() {
  const theme = useUIStore((state) => state.theme);
  const currentTheme = theme === 'light' ? lightTheme : darkTheme;

  return (
    <ThemeProvider theme={currentTheme}>
      <CssBaseline />
      <WebSocketProvider>
        <RouterProvider router={router} />
      </WebSocketProvider>
    </ThemeProvider>
  );
}

export default App;
