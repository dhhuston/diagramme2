import { describe, expect, it } from 'vitest'

import {
  dragVisualDelta,
  nodeBodyOrigin,
  snapPlacementCoord,
  snapPoint,
} from './dragNode'
import type { HitTarget } from '../sceneTypes'

describe('dragNode', () => {
  it('snapPlacementCoord snaps to 3px grid', () => {
    expect(snapPlacementCoord(0)).toBe(0)
    expect(snapPlacementCoord(7)).toBe(6)
    expect(snapPlacementCoord(8)).toBe(9)
  })

  it('nodeBodyOrigin picks the largest hit target for a node', () => {
    const hits: HitTarget[] = [
      {
        id: 'avPlate-1:row-1',
        node_id: 'avPlate-1',
        bounds: { x: 350, y: 480, width: 60, height: 9 },
      },
      {
        id: 'avPlate-1',
        node_id: 'avPlate-1',
        bounds: { x: 346, y: 448, width: 72, height: 72 },
      },
    ]
    expect(nodeBodyOrigin(hits, 'avPlate-1')).toEqual({ x: 346, y: 448 })
  })

  it('dragVisualDelta is null when scene matches target', () => {
    const hits: HitTarget[] = [
      {
        id: 'n1',
        node_id: 'n1',
        bounds: { x: 100, y: 200, width: 72, height: 48 },
      },
    ]
    expect(dragVisualDelta(hits, 'n1', { x: 100, y: 200 })).toBeNull()
  })

  it('dragVisualDelta returns gap when scene lags pointer', () => {
    const hits: HitTarget[] = [
      {
        id: 'n1',
        node_id: 'n1',
        bounds: { x: 100, y: 200, width: 72, height: 48 },
      },
    ]
    expect(dragVisualDelta(hits, 'n1', { x: 107, y: 198 })).toEqual({ x: 7, y: -2 })
  })

  it('snapPoint snaps both axes', () => {
    expect(snapPoint({ x: 107, y: 198 })).toEqual({ x: 108, y: 198 })
  })
})
