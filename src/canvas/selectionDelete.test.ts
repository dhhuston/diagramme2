import { describe, expect, it } from 'vitest'

import type { HitTarget } from './sceneTypes'
import { deleteLabelForTarget, deleteTargetFromHit } from './selectionDelete'

const bounds = { x: 0, y: 0, width: 10, height: 10 }

describe('deleteTargetFromHit', () => {
  it('returns node when node body is selected', () => {
    const hit: HitTarget = { id: 'n1', bounds, node_id: 'n1' }
    expect(deleteTargetFromHit(hit)).toEqual({ kind: 'node', nodeId: 'n1' })
  })

  it('returns edge when wire is selected', () => {
    const hit: HitTarget = { id: 'e1', bounds, edge_id: 'e1' }
    expect(deleteTargetFromHit(hit)).toEqual({ kind: 'edge', edgeId: 'e1' })
  })

  it('returns null for port handles', () => {
    const hit: HitTarget = { id: 'h1', bounds, node_id: 'n1', handle_id: 'port-1' }
    expect(deleteTargetFromHit(hit)).toBeNull()
  })

  it('returns null for wire routing grips', () => {
    const hit: HitTarget = {
      id: 'g1',
      bounds,
      edge_id: 'e1',
      wire_grip_segment: 0,
      wire_grip_orientation: 'h',
    }
    expect(deleteTargetFromHit(hit)).toBeNull()
  })
})

describe('deleteLabelForTarget', () => {
  it('labels node and wire deletes', () => {
    expect(deleteLabelForTarget({ kind: 'node', nodeId: 'n1' })).toBe('Delete node')
    expect(deleteLabelForTarget({ kind: 'edge', edgeId: 'e1' })).toBe('Delete wire')
    expect(deleteLabelForTarget(null)).toBe('Delete')
  })
})
