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

/** Draggable node body — not port, wire, grip, or grouping-zone boundary strip. */
export function isNodeBodyHit(hit: HitTarget): boolean {
  return (
    hit.node_id != null &&
    hit.handle_id == null &&
    hit.edge_id == null &&
    hit.wire_grip_segment == null &&
    !isGroupingZoneBoundaryHit(hit)
  )
}

/** Top-most body hit for a node (selection after palette insert). */
export function findNodeBodyHit(hits: HitTarget[], nodeId: string): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (hit.node_id === nodeId && isNodeBodyHit(hit)) return hit
  }
  return null
}

/** Double-click entry: top-most grouping zone boundary strip under the pointer. */
export function hitTestSceneForGroupingZoneBoundary(
  hits: HitTarget[],
  point: PointPx,
): HitTarget | null {
  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (!isGroupingZoneBoundaryHit(hit)) continue
    if (pointInRect(point, hit.bounds)) return hit
  }
  return null
}

function hitsAtPoint(hits: HitTarget[], point: PointPx): HitTarget[] {
  return hits.filter((hit) => pointInRect(point, hit.bounds))
}

/** Wire routing grips are not selected by click — only after the edge is selected. */
function isWireGripSelectionHit(hit: HitTarget): boolean {
  return hit.wire_grip_segment != null
}

/**
 * Selection: top-most eligible hit in paint order (wires above nodes), except
 * port strips lose to their node body (same node).
 */
export function hitTestSceneForSelection(hits: HitTarget[], point: PointPx): HitTarget | null {
  const matches = hitsAtPoint(hits, point)
  if (matches.length === 0) return null

  const hasNonBoundary = matches.some((h) => !isGroupingZoneBoundaryHit(h))
  const eligible = matches.filter((h) => {
    if (hasNonBoundary && isGroupingZoneBoundaryHit(h)) return false
    if (isWireGripSelectionHit(h)) return false
    return true
  })
  if (eligible.length === 0) return null

  const eligibleIds = new Set(eligible.map((h) => h.id))

  // Port handle strips lose to the node body on the same node.
  for (const hit of eligible) {
    if (hit.node_id != null && hit.handle_id == null && hit.edge_id == null) {
      const portOnSameNode = eligible.some(
        (h) => h.node_id === hit.node_id && h.handle_id != null,
      )
      if (portOnSameNode) return hit
    }
  }

  for (let i = hits.length - 1; i >= 0; i--) {
    const hit = hits[i]
    if (eligibleIds.has(hit.id)) return hit
  }

  return eligible[eligible.length - 1] ?? null
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
