import type { HAlign, PointPx, ScenePolyline, VAlign } from './sceneTypes'

/** sRGB `0xRRGGBB` (matches Rust wire / stroke colors). */
export function colorRgbToCss(color: number): string {
  const rgb = color & 0xffffff
  const r = (rgb >> 16) & 0xff
  const g = (rgb >> 8) & 0xff
  const b = rgb & 0xff
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`
}

export function polylineToKonvaPoints(points: PointPx[]): number[] {
  const flat: number[] = []
  for (const p of points) {
    flat.push(p.x, p.y)
  }
  return flat
}

export function konvaTextAlign(halign: HAlign): 'left' | 'center' | 'right' {
  switch (halign) {
    case 'Left':
      return 'left'
    case 'Center':
      return 'center'
    case 'Right':
      return 'right'
  }
}

export function konvaVerticalAlign(valign: VAlign): 'top' | 'middle' | 'bottom' {
  switch (valign) {
    case 'Top':
      return 'top'
    case 'Middle':
      return 'middle'
    case 'Bottom':
      return 'bottom'
  }
}

export function primitiveKey(kind: string, index: number, polyline?: ScenePolyline): string {
  if (polyline?.edge_id) {
    return `${kind}-${polyline.edge_id}`
  }
  return `${kind}-${index}`
}

export function fitExtentToStage(
  extent: { x: number; y: number; width: number; height: number },
  stageWidth: number,
  stageHeight: number,
  padding = 48,
): { scale: number; x: number; y: number } {
  if (extent.width <= 0 || extent.height <= 0 || stageWidth <= 0 || stageHeight <= 0) {
    return { scale: 1, x: padding, y: padding }
  }
  const innerW = Math.max(stageWidth - padding * 2, 1)
  const innerH = Math.max(stageHeight - padding * 2, 1)
  const scale = Math.min(innerW / extent.width, innerH / extent.height, 4)
  const x = padding - extent.x * scale
  const y = padding - extent.y * scale
  return { scale, x, y }
}
