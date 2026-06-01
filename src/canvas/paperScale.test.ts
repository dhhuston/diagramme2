import { describe, expect, it } from 'vitest'

import {
  CONNECTOR_LINE_PITCH_PX,
  isOnGridStep,
  snapPlacementCoord,
  SNAP_HALF_STEP_PX,
} from './paperScale'

describe('paperScale', () => {
  it('connector row lines in Comp Gym sit on half-grid and 9px pitch', () => {
    const nodeY = 121.5
    const titleDivider = nodeY + 18
    const rowDividers = [titleDivider, titleDivider + 9, titleDivider + 18]
    for (const y of rowDividers) {
      expect(isOnGridStep(y, SNAP_HALF_STEP_PX)).toBe(true)
    }
    const portCenterY = nodeY + 22.5
    expect(isOnGridStep(portCenterY, CONNECTOR_LINE_PITCH_PX)).toBe(true)
    expect(snapPlacementCoord(nodeY)).toBe(121.5)
  })
})
