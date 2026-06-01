import { useCallback, useEffect, useRef, useState } from 'react'
import type { KonvaEventObject } from 'konva/lib/Node'

import { hitTestScene, stagePointerToDiagramPx } from '../hitTest'
import type { HitTarget, PointPx, SceneJson } from '../sceneTypes'
import type { Viewport } from '../useViewport'
import { nodeBodyOrigin, snapPoint, type NodeDragTarget } from './dragNode'
import {
  canConnectPorts,
  portCenterFromHit,
  portFromHit,
  type PortEndpoint,
} from './connectPorts'

const WIRE_PREVIEW_MS = 60

export type WireConnectPreview = {
  from: PortEndpoint
  fromPoint: PointPx
  toPoint: PointPx
}

type UseDiagramInteractionOptions = {
  scene: SceneJson
  viewport: Viewport
  onHit?: (hit: HitTarget | null) => void
  /** Coalesced Rust preview — updates node position + wire routing in scene. */
  onNodeDragPreview?: (nodeId: string, position: PointPx) => void | Promise<void>
  onNodeMoveCommit?: (nodeId: string, position: PointPx) => void | Promise<void>
  onPortConnect?: (from: PortEndpoint, to: PortEndpoint) => void | Promise<void>
  onPan: (next: Pick<Viewport, 'x' | 'y'>) => void
}

type PanSession = {
  startPointer: PointPx
  startViewport: PointPx
}

type DragSession = {
  nodeId: string
  targetOrigin: PointPx
}

type WireConnectSession = {
  from: PortEndpoint
  fromPoint: PointPx
}

export function useDiagramInteraction({
  scene,
  viewport,
  onHit,
  onNodeDragPreview,
  onNodeMoveCommit,
  onPortConnect,
  onPan,
}: UseDiagramInteractionOptions) {
  const dragSession = useRef<DragSession | null>(null)
  const dragGrabOffset = useRef<PointPx | null>(null)
  const [nodeDrag, setNodeDrag] = useState<NodeDragTarget | null>(null)
  const wireSession = useRef<WireConnectSession | null>(null)
  const [wireConnect, setWireConnect] = useState<WireConnectPreview | null>(null)
  const lastDiagramPoint = useRef<PointPx | null>(null)
  const panSession = useRef<PanSession | null>(null)
  const movedDuringGesture = useRef(false)
  const previewFrame = useRef<number | null>(null)
  const previewTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const visualFrame = useRef<number | null>(null)
  const pendingTarget = useRef<PointPx | null>(null)

  const pointerOnStage = useCallback(
    (stage: { getPointerPosition: () => PointPx | null } | null | undefined) => {
      const pointer = stage?.getPointerPosition()
      if (!pointer) return null
      return stagePointerToDiagramPx(pointer, viewport)
    },
    [viewport],
  )

  const cancelPreviewFrame = useCallback(() => {
    if (previewFrame.current != null) {
      cancelAnimationFrame(previewFrame.current)
      previewFrame.current = null
    }
    if (previewTimer.current != null) {
      clearTimeout(previewTimer.current)
      previewTimer.current = null
    }
  }, [])

  const cancelVisualFrame = useCallback(() => {
    if (visualFrame.current != null) {
      cancelAnimationFrame(visualFrame.current)
      visualFrame.current = null
    }
  }, [])

  const endWireConnect = useCallback(() => {
    wireSession.current = null
    setWireConnect(null)
  }, [])

  const queueVisualUpdate = useCallback(() => {
    if (visualFrame.current != null) {
      return
    }
    visualFrame.current = requestAnimationFrame(() => {
      visualFrame.current = null
      const session = dragSession.current
      if (!session) {
        return
      }
      setNodeDrag({
        nodeId: session.nodeId,
        targetOrigin: { ...session.targetOrigin },
      })
    })
  }, [])

  const flushPreview = useCallback(async () => {
    const session = dragSession.current
    const target = pendingTarget.current
    if (!session || !target || !onNodeDragPreview) {
      return
    }
    const position = { ...target }
    try {
      await onNodeDragPreview(session.nodeId, position)
    } finally {
      if (
        pendingTarget.current &&
        dragSession.current &&
        (pendingTarget.current.x !== position.x || pendingTarget.current.y !== position.y)
      ) {
        cancelPreviewFrame()
        previewFrame.current = requestAnimationFrame(() => {
          previewFrame.current = null
          void flushPreview()
        })
      }
    }
  }, [cancelPreviewFrame, onNodeDragPreview])

  const schedulePreview = useCallback(
    (target: PointPx) => {
      pendingTarget.current = target
      if (previewTimer.current != null || previewFrame.current != null) {
        return
      }
      previewTimer.current = setTimeout(() => {
        previewTimer.current = null
        previewFrame.current = requestAnimationFrame(() => {
          previewFrame.current = null
          void flushPreview()
        })
      }, WIRE_PREVIEW_MS)
    },
    [flushPreview],
  )

  const endDrag = useCallback(() => {
    cancelPreviewFrame()
    cancelVisualFrame()
    const session = dragSession.current
    dragSession.current = null
    dragGrabOffset.current = null
    pendingTarget.current = null
    setNodeDrag(null)
    return session
  }, [cancelPreviewFrame, cancelVisualFrame])

  const handlePointerUp = useCallback(() => {
    panSession.current = null

    const wire = wireSession.current
    if (wire) {
      endWireConnect()
      const diagramPoint = lastDiagramPoint.current
      if (diagramPoint && onPortConnect) {
        const hit = hitTestScene(scene.hits, diagramPoint)
        const to = portFromHit(hit)
        if (to && canConnectPorts(wire.from, to)) {
          void onPortConnect(wire.from, to)
        }
      }
      return
    }

    const session = endDrag()
    if (!session || !onNodeMoveCommit) return
    const start = nodeBodyOrigin(scene.hits, session.nodeId)
    if (
      start &&
      Math.abs(session.targetOrigin.x - start.x) < 0.01 &&
      Math.abs(session.targetOrigin.y - start.y) < 0.01
    ) {
      return
    }
    void onNodeMoveCommit(session.nodeId, snapPoint(session.targetOrigin))
  }, [endDrag, endWireConnect, onNodeMoveCommit, onPortConnect, scene.hits])

  const handlePointerDown = useCallback(
    (event: KonvaEventObject<PointerEvent>) => {
      movedDuringGesture.current = false
      const stage = event.target.getStage()
      const diagramPoint = pointerOnStage(stage)
      if (!diagramPoint) return

      const hit = hitTestScene(scene.hits, diagramPoint)
      lastDiagramPoint.current = diagramPoint

      const port = portFromHit(hit)
      if (port && hit && onPortConnect) {
        event.evt.preventDefault()
        endDrag()
        endWireConnect()
        const fromPoint = portCenterFromHit(hit)
        wireSession.current = { from: port, fromPoint }
        setWireConnect({ from: port, fromPoint, toPoint: fromPoint })
        onHit?.(hit)
        return
      }

      if (hit?.node_id && onNodeMoveCommit && !hit.handle_id) {
        event.evt.preventDefault()
        endWireConnect()
        const origin = nodeBodyOrigin(scene.hits, hit.node_id) ?? {
          x: hit.bounds.x,
          y: hit.bounds.y,
        }
        dragGrabOffset.current = {
          x: diagramPoint.x - origin.x,
          y: diagramPoint.y - origin.y,
        }
        dragSession.current = {
          nodeId: hit.node_id,
          targetOrigin: { ...origin },
        }
        setNodeDrag({
          nodeId: hit.node_id,
          targetOrigin: { ...origin },
        })
        onHit?.(hit)
        return
      }

      const pointer = stage?.getPointerPosition()
      if (!pointer) return
      endWireConnect()
      panSession.current = {
        startPointer: pointer,
        startViewport: { x: viewport.x, y: viewport.y },
      }
      onHit?.(hit)
    },
    [onHit, onNodeMoveCommit, onPortConnect, pointerOnStage, scene.hits, viewport.x, viewport.y, endDrag, endWireConnect],
  )

  const handlePointerMove = useCallback(
    (event: KonvaEventObject<PointerEvent>) => {
      const stage = event.target.getStage()
      const diagramPoint = pointerOnStage(stage)
      if (!diagramPoint) return
      lastDiagramPoint.current = diagramPoint

      if (wireSession.current) {
        movedDuringGesture.current = true
        const session = wireSession.current
        setWireConnect({
          from: session.from,
          fromPoint: session.fromPoint,
          toPoint: diagramPoint,
        })
        return
      }

      const session = dragSession.current
      if (session && dragGrabOffset.current) {
        movedDuringGesture.current = true
        session.targetOrigin = {
          x: diagramPoint.x - dragGrabOffset.current.x,
          y: diagramPoint.y - dragGrabOffset.current.y,
        }
        queueVisualUpdate()
        if (onNodeDragPreview) {
          schedulePreview(session.targetOrigin)
        }
        return
      }

      const pan = panSession.current
      const pointer = stage?.getPointerPosition()
      if (!pan || !pointer) return
      movedDuringGesture.current = true
      onPan({
        x: pan.startViewport.x + (pointer.x - pan.startPointer.x),
        y: pan.startViewport.y + (pointer.y - pan.startPointer.y),
      })
    },
    [onNodeDragPreview, onPan, pointerOnStage, queueVisualUpdate, schedulePreview],
  )

  const handleStageClick = useCallback(
    (event: KonvaEventObject<MouseEvent>) => {
      if (movedDuringGesture.current || !onHit) return
      const stage = event.target.getStage()
      const diagramPoint = pointerOnStage(stage)
      if (!diagramPoint) return
      onHit(hitTestScene(scene.hits, diagramPoint))
    },
    [onHit, pointerOnStage, scene.hits],
  )

  useEffect(() => {
    const onWindowPointerUp = () => {
      if (dragSession.current || panSession.current || wireSession.current) {
        handlePointerUp()
      }
    }
    window.addEventListener('pointerup', onWindowPointerUp)
    return () => window.removeEventListener('pointerup', onWindowPointerUp)
  }, [handlePointerUp])

  useEffect(() => {
    return () => {
      cancelPreviewFrame()
      cancelVisualFrame()
    }
  }, [cancelPreviewFrame, cancelVisualFrame])

  return {
    nodeDrag,
    wireConnect,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    handleStageClick,
  }
}
