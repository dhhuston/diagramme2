import type { HitTarget, PointPx } from './sceneTypes'
import type { Viewport } from './useViewport'

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

export function pointInRect(
  point: PointPx,
  rect: { x: number; y: number; width: number; height: number },
): boolean {
  return (
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  )
}

export function hitArea(hit: HitTarget): number {
  return hit.bounds.width * hit.bounds.height
}

/** Grouping zones use `:boundary:` strip hits — interior is click-through. */
export function isGroupingZoneBoundaryHit(hit: HitTarget): boolean {
  return hit.id.includes(':boundary:')
}

function hitsAtPoint(hits: HitTarget[], point: PointPx): HitTarget[] {
  return hits.filter((hit) => pointInRect(point, hit.bounds))
}

/**
 * Selection: prefer real node geometry over grouping-zone frames; prefer node body over port strips.
 */
export function hitTestSceneForSelection(hits: HitTarget[], point: PointPx): HitTarget | null {
  const matches = hitsAtPoint(hits, point)
  if (matches.length === 0) return null

  const content = matches.filter((h) => !isGroupingZoneBoundaryHit(h))
  const pool = content.length > 0 ? content : matches

  const bodies = pool.filter((h) => h.node_id != null && h.handle_id == null && h.edge_id == null)
  const candidates = bodies.length > 0 ? bodies : pool

  let best = candidates[0]
  let bestArea = hitArea(best)
  for (let i = 1; i < candidates.length; i++) {
    const hit = candidates[i]
    const area = hitArea(hit)
    if (area < bestArea) {
      best = hit
      bestArea = area
    }
  }
  return best
}

function wireGripHitActive(hit: HitTarget, selectedEdgeId?: string | null): boolean {
  return (
    hit.wire_grip_segment == null ||
    (selectedEdgeId != null && hit.edge_id === selectedEdgeId)
  )
}

/**
 * Interaction (wiring, drag): top-most hit in paint order — port handles beat node body.
 * Wire routing grips are only pickable when their edge is selected.
 */
export function hitTestSceneForInteraction(
  hits: HitTarget[],
  point: PointPx,
  selectedEdgeId?: string | null,
): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (!wireGripHitActive(hit, selectedEdgeId)) continue
    if (pointInRect(point, hit.bounds)) {
      return hit
    }
  }
  return null
}

/** Stage-relative pointer → diagram px (for window-level pointer events). */
export function stageRelativeToDiagramPx(
  stageX: number,
  stageY: number,
  viewport: Viewport,
): PointPx {
  return {
    x: (stageX - viewport.x) / viewport.scale,
    y: (stageY - viewport.y) / viewport.scale,
  }
}

/** Client coordinates → diagram px using the stage host element's bounding rect. */
export function clientToDiagramPx(
  clientX: number,
  clientY: number,
  hostRect: DOMRect,
  viewport: Viewport,
): PointPx {
  return stageRelativeToDiagramPx(
    clientX - hostRect.left,
    clientY - hostRect.top,
    viewport,
  )
}

/** @deprecated Use hitTestSceneForSelection or hitTestSceneForInteraction */
export function hitTestScene(hits: HitTarget[], point: PointPx): HitTarget | null {
  return hitTestSceneForInteraction(hits, point)
}
