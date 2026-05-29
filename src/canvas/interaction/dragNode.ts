import type { PointPx, RectPx, ScenePrimitive } from '../sceneTypes'

/** Matches Rust `SNAP_GRID_PX` — 1/8" pitch / 3 at 72 dpi. */
export const SNAP_GRID_PX = 3

/** Matches Rust `SCHEMATIC_TAG_BAND_PX`. */
export const SCHEMATIC_TAG_BAND_PX = 15

/** Matches Rust `BUNDLE_STUB_PX` + fillet slack for bracket polylines. */
export const NODE_DRAG_BRACKET_PAD_PX = 24

export function snapPlacementCoord(v: number): number {
  return Math.round(v / SNAP_GRID_PX) * SNAP_GRID_PX
}

/** Region used to collect node-owned primitives (tag band + bundle stubs). */
export function nodeDragCaptureBounds(bounds: RectPx): RectPx {
  return {
    x: bounds.x - NODE_DRAG_BRACKET_PAD_PX,
    y: bounds.y - SCHEMATIC_TAG_BAND_PX,
    width: bounds.width + NODE_DRAG_BRACKET_PAD_PX * 2,
    height: bounds.height + SCHEMATIC_TAG_BAND_PX,
  }
}

function pointInRect(point: PointPx, rect: RectPx): boolean {
  return (
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  )
}

function polylineBounds(points: PointPx[]): RectPx {
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const p of points) {
    minX = Math.min(minX, p.x)
    minY = Math.min(minY, p.y)
    maxX = Math.max(maxX, p.x)
    maxY = Math.max(maxY, p.y)
  }
  if (!Number.isFinite(minX)) {
    return { x: 0, y: 0, width: 0, height: 0 }
  }
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
}

function rectsIntersect(a: RectPx, b: RectPx): boolean {
  return !(
    a.x + a.width < b.x ||
    b.x + b.width < a.x ||
    a.y + a.height < b.y ||
    b.y + b.height < a.y
  )
}

export function belongsToNodeDrag(
  primitive: ScenePrimitive,
  nodeId: string,
  captureBounds: RectPx,
): boolean {
  if ('Polyline' in primitive) {
    const p = primitive.Polyline
    if (p.edge_id) {
      return false
    }
    return rectsIntersect(polylineBounds(p.points), captureBounds)
  }

  if ('Rect' in primitive) {
    const r = primitive.Rect
    if (r.node_id === nodeId) {
      return true
    }
    return rectsIntersect(r.rect, captureBounds)
  }

  if ('Solid' in primitive) {
    const s = primitive.Solid
    if (s.node_id === nodeId) {
      return true
    }
    return s.vertices.some((v) => pointInRect(v, captureBounds))
  }

  const t = primitive.Text
  return pointInRect(t.position, captureBounds)
}

function offsetPoint(p: PointPx, dx: number, dy: number): PointPx {
  return { x: p.x + dx, y: p.y + dy }
}

/** Return a diagram-space primitive shifted by drag delta (preview only). */
export function offsetScenePrimitive(
  primitive: ScenePrimitive,
  dx: number,
  dy: number,
): ScenePrimitive {
  if ('Polyline' in primitive) {
    const p = primitive.Polyline
    return {
      Polyline: {
        ...p,
        points: p.points.map((pt) => offsetPoint(pt, dx, dy)),
      },
    }
  }

  if ('Rect' in primitive) {
    const r = primitive.Rect
    return {
      Rect: {
        ...r,
        rect: {
          ...r.rect,
          x: r.rect.x + dx,
          y: r.rect.y + dy,
        },
      },
    }
  }

  if ('Solid' in primitive) {
    const s = primitive.Solid
    const vertices = s.vertices.map((v) => offsetPoint(v, dx, dy)) as typeof s.vertices
    return {
      Solid: {
        ...s,
        vertices,
      },
    }
  }

  const t = primitive.Text
  return {
    Text: {
      ...t,
      position: offsetPoint(t.position, dx, dy),
    },
  }
}

export type NodeDragPreview = {
  nodeId: string
  captureBounds: RectPx
  origin: PointPx
  dx: number
  dy: number
}

export function snappedNodeOrigin(preview: NodeDragPreview): PointPx {
  return {
    x: snapPlacementCoord(preview.origin.x + preview.dx),
    y: snapPlacementCoord(preview.origin.y + preview.dy),
  }
}
