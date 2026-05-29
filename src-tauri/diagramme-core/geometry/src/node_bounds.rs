//! Node frame bounds in diagram px (mirrors v6 `detailedRevitNodeBoundsInches` frame extents).

use diagramme_schema::Node;

use crate::av_plate_layout::{av_plate_body_grid_row_count, av_plate_groups_from_data};
use crate::device_v2_layout::device_v2_body_grid_row_count;
use crate::paper_scale::{
    DEVICE_V2_WIDTH_PX, WIRETAG_BAR_HEIGHT_PX, WIRETAG_DEFAULT_WIDTH_PX,
};
use crate::schematic_layout::{
    is_patch_panel_node_type, patch_panel_total_height_px, AV_PLATE_GRID_ROW_PX,
    AV_PLATE_TITLE_HEIGHT_PX, DEVICE_V2_GRID_ROW_PX, DEVICE_V2_TITLE_HEIGHT_PX,
    PATCH_PANEL_WIDTH_PX,
};
use crate::symbol_layout::{
    antenna_bounds, flyoff_note_bounds, grouping_zone_bounds, junction_bounds, mic_block_bounds,
    speaker_block_bounds, text_block_bounds, volume_control_bounds,
};
use crate::types::RectPx;

fn node_width(node: &Node, default_w: f64) -> f64 {
    node.width.filter(|w| *w > 0.0).unwrap_or(default_w)
}

fn node_height(node: &Node, default_h: f64) -> f64 {
    node.height.filter(|h| *h > 0.0).unwrap_or(default_h)
}

/// Returns the on-canvas frame bounds for a node in diagram pixels.
///
/// `x`/`y` match the node position (frame top-left). Tag band above the frame is excluded
/// except where v6 export bounds include it (patch panels, junction).
pub fn node_bounds_diagram_px(node: &Node) -> Option<RectPx> {
    let x = node.position.x;
    let y = node.position.y;

    match node.node_type.as_str() {
        "device" | "deviceV2" => {
            let row_count = device_v2_body_grid_row_count(&node.data);
            let height = DEVICE_V2_TITLE_HEIGHT_PX + row_count as f64 * DEVICE_V2_GRID_ROW_PX;
            Some(RectPx::new(x, y, DEVICE_V2_WIDTH_PX, height))
        }
        "avPlate" => {
            let groups = av_plate_groups_from_data(&node.data);
            let row_count = av_plate_body_grid_row_count(&groups);
            let height = AV_PLATE_TITLE_HEIGHT_PX + row_count as f64 * AV_PLATE_GRID_ROW_PX;
            Some(RectPx::new(x, y, PATCH_PANEL_WIDTH_PX, height))
        }
        t if is_patch_panel_node_type(t) => {
            let row_count = node
                .data
                .get("rows")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let height = patch_panel_total_height_px(row_count);
            Some(RectPx::new(x, y, PATCH_PANEL_WIDTH_PX, height))
        }
        "junction" => Some(junction_bounds(node)),
        "speakerBlock" => Some(speaker_block_bounds(node)),
        "micBlock" => Some(mic_block_bounds(node)),
        "volumeControl" => Some(volume_control_bounds(node)),
        "textBlock" => Some(text_block_bounds(node)),
        "flyoffNote" => Some(flyoff_note_bounds(node)),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => Some(antenna_bounds(node)),
        "groupingZone" => Some(grouping_zone_bounds(node)),
        "wiretag" => {
            let w = node_width(node, WIRETAG_DEFAULT_WIDTH_PX);
            let h = node_height(node, WIRETAG_BAR_HEIGHT_PX);
            Some(RectPx::new(x, y, w, h))
        }
        "wireSplit" => None,
        _ => None,
    }
}
