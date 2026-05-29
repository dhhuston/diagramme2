import { describe, expect, it } from 'vitest'

import {
  colorRgbToCss,
  fitExtentToStage,
  polylineToKonvaPoints,
  textAnchorOffsetX,
  textAnchorOffsetY,
} from './sceneRenderUtils'

describe('sceneRenderUtils', () => {
  it('colorRgbToCss formats wire colors', () => {
    expect(colorRgbToCss(0xae3700)).toBe('#ae3700')
    expect(colorRgbToCss(0)).toBe('#000000')
  })

  it('polylineToKonvaPoints flattens diagram points', () => {
    expect(
      polylineToKonvaPoints([
        { x: 1, y: 2 },
        { x: 3, y: 4 },
      ]),
    ).toEqual([1, 2, 3, 4])
  })

  it('fitExtentToStage scales diagram into stage', () => {
    const fit = fitExtentToStage(
      { x: 100, y: 50, width: 1000, height: 500 },
      1200,
      800,
      48,
    )
    expect(fit.scale).toBeGreaterThan(0)
    expect(fit.x).toBeLessThan(48)
    expect(fit.y).toBeLessThan(48)
  })

  it('textAnchorOffsetX matches DXF horizontal alignment anchor', () => {
    expect(textAnchorOffsetX('Left', 40)).toBe(0)
    expect(textAnchorOffsetX('Center', 40)).toBe(20)
    expect(textAnchorOffsetX('Right', 40)).toBe(40)
  })

  it('textAnchorOffsetY matches DXF vertical alignment anchor', () => {
    expect(textAnchorOffsetY('Top', 9)).toBe(0)
    expect(textAnchorOffsetY('Middle', 9)).toBe(4.5)
    expect(textAnchorOffsetY('Bottom', 9)).toBe(9)
  })
})
