import type { HitTarget, PointPx } from '../sceneTypes'
import { snapPoint } from '../paperScale'

export {
  CONNECTOR_LINE_PITCH_PX,
  SNAP_GRID_PX,
  SNAP_HALF_STEP_PX,
  snapPlacementCoord,
  snapPoint,
} from '../paperScale'

/** Raw pointer position → snapped node origin using grab offset from body origin. */
export function snappedNodeOriginFromPointer(
  diagramPoint: PointPx,
  grabOffset: PointPx,
): PointPx {
  return snapPoint({
    x: diagramPoint.x - grabOffset.x,
    y: diagramPoint.y - grabOffset.y,
  })
}

/** Node root position for `move_node` — body hit has the largest bounds for a node. */
export function nodeBodyOrigin(hits: HitTarget[], nodeId: string): PointPx | null {
  const nodeHits = hits.filter((h) => h.node_id === nodeId)
  if (nodeHits.length === 0) {
    return null
  }
  const body = nodeHits.reduce((best, hit) => {
    const bestArea = best.bounds.width * best.bounds.height
    const area = hit.bounds.width * hit.bounds.height
    return area > bestArea ? hit : best
  })
  return { x: body.bounds.x, y: body.bounds.y }
}

export function nodeBodyBounds(hits: HitTarget[], nodeId: string) {
  const nodeHits = hits.filter((h) => h.node_id === nodeId)
  if (nodeHits.length === 0) {
    return null
  }
  return nodeHits.reduce((best, hit) => {
    const bestArea = best.bounds.width * best.bounds.height
    const area = hit.bounds.width * hit.bounds.height
    return area > bestArea ? hit : best
  }).bounds
}

/** Gap between pointer target and last Rust scene rebuild — auto-zeroes when scene catches up. */
export function dragVisualDelta(
  hits: HitTarget[],
  nodeId: string,
  targetOrigin: PointPx,
  epsilon = 0.01,
): PointPx | null {
  const sceneOrigin = nodeBodyOrigin(hits, nodeId)
  if (!sceneOrigin) {
    return null
  }
  const dx = targetOrigin.x - sceneOrigin.x
  const dy = targetOrigin.y - sceneOrigin.y
  if (Math.abs(dx) < epsilon && Math.abs(dy) < epsilon) {
    return null
  }
  return { x: dx, y: dy }
}

export type NodeDragTarget = {
  nodeId: string
  targetOrigin: PointPx
}
