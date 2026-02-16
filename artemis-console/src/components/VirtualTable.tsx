/**
 * VirtualTable component for rendering large lists efficiently
 *
 * Uses react-window for virtualization to handle thousands of rows
 * with minimal performance impact
 */

import { useMemo, useRef, useEffect } from 'react';
import { FixedSizeList as List, ListChildComponentProps } from 'react-window';
import {
  Box,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material';

/**
 * Column definition for VirtualTable
 */
export interface VirtualTableColumn<T> {
  /** Column identifier */
  id: string;
  /** Column header label */
  label: string;
  /** Column width (flex value or px) */
  width?: string | number;
  /** Minimum column width */
  minWidth?: number;
  /** Align content */
  align?: 'left' | 'center' | 'right';
  /** Render cell content */
  render: (row: T, index: number) => React.ReactNode;
}

/**
 * Props for VirtualTable component
 */
export interface VirtualTableProps<T> {
  /** Data array to render */
  data: T[];
  /** Column definitions */
  columns: VirtualTableColumn<T>[];
  /** Row height in pixels (default: 52) */
  rowHeight?: number;
  /** Table height in pixels (default: 600) */
  height?: number;
  /** Show empty state when no data */
  emptyMessage?: string;
  /** Custom row key getter */
  getRowKey?: (row: T, index: number) => string | number;
  /** On row click handler */
  onRowClick?: (row: T, index: number) => void;
}

/**
 * VirtualTable component for efficient rendering of large datasets
 *
 * Features:
 * - Virtualization using react-window
 * - Only renders visible rows
 * - Supports thousands of rows without performance issues
 * - Customizable columns and row rendering
 * - Click handlers and keyboard navigation
 *
 * @example
 * ```tsx
 * <VirtualTable
 *   data={instances}
 *   columns={[
 *     { id: 'id', label: 'Instance ID', render: (row) => row.id },
 *     { id: 'status', label: 'Status', render: (row) => <StatusChip status={row.status} /> },
 *   ]}
 *   rowHeight={52}
 *   height={600}
 *   onRowClick={(row) => console.log('Clicked:', row)}
 * />
 * ```
 */
export function VirtualTable<T>({
  data,
  columns,
  rowHeight = 52,
  height = 600,
  emptyMessage = 'No data available',
  getRowKey,
  onRowClick,
}: VirtualTableProps<T>) {
  const listRef = useRef<List>(null);

  // Reset scroll position when data changes
  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTo(0);
    }
  }, [data]);

  // Calculate column widths
  const columnStyles = useMemo(() => {
    return columns.map((col) => ({
      width: col.width || 'auto',
      minWidth: col.minWidth || 100,
      textAlign: col.align || 'left',
    }));
  }, [columns]);

  // Row renderer for react-window
  const Row = ({ index, style }: ListChildComponentProps) => {
    const row = data[index];
    const key = getRowKey ? getRowKey(row, index) : index;

    return (
      <TableRow
        key={key}
        style={style}
        hover
        onClick={() => onRowClick?.(row, index)}
        sx={{
          cursor: onRowClick ? 'pointer' : 'default',
          display: 'flex',
          alignItems: 'center',
          borderBottom: '1px solid',
          borderColor: 'divider',
          '&:hover': onRowClick
            ? {
                backgroundColor: 'action.hover',
              }
            : undefined,
        }}
      >
        {columns.map((column, colIndex) => (
          <TableCell
            key={column.id}
            align={column.align}
            sx={{
              ...columnStyles[colIndex],
              flex: typeof columnStyles[colIndex].width === 'number' ? 'none' : '1',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              padding: '8px 16px',
            }}
          >
            {column.render(row, index)}
          </TableCell>
        ))}
      </TableRow>
    );
  };

  // Empty state
  if (data.length === 0) {
    return (
      <Paper sx={{ height, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Typography variant="body1" color="text.secondary">
          {emptyMessage}
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper} sx={{ height }}>
      <Table stickyHeader>
        {/* Table Header */}
        <TableHead>
          <TableRow
            sx={{
              display: 'flex',
              '& .MuiTableCell-root': {
                backgroundColor: 'background.paper',
                fontWeight: 'bold',
              },
            }}
          >
            {columns.map((column, colIndex) => (
              <TableCell
                key={column.id}
                align={column.align}
                sx={{
                  ...columnStyles[colIndex],
                  flex: typeof columnStyles[colIndex].width === 'number' ? 'none' : '1',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  padding: '16px',
                }}
              >
                {column.label}
              </TableCell>
            ))}
          </TableRow>
        </TableHead>
      </Table>

      {/* Virtualized Table Body */}
      <Box sx={{ height: height - 56 }}>
        <List
          ref={listRef}
          height={height - 56} // Subtract header height
          itemCount={data.length}
          itemSize={rowHeight}
          width="100%"
          overscanCount={5} // Render 5 extra rows above/below viewport
        >
          {Row}
        </List>
      </Box>
    </TableContainer>
  );
}

/**
 * Simple virtualized list for non-table data
 *
 * @example
 * ```tsx
 * <VirtualList
 *   data={items}
 *   itemHeight={80}
 *   height={400}
 *   renderItem={(item) => <ItemCard item={item} />}
 * />
 * ```
 */
export interface VirtualListProps<T> {
  data: T[];
  itemHeight: number;
  height?: number;
  renderItem: (item: T, index: number) => React.ReactNode;
  getItemKey?: (item: T, index: number) => string | number;
}

export function VirtualList<T>({
  data,
  itemHeight,
  height = 600,
  renderItem,
  getItemKey,
}: VirtualListProps<T>) {
  const listRef = useRef<List>(null);

  const Row = ({ index, style }: ListChildComponentProps) => {
    const item = data[index];
    const key = getItemKey ? getItemKey(item, index) : index;

    return (
      <Box key={key} style={style}>
        {renderItem(item, index)}
      </Box>
    );
  };

  return (
    <List
      ref={listRef}
      height={height}
      itemCount={data.length}
      itemSize={itemHeight}
      width="100%"
      overscanCount={3}
    >
      {Row}
    </List>
  );
}

export default VirtualTable;
