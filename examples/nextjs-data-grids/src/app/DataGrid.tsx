'use client';

import {
  useArrayStorage,
  useMyPresence,
  useOthers,
  type ArrayStorageHookResult
} from '@ember-link/react';
import {
  GridCellKind,
  type EditableGridCell,
  type GridCell,
  type GridColumn,
  type GridSelection,
  type Item,
  type Rectangle,
  type Highlight
} from '@glideapps/glide-data-grid';
import dynamic from 'next/dynamic';
import React from 'react';
import '@glideapps/glide-data-grid/dist/index.css';

declare global {
  interface EmberLink {
    Presence: {
      selectedRectangle: Rectangle | null;
      color: string;
      name: string;
    };
  }
}

const DataEditor = dynamic(
  () => import('@glideapps/glide-data-grid').then((mod) => mod.DataEditor),
  { ssr: false }
);

const columns: GridColumn[] = [
  {
    title: 'Name',
    id: 'name'
  },
  {
    title: 'Company',
    id: 'company'
  },
  {
    title: 'Email',
    id: 'email'
  },
  {
    title: 'Phone',
    id: 'phone'
  }
];

interface DummyItem {
  name: string;
  company: string;
  email: string;
  phone: string;
}

const highlightColors = [
  '#FFEB3B', // Bright Yellow
  '#4CAF50', // Green
  '#2196F3', // Blue
  '#FF9800', // Orange
  '#E91E63', // Pink
  '#9C27B0', // Purple
  '#00BCD4', // Cyan
  '#F44336', // Red
  '#CDDC39', // Lime
  '#795548' // Brown
];

const names = [
  'Lea Thompson',
  'Cyndi Lauper',
  'Tom Cruise',
  'Madonna',
  'Jerry Hall',
  'Joan Collins',
  'Winona Ryder',
  'Christina Applegate',
  'Alyssa Milano',
  'Molly Ringwald',
  'Ally Sheedy',
  'Debbie Harry',
  'Olivia Newton-John',
  'Elton John',
  'Michael J. Fox',
  'Axl Rose',
  'Emilio Estevez',
  'Ralph Macchio',
  'Rob Lowe',
  'Jennifer Grey',
  'Mickey Rourke',
  'John Cusack',
  'Matthew Broderick',
  'Justine Bateman',
  'Lisa Bonet'
];

export function DataGrid() {
  const syncingData: ArrayStorageHookResult<DummyItem> = useArrayStorage('data');
  const [myPresence, setMyPresence] = useMyPresence();
  const [gridSelection, setGridSelection] = React.useState<GridSelection | undefined>(undefined);
  const others = useOthers();

  const myColor = React.useMemo(() => {
    return highlightColors[Math.floor(Math.random() * highlightColors.length)];
  }, []);

  const myName = React.useMemo(() => {
    return names[Math.floor(Math.random() * names.length)];
  }, []);

  const highlighRegionsFromOthers: Highlight[] = React.useMemo(() => {
    return others
      .map((other) => {
        if (other.selectedRectangle) {
          return {
            range: other.selectedRectangle,
            color: other.color
          };
        }
      })
      .filter(Boolean) as Highlight[];
  }, [others]);

  const getCellContent = React.useCallback(
    (cell: Item): GridCell => {
      const [col, row] = cell;
      const dataRow = syncingData.current[row];
      // dumb but simple way to do this
      const indexes: (keyof DummyItem)[] = ['name', 'company', 'email', 'phone'];
      const d = dataRow[indexes[col]];
      return {
        kind: GridCellKind.Text,
        allowOverlay: true,
        readonly: false,
        displayData: d,
        data: d
      };
    },
    [syncingData]
  );

  const onCellEdited = React.useCallback(
    (cell: Item, newValue: EditableGridCell) => {
      if (newValue.kind !== GridCellKind.Text) {
        // we only have text cells, might as well just die here.
        return;
      }

      const indexes: (keyof DummyItem)[] = ['name', 'company', 'email', 'phone'];
      const [col, row] = cell;
      const key = indexes[col];

      const newData = { ...syncingData.current[row], [key]: newValue.data };

      syncingData.replace(row, newData);
    },
    [syncingData]
  );

  const onGridSelectionChange = React.useCallback((newSelection: GridSelection) => {
    setMyPresence({
      selectedRectangle: newSelection.current?.range ?? null,
      color: myColor,
      name: myName
    });

    setGridSelection(newSelection);
  }, []);

  const onSelectionCleared = React.useCallback(() => {
    setMyPresence({
      selectedRectangle: null,
      color: myColor,
      name: myName
    });
    setGridSelection(undefined);
  }, []);

  return (
    <DataEditor
      drawFocusRing={true}
      columns={columns}
      rows={syncingData.current.length}
      highlightRegions={highlighRegionsFromOthers}
      getCellContent={getCellContent}
      onCellEdited={onCellEdited}
      gridSelection={gridSelection}
      onGridSelectionChange={onGridSelectionChange}
      onSelectionCleared={onSelectionCleared}
    />
  );
}
