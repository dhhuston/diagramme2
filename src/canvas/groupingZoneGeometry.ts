export type Pt = [number, number]

export type SegmentDragKind = 'translate' | 'step-inner' | 'step-connector' | 'new-step'

export interface SegmentDragClassification {
  kind: SegmentDragKind
  dragDir: 'H' | 'V'
  stepVertexIndices?: [number, number]
}

export interface LabelAnchor {
  x: number
  y: number
}

export interface PolylineBoundsFit {
  pairs: Pt[]
  offset: { x: number; y: number }
  size: { width: number; height: number }
}

export interface PolylineViewportSize {
  width: number
  height: number
}

export interface PolylineSegment {
  ax: number
  ay: number
  bx: number
  by: number
  dir: 'H' | 'V'
  i: number
}

export function toPairs(pts: number[]): Pt[] {
  const out: Pt[] = []
  for (let i = 0; i + 1 < pts.length; i += 2) out.push([pts[i], pts[i + 1]])
  return out
}

export function fromPairs(pairs: Pt[]): number[] {
  return pairs.flatMap(([x, y]) => [x, y])
}

/** Default orthogonal rectangle for the initial bounding box. */
export function defaultOrtho(w: number, h: number): number[] {
  return [0, 0, w, 0, w, h, 0, h]
}

/**
 * Classify a segment as 'H' (horizontal) or 'V' (vertical).
 */
export function segDir(ax: number, ay: number, bx: number, by: number): 'H' | 'V' {
  return Math.abs(by - ay) < Math.abs(bx - ax) ? 'H' : 'V'
}

export function getLabelAnchor(pairs: Pt[]): LabelAnchor {
  let best: { x: number; y: number } | null = null
  const n = pairs.length

  for (let i = 0; i < n; i += 1) {
    const [ax, ay] = pairs[i]
    const [bx, by] = pairs[(i + 1) % n]
    if (segDir(ax, ay, bx, by) !== 'H') continue

    const x = Math.min(ax, bx)
    const y = ay
    if (
      !best
      || y < best.y
      || (y === best.y && x < best.x)
    ) {
      best = { x, y }
    }
  }

  return best ? { x: best.x, y: best.y } : { x: 0, y: 0 }
}

export function fitPolylineToNodeBounds(pairs: Pt[]): PolylineBoundsFit {
  if (pairs.length === 0) {
    return {
      pairs: [],
      offset: { x: 0, y: 0 },
      size: { width: 0, height: 0 },
    }
  }

  let minX = pairs[0][0]
  let minY = pairs[0][1]
  let maxX = pairs[0][0]
  let maxY = pairs[0][1]

  for (const [x, y] of pairs) {
    minX = Math.min(minX, x)
    minY = Math.min(minY, y)
    maxX = Math.max(maxX, x)
    maxY = Math.max(maxY, y)
  }

  return {
    pairs: pairs.map(([x, y]) => [x - minX, y - minY] as Pt),
    offset: { x: minX, y: minY },
    size: { width: maxX - minX, height: maxY - minY },
  }
}

export function getPolylineViewportSize(
  pairs: Pt[],
  fallbackWidth: number,
  fallbackHeight: number,
): PolylineViewportSize {
  const fitted = fitPolylineToNodeBounds(pairs)
  return {
    width: Math.max(fallbackWidth, fitted.size.width),
    height: Math.max(fallbackHeight, fitted.size.height),
  }
}

export function getClosedPolylineSegments(pairs: Pt[]): PolylineSegment[] {
  return pairs.map(([ax, ay], i) => {
    const [bx, by] = pairs[(i + 1) % pairs.length]
    return { ax, ay, bx, by, dir: segDir(ax, ay, bx, by), i }
  })
}

/**
 * Returns the dominant drag axis for a vertex based on its adjacent segments.
 *   V + V -> 'ew'   (jog on a vertical run - drag horizontally)
 *   H + H -> 'ns'   (jog on a horizontal run - drag vertically)
 *   mixed -> 'move' (true corner - axis-locked by first movement >=4px)
 */
export function vertexDragAxis(pairs: Pt[], idx: number): 'ew' | 'ns' | 'move' {
  const n = pairs.length
  const prevIdx = (idx - 1 + n) % n
  const nextIdx = (idx + 1) % n
  const [px, py] = pairs[prevIdx]
  const [vx, vy] = pairs[idx]
  const [nx, ny] = pairs[nextIdx]
  const inDir = segDir(px, py, vx, vy)
  const outDir = segDir(vx, vy, nx, ny)
  if (inDir === 'V' && outDir === 'V') return 'ew'
  if (inDir === 'H' && outDir === 'H') return 'ns'
  return 'move'
}

/**
 * Walk outward from `startIdx` (which was just updated), away from `fromIdx`,
 * propagating the coordinate change through any consecutive jog vertices until
 * reaching a corner ('move') or wrapping all the way around.
 */
function propagateFromVertex(out: Pt[], startIdx: number, fromIdx: number, n: number): void {
  let cur = startIdx
  let coming = fromIdx
  const visited = new Set<number>()
  for (;;) {
    visited.add(cur)
    const axis = vertexDragAxis(out, cur)
    if (axis === 'move') break
    const prevOfCur = (cur - 1 + n) % n
    const nextOfCur = (cur + 1) % n
    const other = prevOfCur === coming ? nextOfCur : prevOfCur
    if (visited.has(other)) break
    if (axis === 'ew') {
      out[other] = [out[cur][0], out[other][1]]
    } else {
      out[other] = [out[other][0], out[cur][1]]
    }
    coming = cur
    cur = other
  }
}

/**
 * Move vertex at index `idx` to (nx, ny), adjusting the two adjacent vertices
 * so all segments remain orthogonal.
 */
export function moveVertex(pairs: Pt[], idx: number, nx: number, ny: number): Pt[] {
  const n = pairs.length
  const out: Pt[] = pairs.map(([x, y]) => [x, y])

  const prevIdx = (idx - 1 + n) % n
  const nextIdx = (idx + 1) % n

  const [px, py] = out[prevIdx]
  const [qx, qy] = out[nextIdx]

  const inDir = segDir(px, py, out[idx][0], out[idx][1])
  const outDir = segDir(out[idx][0], out[idx][1], qx, qy)

  out[idx] = [nx, ny]

  if (inDir === 'H') {
    out[prevIdx] = [px, ny]
  } else {
    out[prevIdx] = [nx, py]
  }

  if (outDir === 'H') {
    out[nextIdx] = [qx, ny]
  } else {
    out[nextIdx] = [nx, qy]
  }

  propagateFromVertex(out, prevIdx, idx, n)
  propagateFromVertex(out, nextIdx, idx, n)

  return out
}

/**
 * Step an entire segment perpendicularly by `delta` (dX for V seg, dY for H seg).
 * Inserts two new vertices at the stepped position so all adjacent segments
 * remain orthogonal and unchanged.
 */
export function stepSegment(pairs: Pt[], segIdx: number, delta: number): Pt[] {
  const n = pairs.length
  const a = segIdx
  const b = (segIdx + 1) % n
  const [ax, ay] = pairs[a]
  const [bx, by] = pairs[b]
  const dir = segDir(ax, ay, bx, by)
  const out = [...pairs]
  const insertPos = b === 0 ? out.length : b
  if (dir === 'V') {
    out.splice(insertPos, 0, [ax + delta, ay] as Pt, [ax + delta, by] as Pt)
  } else {
    out.splice(insertPos, 0, [ax, ay + delta] as Pt, [bx, ay + delta] as Pt)
  }
  return out
}

function nearlyEqual(a: number, b: number): boolean {
  return Math.abs(a - b) <= 1
}

function detectStepSpan(pairs: Pt[], segIdx: number, requireOffset: boolean): [number, number] | null {
  const n = pairs.length
  if (n <= 4) return null
  const a = segIdx
  const b = (segIdx + 1) % n
  const prevA = (a - 1 + n) % n
  const nextB = (b + 1) % n
  const [ax, ay] = pairs[a]
  const [bx, by] = pairs[b]
  const [px, py] = pairs[prevA]
  const [nx, ny] = pairs[nextB]
  const innerDir = segDir(ax, ay, bx, by)

  if (innerDir === 'H') {
    if (!nearlyEqual(ay, by)) return null
    if (!nearlyEqual(py, ny)) return null
    if (requireOffset && nearlyEqual(py, ay)) return null
    if (!nearlyEqual(px, ax)) return null
    if (!nearlyEqual(nx, bx)) return null
  } else {
    if (!nearlyEqual(ax, bx)) return null
    if (!nearlyEqual(px, nx)) return null
    if (requireOffset && nearlyEqual(px, ax)) return null
    if (!nearlyEqual(py, ay)) return null
    if (!nearlyEqual(ny, by)) return null
  }

  return [a, b]
}

function detectStepInner(pairs: Pt[], segIdx: number): [number, number] | null {
  return detectStepSpan(pairs, segIdx, true)
}

function findStepForSegment(pairs: Pt[], segIdx: number): {
  kind: 'step-inner' | 'step-connector'
  stepVertexIndices: [number, number]
  dragDir: 'H' | 'V'
} | null {
  const n = pairs.length
  const inner = detectStepInner(pairs, segIdx)
  if (inner) {
    const [a, b] = inner
    const [ax, ay] = pairs[a]
    const [bx, by] = pairs[b]
    return { kind: 'step-inner', stepVertexIndices: inner, dragDir: segDir(ax, ay, bx, by) }
  }

  const leftInnerIdx = (segIdx + 1) % n
  const left = detectStepInner(pairs, leftInnerIdx)
  if (left) {
    const [a, b] = left
    const [ax, ay] = pairs[a]
    const [bx, by] = pairs[b]
    return { kind: 'step-connector', stepVertexIndices: left, dragDir: segDir(ax, ay, bx, by) }
  }

  const rightInnerIdx = (segIdx - 1 + n) % n
  const right = detectStepInner(pairs, rightInnerIdx)
  if (right) {
    const [a, b] = right
    const [ax, ay] = pairs[a]
    const [bx, by] = pairs[b]
    return { kind: 'step-connector', stepVertexIndices: right, dragDir: segDir(ax, ay, bx, by) }
  }

  return null
}

export function classifySegmentDrag(pairs: Pt[], segIdx: number): SegmentDragClassification {
  const step = findStepForSegment(pairs, segIdx)
  if (step) return step

  const n = pairs.length
  const [ax, ay] = pairs[segIdx]
  const [bx, by] = pairs[(segIdx + 1) % n]
  const dragDir = segDir(ax, ay, bx, by)
  const axisA = vertexDragAxis(pairs, segIdx)
  const axisB = vertexDragAxis(pairs, (segIdx + 1) % n)
  if (axisA === 'move' && axisB === 'move') return { kind: 'translate', dragDir }
  return { kind: 'new-step', dragDir }
}

function moveStepVertices(pairs: Pt[], idxA: number, idxB: number, delta: number): Pt[] {
  const out = pairs.map(([x, y]) => [x, y] as Pt)
  const [ax, ay] = out[idxA]
  const [bx, by] = out[idxB]
  const dir = segDir(ax, ay, bx, by)

  if (dir === 'V') {
    out[idxA] = [ax + delta, ay]
    out[idxB] = [bx + delta, by]
  } else {
    out[idxA] = [ax, ay + delta]
    out[idxB] = [bx, by + delta]
  }

  return out
}

function samePoint(a: Pt, b: Pt): boolean {
  return nearlyEqual(a[0], b[0]) && nearlyEqual(a[1], b[1])
}

function isAlignedStep(pairs: Pt[], idxA: number, idxB: number): boolean {
  const n = pairs.length
  const prevA = (idxA - 1 + n) % n
  const nextB = (idxB + 1) % n
  const [ax, ay] = pairs[idxA]
  const [bx, by] = pairs[idxB]
  const [px, py] = pairs[prevA]
  const [nx, ny] = pairs[nextB]
  const dir = segDir(ax, ay, bx, by)

  if (dir === 'H') {
    return nearlyEqual(py, ay) && nearlyEqual(ny, by)
  }
  return nearlyEqual(px, ax) && nearlyEqual(nx, bx)
}

function isCollinearBetweenNeighbors(pairs: Pt[], idx: number): boolean {
  const prev = (idx - 1 + pairs.length) % pairs.length
  const next = (idx + 1) % pairs.length
  const [px, py] = pairs[prev]
  const [x, y] = pairs[idx]
  const [nx, ny] = pairs[next]

  if (nearlyEqual(px, x) && nearlyEqual(x, nx)) {
    return y >= Math.min(py, ny) && y <= Math.max(py, ny)
  }
  if (nearlyEqual(py, y) && nearlyEqual(y, ny)) {
    return x >= Math.min(px, nx) && x <= Math.max(px, nx)
  }
  return false
}

function isCollinearBacktrack(pairs: Pt[], idx: number): boolean {
  const prev = (idx - 1 + pairs.length) % pairs.length
  const next = (idx + 1) % pairs.length
  const [px, py] = pairs[prev]
  const [x, y] = pairs[idx]
  const [nx, ny] = pairs[next]

  if (nearlyEqual(py, y) && nearlyEqual(y, ny)) {
    return x < Math.min(px, nx) || x > Math.max(px, nx)
  }
  if (nearlyEqual(px, x) && nearlyEqual(x, nx)) {
    return y < Math.min(py, ny) || y > Math.max(py, ny)
  }
  return false
}

export function normalizePolyline(pairs: Pt[]): Pt[] {
  let out = pairs.map(([x, y]) => [x, y] as Pt)
  let changed = true

  while (changed && out.length > 4) {
    changed = false

    for (let i = 0; i < out.length && out.length > 4; i += 1) {
      const next = (i + 1) % out.length
      if (samePoint(out[i], out[next])) {
        out = out.filter((_, idx) => idx !== next)
        changed = true
        break
      }
    }
    if (changed) continue

    for (let i = 0; i < out.length && out.length > 4; i += 1) {
      const step = detectStepSpan(out, i, false)
      if (step && isAlignedStep(out, step[0], step[1])) {
        out = out.filter((_, idx) => idx !== step[0] && idx !== step[1])
        changed = true
        break
      }
    }
    if (changed) continue

    for (let i = 0; i < out.length && out.length > 4; i += 1) {
      if (isCollinearBetweenNeighbors(out, i)) {
        out = out.filter((_, idx) => idx !== i)
        changed = true
        break
      }
    }
    if (changed) continue

    for (let i = 0; i < out.length && out.length > 4; i += 1) {
      if (isCollinearBacktrack(out, i)) {
        out = out.filter((_, idx) => idx !== i)
        changed = true
        break
      }
    }
  }

  return out
}

export function applySegmentDrag(pairs: Pt[], segIdx: number, delta: number): Pt[] {
  const classification = classifySegmentDrag(pairs, segIdx)
  let out: Pt[]

  if (classification.stepVertexIndices) {
    const [idxA, idxB] = classification.stepVertexIndices
    out = moveStepVertices(pairs, idxA, idxB, delta)
  } else if (classification.kind === 'translate') {
    const [ax, ay] = pairs[segIdx]
    out = moveVertex(
      pairs,
      segIdx,
      classification.dragDir === 'V' ? ax + delta : ax,
      classification.dragDir === 'H' ? ay + delta : ay,
    )
  } else {
    out = stepSegment(pairs, segIdx, delta)
  }

  return normalizePolyline(out)
}

/**
 * Insert a new vertex on segment segIdx at the click position.
 */
export function insertVertex(pairs: Pt[], segIdx: number, cx: number, cy: number): Pt[] {
  const n = pairs.length
  const a = segIdx
  const b = (segIdx + 1) % n
  const [ax, ay] = pairs[a]
  const [bx, by] = pairs[b]
  const dir = segDir(ax, ay, bx, by)

  const snapped: Pt = dir === 'H'
    ? [Math.round(Math.max(Math.min(ax, bx), Math.min(Math.max(ax, bx), cx))), ay]
    : [ax, Math.round(Math.max(Math.min(ay, by), Math.min(Math.max(ay, by), cy)))]

  const result = [...pairs]
  const insertPos = b === 0 ? result.length : b
  result.splice(insertPos, 0, snapped)
  return result
}

/**
 * Remove a vertex that is collinear with its neighbours (makes a zero-length
 * segment on one side, or is on the same line). Used on double-click.
 */
export function removeVertex(pairs: Pt[], idx: number): Pt[] | null {
  if (pairs.length <= 4) return null
  return pairs.filter((_, i) => i !== idx)
}
