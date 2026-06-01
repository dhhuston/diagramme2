import { SNAP_HALF_STEP_PX } from './paperScale'
import type { RectPx } from './sceneTypes'
import type { Viewport } from './useViewport'

/** Visible diagram bounds from stage pan/zoom (diagram px). */
export function visibleDiagramBounds(
  viewport: Viewport,
  stageWidth: number,
  stageHeight: number,
  padding = SNAP_HALF_STEP_PX * 8,
): RectPx {
  const x0 = -viewport.x / viewport.scale - padding
  const y0 = -viewport.y / viewport.scale - padding
  const x1 = (stageWidth - viewport.x) / viewport.scale + padding
  const y1 = (stageHeight - viewport.y) / viewport.scale + padding
  return {
    x: x0,
    y: y0,
    width: Math.max(x1 - x0, gapMin(stageWidth, viewport.scale)),
    height: Math.max(y1 - y0, gapMin(stageHeight, viewport.scale)),
  }
}

function gapMin(stagePx: number, scale: number) {
  return Math.max(stagePx / scale, SNAP_HALF_STEP_PX)
}

export function gridLinePositions(min: number, max: number, step: number): number[] {
  const start = Math.floor(min / step) * step
  const lines: number[] = []
  for (let v = start; v <= max + step; v += step) {
    lines.push(v)
  }
  return lines
}
