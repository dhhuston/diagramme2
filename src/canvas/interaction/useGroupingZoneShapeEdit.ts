import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import type { FlowNode } from '../../tauriIpc'
import {
  applySegmentDrag,
  defaultOrtho,
  fitPolylineToNodeBounds,
  fromPairs,
  getClosedPolylineSegments,
  insertVertex,
  moveVertex,
  removeVertex,
  toPairs,
  vertexDragAxis,
  type Pt,
} from '../groupingZoneGeometry'
import { isGroupingZoneBoundaryHit, pointInRect } from '../hitTest'
import { nodeSelectionBounds } from '../selectionBounds'
import {
  GROUPING_ZONE_MIN_H,
  GROUPING_ZONE_MIN_W,
  groupingZoneData,
  groupingZoneDimensions,
  groupingZoneShape,
  isGroupingZoneNode,
} from '../groupingZoneNode'
import {
  hitTestRectResizeHandle,
  resizeRectFromHandle,
  type DiagramRect,
  type RectResizeHandle,
} from '../groupingZoneRectResize'
import type { HitTarget, PointPx } from '../sceneTypes'

const DIM_PREVIEW_MS = 80
const VERTEX_PICK_RADIUS = 8
const SEGMENT_PICK_RADIUS = 6

function distPointToSegment(
  px: number,
  py: number,
  ax: number,
  ay: number,
  bx: number,
  by: number,
): number {
  const dx = bx - ax
  const dy = by - ay
  const lenSq = dx * dx + dy * dy
  if (lenSq === 0) {
    const ox = px - ax
    const oy = py - ay
    return Math.sqrt(ox * ox + oy * oy)
  }
  const t = Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / lenSq))
  const cx = ax + t * dx
  const cy = ay + t * dy
  const ox = px - cx
  const oy = py - cy
  return Math.sqrt(ox * ox + oy * oy)
}

export type GroupingZoneShapeEditHandlers = {
  nodes: FlowNode[]
  onRectResizePreview?: (nodeId: string, rect: DiagramRect) => void | Promise<void>
  onRectResizeCommit?: (nodeId: string, rect: DiagramRect) => void | Promise<void>
  onPolylineCommit?: (
    nodeId: string,
    polylinePoints: number[],
    position: PointPx,
    size: { width: number; height: number },
  ) => void | Promise<void>
}

type RectResizeSession = {
  handle: RectResizeHandle
  startRect: DiagramRect
  nodeId: string
}

type VertexDragSession = {
  idx: number
  startClientX: number
  startClientY: number
  pairsSnap: Pt[]
  axis: 'ew' | 'ns' | 'move'
  lockedAxis: 'H' | 'V' | null
  nodePosSnap: PointPx
}

type SegmentDragSession = {
  segIdx: number
  startScreen: number
  pairsSnap: Pt[]
  dir: 'H' | 'V'
  nodePosSnap: PointPx
}

export function useGroupingZoneShapeEdit(
  handlers: GroupingZoneShapeEditHandlers,
  viewportScale: number,
  sceneHits: import('../sceneTypes').HitTarget[] = [],
) {
  const [editingNodeId, setEditingNodeId] = useState<string | null>(null)
  const [liveRect, setLiveRect] = useState<DiagramRect | null>(null)
  const [dragPolyPairs, setDragPolyPairs] = useState<Pt[] | null>(null)

  const rectResizeRef = useRef<RectResizeSession | null>(null)
  const liveRectRef = useRef<DiagramRect | null>(null)
  const vtxDragRef = useRef<VertexDragSession | null>(null)
  const segDragRef = useRef<SegmentDragSession | null>(null)
  const segDidDragRef = useRef(false)
  const vtxDidDragRef = useRef(false)
  const previewTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const lastDiagramPointRef = useRef<PointPx | null>(null)

  const handlersRef = useRef(handlers)
  handlersRef.current = handlers

  const editingNode = useMemo((): FlowNode | undefined => {
    if (!editingNodeId) return undefined
    const fromDiagram = handlers.nodes.find((n) => n.id === editingNodeId)
    if (fromDiagram && isGroupingZoneNode(fromDiagram)) return fromDiagram
    const bounds = nodeSelectionBounds(sceneHits, editingNodeId)
    if (!bounds) return undefined
    const data = handlers.nodes.find((n) => n.id === editingNodeId)?.data ?? {}
    return {
      id: editingNodeId,
      type: 'groupingZone',
      position: { x: bounds.x, y: bounds.y },
      width: bounds.width,
      height: bounds.height,
      data,
    }
  }, [editingNodeId, handlers.nodes, sceneHits])

  const cancelPreviewTimer = useCallback(() => {
    if (previewTimer.current != null) {
      clearTimeout(previewTimer.current)
      previewTimer.current = null
    }
  }, [])

  const exitEdit = useCallback(() => {
    cancelPreviewTimer()
    setEditingNodeId(null)
    setLiveRect(null)
    liveRectRef.current = null
    setDragPolyPairs(null)
    rectResizeRef.current = null
    vtxDragRef.current = null
    segDragRef.current = null
  }, [cancelPreviewTimer])

  const enterEdit = useCallback((nodeId: string) => {
    setEditingNodeId(nodeId)
    setLiveRect(null)
    liveRectRef.current = null
    setDragPolyPairs(null)
  }, [])

  const tryEnterOnDoubleClick = useCallback(
    (hit: HitTarget | null) => {
      if (!hit?.node_id || !isGroupingZoneBoundaryHit(hit)) return false
      enterEdit(hit.node_id)
      return true
    },
    [enterEdit],
  )

  const scheduleRectPreview = useCallback(
    (nodeId: string, rect: DiagramRect) => {
      liveRectRef.current = rect
      setLiveRect(rect)
      cancelPreviewTimer()
      previewTimer.current = setTimeout(() => {
        previewTimer.current = null
        void handlersRef.current.onRectResizePreview?.(nodeId, rect)
      }, DIM_PREVIEW_MS)
    },
    [cancelPreviewTimer],
  )

  const persistPolyline = useCallback(async (pairs: Pt[], basePosition?: PointPx) => {
    if (!editingNodeId || !editingNode || !isGroupingZoneNode(editingNode)) return
    const fitted = fitPolylineToNodeBounds(pairs)
    const pts = fromPairs(fitted.pairs)
    const origin = basePosition ?? editingNode.position
    const position = {
      x: Math.round(origin.x + fitted.offset.x),
      y: Math.round(origin.y + fitted.offset.y),
    }
    await handlersRef.current.onPolylineCommit?.(
      editingNodeId,
      pts,
      position,
      fitted.size,
    )
  }, [editingNode, editingNodeId])

  const currentRect = useCallback((): DiagramRect | null => {
    if (!editingNode || !isGroupingZoneNode(editingNode)) return null
    if (liveRect) return liveRect
    return groupingZoneDimensions(editingNode)
  }, [editingNode, liveRect])

  const currentPolyPairs = useCallback((): Pt[] | null => {
    if (!editingNode || !isGroupingZoneNode(editingNode)) return null
    if (dragPolyPairs) return dragPolyPairs
    const dims = groupingZoneDimensions(editingNode)
    const data = groupingZoneData(editingNode)
    const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
    return toPairs(pts)
  }, [dragPolyPairs, editingNode])

  const handlePointerDown = useCallback(
    (diagramPoint: PointPx, clientX: number, clientY: number): boolean => {
      lastDiagramPointRef.current = diagramPoint
      if (!editingNodeId || !editingNode || !isGroupingZoneNode(editingNode)) return false

      if (groupingZoneShape(editingNode) === 'rect') {
        const rect = liveRect ?? groupingZoneDimensions(editingNode)
        const handle = hitTestRectResizeHandle(diagramPoint, rect)
        if (handle) {
          rectResizeRef.current = { handle, startRect: { ...rect }, nodeId: editingNodeId }
          return true
        }
        if (pointInRect(diagramPoint, rect)) return true
        return false
      }

      const dims = groupingZoneDimensions(editingNode)
      const localX = diagramPoint.x - dims.x
      const localY = diagramPoint.y - dims.y
      const data = groupingZoneData(editingNode)
      const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
      const pairs = dragPolyPairs ?? toPairs(pts)

      for (let i = 0; i < pairs.length; i += 1) {
        const [vx, vy] = pairs[i]
        const dx = localX - vx
        const dy = localY - vy
        if (dx * dx + dy * dy <= VERTEX_PICK_RADIUS * VERTEX_PICK_RADIUS) {
          vtxDidDragRef.current = false
          vtxDragRef.current = {
            idx: i,
            startClientX: clientX,
            startClientY: clientY,
            pairsSnap: pairs.map((pair) => [pair[0], pair[1]] as Pt),
            axis: vertexDragAxis(pairs, i),
            lockedAxis: null,
            nodePosSnap: { ...dims },
          }
          setDragPolyPairs(pairs.map((pair) => [pair[0], pair[1]] as Pt))
          return true
        }
      }

      const segments = getClosedPolylineSegments(pairs)
      for (const seg of segments) {
        if (
          distPointToSegment(localX, localY, seg.ax, seg.ay, seg.bx, seg.by) <=
          SEGMENT_PICK_RADIUS
        ) {
          segDidDragRef.current = false
          segDragRef.current = {
            segIdx: seg.i,
            startScreen: seg.dir === 'V' ? clientX : clientY,
            pairsSnap: pairs.map((pair) => [pair[0], pair[1]] as Pt),
            dir: seg.dir,
            nodePosSnap: { x: dims.x, y: dims.y },
          }
          setDragPolyPairs(pairs.map((pair) => [pair[0], pair[1]] as Pt))
          return true
        }
      }

      if (pointInRect(diagramPoint, dims)) return true

      return false
    },
    [dragPolyPairs, editingNode, editingNodeId, liveRect],
  )

  const handlePointerMove = useCallback(
    (diagramPoint: PointPx, clientX: number, clientY: number) => {
      lastDiagramPointRef.current = diagramPoint
      const rectSession = rectResizeRef.current
      if (rectSession) {
        const next = resizeRectFromHandle(
          rectSession.startRect,
          rectSession.handle,
          diagramPoint,
          GROUPING_ZONE_MIN_W,
          GROUPING_ZONE_MIN_H,
        )
        scheduleRectPreview(rectSession.nodeId, next)
        return true
      }

      const vtx = vtxDragRef.current
      if (vtx) {
        let dx = (clientX - vtx.startClientX) / viewportScale
        let dy = (clientY - vtx.startClientY) / viewportScale
        if (vtx.axis === 'ew') {
          dy = 0
        } else if (vtx.axis === 'ns') {
          dx = 0
        } else {
          if (!vtx.lockedAxis) {
            if (Math.abs(dx) >= 4) vtx.lockedAxis = 'H'
            else if (Math.abs(dy) >= 4) vtx.lockedAxis = 'V'
          }
          if (vtx.lockedAxis === 'H') dy = 0
          else if (vtx.lockedAxis === 'V') dx = 0
          else {
            dx = 0
            dy = 0
          }
        }
        if (Math.abs(dx) >= 2 || Math.abs(dy) >= 2) vtxDidDragRef.current = true
        const [ox, oy] = vtx.pairsSnap[vtx.idx]
        setDragPolyPairs(moveVertex(vtx.pairsSnap, vtx.idx, Math.round(ox + dx), Math.round(oy + dy)))
        return true
      }

      const seg = segDragRef.current
      if (seg) {
        const screenNow = seg.dir === 'V' ? clientX : clientY
        const delta = Math.round((screenNow - seg.startScreen) / viewportScale)
        if (Math.abs(delta) >= 2) segDidDragRef.current = true
        setDragPolyPairs(applySegmentDrag(seg.pairsSnap, seg.segIdx, delta))
        return true
      }

      return false
    },
    [scheduleRectPreview, viewportScale],
  )

  const handleSegmentClick = useCallback(
    async (segIdx: number, localPoint: PointPx) => {
      if (segDidDragRef.current) return
      if (!editingNode || !isGroupingZoneNode(editingNode)) return
      const dims = groupingZoneDimensions(editingNode)
      const data = groupingZoneData(editingNode)
      const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
      const pairs = dragPolyPairs ?? toPairs(pts)
      await persistPolyline(
        insertVertex(pairs, segIdx, localPoint.x, localPoint.y),
        dims,
      )
    },
    [dragPolyPairs, editingNode, persistPolyline],
  )

  const handleVertexDoubleClick = useCallback(
    async (idx: number) => {
      if (vtxDidDragRef.current) return
      if (!editingNode || !isGroupingZoneNode(editingNode)) return
      const dims = groupingZoneDimensions(editingNode)
      const data = groupingZoneData(editingNode)
      const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
      const removed = removeVertex(toPairs(pts), idx)
      if (removed) await persistPolyline(removed, dims)
    },
    [editingNode, persistPolyline],
  )

  const handlePointerUp = useCallback(async () => {
    const pendingPreview = previewTimer.current
    cancelPreviewTimer()

    const rectSession = rectResizeRef.current
    if (rectSession) {
      rectResizeRef.current = null
      const rect = liveRectRef.current ?? rectSession.startRect
      liveRectRef.current = null
      setLiveRect(null)
      // Flush the last preview frame if pointer up beat the throttle timer.
      if (pendingPreview) {
        await handlersRef.current.onRectResizePreview?.(rectSession.nodeId, rect)
      }
      await handlersRef.current.onRectResizeCommit?.(rectSession.nodeId, rect)
      return true
    }

    const vtx = vtxDragRef.current
    if (vtx) {
      vtxDragRef.current = null
      const pairs = dragPolyPairs ?? vtx.pairsSnap
      setDragPolyPairs(null)
      await persistPolyline(pairs, vtx.nodePosSnap)
      return true
    }

    const seg = segDragRef.current
    if (seg) {
      const wasClick = !segDidDragRef.current
      const segIdx = seg.segIdx
      const nodePos = seg.nodePosSnap
      segDragRef.current = null
      if (wasClick) {
        const last = lastDiagramPointRef.current
        if (last && editingNode && isGroupingZoneNode(editingNode)) {
          await handleSegmentClick(segIdx, { x: last.x - nodePos.x, y: last.y - nodePos.y })
        }
      } else {
        const pairs = dragPolyPairs ?? seg.pairsSnap
        await persistPolyline(pairs, nodePos)
      }
      setDragPolyPairs(null)
      return true
    }

    return false
  }, [cancelPreviewTimer, dragPolyPairs, editingNode, handleSegmentClick, persistPolyline])

  const tryVertexDoubleClick = useCallback(
    async (diagramPoint: PointPx) => {
      if (!editingNode || !isGroupingZoneNode(editingNode)) return false
      if (groupingZoneShape(editingNode) !== 'polyline') return false
      const dims = groupingZoneDimensions(editingNode)
      const localX = diagramPoint.x - dims.x
      const localY = diagramPoint.y - dims.y
      const data = groupingZoneData(editingNode)
      const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
      const pairs = dragPolyPairs ?? toPairs(pts)
      for (let i = 0; i < pairs.length; i += 1) {
        const [vx, vy] = pairs[i]
        const dx = localX - vx
        const dy = localY - vy
        if (dx * dx + dy * dy <= VERTEX_PICK_RADIUS * VERTEX_PICK_RADIUS) {
          await handleVertexDoubleClick(i)
          return true
        }
      }
      return false
    },
    [dragPolyPairs, editingNode, handleVertexDoubleClick],
  )

  useEffect(() => {
    if (!editingNodeId) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') exitEdit()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [editingNodeId, exitEdit])

  return {
    editingNodeId,
    editingNode: isGroupingZoneNode(editingNode) ? editingNode : null,
    liveRect,
    dragPolyPairs,
    currentRect,
    currentPolyPairs,
    enterEdit,
    exitEdit,
    tryEnterOnDoubleClick,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    handleSegmentClick,
    handleVertexDoubleClick,
    tryVertexDoubleClick,
    isEditing: editingNodeId != null,
  }
}
