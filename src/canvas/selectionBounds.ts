import type { HitTarget, RectPx } from './sceneTypes'

export function unionBounds(rects: RectPx[]): RectPx | null {
  if (rects.length === 0) {
    return null
  }
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const r of rects) {
    minX = Math.min(minX, r.x)
    minY = Math.min(minY, r.y)
    maxX = Math.max(maxX, r.x + r.width)
    maxY = Math.max(maxY, r.y + r.height)
  }
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
}

/** Visual bounds for a selected node (body rect, or union of grouping-zone strips). */
export function nodeSelectionBounds(hits: HitTarget[], nodeId: string): RectPx | null {
  const nodeHits = hits.filter((h) => h.node_id === nodeId && !h.handle_id && !h.edge_id)
  const body = nodeHits.find((h) => h.id === nodeId)
  if (body) {
    return body.bounds
  }
  const boundaries = nodeHits.filter((h) => h.id.includes(':boundary:'))
  if (boundaries.length > 0) {
    return unionBounds(boundaries.map((h) => h.bounds))
  }
  if (nodeHits.length === 0) {
    return null
  }
  return unionBounds(nodeHits.map((h) => h.bounds))
}
