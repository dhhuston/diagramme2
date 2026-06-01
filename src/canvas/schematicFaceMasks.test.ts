import { describe, expect, it } from 'vitest'

import { faceMaskRects } from './schematicFaceMaskUtils'
import type { HitTarget } from './sceneTypes'

describe('faceMaskRects', () => {
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
    expect(faceMaskRects(hits)).toEqual([
      { nodeId: 'dev-1', bounds: { x: 10, y: 20, width: 72, height: 48 } },
    ])
  })
})
