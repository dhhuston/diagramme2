import { useCallback, useEffect, useRef, useState } from 'react'

import { getDiagramScene, getDiagramScenePatch, openDiagram } from '../tauriIpc'
import { applyScenePatch } from './applyScenePatch'
import type { SceneJson } from './sceneTypes'

const DEFAULT_DEBOUNCE_MS = 120

export function useDiagramScene() {
  const [scene, setScene] = useState<SceneJson | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [fitRevision, setFitRevision] = useState(0)
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const refreshGeneration = useRef(0)
  const previewGeneration = useRef(0)

  const clearDebounce = useCallback(() => {
    if (debounceTimer.current != null) {
      clearTimeout(debounceTimer.current)
      debounceTimer.current = null
    }
  }, [])

  const publishScene = useCallback((next: SceneJson) => {
    setScene(next)
    setError(null)
  }, [])

  /** Scene refresh during drag — drops stale IPC responses. */
  const refreshSceneQuiet = useCallback(async (generation?: number) => {
    const next = await getDiagramScene()
    if (generation != null && generation !== previewGeneration.current) {
      return next
    }
    publishScene(next)
    return next
  }, [publishScene])

  /** Merge a scene patch during drag — avoids full scene rebuild + IPC payload. */
  const refreshScenePatchQuiet = useCallback(async (nodeId: string, generation?: number) => {
    const patch = await getDiagramScenePatch(nodeId)
    if (generation != null && generation !== previewGeneration.current) {
      return patch
    }
    setScene((prev) => (prev == null ? prev : applyScenePatch(prev, patch)))
    setError(null)
    return patch
  }, [])

  const beginDragPreview = useCallback(() => {
    previewGeneration.current += 1
    return previewGeneration.current
  }, [])

  const refreshScene = useCallback(async () => {
    const generation = ++refreshGeneration.current
    setBusy(true)
    try {
      const next = await getDiagramScene()
      if (generation !== refreshGeneration.current) return next
      publishScene(next)
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
  }, [publishScene])

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

  const bumpFitToScene = useCallback(() => {
    setFitRevision((revision) => revision + 1)
  }, [])

  const loadDiagramJson = useCallback(
    async (json: string) => {
      await openDiagram(json)
      const next = await refreshScene()
      bumpFitToScene()
      return next
    },
    [bumpFitToScene, refreshScene],
  )

  useEffect(() => clearDebounce, [clearDebounce])

  // Rust starts with ProjectState::default() (Main sheet); load scene on first paint.
  useEffect(() => {
    void refreshScene().then(() => bumpFitToScene())
  }, [bumpFitToScene, refreshScene])

  return {
    scene,
    error,
    busy,
    fitRevision,
    beginDragPreview,
    bumpFitToScene,
    refreshScene,
    refreshSceneQuiet,
    refreshScenePatchQuiet,
    refreshSceneDebounced,
    loadDiagramJson,
  }
}
