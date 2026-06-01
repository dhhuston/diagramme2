import { describe, expect, it } from 'vitest'

import {
  hitArea,
  hitTestSceneForInteraction,
  hitTestSceneForSelection,
  isGroupingZoneBoundaryHit,
  pointInRect,
  stagePointerToDiagramPx,
} from './hitTest'
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

  it('hitTestSceneForInteraction returns top-most target', () => {
    const hits: HitTarget[] = [
      { id: 'back', bounds: { x: 0, y: 0, width: 100, height: 100 } },
      {
        id: 'front',
        bounds: { x: 10, y: 10, width: 20, height: 20 },
        node_id: 'node-a',
      },
    ]
    expect(hitTestSceneForInteraction(hits, { x: 15, y: 15 })?.id).toBe('front')
    expect(hitTestSceneForInteraction(hits, { x: 90, y: 90 })?.id).toBe('back')
  })

  it('hitTestSceneForSelection prefers node over grouping zone interior', () => {
    const zone: HitTarget = {
      id: 'zone-1:boundary:0',
      bounds: { x: 0, y: 0, width: 200, height: 8 },
      node_id: 'zone-1',
    }
    const device: HitTarget = {
      id: 'device-1',
      bounds: { x: 50, y: 50, width: 80, height: 60 },
      node_id: 'device-1',
    }
    const hits = [zone, device]
    expect(isGroupingZoneBoundaryHit(zone)).toBe(true)
    expect(hitTestSceneForSelection(hits, { x: 80, y: 80 })?.node_id).toBe('device-1')
    expect(hitTestSceneForSelection(hits, { x: 10, y: 4 })?.node_id).toBe('zone-1')
  })

  it('hitTestSceneForSelection prefers wire segment over overlapping speaker body', () => {
    const speaker: HitTarget = {
      id: 'speakerBlock-1',
      bounds: { x: 850, y: 20, width: 120, height: 40 },
      node_id: 'speakerBlock-1',
    }
    const wire: HitTarget = {
      id: 'e-spk:seg:2',
      bounds: { x: 900, y: 38, width: 60, height: 8 },
      edge_id: 'e-spk',
    }
    const hits = [speaker, wire]
    expect(hitTestSceneForSelection(hits, { x: 920, y: 42 })?.edge_id).toBe('e-spk')
  })

  it('hitTestSceneForSelection prefers node body over port strip on same node', () => {
    const body: HitTarget = {
      id: 'n1',
      bounds: { x: 100, y: 100, width: 80, height: 50 },
      node_id: 'n1',
    }
    const port: HitTarget = {
      id: 'n1:L-0-in',
      bounds: { x: 100, y: 120, width: 40, height: 9 },
      node_id: 'n1',
      handle_id: 'L-0-in',
    }
    const hits = [body, port]
    expect(hitArea(body)).toBeGreaterThan(hitArea(port))
    expect(hitTestSceneForSelection(hits, { x: 110, y: 122 })?.id).toBe('n1')
    expect(hitTestSceneForInteraction(hits, { x: 110, y: 122 })?.handle_id).toBe('L-0-in')
  })

  it('hitTestSceneForInteraction skips wire grips unless edge is selected', () => {
    const wire: HitTarget = {
      id: 'edge-1:seg:0',
      bounds: { x: 40, y: 98, width: 80, height: 8 },
      edge_id: 'edge-1',
    }
    const grip: HitTarget = {
      id: 'edge-1:grip:0:h',
      bounds: { x: 76, y: 96, width: 8, height: 8 },
      edge_id: 'edge-1',
      wire_grip_segment: 0,
      wire_grip_orientation: 'h',
    }
    const hits = [wire, grip]
    expect(hitTestSceneForInteraction(hits, { x: 80, y: 100 })?.id).toBe('edge-1:seg:0')
    expect(hitTestSceneForInteraction(hits, { x: 80, y: 100 }, 'edge-1')?.id).toBe(
      'edge-1:grip:0:h',
    )
  })
})
