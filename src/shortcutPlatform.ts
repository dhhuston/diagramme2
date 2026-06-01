import { isMacOS } from './utils/isTauri'

export { isMacOS }

export type ShortcutLabels = {
  primary: string
  save: string
  saveAs: string
  open: string
  newDiagram: string
  undo: string
  redo: string
  copy: string
  cut: string
  paste: string
  duplicate: string
  selectAll: string
  quitApp: string
}

export function getShortcutLabels(): ShortcutLabels {
  const mac = isMacOS()
  const primary = mac ? 'Cmd' : 'Ctrl'

  return {
    primary,
    save: `${primary}+S`,
    saveAs: `${primary}+Shift+S`,
    open: `${primary}+O`,
    newDiagram: `${primary}+N`,
    undo: `${primary}+Z`,
    redo: mac ? 'Cmd+Shift+Z' : 'Ctrl+Y',
    copy: `${primary}+C`,
    cut: `${primary}+X`,
    paste: `${primary}+V`,
    duplicate: `${primary}+D`,
    selectAll: `${primary}+A`,
    quitApp: mac ? '⌘Q' : 'Ctrl+Q',
  }
}
