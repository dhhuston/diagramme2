import type { AppMenuBarBucket, MenuBarRow } from './components/AppMenuBar'
import { getShortcutLabels, isMacOS } from './shortcutPlatform'
import { isTauriApp } from './utils/isTauri'

export type AppMenuActions = {
  onExportDxf: () => void
  onUndo: () => void
  onRedo: () => void
  toggleWiringMode: () => void
  toggleFocusMode: () => void
  toggleAlignmentGuides: () => void
  onLoadGoldenDiagram?: () => void
  wiringMode: boolean
  focusMode: boolean
  alignmentGuides: boolean
}

const sep: MenuBarRow = { kind: 'separator' }

function disabledItem(id: string, label: string, shortcut?: string): MenuBarRow {
  return { kind: 'item', id, label, shortcut, disabled: true }
}

export function buildAppMenus(actions: AppMenuActions): AppMenuBarBucket[] {
  if (isTauriApp()) return []

  const shortcutLabels = getShortcutLabels()
  const exitLabel = isMacOS() ? 'Quit Diagramme' : 'Exit Diagramme'

  const fileRows: MenuBarRow[] = [
    disabledItem('file-new', 'New Diagram', shortcutLabels.newDiagram),
    sep,
    disabledItem('file-open', 'Open Diagram…', shortcutLabels.open),
    disabledItem('file-save', 'Save', shortcutLabels.save),
    disabledItem('file-save-as', 'Save As…', shortcutLabels.saveAs),
    disabledItem('file-grouped-report', 'Grouped Inventory Report'),
    disabledItem('file-device-tags-report', 'Device Tags Report'),
    disabledItem('file-plate-connections-report', 'Plate Connections Report'),
    disabledItem('file-export-grouped', 'Export Grouped Inventory (.xlsx)'),
    disabledItem('file-export-device-tags', 'Export Device Tags (.xlsx)'),
    disabledItem('file-export-plate-connections', 'Export Plate Connections (.xlsx)'),
    disabledItem('file-export-equipment', 'Export Equipment List (.xlsx)'),
    {
      kind: 'item',
      id: 'file-export-dxf',
      label: 'Export DXF (Revit)',
      onSelect: actions.onExportDxf,
    },
    sep,
    disabledItem('file-exit', exitLabel, shortcutLabels.quitApp),
  ]

  const editRows: MenuBarRow[] = [
    {
      kind: 'item',
      id: 'edit-undo',
      label: 'Undo',
      shortcut: shortcutLabels.undo,
      onSelect: actions.onUndo,
    },
    {
      kind: 'item',
      id: 'edit-redo',
      label: 'Redo',
      shortcut: shortcutLabels.redo,
      onSelect: actions.onRedo,
    },
    sep,
    disabledItem('edit-cut', 'Cut', shortcutLabels.cut),
    disabledItem('edit-copy', 'Copy', shortcutLabels.copy),
    disabledItem('edit-paste', 'Paste', shortcutLabels.paste),
    disabledItem('edit-duplicate', 'Duplicate', shortcutLabels.duplicate),
    disabledItem('edit-select-all', 'Select All', shortcutLabels.selectAll),
  ]

  const diagramViewRows: MenuBarRow[] = [
    {
      kind: 'item',
      id: 'view-wiring-mode',
      label: actions.wiringMode ? 'Wiring mode: On' : 'Wiring mode: Off',
      shortcut: 'W',
      onSelect: actions.toggleWiringMode,
    },
    {
      kind: 'item',
      id: 'view-alignment-guides',
      label: actions.alignmentGuides ? 'Alignment guides: On' : 'Alignment guides: Off',
      onSelect: actions.toggleAlignmentGuides,
    },
    {
      kind: 'item',
      id: 'view-focus-mode',
      label: actions.focusMode ? 'Focus mode: On' : 'Focus mode: Off',
      shortcut: 'F',
      onSelect: actions.toggleFocusMode,
    },
    disabledItem('view-wire-geometry-overlay', 'Show Wire Geometry Overlay'),
    sep,
    disabledItem('view-show-page-boundary', 'Show Page Boundary'),
    disabledItem('view-page-boundary-settings', 'Page Boundary Settings…'),
    disabledItem('view-fit-page-boundary', 'Fit View to Page'),
  ]

  const helpRows: MenuBarRow[] = [
    disabledItem('help-user-guide', 'User Guide…'),
    ...(actions.onLoadGoldenDiagram
      ? [
          {
            kind: 'item' as const,
            id: 'help-load-golden',
            label: 'Load Comp Gym (dev)…',
            onSelect: actions.onLoadGoldenDiagram,
          },
        ]
      : []),
    sep,
    disabledItem('help-about-show', 'About Diagramme'),
  ]

  return [
    { id: 'file', title: 'File', rows: fileRows },
    { id: 'edit', title: 'Edit', rows: editRows },
    { id: 'view', title: 'View', rows: diagramViewRows },
    { id: 'help', title: 'Help', rows: helpRows },
  ]
}
