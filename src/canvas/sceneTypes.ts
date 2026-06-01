/** Mirrors Rust `diagramme_scene::Scene` JSON from `get_diagram_scene`. */

export interface PointPx {
  x: number
  y: number
}

export interface RectPx {
  x: number
  y: number
  width: number
  height: number
}

export type HAlign = 'Left' | 'Center' | 'Right'
export type VAlign = 'Top' | 'Middle' | 'Bottom'

export interface SceneText {
  position: PointPx
  content: string
  height_px: number
  halign: HAlign
  valign: VAlign
  font: string
  owner_node_id?: string
}

export interface ScenePolyline {
  points: PointPx[]
  stroke_px: number
  layer: string
  color: number
  closed?: boolean
  edge_id?: string
  owner_node_id?: string
}

export interface SceneRect {
  rect: RectPx
  stroke_px: number
  fill?: number
  layer: string
  node_id?: string
}

export interface SceneSolid {
  vertices: [PointPx, PointPx, PointPx, PointPx]
  layer: string
  node_id?: string
}

export type ScenePrimitive =
  | { Polyline: ScenePolyline }
  | { Rect: SceneRect }
  | { Solid: SceneSolid }
  | { Text: SceneText }

export interface HitTarget {
  id: string
  bounds: RectPx
  node_id?: string
  edge_id?: string
  handle_id?: string
  /** Opaque canvas face; omitted when tags/text sit outside the filled frame. */
  face_mask_bounds?: RectPx
}

export interface SceneJson {
  primitives: ScenePrimitive[]
  extent: RectPx
  hits: HitTarget[]
}

/** Partial scene from `get_diagram_scene_patch` after drag preview. */
export interface ScenePatchJson {
  node_ids: string[]
  edge_ids: string[]
  primitives: ScenePrimitive[]
  hits: HitTarget[]
}
