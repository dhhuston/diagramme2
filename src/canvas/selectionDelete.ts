import type { HitTarget } from './sceneTypes'
import { isWireGripHit } from './wireGripUtils'

export type DeleteSelectionTarget =
  | { kind: 'node'; nodeId: string }
  | { kind: 'edge'; edgeId: string }

/** Whether the current canvas selection can be deleted (node body or whole wire). */
export function deleteTargetFromHit(hit: HitTarget | null | undefined): DeleteSelectionTarget | null {
  if (!hit) return null
  if (isWireGripHit(hit)) return null
  if (hit.handle_id) return null
  if (hit.edge_id) return { kind: 'edge', edgeId: hit.edge_id }
  if (hit.node_id) return { kind: 'node', nodeId: hit.node_id }
  return null
}

export function deleteLabelForTarget(target: DeleteSelectionTarget | null): string {
  if (!target) return 'Delete'
  return target.kind === 'edge' ? 'Delete wire' : 'Delete node'
}
