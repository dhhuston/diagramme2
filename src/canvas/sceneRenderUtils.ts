import type Konva from 'konva'

import type { HAlign, PointPx, ScenePolyline, VAlign } from './sceneTypes'

/**
 * Arial Narrow cap-height / CSS em-size for schematic labels.
 * Scene `height_px` and DXF TEXT height are cap height; canvas `fontSize` is em box.
 * Calibrated to v6 canvas typography (e.g. 9px em → 6.75px cap / 3⁄32").
 */
export const ARIAL_NARROW_CAP_TO_EM = 0.75

/** Konva `fontSize` that renders cap height equal to scene/DXF `height_px`. */
export function sceneCapHeightToFontSizePx(capHeightPx: number): number {
  if (capHeightPx <= 0) {
    return 0
  }
  return capHeightPx / ARIAL_NARROW_CAP_TO_EM
}

/** Konva hairline for schematic ink — v6 SVG uses 0.5px; DXF emits CAD hairline (same optical weight). */
export const SCHEMATIC_INK_STROKE_PX = 0.5

/** Map scene stroke to Konva width (wires stay full px; frame/rules use schematic hairline). */
export function konvaStrokeWidthPx(strokePx: number, edgeId?: string): number {
  if (edgeId) {
    return strokePx
  }
  return SCHEMATIC_INK_STROKE_PX
}

/** Scene/DXF text anchor offsets from measured Konva text metrics. */
export function textAnchorOffsetX(halign: HAlign, width: number): number {
  switch (halign) {
    case 'Left':
      return 0
    case 'Center':
      return width / 2
    case 'Right':
      return width
  }
}

/** Vertical offset from measured text box (DXF insertion semantics). */
export function textAnchorOffsetY(valign: VAlign, textHeightPx: number): number {
  switch (valign) {
    case 'Top':
      return 0
    case 'Middle':
      return textHeightPx / 2
    case 'Bottom':
      return textHeightPx
  }
}

/** Apply scene insertion-point semantics to a Konva Text node. */
export function applySceneTextAnchor(node: Konva.Text, halign: HAlign, valign: VAlign): void {
  const width = node.getTextWidth()
  const textHeight = node.height()
  node.offsetX(textAnchorOffsetX(halign, width))
  node.offsetY(textAnchorOffsetY(valign, textHeight))
}

/** Header row fills — matches v6 `--clr-schematic-header` / DXF FILLS layer (ACI 9). */
export const SCHEMATIC_HEADER_FILL = '#bfbfbf'

/** Flyoff triangles and other ink fills — matches v6 `--clr-schematic-ink`. */
export const SCHEMATIC_INK_FILL = '#000000'

/** Canvas fill for scene SOLID primitives (layer encodes intent; DXF uses layer color). */
export function solidLayerFillCss(layer: string): string {
  switch (layer) {
    case 'FILLS':
      return SCHEMATIC_HEADER_FILL
    case 'INKFILL':
      return SCHEMATIC_INK_FILL
    default:
      return SCHEMATIC_INK_FILL
  }
}

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

/** Stable React key — index disambiguates multiple polylines per wire (crossing-gap splits). */
export function primitiveKey(kind: string, index: number, polyline?: ScenePolyline): string {
  if (polyline?.edge_id) {
    return `${kind}-${polyline.edge_id}-${index}`
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
