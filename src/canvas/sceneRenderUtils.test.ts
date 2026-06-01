import { describe, expect, it } from 'vitest'

import {
  colorRgbToCss,
  fitExtentToStage,
  konvaStrokeWidthPx,
  polylineToKonvaPoints,
  primitiveKey,
  sceneCapHeightToFontSizePx,
  solidLayerFillCss,
  textAnchorOffsetX,
  textAnchorOffsetY,
} from './sceneRenderUtils'

describe('sceneRenderUtils', () => {
  it('colorRgbToCss formats wire colors', () => {
    expect(colorRgbToCss(0xae3700)).toBe('#ae3700')
    expect(colorRgbToCss(0)).toBe('#000000')
  })

  it('solidLayerFillCss maps DXF layers to canvas fills', () => {
    expect(solidLayerFillCss('FILLS')).toBe('#c0c0c0')
    expect(solidLayerFillCss('INKFILL')).toBe('#000000')
  })

  it('konvaStrokeWidthPx uses schematic hairline for ink and wires', () => {
    expect(konvaStrokeWidthPx(1.0, 'edge-1')).toBe(0.5)
    expect(konvaStrokeWidthPx(1.0)).toBe(0.5)
  })

  it('sceneCapHeightToFontSizePx converts DXF cap height to canvas em size', () => {
    expect(sceneCapHeightToFontSizePx(6.75)).toBe(9)
    expect(sceneCapHeightToFontSizePx(5)).toBeCloseTo(20 / 3, 6)
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

  it('textAnchorOffsetY uses measured text box height', () => {
    expect(textAnchorOffsetY('Top', 9)).toBe(0)
    expect(textAnchorOffsetY('Middle', 9)).toBe(4.5)
    expect(textAnchorOffsetY('Bottom', 9)).toBe(9)
  })

  it('primitiveKey disambiguates wire segments that share edge_id', () => {
    const wire = { edge_id: 'e-1', points: [], stroke_px: 1, layer: 'WIRES', color: 0 }
    expect(primitiveKey('polyline', 3, wire)).toBe('polyline-e-1-3')
    expect(primitiveKey('polyline', 4, wire)).toBe('polyline-e-1-4')
    expect(primitiveKey('polyline', 3, wire)).not.toBe(primitiveKey('polyline', 4, wire))
  })
})
