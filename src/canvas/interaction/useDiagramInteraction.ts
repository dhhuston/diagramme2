import { useCallback, useEffect, useRef, useState } from 'react'
import type { KonvaEventObject } from 'konva/lib/Node'

import { hitTestScene, stagePointerToDiagramPx } from '../hitTest'
import type { HitTarget, PointPx, SceneJson } from '../sceneTypes'
import type { Viewport } from '../useViewport'
import {
  nodeDragCaptureBounds,
  snappedNodeOrigin,
  type NodeDragPreview,
} from './dragNode'

type UseDiagramInteractionOptions = {
  scene: SceneJson
  viewport: Viewport
  onHit?: (hit: HitTarget | null) => void
  onNodeMove?: (nodeId: string, position: PointPx) => void | Promise<void>
  onPan: (next: Pick<Viewport, 'x' | 'y'>) => void
}

type PanSession = {
  startPointer: PointPx
  startViewport: PointPx
}

export function useDiagramInteraction({
  scene,
  viewport,
  onHit,
  onNodeMove,
  onPan,
}: UseDiagramInteractionOptions) {
  const [nodeDrag, setNodeDrag] = useState<NodeDragPreview | null>(null)
  const nodeDragRef = useRef<NodeDragPreview | null>(null)
  const panSession = useRef<PanSession | null>(null)
  const dragGrabOffset = useRef<PointPx | null>(null)
  const movedDuringGesture = useRef(false)

  useEffect(() => {
    nodeDragRef.current = nodeDrag
  }, [nodeDrag])

  const pointerOnStage = useCallback(
    (stage: { getPointerPosition: () => PointPx | null } | null | undefined) => {
      const pointer = stage?.getPointerPosition()
      if (!pointer) return null
      return stagePointerToDiagramPx(pointer, viewport)
    },
    [viewport],
  )

  const handlePointerUp = useCallback(() => {
    const preview = nodeDragRef.current
    setNodeDrag(null)
    dragGrabOffset.current = null
    panSession.current = null

    if (!preview || !onNodeMove) return
    if (preview.dx === 0 && preview.dy === 0) return
    void onNodeMove(preview.nodeId, snappedNodeOrigin(preview))
  }, [onNodeMove])

  const handlePointerDown = useCallback(
    (event: KonvaEventObject<PointerEvent>) => {
      movedDuringGesture.current = false
      const stage = event.target.getStage()
      const diagramPoint = pointerOnStage(stage)
      if (!diagramPoint) return

      const hit = hitTestScene(scene.hits, diagramPoint)
      if (hit?.node_id && onNodeMove) {
        event.evt.preventDefault()
        dragGrabOffset.current = {
          x: diagramPoint.x - hit.bounds.x,
          y: diagramPoint.y - hit.bounds.y,
        }
        const preview: NodeDragPreview = {
          nodeId: hit.node_id,
          captureBounds: nodeDragCaptureBounds(hit.bounds),
          origin: { x: hit.bounds.x, y: hit.bounds.y },
          dx: 0,
          dy: 0,
        }
        setNodeDrag(preview)
        onHit?.(hit)
        return
      }

      const pointer = stage?.getPointerPosition()
      if (!pointer) return
      panSession.current = {
        startPointer: pointer,
        startViewport: { x: viewport.x, y: viewport.y },
      }
      onHit?.(hit)
    },
    [onHit, onNodeMove, pointerOnStage, scene.hits, viewport.x, viewport.y],
  )

  const handlePointerMove = useCallback(
    (event: KonvaEventObject<PointerEvent>) => {
      const stage = event.target.getStage()
      const diagramPoint = pointerOnStage(stage)
      if (!diagramPoint) return

      if (nodeDragRef.current && dragGrabOffset.current) {
        movedDuringGesture.current = true
        const preview = nodeDragRef.current
        const nextOrigin = {
          x: diagramPoint.x - dragGrabOffset.current.x,
          y: diagramPoint.y - dragGrabOffset.current.y,
        }
        setNodeDrag({
          ...preview,
          dx: nextOrigin.x - preview.origin.x,
          dy: nextOrigin.y - preview.origin.y,
        })
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
    [onPan, pointerOnStage],
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
      if (nodeDragRef.current || panSession.current) {
        handlePointerUp()
      }
    }
    window.addEventListener('pointerup', onWindowPointerUp)
    return () => window.removeEventListener('pointerup', onWindowPointerUp)
  }, [handlePointerUp])

  return {
    nodeDrag,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    handleStageClick,
  }
}
