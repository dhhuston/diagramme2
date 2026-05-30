import { describe, expect, it } from 'vitest'

import { applyScenePatch, captureNodePrimitiveIndices, primitiveBounds } from './scenePatch'
import type { SceneJson } from './sceneTypes'

const baseScene: SceneJson = {
  extent: { x: 0, y: 0, width: 100, height: 100 },
  hits: [
    { id: 'n1', bounds: { x: 10, y: 10, width: 40, height: 20 }, node_id: 'n1' },
    { id: 'n2', bounds: { x: 80, y: 10, width: 10, height: 10 }, node_id: 'n2' },
  ],
  primitives: [
    { Rect: { rect: { x: 10, y: 10, width: 40, height: 20 }, stroke_px: 1, layer: '0' } },
    {
      Polyline: {
        points: [
          { x: 50, y: 20 },
          { x: 90, y: 20 },
        ],
        stroke_px: 1,
        layer: 'WIRES',
        color: 0,
        edge_id: 'e1',
      },
    },
    { Text: { position: { x: 15, y: 15 }, content: 'TAG', height_px: 9, halign: 'Left', valign: 'Top', font: 'Arial Narrow' } },
  ],
}

describe('scenePatch', () => {
  it('captureNodePrimitiveIndices selects non-wire geometry inside body', () => {
    const body = { x: 10, y: 10, width: 40, height: 20 }
    expect(captureNodePrimitiveIndices(baseScene, body).sort()).toEqual([0, 2])
  })

  it('applyScenePatch replaces wire polylines and node hits only', () => {
    const next = applyScenePatch(baseScene, {
      node_ids: ['n1'],
      edge_ids: ['e1'],
      primitives: [
        {
          Polyline: {
            points: [
              { x: 50, y: 25 },
              { x: 90, y: 25 },
            ],
            stroke_px: 1,
            layer: 'WIRES',
            color: 0,
            edge_id: 'e1',
          },
        },
      ],
      hits: [{ id: 'n1', bounds: { x: 20, y: 10, width: 40, height: 20 }, node_id: 'n1' }],
    })
    expect(next.primitives).toHaveLength(3)
    const wire = next.primitives.find(
      (p) => 'Polyline' in p && p.Polyline.edge_id === 'e1',
    )
    expect(wire).toEqual({
      Polyline: {
        points: [
          { x: 50, y: 25 },
          { x: 90, y: 25 },
        ],
        stroke_px: 1,
        layer: 'WIRES',
        color: 0,
        edge_id: 'e1',
      },
    })
    expect(next.hits.map((h) => h.id)).toEqual(['n2', 'n1'])
  })

  it('primitiveBounds covers polyline extents', () => {
    const b = primitiveBounds(baseScene.primitives[1])
    expect(b).toEqual({ x: 50, y: 20, width: 40, height: 0 })
  })
})
