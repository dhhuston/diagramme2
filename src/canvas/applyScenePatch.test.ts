import { describe, expect, it } from 'vitest'

import { applyScenePatch } from './applyScenePatch'
import type { SceneJson, ScenePatchJson } from './sceneTypes'

const baseScene: SceneJson = {
  extent: { x: 0, y: 0, width: 100, height: 100 },
  primitives: [
    { Polyline: { points: [{ x: 0, y: 0 }, { x: 1, y: 1 }], stroke_px: 1, layer: 'Wires', color: 0, owner_node_id: 'n1' } },
    { Polyline: { points: [{ x: 2, y: 2 }, { x: 3, y: 3 }], stroke_px: 1, layer: 'Wires', color: 0, edge_id: 'e1' } },
    { Polyline: { points: [{ x: 4, y: 4 }, { x: 5, y: 5 }], stroke_px: 1, layer: 'Wires', color: 0, edge_id: 'e2' } },
  ],
  hits: [
    { id: 'n1', bounds: { x: 0, y: 0, width: 10, height: 10 }, node_id: 'n1' },
    { id: 'n2', bounds: { x: 20, y: 0, width: 10, height: 10 }, node_id: 'n2' },
  ],
}

describe('applyScenePatch', () => {
  it('replaces owned node geometry and connected wire polylines', () => {
    const patch: ScenePatchJson = {
      node_ids: ['n1'],
      edge_ids: ['e1'],
      primitives: [
        { Polyline: { points: [{ x: 9, y: 9 }, { x: 10, y: 10 }], stroke_px: 1, layer: 'Wires', color: 0, owner_node_id: 'n1' } },
        { Polyline: { points: [{ x: 11, y: 11 }, { x: 12, y: 12 }], stroke_px: 1, layer: 'Wires', color: 0, edge_id: 'e1' } },
      ],
      hits: [{ id: 'n1', bounds: { x: 1, y: 1, width: 10, height: 10 }, node_id: 'n1' }],
    }

    const next = applyScenePatch(baseScene, patch)
    expect(next.primitives).toHaveLength(3)
    expect(next.primitives.some((p) => 'Polyline' in p && p.Polyline.owner_node_id === 'n1' && p.Polyline.points[0]?.x === 9)).toBe(true)
    expect(next.primitives.some((p) => 'Polyline' in p && p.Polyline.edge_id === 'e2')).toBe(true)
    expect(next.hits).toHaveLength(2)
    expect(next.hits.find((h) => h.node_id === 'n1')?.bounds.x).toBe(1)
    expect(next.hits.find((h) => h.node_id === 'n2')).toBeDefined()
  })
})
