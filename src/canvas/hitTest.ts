import type { Viewport } from './useViewport'
import type { HitTarget, PointPx } from './sceneTypes'

/** Convert stage pointer position to diagram px using viewport pan/zoom. */
export function stagePointerToDiagramPx(
  pointer: PointPx,
  viewport: Viewport,
): PointPx {
  return {
    x: (pointer.x - viewport.x) / viewport.scale,
    y: (pointer.y - viewport.y) / viewport.scale,
  }
}

export function pointInRect(point: PointPx, rect: { x: number; y: number; width: number; height: number }): boolean {
  return (
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  )
}

/** Top-most hit target at diagram point (reverse paint order). */
export function hitTestScene(hits: HitTarget[], point: PointPx): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (pointInRect(point, hit.bounds)) {
      return hit
    }
  }
  return null
}
