import { defaultOrtho, toPairs, type Pt } from './groupingZoneGeometry'
import { isGroupingZoneBoundaryHit } from './hitTest'
import { groupingZoneData, groupingZoneDimensions, groupingZoneShape } from './groupingZoneNode'
import type { HitTarget, PointPx } from './sceneTypes'
import type { FlowNode } from '../tauriIpc'

const DEFAULT_BOUNDARY_TOLERANCE_PX = 8

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

function distanceToRectBorder(
  point: PointPx,
  rect: { x: number; y: number; width: number; height: number },
): number {
  const left = point.x - rect.x
  const top = point.y - rect.y
  const right = rect.x + rect.width - point.x
  const bottom = rect.y + rect.height - point.y
  if (left >= 0 && top >= 0 && right >= 0 && bottom >= 0) {
    return Math.min(left, top, right, bottom)
  }
  const dx = left < 0 ? -left : right < 0 ? -right : 0
  const dy = top < 0 ? -top : bottom < 0 ? -bottom : 0
  return Math.sqrt(dx * dx + dy * dy)
}

function distanceToPolylineBorder(node: FlowNode, point: PointPx): number {
  const dims = groupingZoneDimensions(node)
  const data = groupingZoneData(node)
  const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
  const pairs = toPairs(pts)
  if (pairs.length < 2) return Infinity
  let min = Infinity
  for (let i = 0; i < pairs.length; i += 1) {
    const [ax, ay] = pairs[i]
    const [bx, by] = pairs[(i + 1) % pairs.length]
    min = Math.min(
      min,
      distPointToSegment(
        point.x,
        point.y,
        dims.x + ax,
        dims.y + ay,
        dims.x + bx,
        dims.y + by,
      ),
    )
  }
  return min
}

function distanceToGroupingZoneBorder(node: FlowNode, point: PointPx): number {
  if (groupingZoneShape(node) === 'polyline') {
    return distanceToPolylineBorder(node, point)
  }
  return distanceToRectBorder(point, groupingZoneDimensions(node))
}

function boundaryHitForNode(hits: HitTarget[], nodeId: string): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (hit.node_id === nodeId && isGroupingZoneBoundaryHit(hit)) return hit
  }
  return null
}

/** Scene strip hit first, then distance to the visible border (v6 stroke-style picking). */
export function hitTestGroupingZoneBoundary(
  hits: HitTarget[],
  point: PointPx,
  nodes: FlowNode[],
  tolerancePx = DEFAULT_BOUNDARY_TOLERANCE_PX,
): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (!isGroupingZoneBoundaryHit(hit)) continue
    const { bounds } = hit
    if (
      point.x >= bounds.x &&
      point.x <= bounds.x + bounds.width &&
      point.y >= bounds.y &&
      point.y <= bounds.y + bounds.height
    ) {
      return hit
    }
  }

  let best: { node: FlowNode; distance: number } | null = null
  for (const node of nodes) {
    if (node.type !== 'groupingZone') continue
    const distance = distanceToGroupingZoneBorder(node, point)
    if (distance > tolerancePx) continue
    if (!best || distance < best.distance) {
      best = { node, distance }
    }
  }
  if (!best) return null

  return (
    boundaryHitForNode(hits, best.node.id) ?? {
      id: `${best.node.id}:boundary:0`,
      bounds: {
        x: best.node.position.x,
        y: best.node.position.y,
        width: best.node.width ?? 0,
        height: best.node.height ?? 0,
      },
      node_id: best.node.id,
    }
  )
}

export function isGroupingZoneNodeSelection(hits: HitTarget[], nodeId: string): boolean {
  const hasBody = hits.some((h) => h.id === nodeId && h.node_id === nodeId)
  if (hasBody) return false
  return hits.some((h) => h.node_id === nodeId && isGroupingZoneBoundaryHit(h))
}

/** Polyline local-space vertex pairs for selection chrome. */
export function groupingZonePolylinePairs(node: FlowNode): Pt[] | null {
  if (groupingZoneShape(node) !== 'polyline') return null
  const dims = groupingZoneDimensions(node)
  const data = groupingZoneData(node)
  const pts = data.polylinePoints ?? defaultOrtho(dims.width, dims.height)
  return toPairs(pts)
}
