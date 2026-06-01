import { describe, expect, it } from 'vitest'

import { hitTestGroupingZoneBoundary } from './groupingZoneHitTest'
import type { HitTarget } from './sceneTypes'
import type { FlowNode } from '../tauriIpc'

describe('groupingZoneHitTest', () => {
  const zoneNode: FlowNode = {
    id: 'zone-1',
    type: 'groupingZone',
    position: { x: 100, y: 100 },
    width: 200,
    height: 120,
    data: { shape: 'rect' },
  }

  const boundaryHit: HitTarget = {
    id: 'zone-1:boundary:0',
    bounds: { x: 100, y: 100, width: 200, height: 6 },
    node_id: 'zone-1',
  }

  const wireHit: HitTarget = {
    id: 'edge-1:seg:0',
    bounds: { x: 140, y: 102, width: 80, height: 8 },
    edge_id: 'edge-1',
  }

  it('finds boundary strip hits under overlapping wires', () => {
    const hits = [boundaryHit, wireHit]
    expect(hitTestGroupingZoneBoundary(hits, { x: 150, y: 103 }, [zoneNode])?.node_id).toBe(
      'zone-1',
    )
  })

  it('falls back to distance when strip is thin but pointer is on visible border', () => {
    const hits: HitTarget[] = [wireHit]
    expect(
      hitTestGroupingZoneBoundary(hits, { x: 150, y: 100.5 }, [zoneNode], 8)?.node_id,
    ).toBe('zone-1')
  })
})
