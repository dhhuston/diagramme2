import { describe, expect, it } from 'vitest'

import { nodeSelectionBounds, unionBounds } from './selectionBounds'
import type { HitTarget } from './sceneTypes'

describe('selectionBounds', () => {
  it('unionBounds merges rectangles', () => {
    expect(
      unionBounds([
        { x: 0, y: 0, width: 10, height: 10 },
        { x: 8, y: 5, width: 20, height: 4 },
      ]),
    ).toEqual({ x: 0, y: 0, width: 28, height: 10 })
  })

  it('nodeSelectionBounds prefers body hit id matching node', () => {
    const hits: HitTarget[] = [
      {
        id: 'n1:L-0-in',
        node_id: 'n1',
        handle_id: 'L-0-in',
        bounds: { x: 0, y: 10, width: 40, height: 9 },
      },
      {
        id: 'n1',
        node_id: 'n1',
        bounds: { x: 0, y: 0, width: 80, height: 50 },
      },
    ]
    expect(nodeSelectionBounds(hits, 'n1')).toEqual({ x: 0, y: 0, width: 80, height: 50 })
  })

  it('nodeSelectionBounds unions grouping zone boundary strips', () => {
    const hits: HitTarget[] = [
      {
        id: 'zone:boundary:0',
        node_id: 'zone',
        bounds: { x: 100, y: 100, width: 200, height: 6 },
      },
      {
        id: 'zone:boundary:2',
        node_id: 'zone',
        bounds: { x: 100, y: 100, width: 6, height: 120 },
      },
      {
        id: 'zone:boundary:1',
        node_id: 'zone',
        bounds: { x: 100, y: 214, width: 200, height: 6 },
      },
      {
        id: 'zone:boundary:3',
        node_id: 'zone',
        bounds: { x: 294, y: 100, width: 6, height: 120 },
      },
    ]
    expect(nodeSelectionBounds(hits, 'zone')).toEqual({
      x: 100,
      y: 100,
      width: 200,
      height: 120,
    })
  })
})
