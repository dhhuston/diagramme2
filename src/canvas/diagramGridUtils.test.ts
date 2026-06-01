import { describe, expect, it } from 'vitest'

import { visibleDiagramBounds } from './diagramGridUtils'

describe('diagramGridUtils', () => {
  it('visibleDiagramBounds maps stage corners to diagram space', () => {
    const bounds = visibleDiagramBounds({ x: 0, y: 0, scale: 1 }, 800, 600, 0)
    expect(bounds.x).toBeCloseTo(0)
    expect(bounds.y).toBeCloseTo(0)
    expect(bounds.width).toBe(800)
    expect(bounds.height).toBe(600)
  })
})
