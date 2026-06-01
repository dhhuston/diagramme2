import { useCallback, useEffect, useRef, useState, type MutableRefObject } from 'react'
import type { KonvaEventObject } from 'konva/lib/Node'
import { Layer, Stage } from 'react-konva'

import { DiagramGrid } from './DiagramGrid'
import { GroupingZoneShapeEditOverlay } from './GroupingZoneShapeEditOverlay'
import { SchematicFaceMasks } from './SchematicFaceMasks'
import {
  hitTestGroupingZoneBoundary,
} from './groupingZoneHitTest'
import {
  stagePointerToDiagramPx,
} from './hitTest'
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
import { WireConnectOverlay } from './WireConnectOverlay'
import { WireRoutingGrips } from './WireRoutingGrips'
import type { WireSegmentAdjustHandlers } from './interaction/useWireSegmentAdjust'
import { useViewport } from './useViewport'

const BOUNDARY_TAP_MOVE_PX = 5
const BOUNDARY_DOUBLE_TAP_MS = 450
const BOUNDARY_DOUBLE_TAP_DIAGRAM_PX = 12

type BoundaryTapSession = {
  hit: HitTarget
  diagramPoint: PointPx
  startClientX: number
  startClientY: number
}

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
  /** Updated with a function that returns the diagram point at the stage center. */
  insertPositionRef?: MutableRefObject<(() => PointPx | null) | null>
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
  insertPositionRef,
}: DiagramStageProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ width: 800, height: 600 })
  const { viewport, setFit, setPan, onWheel } = useViewport()
  const { showGrid } = useCanvasPreferences()

  const shapeEdit = useGroupingZoneShapeEdit(
    groupingZoneShapeEdit ?? { nodes: diagramNodes },
    viewport.scale,
    scene.hits,
  )

  const skipExitEditOnceRef = useRef(false)
  const boundaryTapRef = useRef<BoundaryTapSession | null>(null)
  const lastBoundaryTapRef = useRef<{
    at: number
    nodeId: string
    x: number
    y: number
  } | null>(null)
  const suppressNextStageClickRef = useRef(false)

  const zoneNodes = groupingZoneShapeEdit?.nodes ?? diagramNodes

  const boundaryHitAt = useCallback(
    (diagramPoint: PointPx) =>
      hitTestGroupingZoneBoundary(scene.hits, diagramPoint, zoneNodes),
    [scene.hits, zoneNodes],
  )

  const tryEnterGroupingZoneEdit = useCallback(
    (diagramPoint: PointPx, boundaryHit?: HitTarget | null) => {
      if (shapeEdit.editingNodeId) return false
      const hit = boundaryHit ?? boundaryHitAt(diagramPoint)
      if (!hit) return false
      onHit?.(hit)
      skipExitEditOnceRef.current = true
      return shapeEdit.tryEnterOnDoubleClick(hit)
    },
    [boundaryHitAt, onHit, shapeEdit.editingNodeId, shapeEdit.tryEnterOnDoubleClick],
  )

  const finishBoundaryTap = useCallback(
    (session: BoundaryTapSession) => {
      const { hit, diagramPoint } = session
      if (!hit.node_id) return

      onHit?.(hit)
      suppressNextStageClickRef.current = true

      const now = Date.now()
      const last = lastBoundaryTapRef.current
      const isDoubleTap =
        last != null &&
        last.nodeId === hit.node_id &&
        now - last.at <= BOUNDARY_DOUBLE_TAP_MS &&
        Math.hypot(diagramPoint.x - last.x, diagramPoint.y - last.y) <=
          BOUNDARY_DOUBLE_TAP_DIAGRAM_PX

      if (isDoubleTap) {
        lastBoundaryTapRef.current = null
        tryEnterGroupingZoneEdit(diagramPoint, hit)
        return
      }

      lastBoundaryTapRef.current = {
        at: now,
        nodeId: hit.node_id,
        x: diagramPoint.x,
        y: diagramPoint.y,
      }
    },
    [onHit, tryEnterGroupingZoneEdit],
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
    handleStageClick: interactionStageClick,
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
    if (skipExitEditOnceRef.current) {
      skipExitEditOnceRef.current = false
      return
    }
    if (selectedHit?.node_id === shapeEdit.editingNodeId) return
    shapeEdit.exitEdit()
  }, [selectedHit?.node_id, shapeEdit.editingNodeId, shapeEdit.exitEdit])

  useEffect(() => {
    if (!shapeEdit.isEditing) return
    const onWindowPointerUp = () => {
      void shapeEdit.handlePointerUp()
    }
    window.addEventListener('pointerup', onWindowPointerUp)
    return () => window.removeEventListener('pointerup', onWindowPointerUp)
  }, [shapeEdit.isEditing, shapeEdit.handlePointerUp])

  useEffect(() => {
    if (!insertPositionRef) return
    insertPositionRef.current = () => {
      const center = { x: size.width / 2, y: size.height / 2 }
      return stagePointerToDiagramPx(center, viewport)
    }
    return () => {
      insertPositionRef.current = null
    }
  }, [insertPositionRef, size.width, size.height, viewport])

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

    if (diagramPoint) {
      const boundaryHit = boundaryHitAt(diagramPoint)
      if (boundaryHit) {
        boundaryTapRef.current = {
          hit: boundaryHit,
          diagramPoint,
          startClientX: event.evt.clientX,
          startClientY: event.evt.clientY,
        }
        event.evt.preventDefault()
        return
      }
    }

    boundaryTapRef.current = null
    interactionPointerDown(event)
  }

  const handlePointerMove = (event: KonvaEventObject<PointerEvent>) => {
    const boundaryTap = boundaryTapRef.current
    if (boundaryTap) {
      const dx = event.evt.clientX - boundaryTap.startClientX
      const dy = event.evt.clientY - boundaryTap.startClientY
      if (dx * dx + dy * dy > BOUNDARY_TAP_MOVE_PX * BOUNDARY_TAP_MOVE_PX) {
        boundaryTapRef.current = null
        lastBoundaryTapRef.current = null
        interactionPointerDown(event)
        interactionPointerMove(event)
      }
      return
    }

    const diagramPoint = diagramPointFromEvent(event)
    if (
      diagramPoint &&
      shapeEdit.handlePointerMove(diagramPoint, event.evt.clientX, event.evt.clientY)
    ) {
      return
    }
    interactionPointerMove(event)
  }

  const handlePointerUp = (_event: KonvaEventObject<PointerEvent>) => {
    const boundaryTap = boundaryTapRef.current
    if (boundaryTap) {
      boundaryTapRef.current = null
      finishBoundaryTap(boundaryTap)
      return
    }

    if (shapeEdit.isEditing) {
      void shapeEdit.handlePointerUp()
      return
    }
    interactionPointerUp()
  }

  const handleStageClick = (event: KonvaEventObject<MouseEvent>) => {
    if (suppressNextStageClickRef.current) {
      suppressNextStageClickRef.current = false
      return
    }
    interactionStageClick(event)
  }

  const handleStageDoubleClick = (event: KonvaEventObject<MouseEvent>) => {
    const diagramPoint = diagramPointFromEvent(event)
    if (!diagramPoint) return

    if (shapeEdit.editingNodeId) {
      void shapeEdit.tryVertexDoubleClick(diagramPoint)
      return
    }

    const boundaryHit = boundaryHitAt(diagramPoint)
    if (boundaryHit) {
      lastBoundaryTapRef.current = null
      if (tryEnterGroupingZoneEdit(diagramPoint, boundaryHit)) {
        suppressNextStageClickRef.current = true
        event.evt.preventDefault()
      }
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
        onPointerUp={(event) => handlePointerUp(event)}
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
          <SceneRenderer
            scene={scene}
            selectedHit={selectedHit}
            nodeDrag={nodeDrag}
            diagramNodes={groupingZoneShapeEdit?.nodes ?? diagramNodes}
          />
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
