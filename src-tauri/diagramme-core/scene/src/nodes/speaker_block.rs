//! Speaker block scene primitives — ported from v6 `appendSpeakerBlockRevitDxf`.

use diagramme_geometry::{
    PointPx, RectPx, TextHAlign, TextRole, MIC_SPEAKER_FRAME_HEIGHT_PX,
    MIC_SPEAKER_LABEL_PAIR_HALF_SPACING_PX, SPEAKER_BLOCK_DEFAULT_WIDTH_PX,
    SPEAKER_SCHEMATIC_SVG_WIDTH_PX, SPEAKER_SYMBOL_ROW_TOP_FROM_ROOT_PX,
    speaker_label_mid_y_from_root_px,
};
use diagramme_schema::Node;

use crate::nodes::emit::{
    push_line, push_polyline, push_text, scene_text_from_role, DEFAULT_LAYER,
};
use crate::scene::Scene;
use crate::text::sanitize_text;

/// Scene bounds (diagram px).
pub fn speaker_block_scene_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        SPEAKER_BLOCK_DEFAULT_WIDTH_PX,
        MIC_SPEAKER_FRAME_HEIGHT_PX,
    )
}

/// Append speaker block drawable primitives to `scene`.
pub fn append_speaker_block_scene(scene: &mut Scene, node: &Node) {
    let data = &node.data;
    let nx = node.position.x;
    let ny = node.position.y;
    let sym_ny = ny + SPEAKER_SYMBOL_ROW_TOP_FROM_ROOT_PX;

    let pad = 0.25;
    let sq_left = pad;
    let sq_right = pad + 4.5;
    let mid_y = 7.0;
    let sq_top = mid_y - 3.375;
    let sq_bottom = mid_y + 3.375;
    let cone_right = pad + 13.5;
    let cone_top = mid_y - 6.75;
    let cone_bottom = mid_y + 6.75;

    let sym_pt = |lx: f64, ly: f64| PointPx {
        x: nx + lx,
        y: sym_ny + ly,
    };

    push_polyline(
        scene,
        vec![
            sym_pt(sq_left, sq_top),
            sym_pt(sq_right, sq_top),
            sym_pt(cone_right, cone_top),
            sym_pt(cone_right, cone_bottom),
            sym_pt(sq_right, sq_bottom),
            sym_pt(sq_left, sq_bottom),
        ],
        true,
        DEFAULT_LAYER,
    );
    push_line(scene, sq_right, sq_top, sq_right, sq_bottom, nx, sym_ny);

    let kind = data
        .get("symbolKind")
        .and_then(|v| v.as_str())
        .unwrap_or("standard");
    match kind {
        "70v" => {
            push_line(scene, sq_left, sq_top, sq_right, sq_bottom, nx, sym_ny);
            push_line(scene, sq_right, sq_top, sq_left, sq_bottom, nx, sym_ny);
        }
        "active" => {
            let tx = sq_left + 0.25;
            let ty = sq_top + 0.25;
            let bx = sq_left + 0.25;
            let by = sq_bottom - 0.25;
            let mx = sq_right - 0.25;
            push_polyline(
                scene,
                vec![sym_pt(tx, ty), sym_pt(mx, mid_y), sym_pt(bx, by)],
                false,
                DEFAULT_LAYER,
            );
        }
        _ => {}
    }

    let gap = 6.0;
    let text_left = nx + SPEAKER_SCHEMATIC_SVG_WIDTH_PX + gap;
    let line1 = sanitize_text(
        data.get("line1").and_then(|v| v.as_str()).unwrap_or(""),
        48,
    );
    let line2 = sanitize_text(
        data.get("line2").and_then(|v| v.as_str()).unwrap_or(""),
        64,
    );
    let label_mid_y = ny + speaker_label_mid_y_from_root_px();
    let half = MIC_SPEAKER_LABEL_PAIR_HALF_SPACING_PX;

    push_text(
        scene,
        scene_text_from_role(
            TextRole::SpeakerPrimary,
            PointPx {
                x: text_left,
                y: label_mid_y - half,
            },
            if line1.is_empty() {
                " ".to_string()
            } else {
                line1
            },
            Some(TextHAlign::Left),
            None,
        ),
    );
    push_text(
        scene,
        scene_text_from_role(
            TextRole::SpeakerSecondary,
            PointPx {
                x: text_left,
                y: label_mid_y + half,
            },
            if line2.is_empty() {
                " ".to_string()
            } else {
                line2
            },
            Some(TextHAlign::Left),
            None,
        ),
    );
}
