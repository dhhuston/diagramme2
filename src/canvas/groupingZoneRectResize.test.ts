import { describe, expect, it } from 'vitest'

import {
  hitTestRectResizeHandle,
  resizeRectFromHandle,
} from './groupingZoneRectResize'

describe('groupingZoneRectResize', () => {
  const start = { x: 100, y: 100, width: 200, height: 120 }

  it('hit-tests corner handles', () => {
    expect(hitTestRectResizeHandle({ x: 100, y: 100 }, start)).toBe('nw')
    expect(hitTestRectResizeHandle({ x: 300, y: 220 }, start)).toBe('se')
  })

  it('resizes from east handle', () => {
    expect(resizeRectFromHandle(start, 'e', { x: 360, y: 160 }, 80, 60)).toEqual({
      x: 100,
      y: 100,
      width: 260,
      height: 120,
    })
  })

  it('resizes from west handle and shifts origin', () => {
    expect(resizeRectFromHandle(start, 'w', { x: 80, y: 160 }, 80, 60)).toEqual({
      x: 80,
      y: 100,
      width: 220,
      height: 120,
    })
  })

  it('enforces minimum size', () => {
    expect(resizeRectFromHandle(start, 'e', { x: 120, y: 160 }, 80, 60)).toEqual({
      x: 100,
      y: 100,
      width: 80,
      height: 120,
    })
  })
})
