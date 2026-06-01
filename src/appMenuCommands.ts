/** Native menu command ids (Tauri `app-menu-command` payload). */
export type AppMenuCommand =
  | 'file.new'
  | 'file.open'
  | 'app.quit'
  | 'file.save'
  | 'file.saveAs'
  | 'file.groupedInventoryReport'
  | 'file.deviceTagsReport'
  | 'file.plateConnectionsReport'
  | 'file.exportGroupedInventoryXlsx'
  | 'file.exportDeviceTagsXlsx'
  | 'file.exportPlateConnectionsXlsx'
  | 'file.exportEquipmentXlsx'
  | 'file.exportRevitDxf'
  | 'edit.undo'
  | 'edit.redo'
  | 'edit.copy'
  | 'edit.cut'
  | 'edit.paste'
  | 'edit.duplicate'
  | 'edit.selectAll'
  | 'view.toggleWiringMode'
  | 'view.toggleAlignmentGuides'
  | 'view.toggleFocusMode'
  | 'view.toggleWireGeometryOverlay'
  | 'view.togglePageBoundary'
  | 'view.pageBoundarySettings'
  | 'view.fitPageBoundary'
  | 'about.show'
  | 'help.userGuide'
  | 'help.loadDxfExportTestDiagram'

const MENU_COMMANDS = new Set<string>([
  'file.new',
  'file.open',
  'file.save',
  'file.saveAs',
  'file.groupedInventoryReport',
  'file.deviceTagsReport',
  'file.plateConnectionsReport',
  'file.exportGroupedInventoryXlsx',
  'file.exportDeviceTagsXlsx',
  'file.exportPlateConnectionsXlsx',
  'file.exportEquipmentXlsx',
  'file.exportRevitDxf',
  'edit.undo',
  'edit.redo',
  'edit.copy',
  'edit.cut',
  'edit.paste',
  'edit.duplicate',
  'edit.selectAll',
  'view.toggleWiringMode',
  'view.toggleAlignmentGuides',
  'view.toggleFocusMode',
  'view.toggleWireGeometryOverlay',
  'view.togglePageBoundary',
  'view.pageBoundarySettings',
  'view.fitPageBoundary',
  'about.show',
  'help.userGuide',
  'help.loadDxfExportTestDiagram',
])

export function isAppMenuCommand(id: string): id is AppMenuCommand {
  return MENU_COMMANDS.has(id)
}
