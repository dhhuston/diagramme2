import type { HitTarget } from './sceneTypes'

export const WIRE_GRIP_RADIUS_PX = 4

export function isWireGripHit(hit: HitTarget | null | undefined): hit is HitTarget & {
  edge_id: string
  wire_grip_segment: number
  wire_grip_orientation: 'h' | 'v'
} {
  return (
    hit != null &&
    hit.edge_id != null &&
    hit.wire_grip_segment != null &&
    (hit.wire_grip_orientation === 'h' || hit.wire_grip_orientation === 'v')
  )
}

export function wireGripHits(hits: HitTarget[], selectedEdgeId?: string | null): HitTarget[] {
  if (!selectedEdgeId) return []
  return hits.filter((h) => isWireGripHit(h) && h.edge_id === selectedEdgeId)
}

export function wireGripCursor(orientation: 'h' | 'v'): string {
  return orientation === 'h' ? 'ns-resize' : 'ew-resize'
}
