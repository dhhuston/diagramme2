import { describe, expect, it } from 'vitest'

import { hitTestScene, pointInRect, stagePointerToDiagramPx } from './hitTest'
import type { HitTarget } from './sceneTypes'

describe('hitTest', () => {
  it('stagePointerToDiagramPx inverts viewport transform', () => {
    const diagram = stagePointerToDiagramPx(
      { x: 148, y: 98 },
      { scale: 2, x: 48, y: 48 },
    )
    expect(diagram).toEqual({ x: 50, y: 25 })
  })

  it('pointInRect uses inclusive bounds', () => {
    const rect = { x: 10, y: 20, width: 30, height: 40 }
    expect(pointInRect({ x: 10, y: 20 }, rect)).toBe(true)
    expect(pointInRect({ x: 40, y: 60 }, rect)).toBe(true)
    expect(pointInRect({ x: 41, y: 60 }, rect)).toBe(false)
  })

  it('hitTestScene returns top-most target', () => {
    const hits: HitTarget[] = [
      { id: 'back', bounds: { x: 0, y: 0, width: 100, height: 100 } },
      { id: 'front', bounds: { x: 10, y: 10, width: 20, height: 20 }, node_id: 'node-a' },
    ]
    expect(hitTestScene(hits, { x: 15, y: 15 })?.id).toBe('front')
    expect(hitTestScene(hits, { x: 90, y: 90 })?.id).toBe('back')
    expect(hitTestScene(hits, { x: 200, y: 200 })).toBeNull()
  })
})
