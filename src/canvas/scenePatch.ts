import type { HitTarget, RectPx, SceneJson, ScenePrimitive } from './sceneTypes'

export interface ScenePatchJson {
  node_ids: string[]
  edge_ids: string[]
  primitives: ScenePrimitive[]
  hits: HitTarget[]
}

function rectsIntersect(a: RectPx, b: RectPx): boolean {
  return (
    a.x <= b.x + b.width &&
    a.x + a.width >= b.x &&
    a.y <= b.y + b.height &&
    a.y + a.height >= b.y
  )
}

/** Bounding box for hit testing which primitives belong to a dragged node. */
export function primitiveBounds(primitive: ScenePrimitive): RectPx | null {
  if ('Polyline' in primitive) {
    const pts = primitive.Polyline.points
    if (pts.length === 0) return null
    let minX = pts[0].x
    let minY = pts[0].y
    let maxX = pts[0].x
    let maxY = pts[0].y
    for (const p of pts) {
      minX = Math.min(minX, p.x)
      minY = Math.min(minY, p.y)
      maxX = Math.max(maxX, p.x)
      maxY = Math.max(maxY, p.y)
    }
    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
  }
  if ('Rect' in primitive) {
    return primitive.Rect.rect
  }
  if ('Solid' in primitive) {
    const verts = primitive.Solid.vertices
    let minX = verts[0].x
    let minY = verts[0].y
    let maxX = verts[0].x
    let maxY = verts[0].y
    for (const v of verts) {
      minX = Math.min(minX, v.x)
      minY = Math.min(minY, v.y)
      maxX = Math.max(maxX, v.x)
      maxY = Math.max(maxY, v.y)
    }
    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
  }
  const t = primitive.Text
  return { x: t.position.x, y: t.position.y, width: 0, height: t.height_px }
}

/** Non-wire primitives overlapping the node body at drag start — Konva-local translate during drag. */
export function captureNodePrimitiveIndices(scene: SceneJson, body: RectPx): number[] {
  const pad = 2
  const region: RectPx = {
    x: body.x - pad,
    y: body.y - pad,
    width: body.width + pad * 2,
    height: body.height + pad * 2,
  }
  return scene.primitives.flatMap((primitive, index) => {
    if ('Polyline' in primitive && primitive.Polyline.edge_id) {
      return []
    }
    const bounds = primitiveBounds(primitive)
    if (!bounds) return []
    return rectsIntersect(bounds, region) ? [index] : []
  })
}

/** Merge wire polylines and hit targets from a Rust drag-preview patch. */
export function applyScenePatch(scene: SceneJson, patch: ScenePatchJson): SceneJson {
  const edgeSet = new Set(patch.edge_ids)
  const nodeSet = new Set(patch.node_ids)
  const primitives = scene.primitives
    .filter((p) => {
      if ('Polyline' in p && p.Polyline.edge_id) {
        return !edgeSet.has(p.Polyline.edge_id)
      }
      return true
    })
    .concat(patch.primitives)
  const hits = scene.hits
    .filter((h) => !h.node_id || !nodeSet.has(h.node_id))
    .concat(patch.hits)
  return { ...scene, primitives, hits }
}
