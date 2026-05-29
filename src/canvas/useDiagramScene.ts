import { useCallback, useEffect, useRef, useState } from 'react'

import { getDiagramScene, openDiagram } from '../tauriIpc'
import type { SceneJson } from './sceneTypes'

const DEFAULT_DEBOUNCE_MS = 120

export function useDiagramScene() {
  const [scene, setScene] = useState<SceneJson | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const refreshGeneration = useRef(0)

  const clearDebounce = useCallback(() => {
    if (debounceTimer.current != null) {
      clearTimeout(debounceTimer.current)
      debounceTimer.current = null
    }
  }, [])

  const refreshScene = useCallback(async () => {
    const generation = ++refreshGeneration.current
    setBusy(true)
    try {
      const next = await getDiagramScene()
      if (generation !== refreshGeneration.current) return next
      setScene(next)
      setError(null)
      return next
    } catch (err) {
      if (generation === refreshGeneration.current) {
        setError(String(err))
      }
      throw err
    } finally {
      if (generation === refreshGeneration.current) {
        setBusy(false)
      }
    }
  }, [])

  const refreshSceneDebounced = useCallback(
    (debounceMs = DEFAULT_DEBOUNCE_MS) => {
      clearDebounce()
      debounceTimer.current = setTimeout(() => {
        debounceTimer.current = null
        void refreshScene()
      }, debounceMs)
    },
    [clearDebounce, refreshScene],
  )

  const loadDiagramJson = useCallback(
    async (json: string) => {
      await openDiagram(json)
      return refreshScene()
    },
    [refreshScene],
  )

  useEffect(() => clearDebounce, [clearDebounce])

  return {
    scene,
    error,
    busy,
    refreshScene,
    refreshSceneDebounced,
    loadDiagramJson,
  }
}
