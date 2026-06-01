import { useCallback, useState } from 'react'

import { AppShell } from './AppShell'
import { DiagramStage } from './canvas/DiagramStage'
import type { HitTarget, PointPx } from './canvas/sceneTypes'
import { useDiagramScene } from './canvas/useDiagramScene'
import type { PortEndpoint } from './canvas/interaction/connectPorts'
import type { AppMenuCommand } from './appMenuCommands'
import { exportRevitDxf, addEdge, moveNode, undo, redo } from './tauriIpc'

export default function App() {
  const {
    scene,
    error,
    busy,
    fitRevision,
    beginDragPreview,
    loadDiagramJson,
    refreshScene,
    refreshScenePatchQuiet,
    bumpFitToScene,
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
  }, [])

  const handleRefreshScene = useCallback(async () => {
    setStatus(null)
    try {
      const next = await refreshScene()
      bumpFitToScene()
      setStatus(`Scene refreshed (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Refresh failed: ${String(err)}`)
    }
  }, [bumpFitToScene, refreshScene])

  const handleUndo = useCallback(async () => {
    setStatus(null)
    try {
      await undo()
      const next = await refreshScene()
      bumpFitToScene()
      setStatus(`Undo (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Undo failed: ${String(err)}`)
    }
  }, [bumpFitToScene, refreshScene])

  const handleRedo = useCallback(async () => {
    setStatus(null)
    try {
      await redo()
      const next = await refreshScene()
      bumpFitToScene()
      setStatus(`Redo (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Redo failed: ${String(err)}`)
    }
  }, [bumpFitToScene, refreshScene])

  const handleNodeDragPreview = useCallback(
    async (nodeId: string, position: PointPx) => {
      const gen = beginDragPreview()
      await moveNode(nodeId, position, null, true)
      await refreshScenePatchQuiet(nodeId, gen)
    },
    [beginDragPreview, refreshScenePatchQuiet],
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

  const handlePortConnect = useCallback(
    async (from: PortEndpoint, to: PortEndpoint) => {
      await addEdge({
        source: from.nodeId,
        target: to.nodeId,
        sourceHandle: from.handleId,
        targetHandle: to.handleId,
      })
      const next = await refreshScene()
      setStatus(
        `Connected ${from.handleId} → ${to.handleId}; ${next.primitives.length} primitives`,
      )
    },
    [refreshScene],
  )

  const handleMenuUnavailable = useCallback((command: AppMenuCommand) => {
    setStatus(`Not available yet (${command})`)
  }, [])

  const displayStatus = error ?? status

  return (
    <AppShell
      scene={scene}
      selectedHit={selectedHit}
      status={displayStatus}
      busy={busy}
      onExportDxf={handleExportDxf}
      onUndo={handleUndo}
      onRedo={handleRedo}
      onRefreshScene={handleRefreshScene}
      onLoadGoldenDiagram={loadGoldenDiagram}
      onMenuUnavailable={handleMenuUnavailable}
      onClearSelection={() => setSelectedHit(null)}
      canvas={
        scene ? (
          <DiagramStage
            scene={scene}
            selectedHit={selectedHit}
            fitRevision={fitRevision}
            onHit={handleHit}
            onNodeDragPreview={handleNodeDragPreview}
            onNodeMoveCommit={handleNodeMoveCommit}
            onPortConnect={handlePortConnect}
          />
        ) : null
      }
    />
  )
}
