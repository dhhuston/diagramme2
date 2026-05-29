//! Shared scene emission helpers for schematic node appenders.

use diagramme_geometry::{
    text_style_for_role, PointPx, TextHAlign, TextRole, TextVAlign,
};
use diagramme_schema::Node;

use crate::scene::{HAlign, Scene, ScenePrimitive, SceneText, VAlign};

pub const DEFAULT_LAYER: &str = "0";
pub const INKFILL_LAYER: &str = "INKFILL";
pub const GUIDES_LAYER: &str = "GUIDES";
pub const HAIRLINE_STROKE_PX: f64 = 1.0;

pub fn local_to_diagram(nx: f64, ny: f64, lx: f64, ly: f64) -> PointPx {
    PointPx {
        x: nx + lx,
        y: ny + ly,
    }
}

pub fn to_halign(align: TextHAlign) -> HAlign {
    match align {
        TextHAlign::Left => HAlign::Left,
        TextHAlign::Center => HAlign::Center,
        TextHAlign::Right => HAlign::Right,
    }
}

pub fn to_valign(align: TextVAlign) -> VAlign {
    match align {
        TextVAlign::Top => VAlign::Top,
        TextVAlign::Middle => VAlign::Middle,
        TextVAlign::Bottom => VAlign::Bottom,
    }
}

pub fn push_polyline(scene: &mut Scene, points: Vec<PointPx>, closed: bool, layer: &str) {
    scene.primitives.push(ScenePrimitive::Polyline {
        points,
        stroke_px: HAIRLINE_STROKE_PX,
        layer: layer.to_string(),
        color: 0,
        closed,
        edge_id: None,
    });
}

pub fn push_line(scene: &mut Scene, x0: f64, y0: f64, x1: f64, y1: f64, nx: f64, ny: f64) {
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, x0, y0),
            local_to_diagram(nx, ny, x1, y1),
        ],
        false,
        DEFAULT_LAYER,
    );
}

pub fn push_solid_triangle(scene: &mut Scene, p0: PointPx, p1: PointPx, p2: PointPx, node_id: &str) {
    scene.primitives.push(ScenePrimitive::Solid {
        vertices: [p0, p1, p2, p2],
        layer: INKFILL_LAYER.to_string(),
        node_id: Some(node_id.to_string()),
    });
}

pub fn push_circle_polyline(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    cx: f64,
    cy: f64,
    r: f64,
    segments: usize,
) {
    let steps = segments.max(3);
    let mut pts = Vec::with_capacity(steps);
    for i in 0..steps {
        let a = (2.0 * std::f64::consts::PI * i as f64) / steps as f64;
        pts.push(local_to_diagram(nx, ny, cx + r * a.cos(), cy + r * a.sin()));
    }
    push_polyline(scene, pts, true, DEFAULT_LAYER);
}

pub fn push_dashed_line_px(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
) {
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let dash = 8.0;
    let gap = 5.0;
    let mut t = 0.0;
    while t < len {
        let seg_start = t;
        let seg_end = len.min(t + dash);
        push_polyline(
            scene,
            vec![
                local_to_diagram(nx, ny, ax + ux * seg_start, ay + uy * seg_start),
                local_to_diagram(nx, ny, ax + ux * seg_end, ay + uy * seg_end),
            ],
            false,
            GUIDES_LAYER,
        );
        t += dash + gap;
    }
}

pub fn scene_text_from_role(
    role: TextRole,
    position: PointPx,
    content: String,
    halign_override: Option<TextHAlign>,
    valign_override: Option<TextVAlign>,
) -> SceneText {
    let style = text_style_for_role(role);
    SceneText {
        position,
        content,
        height_px: style.height_px,
        halign: to_halign(halign_override.unwrap_or(style.halign)),
        valign: to_valign(valign_override.unwrap_or(style.valign)),
        font: style.font.to_string(),
    }
}

pub fn push_text(scene: &mut Scene, text: SceneText) {
    scene.primitives.push(ScenePrimitive::Text(text));
}

pub fn node_width(node: &Node, default_w: f64) -> f64 {
    node.width.filter(|w| *w > 0.0).unwrap_or(default_w)
}

pub fn node_height(node: &Node, default_h: f64) -> f64 {
    node.height.filter(|h| *h > 0.0).unwrap_or(default_h)
}

pub fn with_split_suffix(base: &str, split_instance: Option<u64>) -> String {
    match split_instance {
        Some(n) => format!("{} ({n})", base.trim()),
        None => base.trim().to_string(),
    }
}
