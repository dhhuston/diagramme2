//! Mic block scene primitives — ported from v6 `appendMicBlockRevitDxf`.

use diagramme_geometry::{
    mic_block_outer_width_snapped_px, mic_block_strip_top_inset_px, PointPx, RectPx,
    TextHAlign, TextRole, MIC_BLOCK_FRAME_HEIGHT_PX, MIC_BUS_X, MIC_CX, MIC_CY, MIC_R,
    MIC_SCHEMATIC_SVG_WIDTH_PX, MIC_SPEAKER_LABEL_PAIR_HALF_SPACING_PX,
    MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX, MIC_SPEAKER_VC_STRIP_HEIGHT_PX, MIC_SVG_H,
};
use diagramme_schema::Node;

use crate::nodes::emit::{
    push_circle_polyline, push_line, push_node_body_hit_with_face_mask, push_text,
    scene_text_from_role,
};
use crate::scene::Scene;
use crate::text::sanitize_text;

/// Filled circle matching the mic symbol (diagram px).
pub fn mic_symbol_face_mask_polygon(nx: f64, ny: f64, outer_w: f64) -> Vec<PointPx> {
    let row_top = ny + mic_block_strip_top_inset_px();
    let sym_left = nx + outer_w - MIC_SCHEMATIC_SVG_WIDTH_PX;
    let cx = sym_left + MIC_CX;
    let cy = row_top + MIC_CY;
    let steps = 32;
    (0..steps)
        .map(|i| {
            let a = (2.0 * std::f64::consts::PI * i as f64) / steps as f64;
            PointPx {
                x: cx + MIC_R * a.cos(),
                y: cy + MIC_R * a.sin(),
            }
        })
        .collect()
}

/// Scene bounds (diagram px).
pub fn mic_block_scene_bounds(node: &Node) -> RectPx {
    let line1 = node.data.get("line1").and_then(|v| v.as_str()).unwrap_or("");
    let line2 = node.data.get("line2").and_then(|v| v.as_str()).unwrap_or("");
    let w = mic_block_outer_width_snapped_px(line1, line2);
    RectPx::new(node.position.x, node.position.y, w, MIC_BLOCK_FRAME_HEIGHT_PX)
}

/// Append mic block drawable primitives to `scene`.
pub fn append_mic_block_scene(scene: &mut Scene, node: &Node) {
    let data = &node.data;
    let nx = node.position.x;
    let ny = node.position.y;
    let line1 = data.get("line1").and_then(|v| v.as_str()).unwrap_or("");
    let line2 = data.get("line2").and_then(|v| v.as_str()).unwrap_or("");
    let outer_w = mic_block_outer_width_snapped_px(line1, line2);
    let row_top = ny + mic_block_strip_top_inset_px();
    let sym_left = nx + outer_w - MIC_SCHEMATIC_SVG_WIDTH_PX;

    push_line(scene, MIC_BUS_X, 0.0, MIC_BUS_X, MIC_SVG_H, sym_left, row_top);
    push_circle_polyline(scene, sym_left, row_top, MIC_CX, MIC_CY, MIC_R, 32);

    let ch = sanitize_text(
        data.get("channelNumber")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        8,
    );
    push_text(
        scene,
        scene_text_from_role(
            TextRole::MicChannel,
            PointPx {
                x: sym_left + MIC_CX,
                y: row_top + MIC_CY,
            },
            if ch.is_empty() { " ".to_string() } else { ch },
            Some(TextHAlign::Center),
            None,
        ),
    );

    let label_right = sym_left - MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX;
    let line1_text = sanitize_text(line1, 48);
    let line2_text = sanitize_text(line2, 64);
    let label_mid_y = ny + MIC_BLOCK_FRAME_HEIGHT_PX / 2.0;
    let half = MIC_SPEAKER_LABEL_PAIR_HALF_SPACING_PX;

    push_text(
        scene,
        scene_text_from_role(
            TextRole::SpeakerPrimary,
            PointPx {
                x: label_right,
                y: label_mid_y - half,
            },
            if line1_text.is_empty() {
                " ".to_string()
            } else {
                line1_text
            },
            Some(TextHAlign::Right),
            None,
        ),
    );
    push_text(
        scene,
        scene_text_from_role(
            TextRole::SpeakerSecondary,
            PointPx {
                x: label_right,
                y: label_mid_y + half,
            },
            if line2_text.is_empty() {
                " ".to_string()
            } else {
                line2_text
            },
            Some(TextHAlign::Right),
            None,
        ),
    );

    let _ = MIC_SPEAKER_VC_STRIP_HEIGHT_PX;

    push_node_body_hit_with_face_mask(
        scene,
        node,
        mic_block_scene_bounds(node),
        None,
        Some(mic_symbol_face_mask_polygon(nx, ny, outer_w)),
    );
}
