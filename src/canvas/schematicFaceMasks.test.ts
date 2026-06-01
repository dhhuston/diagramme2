import { describe, expect, it } from 'vitest'

import { faceMaskShapes } from './schematicFaceMaskUtils'
import type { HitTarget } from './sceneTypes'

describe('faceMaskShapes', () => {
  it('uses face_mask_bounds when present and skips external-only nodes', () => {
    const hits: HitTarget[] = [
      {
        id: 'dev-1',
        node_id: 'dev-1',
        bounds: { x: 10, y: 8, width: 72, height: 60 },
        face_mask_bounds: { x: 10, y: 20, width: 72, height: 48 },
      },
      {
        id: 'fly-1',
        node_id: 'fly-1',
        bounds: { x: 0, y: 0, width: 120, height: 12 },
      },
      {
        id: 'zone:boundary:0',
        node_id: 'zone',
        bounds: { x: 0, y: 0, width: 200, height: 6 },
      },
    ]
    expect(faceMaskShapes(hits)).toEqual([
      {
        nodeId: 'dev-1',
        kind: 'rect',
        bounds: { x: 10, y: 20, width: 72, height: 48 },
      },
    ])
  })

  it('prefers face_mask_polygon over bounds', () => {
    const hits: HitTarget[] = [
      {
        id: 'wt-1',
        node_id: 'wt-1',
        bounds: { x: 0, y: 0, width: 80, height: 12 },
        face_mask_bounds: { x: 0, y: 0, width: 80, height: 12 },
        face_mask_polygon: [
          { x: 0, y: 0 },
          { x: 70, y: 0 },
          { x: 80, y: 6 },
          { x: 0, y: 12 },
        ],
      },
    ]
    expect(faceMaskShapes(hits)).toEqual([
      {
        nodeId: 'wt-1',
        kind: 'polygon',
        points: [
          { x: 0, y: 0 },
          { x: 70, y: 0 },
          { x: 80, y: 6 },
          { x: 0, y: 12 },
        ],
      },
    ])
  })
})
