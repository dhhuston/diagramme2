import { describe, expect, it } from 'vitest'
import {
  applySegmentDrag,
  classifySegmentDrag,
  getClosedPolylineSegments,
  fitPolylineToNodeBounds,
  getLabelAnchor,
  getPolylineViewportSize,
  stepSegment,
  type Pt,
} from './groupingZoneGeometry'

describe('grouping zone segment drag geometry', () => {
  it('moves an existing horizontal step from its current jog y without adding vertices', () => {
    const pairs: Pt[] = [[0, 0], [0, 30], [100, 30], [100, 0], [100, 80], [0, 80]]

    expect(applySegmentDrag(pairs, 1, 20)).toEqual([
      [0, 50],
      [100, 50],
      [100, 80],
      [0, 80],
    ])
  })

  it('moves an existing vertical step from its current jog x without adding vertices', () => {
    const pairs: Pt[] = [[0, 0], [40, 0], [40, 100], [0, 100], [120, 100], [120, 0]]

    expect(applySegmentDrag(pairs, 1, 25)).toEqual([
      [65, 0],
      [65, 100],
      [120, 100],
      [120, 0],
    ])
  })

  it('moves either generated connector segment without inserting extra vertices', () => {
    const pairs: Pt[] = [[0, 0], [0, 30], [100, 30], [100, 0], [100, 80], [0, 80]]

    expect(classifySegmentDrag(pairs, 0)).toMatchObject({ kind: 'step-connector', dragDir: 'H' })
    expect(classifySegmentDrag(pairs, 2)).toMatchObject({ kind: 'step-connector', dragDir: 'H' })
    expect(applySegmentDrag(pairs, 0, 15)).toEqual([
      [0, 45],
      [100, 45],
      [100, 80],
      [0, 80],
    ])
    expect(applySegmentDrag(pairs, 2, -10)).toEqual([
      [0, 20],
      [100, 20],
      [100, 80],
      [0, 80],
    ])
  })

  it('inserts exactly two jog vertices for a fresh baseline segment', () => {
    const pairs: Pt[] = [[0, 0], [50, 0], [100, 0], [100, 80], [0, 80]]

    expect(stepSegment(pairs, 0, 30)).toEqual([[0, 0], [0, 30], [50, 30], [50, 0], [100, 0], [100, 80], [0, 80]])
    expect(classifySegmentDrag(pairs, 0).kind).toBe('new-step')
    expect(applySegmentDrag(pairs, 0, 30)).toEqual([[0, 30], [50, 30], [50, 0], [100, 0], [100, 80], [0, 80]])
  })

  it('merges a horizontal step back into one straight segment when dragged in line', () => {
    const pairs: Pt[] = [[0, 0], [0, 30], [100, 30], [100, 0], [100, 80], [0, 80]]

    expect(applySegmentDrag(pairs, 1, -30)).toEqual([[0, 0], [100, 0], [100, 80], [0, 80]])
  })

  it('merges a vertical step back into one straight segment when dragged in line', () => {
    const pairs: Pt[] = [[0, 0], [40, 0], [40, 100], [0, 100], [120, 100], [120, 0]]

    expect(applySegmentDrag(pairs, 1, -40)).toEqual([[0, 0], [0, 100], [120, 100], [120, 0]])
  })

  it('normalizes an existing step for zero and tiny deltas when geometry is backtracking', () => {
    const pairs: Pt[] = [[0, 0], [0, 30], [100, 30], [100, 0], [100, 80], [0, 80]]

    expect(applySegmentDrag(pairs, 1, 0)).toEqual([
      [0, 30],
      [100, 30],
      [100, 80],
      [0, 80],
    ])
    expect(applySegmentDrag(pairs, 1, 1)).toEqual([
      [0, 31],
      [100, 31],
      [100, 80],
      [0, 80],
    ])
  })

  it('removes a stale collinear bottom vertex when translating the outer right segment inward', () => {
    const pairs: Pt[] = [[0, 0], [360, 0], [360, 240], [480, 240], [480, 340], [360, 340], [0, 340]]

    expect(applySegmentDrag(pairs, 3, -80)).toEqual([
      [0, 0],
      [360, 0],
      [360, 240],
      [400, 240],
      [400, 340],
      [0, 340],
    ])
  })

  it('steps a split right-side lower segment outward and inward without leaving the old corner', () => {
    const pairs: Pt[] = [[0, 0], [900, 0], [900, 240], [900, 480], [0, 480]]

    expect(applySegmentDrag(pairs, 2, 140)).toEqual([
      [0, 0],
      [900, 0],
      [900, 240],
      [1040, 240],
      [1040, 480],
      [0, 480],
    ])
    expect(applySegmentDrag(pairs, 2, -150)).toEqual([
      [0, 0],
      [900, 0],
      [900, 240],
      [750, 240],
      [750, 480],
      [0, 480],
    ])
  })

  it('steps split left-side vertical tail segments without leaving stale corners', () => {
    const upperTail: Pt[] = [[0, 0], [900, 0], [900, 480], [0, 480], [0, 240]]
    const lowerTail: Pt[] = [[0, 0], [900, 0], [900, 480], [0, 480], [0, 240]]

    expect(applySegmentDrag(upperTail, 4, -120)).toEqual([
      [900, 0],
      [900, 480],
      [0, 480],
      [0, 240],
      [-120, 240],
      [-120, 0],
    ])
    expect(applySegmentDrag(lowerTail, 3, -120)).toEqual([
      [0, 0],
      [900, 0],
      [900, 480],
      [-120, 480],
      [-120, 240],
      [0, 240],
    ])
  })

  it('steps split top-side horizontal tail segments without leaving stale corners', () => {
    const leftTail: Pt[] = [[0, 0], [450, 0], [900, 0], [900, 480], [0, 480]]
    const rightTail: Pt[] = [[0, 0], [450, 0], [900, 0], [900, 480], [0, 480]]

    expect(applySegmentDrag(leftTail, 0, -100)).toEqual([
      [0, -100],
      [450, -100],
      [450, 0],
      [900, 0],
      [900, 480],
      [0, 480],
    ])
    expect(applySegmentDrag(rightTail, 1, -100)).toEqual([
      [0, 0],
      [450, 0],
      [450, -100],
      [900, -100],
      [900, 480],
      [0, 480],
    ])
  })

  it('steps split bottom-side horizontal tail segments without leaving stale corners', () => {
    const rightTail: Pt[] = [[0, 0], [900, 0], [900, 480], [450, 480], [0, 480]]
    const leftTail: Pt[] = [[0, 0], [900, 0], [900, 480], [450, 480], [0, 480]]

    expect(applySegmentDrag(rightTail, 2, 100)).toEqual([
      [0, 0],
      [900, 0],
      [900, 580],
      [450, 580],
      [450, 480],
      [0, 480],
    ])
    expect(applySegmentDrag(leftTail, 3, 100)).toEqual([
      [0, 0],
      [900, 0],
      [900, 480],
      [450, 480],
      [450, 580],
      [0, 580],
    ])
  })

  it('steps a middle sub-run on a multi-split side while preserving visible neighboring split points', () => {
    const pairs: Pt[] = [[0, 0], [900, 0], [900, 160], [900, 320], [900, 480], [0, 480]]

    expect(applySegmentDrag(pairs, 2, 120)).toEqual([
      [0, 0],
      [900, 0],
      [900, 160],
      [1020, 160],
      [1020, 320],
      [900, 320],
      [900, 480],
      [0, 480],
    ])
  })

  it('merges a split-tail step back into the original side when dragged back in line', () => {
    const pairs: Pt[] = [[0, 0], [900, 0], [900, 240], [1040, 240], [1040, 480], [0, 480]]

    expect(applySegmentDrag(pairs, 3, -140)).toEqual([
      [0, 0],
      [900, 0],
      [900, 480],
      [0, 480],
    ])
  })
})

describe('grouping zone label anchor geometry', () => {
  it('anchors a rectangle label to the left endpoint of the top horizontal segment', () => {
    expect(getLabelAnchor([[0, 0], [240, 0], [240, 180], [0, 180]])).toEqual({ x: 0, y: 0 })
  })

  it('anchors to the left endpoint of the topmost horizontal segment in a notched shape', () => {
    const pairs: Pt[] = [[0, 80], [420, 80], [420, 40], [260, 40], [260, 220], [0, 220]]

    expect(getLabelAnchor(pairs)).toEqual({ x: 260, y: 40 })
  })

  it('chooses the leftmost segment when multiple horizontal segments share the topmost y', () => {
    const pairs: Pt[] = [[80, 0], [240, 0], [240, 80], [320, 80], [320, 0], [420, 0], [420, 180], [80, 180]]

    expect(getLabelAnchor(pairs)).toEqual({ x: 80, y: 0 })
  })

  it('uses the left endpoint regardless of horizontal segment point order', () => {
    const pairs: Pt[] = [[0, 180], [240, 180], [240, 0], [0, 0]]

    expect(getLabelAnchor(pairs)).toEqual({ x: 0, y: 0 })
  })

  it('anchors to the topmost horizontal segment after split/generated geometry is normalized', () => {
    const pairs: Pt[] = [[0, 0], [900, 0], [900, 160], [1020, 160], [1020, 320], [900, 320], [900, 480], [0, 480]]

    expect(getLabelAnchor(pairs)).toEqual({ x: 0, y: 0 })
  })
})

describe('grouping zone bounds fitting', () => {
  it('keeps already in-bounds geometry at the same local coordinates', () => {
    const fitted = fitPolylineToNodeBounds([[0, 0], [240, 0], [240, 180], [0, 180]])

    expect(fitted).toEqual({
      pairs: [[0, 0], [240, 0], [240, 180], [0, 180]],
      offset: { x: 0, y: 0 },
      size: { width: 240, height: 180 },
    })
  })

  it('rebases geometry that extends right and down beyond the current node box', () => {
    const fitted = fitPolylineToNodeBounds([[0, 0], [900, 0], [900, 160], [1020, 160], [1020, 320], [0, 320]])

    expect(fitted).toEqual({
      pairs: [[0, 0], [900, 0], [900, 160], [1020, 160], [1020, 320], [0, 320]],
      offset: { x: 0, y: 0 },
      size: { width: 1020, height: 320 },
    })
  })

  it('rebases geometry that extends left and up outside the current node origin', () => {
    const fitted = fitPolylineToNodeBounds([[-120, -40], [240, -40], [240, 180], [-120, 180]])

    expect(fitted).toEqual({
      pairs: [[0, 0], [360, 0], [360, 220], [0, 220]],
      offset: { x: -120, y: -40 },
      size: { width: 360, height: 220 },
    })
  })

  it('uses the live polyline extents when they exceed the fallback viewport', () => {
    expect(getPolylineViewportSize([[0, 0], [360, 0], [360, 220], [0, 220]], 240, 180)).toEqual({
      width: 360,
      height: 220,
    })
  })

  it('keeps the fallback viewport when it is already large enough', () => {
    expect(getPolylineViewportSize([[0, 0], [240, 0], [240, 180], [0, 180]], 360, 220)).toEqual({
      width: 360,
      height: 220,
    })
  })
})

describe('grouping zone boundary segments', () => {
  it('returns one closed segment per edge with direction metadata', () => {
    expect(getClosedPolylineSegments([[0, 0], [240, 0], [240, 180], [0, 180]])).toEqual([
      { ax: 0, ay: 0, bx: 240, by: 0, dir: 'H', i: 0 },
      { ax: 240, ay: 0, bx: 240, by: 180, dir: 'V', i: 1 },
      { ax: 240, ay: 180, bx: 0, by: 180, dir: 'H', i: 2 },
      { ax: 0, ay: 180, bx: 0, by: 0, dir: 'V', i: 3 },
    ])
  })

  it('preserves split and stepped edges for boundary-only hit targets', () => {
    expect(getClosedPolylineSegments([[0, 0], [240, 0], [240, 90], [320, 90], [320, 180], [0, 180]])).toEqual([
      { ax: 0, ay: 0, bx: 240, by: 0, dir: 'H', i: 0 },
      { ax: 240, ay: 0, bx: 240, by: 90, dir: 'V', i: 1 },
      { ax: 240, ay: 90, bx: 320, by: 90, dir: 'H', i: 2 },
      { ax: 320, ay: 90, bx: 320, by: 180, dir: 'V', i: 3 },
      { ax: 320, ay: 180, bx: 0, by: 180, dir: 'H', i: 4 },
      { ax: 0, ay: 180, bx: 0, by: 0, dir: 'V', i: 5 },
    ])
  })
})
