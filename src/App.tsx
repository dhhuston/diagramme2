import { useCallback, useMemo, useState } from 'react'

import { AppShell } from './AppShell'
import { DiagramStage } from './canvas/DiagramStage'
import type { HitTarget, PointPx } from './canvas/sceneTypes'
import { useDiagramScene } from './canvas/useDiagramScene'
import type { PortEndpoint } from './canvas/interaction/connectPorts'
import type { AppMenuCommand } from './appMenuCommands'
import { DEV_FIXTURES, fetchDevFixture } from './devFixtures'
import type { WireSegmentArm } from './canvas/interaction/useWireSegmentAdjust'
import { exportRevitDxf, addEdge, moveNode, undo, redo, dragWireSegment, updateEdgeInnerCorners, getState, getWireInnerChain } from './tauriIpc'

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

  const loadDevFixture = useCallback(
    async (path: string, label: string) => {
      setStatus(null)
      setSelectedHit(null)
      try {
        const json = await fetchDevFixture(path)
        const next = await loadDiagramJson(json)
        setStatus(`Loaded ${label} (${next.primitives.length} primitives)`)
      } catch (err) {
        setStatus(`Load failed: ${String(err)}`)
      }
    },
    [loadDiagramJson],
  )

  const loadGoldenDiagram = useCallback(
    () => loadDevFixture(DEV_FIXTURES.compGym, 'Comp Gym'),
    [loadDevFixture],
  )
  const loadCafeteriaDiagram = useCallback(
    () => loadDevFixture(DEV_FIXTURES.cafeteria, 'Cafeteria D104A'),
    [loadDevFixture],
  )
  const loadSplitFaceDemoDiagram = useCallback(
    () => loadDevFixture(DEV_FIXTURES.splitFaceDemo, 'Split face demo'),
    [loadDevFixture],
  )

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

  const readEdgeInnerCorners = useCallback(async (edgeId: string) => {
    const state = await getState()
    const edge = state.edges.find((e) => e.id === edgeId)
    const data = edge?.data as { innerCorners?: PointPx[] } | undefined
    return data?.innerCorners
  }, [])

  const readWireInnerChain = useCallback(async (edgeId: string) => {
    const chain = await getWireInnerChain(edgeId)
    return chain ?? undefined
  }, [])

  const handleWireSegmentPreview = useCallback(
    async (arm: WireSegmentArm, delta: PointPx) => {
      await dragWireSegment(
        arm.edgeId,
        arm.segmentIndex,
        arm.orientation,
        delta,
        arm.chain0,
        true,
      )
      await refreshScene()
    },
    [refreshScene],
  )

  const handleWireSegmentCommit = useCallback(
    async (arm: WireSegmentArm, delta: PointPx) => {
      await dragWireSegment(
        arm.edgeId,
        arm.segmentIndex,
        arm.orientation,
        delta,
        arm.chain0,
        true,
      )
      await dragWireSegment(
        arm.edgeId,
        arm.segmentIndex,
        arm.orientation,
        delta,
        arm.chain0,
        false,
      )
      const next = await refreshScene()
      setStatus(`Wire route updated; ${next.primitives.length} primitives`)
    },
    [refreshScene],
  )

  const handleWireSegmentCancel = useCallback(
    async (arm: WireSegmentArm) => {
      await updateEdgeInnerCorners(arm.edgeId, arm.priorCorners ?? null, false)
      await refreshScene()
    },
    [refreshScene],
  )

  const wireSegmentAdjust = useMemo(
    () => ({
      readEdgeInnerCorners,
      readWireInnerChain,
      onWireSegmentPreview: handleWireSegmentPreview,
      onWireSegmentCommit: handleWireSegmentCommit,
      onWireSegmentCancel: handleWireSegmentCancel,
    }),
    [
      handleWireSegmentCancel,
      handleWireSegmentCommit,
      handleWireSegmentPreview,
      readEdgeInnerCorners,
      readWireInnerChain,
    ],
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
      onLoadCafeteriaDiagram={loadCafeteriaDiagram}
      onLoadSplitFaceDemoDiagram={loadSplitFaceDemoDiagram}
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
            wireSegmentAdjust={wireSegmentAdjust}
          />
        ) : null
      }
    />
  )
}
