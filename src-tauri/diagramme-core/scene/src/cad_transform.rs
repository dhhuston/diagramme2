//! Strict mirror: diagram px (Y-down) → CAD inches (Y-up). Linear scale only (`1/72`).

use crate::scene::{HAlign, Scene, ScenePrimitive, SceneText, VAlign};
use diagramme_geometry::{PointPx, RectPx};

pub use diagramme_geometry::units::px_to_in;

/// CAD-space axis-aligned extent in inches (diagram-down Y before mirror).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtentIn {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// Point in CAD inches (Y-up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointIn {
    pub x: f64,
    pub y: f64,
}

/// Axis-aligned rectangle in CAD inches (`origin` = top-left after Y mirror).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectIn {
    pub origin: PointIn,
    pub width_in: f64,
    pub height_in: f64,
}

/// Horizontal text alignment in CAD space (same semantics as scene).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadHAlign {
    Left,
    Center,
    Right,
}

/// Vertical text alignment in CAD space (same semantics as scene).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadVAlign {
    Top,
    Middle,
    Bottom,
}

/// Text primitive in CAD inches.
#[derive(Debug, Clone, PartialEq)]
pub struct CadText {
    pub position: PointIn,
    pub content: String,
    pub height_in: f64,
    pub halign: CadHAlign,
    pub valign: CadVAlign,
    pub font: String,
}

/// Drawable primitive in CAD inches (output of `scene_to_cad`).
#[derive(Debug, Clone, PartialEq)]
pub enum CadPrimitive {
    Polyline {
        points: Vec<PointIn>,
        stroke_in: f64,
        layer: String,
        color: u32,
        edge_id: Option<String>,
    },
    Rect {
        rect: RectIn,
        stroke_in: f64,
        fill: Option<u32>,
        layer: String,
        node_id: Option<String>,
    },
    Solid {
        vertices: [PointIn; 4],
        layer: String,
        node_id: Option<String>,
    },
    Text(CadText),
}

/// Scene transformed to CAD inches for DXF emit and parity tests.
#[derive(Debug, Clone, PartialEq)]
pub struct CadScene {
    pub primitives: Vec<CadPrimitive>,
    pub extent: ExtentIn,
}

/// Diagram px (Y-down) → CAD inches (Y-up). Only linear scale + Y mirror.
#[inline]
pub fn scene_point_to_cad(p: PointPx, ext: ExtentIn) -> PointIn {
    PointIn {
        x: px_to_in(p.x),
        y: ext.min_y + ext.max_y - px_to_in(p.y),
    }
}

/// Scene bounding rect (diagram px) → extent in diagram-down inches for mirror + DXF header.
pub fn extent_in_from_rect(rect: RectPx) -> ExtentIn {
    ExtentIn {
        min_x: px_to_in(rect.x),
        min_y: px_to_in(rect.y),
        max_x: px_to_in(rect.x + rect.width),
        max_y: px_to_in(rect.y + rect.height),
    }
}

/// Union of diagram-space rects → CAD extent in inches (DXF header / Y mirror).
pub fn extent_from_rects(rects: impl IntoIterator<Item = RectPx>) -> ExtentIn {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut any = false;

    for rect in rects {
        any = true;
        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.x + rect.width;
        let y1 = rect.y + rect.height;
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    }

    if !any {
        return ExtentIn {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
        };
    }

    ExtentIn {
        min_x: px_to_in(min_x),
        min_y: px_to_in(min_y),
        max_x: px_to_in(max_x),
        max_y: px_to_in(max_y),
    }
}

fn rect_px_to_cad(rect: RectPx, ext: ExtentIn) -> RectIn {
    RectIn {
        origin: scene_point_to_cad(PointPx { x: rect.x, y: rect.y }, ext),
        width_in: px_to_in(rect.width),
        height_in: px_to_in(rect.height),
    }
}

fn scene_text_to_cad(text: &SceneText, ext: ExtentIn) -> CadText {
    CadText {
        position: scene_point_to_cad(text.position, ext),
        content: text.content.clone(),
        height_in: px_to_in(text.height_px),
        halign: match text.halign {
            HAlign::Left => CadHAlign::Left,
            HAlign::Center => CadHAlign::Center,
            HAlign::Right => CadHAlign::Right,
        },
        valign: match text.valign {
            VAlign::Top => CadVAlign::Top,
            VAlign::Middle => CadVAlign::Middle,
            VAlign::Bottom => CadVAlign::Bottom,
        },
        font: text.font.clone(),
    }
}

fn primitive_to_cad(primitive: &ScenePrimitive, ext: ExtentIn) -> CadPrimitive {
    match primitive {
        ScenePrimitive::Polyline {
            points,
            stroke_px,
            layer,
            color,
            edge_id,
        } => CadPrimitive::Polyline {
            points: points
                .iter()
                .map(|&p| scene_point_to_cad(p, ext))
                .collect(),
            stroke_in: px_to_in(*stroke_px),
            layer: layer.clone(),
            color: *color,
            edge_id: edge_id.clone(),
        },
        ScenePrimitive::Rect {
            rect,
            stroke_px,
            fill,
            layer,
            node_id,
        } => CadPrimitive::Rect {
            rect: rect_px_to_cad(*rect, ext),
            stroke_in: px_to_in(*stroke_px),
            fill: *fill,
            layer: layer.clone(),
            node_id: node_id.clone(),
        },
        ScenePrimitive::Solid {
            vertices,
            layer,
            node_id,
        } => CadPrimitive::Solid {
            vertices: vertices.map(|p| scene_point_to_cad(p, ext)),
            layer: layer.clone(),
            node_id: node_id.clone(),
        },
        ScenePrimitive::Text(text) => CadPrimitive::Text(scene_text_to_cad(text, ext)),
    }
}

/// Transform the full scene to CAD inches — the only path from scene geometry to DXF.
pub fn scene_to_cad(scene: &Scene) -> CadScene {
    let extent = extent_in_from_rect(scene.extent);
    CadScene {
        primitives: scene
            .primitives
            .iter()
            .map(|p| primitive_to_cad(p, extent))
            .collect(),
        extent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    #[test]
    fn scene_point_y_mirror_with_144px_extent() {
        let ext = extent_in_from_rect(RectPx::new(0.0, 0.0, 0.0, 144.0));
        let cad = scene_point_to_cad(PointPx { x: 0.0, y: 72.0 }, ext);
        assert!((cad.x - 0.0).abs() < 1e-12);
        assert!(
            (cad.y - 1.0).abs() < 1e-12,
            "midline diagram y=72\" should mirror to cad y=1.0\", got {}",
            cad.y
        );
        let top = scene_point_to_cad(PointPx { x: 0.0, y: 0.0 }, ext);
        assert!((top.y - 2.0).abs() < 1e-12);
        let bottom = scene_point_to_cad(PointPx { x: 0.0, y: 144.0 }, ext);
        assert!((bottom.y - 0.0).abs() < 1e-12);
    }

    #[test]
    fn text_cap_height_px_to_in() {
        assert!((px_to_in(9.0) - 0.125).abs() < 1e-12);
    }

    #[test]
    fn extent_from_rects_unions_bounds() {
        let ext = extent_from_rects([
            RectPx::new(0.0, 0.0, 72.0, 72.0),
            RectPx::new(72.0, 72.0, 72.0, 72.0),
        ]);
        assert!((ext.min_x - 0.0).abs() < 1e-12);
        assert!((ext.min_y - 0.0).abs() < 1e-12);
        assert!((ext.max_x - 2.0).abs() < 1e-12);
        assert!((ext.max_y - 2.0).abs() < 1e-12);
    }

    #[test]
    fn scene_to_cad_empty_scene() {
        let cad = scene_to_cad(&Scene::default());
        assert!(cad.primitives.is_empty());
        assert_eq!(cad.extent.min_x, 0.0);
        assert_eq!(cad.extent.max_y, 0.0);
    }
}
