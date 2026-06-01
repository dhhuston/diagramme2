/**
 * Canvas scale constants — keep in sync with `diagramme_geometry::paper_scale`.
 *
 * - {@link SNAP_HALF_STEP_PX}: node placement / React Flow snap (v6 `snapGrid`)
 * - {@link CONNECTOR_LINE_PITCH_PX}: port row center-to-center pitch (1/8″)
 * - {@link SNAP_GRID_PX}: fine subdivision (half-step × 2); visible minor lines
 */

export const PX_PER_INCH = 72

/** 1/8″ between connector rule centers. */
export const CONNECTOR_LINE_PITCH_PX = PX_PER_INCH / 8

/** Full placement cell (3px @ 72 dpi). */
export const SNAP_GRID_PX = CONNECTOR_LINE_PITCH_PX / 3

/** Half placement step — node origins and row strokes land here (v6). */
export const SNAP_HALF_STEP_PX = SNAP_GRID_PX / 2

/** Volume control hex frame height (3/16″). */
export const VOLUME_CONTROL_HEX_VERTEX_SPAN_PX = (3 / 16) * PX_PER_INCH

export const VOLUME_CONTROL_FRAME_HEIGHT_PX = VOLUME_CONTROL_HEX_VERTEX_SPAN_PX

/** Wiretag bar height (2× snap grid). */
export const WIRETAG_BAR_HEIGHT_PX = 2 * SNAP_GRID_PX

/** Initial wiretag width before label growth. */
export const WIRETAG_INITIAL_WIDTH_PX = 40

export function snapPlacementCoord(v: number): number {
  return Math.round(v / SNAP_HALF_STEP_PX) * SNAP_HALF_STEP_PX
}

export function snapPoint(point: { x: number; y: number }): { x: number; y: number } {
  return {
    x: snapPlacementCoord(point.x),
    y: snapPlacementCoord(point.y),
  }
}

export function isOnGridStep(value: number, step: number, epsilon = 1e-6): boolean {
  const n = Math.round(value / step)
  return Math.abs(value - n * step) < epsilon
}
