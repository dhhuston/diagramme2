import { useEffect, useRef, useState } from 'react'
import { Layer, Stage } from 'react-konva'

import { DiagramGrid } from './DiagramGrid'
import { SchematicFaceMasks } from './SchematicFaceMasks'
import { useDiagramInteraction } from './interaction/useDiagramInteraction'
import { useCanvasPreferences } from '../hooks/useCanvasPreferences'
import type { PortEndpoint } from './interaction/connectPorts'
import { fitExtentToStage } from './sceneRenderUtils'
import { SceneRenderer } from './SceneRenderer'
import type { HitTarget, PointPx, SceneJson } from './sceneTypes'
import { useViewport } from './useViewport'
import { WireConnectOverlay } from './WireConnectOverlay'
import { WireRoutingGrips } from './WireRoutingGrips'
import type { WireSegmentAdjustHandlers } from './interaction/useWireSegmentAdjust'

type DiagramStageProps = {
  scene: SceneJson
  selectedHit?: HitTarget | null
  /** Increment when a new diagram loads to fit the viewport once. */
  fitRevision: number
  onHit?: (hit: HitTarget | null) => void
  onNodeDragPreview?: (nodeId: string, position: PointPx) => void | Promise<void>
  onNodeMoveCommit?: (nodeId: string, position: PointPx) => void | Promise<void>
  onPortConnect?: (from: PortEndpoint, to: PortEndpoint) => void | Promise<void>
  wireSegmentAdjust?: WireSegmentAdjustHandlers
}

/** Konva stage: 1 diagram px = 1 unit at scale 1; wheel zoom + drag pan. */
export function DiagramStage({
  scene,
  selectedHit = null,
  fitRevision,
  onHit,
  onNodeDragPreview,
  onNodeMoveCommit,
  onPortConnect,
  wireSegmentAdjust,
}: DiagramStageProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ width: 800, height: 600 })
  const { viewport, setFit, setPan, onWheel } = useViewport()
  const { showGrid } = useCanvasPreferences()

  const {
    nodeDrag,
    wireConnect,
    activeWireGripId,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    handleStageClick,
  } = useDiagramInteraction({
    scene,
    viewport,
    selectedEdgeId: selectedHit?.edge_id ?? null,
    hostRef,
    onHit,
    onNodeDragPreview,
    onNodeMoveCommit,
    onPortConnect,
    onPan: setPan,
    wireSegmentAdjust,
  })

  useEffect(() => {
    const el = hostRef.current
    if (!el) return

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) return
      const { width, height } = entry.contentRect
      setSize({ width: Math.max(width, 1), height: Math.max(height, 1) })
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  // Fit on diagram load and window resize — not on drag scene rebuilds.
  useEffect(() => {
    setFit(fitExtentToStage(scene.extent, size.width, size.height))
  }, [fitRevision, size.width, size.height, setFit])

  return (
    <div ref={hostRef} className="diagram-stage-host">
      <Stage
        width={size.width}
        height={size.height}
        onWheel={onWheel}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onClick={handleStageClick}
      >
        <Layer
          x={viewport.x}
          y={viewport.y}
          scaleX={viewport.scale}
          scaleY={viewport.scale}
          clearBeforeDraw
          hitGraphEnabled={false}
        >
          {showGrid ? (
            <DiagramGrid
              extent={scene.extent}
              viewport={viewport}
              stageWidth={size.width}
              stageHeight={size.height}
            />
          ) : null}
          <SchematicFaceMasks hits={scene.hits} />
          <SceneRenderer scene={scene} selectedHit={selectedHit} nodeDrag={nodeDrag} />
          <WireRoutingGrips
            hits={scene.hits}
            selectedEdgeId={selectedHit?.edge_id ?? null}
            activeGripId={activeWireGripId}
          />
          <WireConnectOverlay preview={wireConnect} />
        </Layer>
      </Stage>
    </div>
  )
}
