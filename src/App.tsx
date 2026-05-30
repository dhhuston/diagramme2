import { useCallback, useState } from 'react'

import { DiagramStage } from './canvas/DiagramStage'
import type { HitTarget, PointPx } from './canvas/sceneTypes'
import { useDiagramScene } from './canvas/useDiagramScene'
import { exportRevitDxf, moveNode } from './tauriIpc'
import './App.css'

export default function App() {
  const {
    scene,
    error,
    busy,
    fitRevision,
    beginDragPreview,
    applyScenePatchQuiet,
    loadDiagramJson,
    refreshScene,
  } = useDiagramScene()
  const [status, setStatus] = useState<string | null>(null)
  const [selectedHit, setSelectedHit] = useState<HitTarget | null>(null)

  const loadGoldenDiagram = useCallback(async () => {
    setStatus(null)
    setSelectedHit(null)
    try {
      const json = await fetch('/fixtures/golden/Comp Gym F102A.diagramme').then((r) => {
        if (!r.ok) throw new Error(`fixture fetch ${r.status}`)
        return r.text()
      })
      const next = await loadDiagramJson(json)
      setStatus(`Loaded Comp Gym (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Load failed: ${String(err)}`)
    }
  }, [loadDiagramJson])

  const handleExportDxf = useCallback(async () => {
    setStatus(null)
    try {
      const dxf = await exportRevitDxf()
      setStatus(`DXF exported (${dxf.length.toLocaleString()} chars)`)
    } catch (err) {
      setStatus(`Export failed: ${String(err)}`)
    }
  }, [])

  const handleHit = useCallback((hit: HitTarget | null) => {
    setSelectedHit(hit)
    if (hit?.node_id) {
      setStatus(`Selected node ${hit.node_id}`)
    } else if (hit) {
      setStatus(`Selected ${hit.id}`)
    }
  }, [])

  const handleRefreshScene = useCallback(async () => {
    setStatus(null)
    try {
      const next = await refreshScene()
      setStatus(`Scene refreshed (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Refresh failed: ${String(err)}`)
    }
  }, [refreshScene])

  const handleNodeDragPreview = useCallback(
    async (nodeId: string, position: PointPx) => {
      const gen = beginDragPreview()
      await moveNode(nodeId, position, null, true)
      await applyScenePatchQuiet(nodeId, gen)
    },
    [applyScenePatchQuiet, beginDragPreview],
  )

  const handleNodeMoveCommit = useCallback(
    async (nodeId: string, position: PointPx) => {
      beginDragPreview()
      await moveNode(nodeId, position)
      const next = await refreshScene()
      setStatus(`Moved ${nodeId} → (${position.x}, ${position.y}); ${next.primitives.length} primitives`)
    },
    [beginDragPreview, refreshScene],
  )

  const displayStatus = error ?? status

  return (
    <div className="app-shell">
      <header className="app-toolbar">
        <h1>Diagramme v2</h1>
        <div className="app-toolbar-actions">
          <button type="button" disabled={busy} onClick={() => void loadGoldenDiagram()}>
            Load Comp Gym
          </button>
          <button type="button" disabled={busy || !scene} onClick={() => void handleRefreshScene()}>
            Refresh scene
          </button>
          <button type="button" disabled={busy || !scene} onClick={() => void handleExportDxf()}>
            Export DXF
          </button>
        </div>
        {displayStatus ? <p className="app-status">{displayStatus}</p> : null}
        {selectedHit?.node_id ? (
          <p className="app-status">Hit: {selectedHit.node_id}</p>
        ) : null}
      </header>
      <main className="app-canvas">
        {scene ? (
          <DiagramStage
            scene={scene}
            fitRevision={fitRevision}
            onHit={handleHit}
            onNodeDragPreview={handleNodeDragPreview}
            onNodeMoveCommit={handleNodeMoveCommit}
          />
        ) : (
          <div className="app-placeholder">
            <p>Load Comp Gym to render the Rust scene on Konva.</p>
          </div>
        )}
      </main>
    </div>
  )
}
