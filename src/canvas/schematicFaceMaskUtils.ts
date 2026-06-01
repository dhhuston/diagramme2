import type { HitTarget } from './sceneTypes'

export type FaceMaskRect = {
  nodeId: string
  bounds: HitTarget['bounds']
}

/** One opaque face per node that declares `face_mask_bounds` (inset frame, not external tags). */
export function faceMaskRects(hits: HitTarget[]): FaceMaskRect[] {
  const seen = new Set<string>()
  const masks: FaceMaskRect[] = []
  for (const hit of hits) {
    if (hit.node_id == null || hit.handle_id != null || hit.edge_id != null) {
      continue
    }
    if (hit.id !== hit.node_id || seen.has(hit.node_id)) {
      continue
    }
    const mask = hit.face_mask_bounds
    if (!mask) {
      continue
    }
    seen.add(hit.node_id)
    masks.push({ nodeId: hit.node_id, bounds: mask })
  }
  return masks
}
