//! Text block scene primitives — ported from v6 `appendTextBlockRevitDxf`.

use diagramme_geometry::{RectPx, TextHAlign, TextRole, TextVAlign};
use diagramme_schema::Node;

use crate::nodes::emit::{local_to_diagram, push_line, push_text, scene_text_from_role, node_height, node_width};
use crate::scene::Scene;
use crate::text::sanitize_text;

const TEXT_BLOCK_DEFAULT_W: f64 = 200.0;
const TEXT_BLOCK_DEFAULT_H: f64 = 80.0;

/// Scene bounds (diagram px).
pub fn text_block_scene_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        node_width(node, TEXT_BLOCK_DEFAULT_W),
        node_height(node, TEXT_BLOCK_DEFAULT_H),
    )
}

fn text_block_font_px(data: &serde_json::Value) -> f64 {
    data.get("fontSizePx")
        .and_then(|v| v.as_f64())
        .map(|n| n.clamp(8.0, 18.0))
        .unwrap_or(14.0)
}

/// Append text block drawable primitives to `scene`.
pub fn append_text_block_scene(scene: &mut Scene, node: &Node) {
    let nx = node.position.x;
    let ny = node.position.y;
    let w = node_width(node, TEXT_BLOCK_DEFAULT_W);
    let h = node_height(node, TEXT_BLOCK_DEFAULT_H);
    let inset = 0.25;

    push_line(scene, inset, inset, w - inset, inset, nx, ny);
    push_line(scene, inset, inset, inset, h - inset, nx, ny);
    push_line(scene, w - inset, inset, w - inset, h - inset, nx, ny);
    push_line(scene, inset, h - inset, w - inset, h - inset, nx, ny);

    let font_size = text_block_font_px(&node.data);
    let text = node
        .data
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let lines: Vec<&str> = text.split('\n').take(24).collect();
    let line_pitch = font_size * 1.15;
    let mut ly = 6.0 + font_size * 0.35;

    for line in lines {
        let t = sanitize_text(if line.is_empty() { " " } else { line }, 200);
        let mut label = scene_text_from_role(
            TextRole::TextBlock,
            local_to_diagram(nx, ny, 6.0, ly),
            t,
            Some(TextHAlign::Left),
            Some(TextVAlign::Middle),
        );
        label.height_px = font_size;
        push_text(scene, label);
        ly += line_pitch;
        if ly > h - 4.0 {
            break;
        }
    }

}
