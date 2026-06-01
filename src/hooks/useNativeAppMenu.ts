import { useEffect, useRef } from 'react'

import type { AppMenuCommand } from '../appMenuCommands'
import { setAppMenuCommandHandler } from '../tauriMenuBridge'

export type NativeAppMenuHandlers = {
  onExportDxf: () => void | Promise<void>
  onUndo: () => void | Promise<void>
  onRedo: () => void | Promise<void>
  onLoadGoldenDiagram?: () => void | Promise<void>
  toggleWiringMode: () => void
  toggleFocusMode: () => void
  toggleAlignmentGuides: () => void
  onUnavailable?: (command: AppMenuCommand) => void
}

export function useNativeAppMenu(handlers: NativeAppMenuHandlers) {
  const handlersRef = useRef(handlers)
  handlersRef.current = handlers

  useEffect(() => {
    setAppMenuCommandHandler((command) => {
      const h = handlersRef.current
      switch (command) {
        case 'file.exportRevitDxf':
          void h.onExportDxf()
          return
        case 'edit.undo':
          void h.onUndo()
          return
        case 'edit.redo':
          void h.onRedo()
          return
        case 'view.toggleWiringMode':
          h.toggleWiringMode()
          return
        case 'view.toggleFocusMode':
          h.toggleFocusMode()
          return
        case 'view.toggleAlignmentGuides':
          h.toggleAlignmentGuides()
          return
        case 'help.loadDxfExportTestDiagram':
          h.onLoadGoldenDiagram?.()
          return
        default:
          h.onUnavailable?.(command)
      }
    })
    return () => setAppMenuCommandHandler(null)
  }, [])
}
