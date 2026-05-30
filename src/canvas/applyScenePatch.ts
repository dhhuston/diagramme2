import type { HitTarget, SceneJson, ScenePatchJson, ScenePrimitive } from './sceneTypes'

function primitiveOwnerNodeId(prim: ScenePrimitive): string | undefined {
  if ('Polyline' in prim) {
    return prim.Polyline.owner_node_id
  }
  if ('Text' in prim) {
    return prim.Text.owner_node_id
  }
  if ('Rect' in prim) {
    return prim.Rect.node_id
  }
  if ('Solid' in prim) {
    return prim.Solid.node_id
  }
  return undefined
}

function primitiveEdgeId(prim: ScenePrimitive): string | undefined {
  if ('Polyline' in prim) {
    return prim.Polyline.edge_id
  }
  return undefined
}

/** Merge a drag-preview patch into the current scene without a full rebuild. */
export function applyScenePatch(scene: SceneJson, patch: ScenePatchJson): SceneJson {
  const nodeIds = new Set(patch.node_ids)
  const edgeIds = new Set(patch.edge_ids)

  const primitives = scene.primitives.filter((prim) => {
    const owner = primitiveOwnerNodeId(prim)
    if (owner != null && nodeIds.has(owner)) {
      return false
    }
    const edgeId = primitiveEdgeId(prim)
    if (edgeId != null && edgeIds.has(edgeId)) {
      return false
    }
    return true
  })

  const hits = scene.hits.filter((hit) => {
    if (hit.node_id != null && nodeIds.has(hit.node_id)) {
      return false
    }
    return true
  })

  return {
    extent: scene.extent,
    primitives: [...primitives, ...patch.primitives],
    hits: mergeHits(hits, patch.hits),
  }
}

/** Replace hits for patched nodes; keep unrelated hits from the prior scene. */
function mergeHits(existing: HitTarget[], patchHits: HitTarget[]): HitTarget[] {
  const patchNodeIds = new Set(
    patchHits.map((h) => h.node_id).filter((id): id is string => id != null),
  )
  const kept = existing.filter((h) => h.node_id == null || !patchNodeIds.has(h.node_id))
  return [...kept, ...patchHits]
}
