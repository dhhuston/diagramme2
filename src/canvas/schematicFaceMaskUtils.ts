import type { HitTarget, PointPx, RectPx } from './sceneTypes'

export type FaceMaskRect = {
  nodeId: string
  kind: 'rect'
  bounds: RectPx
}

export type FaceMaskPolygon = {
  nodeId: string
  kind: 'polygon'
  points: PointPx[]
}

export type FaceMaskShape = FaceMaskRect | FaceMaskPolygon

/** Opaque face shapes declared on body hits (inset frames, symbol geometry, wiretag hull). */
export function faceMaskShapes(hits: HitTarget[]): FaceMaskShape[] {
  const seen = new Set<string>()
  const masks: FaceMaskShape[] = []
  for (const hit of hits) {
    if (hit.node_id == null || hit.handle_id != null || hit.edge_id != null) {
      continue
    }
    if (hit.id !== hit.node_id || seen.has(hit.node_id)) {
      continue
    }
    const polygon = hit.face_mask_polygon
    if (polygon && polygon.length >= 3) {
      seen.add(hit.node_id)
      masks.push({ nodeId: hit.node_id, kind: 'polygon', points: polygon })
      continue
    }
    const bounds = hit.face_mask_bounds
    if (!bounds) {
      continue
    }
    seen.add(hit.node_id)
    masks.push({ nodeId: hit.node_id, kind: 'rect', bounds })
  }
  return masks
}

/** @deprecated Use faceMaskShapes */
export function faceMaskRects(hits: HitTarget[]): FaceMaskRect[] {
  return faceMaskShapes(hits).filter((m): m is FaceMaskRect => m.kind === 'rect')
}
