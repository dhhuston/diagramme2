import { describe, expect, it } from 'vitest'

import {
  belongsToNodeDrag,
  nodeDragCaptureBounds,
  offsetScenePrimitive,
  snapPlacementCoord,
  snappedNodeOrigin,
} from './dragNode'
import type { ScenePrimitive } from '../sceneTypes'

describe('dragNode', () => {
  it('snapPlacementCoord snaps to 3px grid', () => {
    expect(snapPlacementCoord(0)).toBe(0)
    expect(snapPlacementCoord(7)).toBe(6)
    expect(snapPlacementCoord(8)).toBe(9)
  })

  it('nodeDragCaptureBounds includes tag band and bracket pad', () => {
    const bounds = nodeDragCaptureBounds({ x: 100, y: 200, width: 72, height: 48 })
    expect(bounds.x).toBe(76)
    expect(bounds.y).toBe(185)
    expect(bounds.width).toBe(120)
    expect(bounds.height).toBe(63)
  })

  it('belongsToNodeDrag excludes wires and includes node solids', () => {
    const capture = { x: 0, y: 0, width: 100, height: 100 }
    const wire: ScenePrimitive = {
      Polyline: {
        points: [{ x: 10, y: 10 }, { x: 50, y: 50 }],
        stroke_px: 1,
        layer: 'WIRES',
        color: 0,
        edge_id: 'edge-1',
      },
    }
    const solid: ScenePrimitive = {
      Solid: {
        vertices: [
          { x: 10, y: 10 },
          { x: 20, y: 10 },
          { x: 20, y: 20 },
          { x: 10, y: 20 },
        ],
        layer: 'FILLS',
        node_id: 'node-a',
      },
    }
    expect(belongsToNodeDrag(wire, 'node-a', capture)).toBe(false)
    expect(belongsToNodeDrag(solid, 'node-a', capture)).toBe(true)
  })

  it('offsetScenePrimitive shifts text position', () => {
    const text: ScenePrimitive = {
      Text: {
        position: { x: 1, y: 2 },
        content: 'A',
        height_px: 5,
        halign: 'Left',
        valign: 'Middle',
        font: 'Arial Narrow',
      },
    }
    const moved = offsetScenePrimitive(text, 3, 4)
    expect(moved).toEqual({
      Text: {
        position: { x: 4, y: 6 },
        content: 'A',
        height_px: 5,
        halign: 'Left',
        valign: 'Middle',
        font: 'Arial Narrow',
      },
    })
  })

  it('snappedNodeOrigin applies grid snap on drop', () => {
    const preview = {
      nodeId: 'n1',
      captureBounds: { x: 0, y: 0, width: 10, height: 10 },
      origin: { x: 100, y: 200 },
      dx: 7,
      dy: -2,
    }
    expect(snappedNodeOrigin(preview)).toEqual({ x: 108, y: 198 })
  })
})
