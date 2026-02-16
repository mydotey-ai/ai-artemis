# Layout Components

Core layout components for the Artemis Console application. These components provide the foundational structure for all pages in the application.

## Components Overview

### MainLayout

The primary layout wrapper for authenticated pages. Combines the Header and Sidebar in a responsive box layout.

**Features:**
- Fixed header at the top (z-index: 1100)
- Responsive sidebar (permanent on desktop, collapsible on mobile)
- Main content area with proper spacing and padding
- Outlet support for react-router-dom routing
- Smooth transitions between states
- Theme-aware styling with Material-UI

**Layout Structure:**
```
┌─────────────────────────────────┐
│       Header (Fixed, z: 1100)   │ (64px height)
├──────────────┬──────────────────┤
│              │                  │
│ Sidebar      │ Main Content     │
│ (Desktop:    │ Area with        │
│  Permanent)  │ Outlet           │
│              │                  │
│ (Mobile:     │                  │
│  Drawer)     │                  │
│              │                  │
└──────────────┴──────────────────┘
```

**Props:**
```typescript
interface MainLayoutProps {
  children?: ReactNode;           // Optional children (alternative to Outlet)
  maxWidth?: ContainerMaxWidth;   // Container max width (default: 'lg')
  containerSx?: SxProps<Theme>;  // Additional sx styles
  drawerWidth?: number;           // Sidebar width in pixels (default: 240)
}
```

**Usage Example:**
```tsx
import { MainLayout } from '@/components/Layout';
import { Outlet } from 'react-router-dom';

// Option 1: Using Outlet for routing
function App() {
  return (
    <MainLayout>
      <Outlet />
    </MainLayout>
  );
}

// Option 2: Using MainLayout with auto Outlet
function App() {
  return <MainLayout />;
}

// Option 3: Custom children
function App() {
  return (
    <MainLayout maxWidth="md">
      <YourComponent />
    </MainLayout>
  );
}
```

**Responsive Behavior:**
- **Desktop (lg+)**: Sidebar is permanent (240px wide), takes up left side of main area
- **Tablet/Mobile (below lg)**: Sidebar becomes a temporary drawer that slides in from the left
- **Header**: Always fixed at the top, adjusts padding-left for sidebar width on desktop
- **Content Area**: Automatically adjusts margins to accommodate sidebar on desktop

### Header

Top navigation bar component with branding, theme toggle, and sidebar controls.

**Features:**
- Logo/branding area with app title
- Theme toggle button (light/dark mode)
- Sidebar menu toggle (visible on mobile only)
- Responsive layout with proper spacing
- Material-UI AppBar styling
- Icon animations and transitions

**Props:**
```typescript
interface HeaderProps {
  title?: string;              // Custom title (default: 'Artemis Console')
  onMenuClick?: () => void;    // Custom menu click handler
  sx?: SxProps<Theme>;         // Additional sx styles
}
```

**Usage Example:**
```tsx
import { Header } from '@/components/Layout';

// Default usage
<Header />

// Custom title
<Header title="My Custom Title" />

// With custom menu handler
<Header
  title="Custom App"
  onMenuClick={() => console.log('Menu clicked')}
/>
```

### Sidebar

Responsive navigation sidebar with Material-UI Drawer.

**Features:**
- Desktop: Permanent drawer (lg breakpoint and above)
- Mobile: Temporary drawer that opens/closes (below lg)
- Navigation menu with 9 standard menu items
- Active route highlighting
- Smooth transitions
- Header and footer sections
- Theme-aware styling

**Menu Items:**
1. Dashboard - `/dashboard`
2. Services - `/services`
3. Instances - `/instances`
4. Cluster - `/cluster`
5. Routing - `/routing`
6. Audit Log - `/audit-log`
7. Zone Ops - `/zone-ops`
8. Canary - `/canary`
9. Users - `/users`

**Props:**
```typescript
interface SidebarProps {
  drawerWidth?: number;    // Custom width in pixels (default: 280px)
  className?: string;      // Optional CSS class name
}
```

**Usage Example:**
```tsx
import { Sidebar } from '@/components/Layout';

// Default usage
<Sidebar />

// Custom width
<Sidebar drawerWidth={300} />

// With CSS class
<Sidebar className="custom-sidebar" />
```

## Z-Index Hierarchy

The layout components use a structured z-index hierarchy to ensure proper layering:

```
1100  ← Header (AppBar)
1000  ← Sidebar (on desktop)
0     ← Main content
```

This ensures that:
- Header is always visible and interactive
- Sidebar (on desktop) appears below header but above content
- Mobile drawer appears above everything (1300+ by default with Material-UI)

## Responsive Breakpoints

Material-UI breakpoints used:
- `xs`: 0px - phones
- `sm`: 600px - tablets
- `md`: 960px - small laptops
- `lg`: 1280px - laptops/desktops (sidebar transition point)
- `xl`: 1920px - large displays

Key breakpoint: `lg` (1280px) - switches sidebar from temporary (mobile) to permanent (desktop)

## State Management

The layout components use Zustand store (`useUIStore`) for state:

**Sidebar State:**
- `sidebarOpen: boolean` - Whether sidebar is open (mobile only)
- `toggleSidebar()` - Toggle sidebar visibility
- `setSidebarOpen(open: boolean)` - Set sidebar state explicitly

**Theme State:**
- `theme: 'light' | 'dark'` - Current theme
- `toggleTheme()` - Toggle between light and dark theme
- `setTheme(theme: Theme)` - Set specific theme

**Persistence:**
- Sidebar state is saved to `localStorage` as `artemis_sidebar_open`
- Theme preference is saved to `localStorage` as `artemis_theme`
- Both states are restored on app reload

## Styling and Theming

All layout components use Material-UI theming system:

**Theme Variables Used:**
- `theme.palette.primary.main` - Primary brand color
- `theme.palette.background.paper` - Header/Sidebar background
- `theme.palette.background.default` - Main content area background
- `theme.palette.text.primary` - Primary text color
- `theme.palette.divider` - Border colors
- `theme.transitions` - Smooth animations

**Custom Styling:**
Both MainLayout and Header accept custom `sx` props for Material-UI styling:

```tsx
<MainLayout
  containerSx={{
    backgroundColor: '#f5f5f5',
    padding: 4,
  }}
/>

<Header
  sx={{
    boxShadow: 3,
    backgroundColor: '#fff',
  }}
/>
```

## Integration with React Router

The MainLayout component integrates seamlessly with react-router-dom:

**Setup in App Component:**
```tsx
import { MainLayout } from '@/components/Layout';
import { BrowserRouter, Routes, Route, Outlet } from 'react-router-dom';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<MainLayout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/services" element={<Services />} />
          <Route path="/instances" element={<Instances />} />
          {/* More routes... */}
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
```

The `<MainLayout />` component automatically renders `<Outlet />` for child routes.

## Performance Considerations

**Optimizations:**
1. Memoized calculations for active route detection
2. Memoized icons to avoid unnecessary re-renders
3. React.FC with proper TypeScript types
4. useMediaQuery for efficient breakpoint detection
5. Selective re-renders using Zustand selectors
6. CSS transitions for smooth animations

**Best Practices:**
- Use `selectSidebarOpen` selector for sidebar state
- Use `selectTheme` selector for theme state
- Avoid passing complex objects as props
- Use sx prop for dynamic styling instead of inline styles

## Common Issues and Solutions

### Issue: Sidebar overlaps content on mobile

**Solution:** Ensure you're using `MainLayout` wrapper for all pages. The temporary drawer on mobile has proper modal backdrop and z-indexing.

### Issue: Content not scrolling properly

**Solution:** MainLayout uses `overflow: auto` on content wrapper. Ensure Container and child components have proper height settings.

### Issue: Header padding doesn't adjust on desktop

**Solution:** MainLayout handles this automatically. Check that the parent component is using MainLayout, not building custom layout.

### Issue: Theme toggle not persisting

**Solution:** Theme is automatically saved to localStorage. Check browser console for localStorage errors and verify localStorage is enabled.

## File Structure

```
src/components/Layout/
├── MainLayout.tsx      # Primary layout wrapper
├── Header.tsx          # Top navigation bar
├── Sidebar.tsx         # Navigation sidebar
├── index.ts            # Exports
└── README.md           # This file
```

## Type Definitions

All components are fully typed with TypeScript:

```typescript
// MainLayout
interface MainLayoutProps {
  children?: ReactNode;
  maxWidth?: ContainerMaxWidth;
  containerSx?: SxProps<Theme>;
  drawerWidth?: number;
}

// Header
interface HeaderProps {
  title?: string;
  onMenuClick?: () => void;
  sx?: SxProps<Theme>;
}

// Sidebar
interface SidebarProps {
  drawerWidth?: number;
  className?: string;
}
```

## Browser Support

The layout components support all modern browsers:
- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers (iOS Safari, Chrome Mobile)

## Contributing

When modifying layout components:
1. Maintain TypeScript types
2. Follow Material-UI conventions
3. Test responsive behavior at all breakpoints
4. Update this README with any changes
5. Run `npm run lint` and `npm run build` before committing

## Related Documentation

- Material-UI Documentation: https://mui.com/
- React Router Documentation: https://reactrouter.com/
- Zustand Documentation: https://github.com/pmndrs/zustand
