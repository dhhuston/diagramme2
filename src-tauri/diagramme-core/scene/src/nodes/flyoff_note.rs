//! Flyoff note scene primitives — ported from v6 `appendFlyoffNoteRevitDxf`.

use diagramme_geometry::{
    PointPx, RectPx, TextRole, FLYOFF_TEXT_FONT_PX, FLYOFF_TRI_H, FLYOFF_TRI_W,
};
use diagramme_schema::Node;

use crate::nodes::emit::{
    local_to_diagram, push_node_body_hit_with_face_mask, push_solid_triangle, push_text,
    scene_text_from_role, node_width,
};
use crate::scene::Scene;
use crate::text::sanitize_text;

const FLYOFF_DEFAULT_W: f64 = 120.0;

/// Scene bounds (diagram px).
pub fn flyoff_note_scene_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        node_width(node, FLYOFF_DEFAULT_W),
        FLYOFF_TRI_H,
    )
}

/// Append flyoff note drawable primitives to `scene`.
pub fn append_flyoff_note_scene(scene: &mut Scene, node: &Node) {
    let data = &node.data;
    let nx = node.position.x;
    let ny = node.position.y;
    let w = node_width(node, FLYOFF_DEFAULT_W);
    let tri_h = FLYOFF_TRI_H;
    let input = data
        .get("portDirection")
        .and_then(|v| v.as_str())
        .map(|s| s == "input")
        .unwrap_or(false);

    let text = sanitize_text(
        &data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_uppercase(),
        500,
    );
    let display_text = if text.is_empty() { " ".to_string() } else { text };

    let mut flyoff_text = scene_text_from_role(
        TextRole::Flyoff,
        PointPx { x: 0.0, y: 0.0 },
        display_text,
        None,
        None,
    );
    flyoff_text.height_px = FLYOFF_TEXT_FONT_PX;

    if input {
        push_solid_triangle(
            scene,
            local_to_diagram(nx, ny, 0.0, tri_h / 2.0),
            local_to_diagram(nx, ny, FLYOFF_TRI_W, 0.0),
            local_to_diagram(nx, ny, FLYOFF_TRI_W, tri_h),
            &node.id,
        );
        flyoff_text.position = local_to_diagram(nx, ny, FLYOFF_TRI_W + 6.0, tri_h / 2.0);
    } else {
        let text_w = (w - FLYOFF_TRI_W).max(0.0);
        push_solid_triangle(
            scene,
            local_to_diagram(nx, ny, text_w, 0.0),
            local_to_diagram(nx, ny, text_w + FLYOFF_TRI_W, tri_h / 2.0),
            local_to_diagram(nx, ny, text_w, tri_h),
            &node.id,
        );
        flyoff_text.position = local_to_diagram(nx, ny, w - FLYOFF_TRI_W - 6.0, tri_h / 2.0);
        flyoff_text.halign = crate::scene::HAlign::Right;
    }
    flyoff_text.halign = if input {
        crate::scene::HAlign::Left
    } else {
        crate::scene::HAlign::Right
    };
    push_text(scene, flyoff_text);
    push_node_body_hit_with_face_mask(scene, node, flyoff_note_scene_bounds(node), None, None);
}
