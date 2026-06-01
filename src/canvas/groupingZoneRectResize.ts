import type { PointPx } from './sceneTypes'

export type RectResizeHandle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w'

export const RECT_RESIZE_HANDLES: RectResizeHandle[] = [
  'nw',
  'n',
  'ne',
  'e',
  'se',
  's',
  'sw',
  'w',
]

export type DiagramRect = {
  x: number
  y: number
  width: number
  height: number
}

export function rectResizeHandlePosition(rect: DiagramRect, handle: RectResizeHandle): PointPx {
  const { x, y, width: w, height: h } = rect
  switch (handle) {
    case 'nw':
      return { x, y }
    case 'n':
      return { x: x + w / 2, y }
    case 'ne':
      return { x: x + w, y }
    case 'e':
      return { x: x + w, y: y + h / 2 }
    case 'se':
      return { x: x + w, y: y + h }
    case 's':
      return { x: x + w / 2, y: y + h }
    case 'sw':
      return { x, y: y + h }
    case 'w':
      return { x, y: y + h / 2 }
  }
}

export function hitTestRectResizeHandle(
  point: PointPx,
  rect: DiagramRect,
  pickRadius = 8,
): RectResizeHandle | null {
  const r2 = pickRadius * pickRadius
  for (const handle of RECT_RESIZE_HANDLES) {
    const pos = rectResizeHandlePosition(rect, handle)
    const dx = point.x - pos.x
    const dy = point.y - pos.y
    if (dx * dx + dy * dy <= r2) {
      return handle
    }
  }
  return null
}

export function resizeRectFromHandle(
  start: DiagramRect,
  handle: RectResizeHandle,
  pointer: PointPx,
  minWidth: number,
  minHeight: number,
): DiagramRect {
  let { x, y, width, height } = start
  const right = x + width
  const bottom = y + height
  const px = Math.round(pointer.x)
  const py = Math.round(pointer.y)

  if (handle === 'e' || handle === 'ne' || handle === 'se') {
    width = Math.max(minWidth, px - x)
  }
  if (handle === 'w' || handle === 'nw' || handle === 'sw') {
    const newW = Math.max(minWidth, right - px)
    x = right - newW
    width = newW
  }
  if (handle === 's' || handle === 'se' || handle === 'sw') {
    height = Math.max(minHeight, py - y)
  }
  if (handle === 'n' || handle === 'ne' || handle === 'nw') {
    const newH = Math.max(minHeight, bottom - py)
    y = bottom - newH
    height = newH
  }

  return { x, y, width, height }
}

export function cursorForRectResizeHandle(handle: RectResizeHandle): string {
  const map: Record<RectResizeHandle, string> = {
    nw: 'nwse-resize',
    n: 'ns-resize',
    ne: 'nesw-resize',
    e: 'ew-resize',
    se: 'nwse-resize',
    s: 'ns-resize',
    sw: 'nesw-resize',
    w: 'ew-resize',
  }
  return map[handle]
}
