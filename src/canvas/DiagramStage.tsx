import { useEffect, useRef, useState } from 'react'
import type { KonvaEventObject } from 'konva/lib/Node'
import { Layer, Stage } from 'react-konva'

import { DiagramGrid } from './DiagramGrid'
import { GroupingZoneShapeEditOverlay } from './GroupingZoneShapeEditOverlay'
import { SchematicFaceMasks } from './SchematicFaceMasks'
import { hitTestSceneForInteraction } from './hitTest'
import { stagePointerToDiagramPx } from './hitTest'
import { useDiagramInteraction } from './interaction/useDiagramInteraction'
import {
  useGroupingZoneShapeEdit,
  type GroupingZoneShapeEditHandlers,
} from './interaction/useGroupingZoneShapeEdit'
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
  diagramNodes?: import('../tauriIpc').FlowNode[]
  groupingZoneShapeEdit?: GroupingZoneShapeEditHandlers
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
  diagramNodes = [],
  groupingZoneShapeEdit,
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

  const shapeEdit = useGroupingZoneShapeEdit(
    groupingZoneShapeEdit ?? { nodes: diagramNodes },
    viewport.scale,
  )

  const diagramPointFromEvent = (
    event: KonvaEventObject<PointerEvent | MouseEvent>,
  ): PointPx | null => {
    const stage = event.target.getStage()
    const pointer = stage?.getPointerPosition()
    if (!pointer) return null
    return stagePointerToDiagramPx(pointer, viewport)
  }

  const {
    nodeDrag,
    wireConnect,
    activeWireGripId,
    handlePointerDown: interactionPointerDown,
    handlePointerMove: interactionPointerMove,
    handlePointerUp: interactionPointerUp,
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
    if (!shapeEdit.editingNodeId) return
    if (!selectedHit?.node_id || selectedHit.node_id !== shapeEdit.editingNodeId) {
      shapeEdit.exitEdit()
    }
  }, [selectedHit?.node_id, shapeEdit.editingNodeId, shapeEdit.exitEdit])

  useEffect(() => {
    if (!shapeEdit.isEditing) return
    const onWindowPointerUp = () => {
      void shapeEdit.handlePointerUp()
    }
    window.addEventListener('pointerup', onWindowPointerUp)
    return () => window.removeEventListener('pointerup', onWindowPointerUp)
  }, [shapeEdit.isEditing, shapeEdit.handlePointerUp])

  const handlePointerDown = (event: KonvaEventObject<PointerEvent>) => {
    const diagramPoint = diagramPointFromEvent(event)
    if (
      diagramPoint &&
      shapeEdit.handlePointerDown(diagramPoint, event.evt.clientX, event.evt.clientY)
    ) {
      event.evt.preventDefault()
      return
    }
    if (shapeEdit.isEditing) return
    interactionPointerDown(event)
  }

  const handlePointerMove = (event: KonvaEventObject<PointerEvent>) => {
    const diagramPoint = diagramPointFromEvent(event)
    if (
      diagramPoint &&
      shapeEdit.handlePointerMove(diagramPoint, event.evt.clientX, event.evt.clientY)
    ) {
      return
    }
    interactionPointerMove(event)
  }

  const handlePointerUp = () => {
    if (shapeEdit.isEditing) {
      void shapeEdit.handlePointerUp()
      return
    }
    interactionPointerUp()
  }

  const handleStageDoubleClick = (event: KonvaEventObject<MouseEvent>) => {
    const diagramPoint = diagramPointFromEvent(event)
    if (!diagramPoint) return

    if (shapeEdit.editingNodeId) {
      void shapeEdit.tryVertexDoubleClick(diagramPoint)
      return
    }

    const hit = hitTestSceneForInteraction(scene.hits, diagramPoint, selectedHit?.edge_id ?? null)
    if (shapeEdit.tryEnterOnDoubleClick(hit)) {
      onHit?.(hit)
    }
  }

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
        onDblClick={handleStageDoubleClick}
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
          {shapeEdit.editingNode ? (
            <GroupingZoneShapeEditOverlay
              node={shapeEdit.editingNode}
              liveRect={shapeEdit.liveRect}
              polyPairs={shapeEdit.currentPolyPairs()}
            />
          ) : null}
          <WireConnectOverlay preview={wireConnect} />
        </Layer>
      </Stage>
    </div>
  )
}
