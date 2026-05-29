//! Junction box scene primitives — ported from v6 `appendJunctionRevitDxf`.

use diagramme_geometry::{
    junction_scene_bounds_rect, RectPx, JUNCTION_BOX_BODY_TOP_PX, PATCH_GRID_ROW_PX,
    PATCH_PANEL_WIDTH_PX, SCHEMATIC_FRAME_INSET_PX,
};
use diagramme_schema::Node;

use crate::nodes::emit::{push_line, with_split_suffix};
use crate::nodes::patch_panel::push_patch_panel_frame;
use crate::scene::Scene;
use crate::text::sanitize_text;

fn junction_total_height_px(row_count: usize) -> f64 {
    JUNCTION_BOX_BODY_TOP_PX + row_count as f64 * PATCH_GRID_ROW_PX + PATCH_GRID_ROW_PX
}

/// Scene bounds including tag band (diagram px).
pub fn junction_scene_bounds(node: &Node) -> RectPx {
    junction_scene_bounds_rect(node)
}

/// Append junction drawable primitives to `scene`. Returns false when `rowCount` is zero.
pub fn append_junction_scene(scene: &mut Scene, node: &Node) -> bool {
    let data = &node.data;
    let row_count = data
        .get("rowCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if row_count == 0 {
        return false;
    }

    let nx = node.position.x;
    let ny = node.position.y;
    let total_h = junction_total_height_px(row_count);

    let tag_code = data
        .get("tagCode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let tag_number = data
        .get("tagNumber")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let tag = format!("{tag_code} / {tag_number}").trim().to_string();
    let split_instance = data.get("splitInstance").and_then(|v| v.as_u64());
    let title_line = sanitize_text(
        &with_split_suffix(tag_code, split_instance),
        48,
    );

    let f_top = push_patch_panel_frame(
        scene,
        node,
        total_h,
        &tag,
        &[title_line],
        split_instance.is_some(),
    );

    let inset = SCHEMATIC_FRAME_INSET_PX;
    for i in 0..row_count {
        let row_top = f_top + JUNCTION_BOX_BODY_TOP_PX + i as f64 * PATCH_GRID_ROW_PX;
        let y_line = row_top + PATCH_GRID_ROW_PX / 2.0;
        push_line(
            scene,
            inset,
            y_line,
            PATCH_PANEL_WIDTH_PX - inset,
            y_line,
            nx,
            ny,
        );
    }

    true
}
