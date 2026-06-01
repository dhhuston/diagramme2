import { useCallback, useState } from 'react'

import { isEmptyStateDismissed } from '../components/CanvasEmptyState'

function readStoredBoolean(key: string, fallback: boolean) {
  try {
    const stored = localStorage.getItem(key)
    return stored == null ? fallback : stored !== 'false'
  } catch {
    return fallback
  }
}

function useStoredBoolean(key: string, fallback: boolean) {
  const [value, setValue] = useState(() => readStoredBoolean(key, fallback))

  const toggle = useCallback(() => {
    setValue((current) => {
      const next = !current
      try {
        localStorage.setItem(key, String(next))
      } catch {
        // ignore
      }
      return next
    })
  }, [key])

  return [value, setValue, toggle] as const
}

export function useCanvasPreferences() {
  const [focusMode, setFocusMode] = useState(false)
  const [showGrid, , toggleGrid] = useStoredBoolean('diagramme.grid.show', true)
  const [wiringMode, , toggleWiringMode] = useStoredBoolean('diagramme.wiring.mode', true)
  const [alignmentGuides, , toggleAlignmentGuides] = useStoredBoolean(
    'diagramme.alignment.guides',
    true,
  )
  const [showWireGeometryOverlay, , toggleWireGeometryOverlay] = useStoredBoolean(
    'diagramme.wireGeometry.overlay',
    false,
  )
  const [showEmptyHint, setShowEmptyHint] = useState(() => !isEmptyStateDismissed())

  const toggleFocusMode = useCallback(() => {
    setFocusMode((current) => !current)
  }, [])

  return {
    focusMode,
    setShowEmptyHint,
    showEmptyHint,
    showGrid,
    alignmentGuides,
    toggleFocusMode,
    toggleGrid,
    toggleAlignmentGuides,
    toggleWireGeometryOverlay,
    toggleWiringMode,
    wiringMode,
    showWireGeometryOverlay,
  }
}
