//! Analytical port centers in diagram px (mirrors v6 `getAnalyticalPortXY`).

use diagramme_schema::Node;

use crate::av_plate_layout::{flatten_av_plate_body_rows, av_plate_groups_from_data, AvPlateBodyRow};
use crate::device_v2_layout::{
    bundled_bracket_slots, device_v2_normalized_columns, flatten_device_v2_body_rows,
    DeviceV2BodySlot, Side,
};
use crate::paper_scale::{
    DEVICE_V2_WIDTH_PX, MIC_SPEAKER_HANDLE_CENTER_Y_PX, PX_PER_INCH, WIRETAG_BAR_HEIGHT_PX,
    WIRETAG_DEFAULT_WIDTH_PX,
};
use crate::schematic_layout::{
    is_patch_panel_node_type, BUNDLE_FILLET_PX, BUNDLE_STUB_PX, DEVICE_V2_GRID_ROW_PX,
    DEVICE_V2_ROW_CENTER_Y_PX, DEVICE_V2_TITLE_HEIGHT_PX, PATCH_BODY_TOP_PX,
    PATCH_CIRCUIT_HEIGHT_PX, PATCH_PANEL_WIDTH_PX, PATCH_ROW_CENTER_Y_PX,
};
use crate::symbol_layout::{
    antenna_symbol_port_xy, flyoff_note_bounds_width_px, mic_block_outer_width_snapped_px,
    FLYOFF_TRI_H, SPEAKER_HANDLE_CENTER_Y_FROM_ROOT_PX, SPEAKER_PASSTHRU_HANDLE_CENTER_Y_FROM_ROOT_PX,
    SPEAKER_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX, VC_HANDLE_CENTER_Y_FROM_ROOT_PX,
    VC_SCHEMATIC_SVG_WIDTH_PX, VC_SOURCE_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX,
    VC_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX,
};
use crate::types::PointPx;

/// Wiretag source handle id (`conn-src`).
pub const WIRETAG_CONN_SRC: &str = "conn-src";

/// Wiretag target handle id (`conn-tgt`).
pub const WIRETAG_CONN_TGT: &str = "conn-tgt";

/// Returns the exact diagram-space center of a port handle, computed from node data.
///
/// Returns `None` for unrecognised node types or handle ids — callers may fall back to DOM measurement.
pub fn get_analytical_port_xy(node: &Node, handle_id: &str) -> Option<PointPx> {
    if is_patch_panel_node_type(&node.node_type) {
        return patch_panel_port_xy(node, handle_id);
    }

    let pos = node.position;

    match node.node_type.as_str() {
        "avPlate" => av_plate_port_xy(node, handle_id, pos.x, pos.y),
        "wiretag" => wiretag_port_xy(node, handle_id, pos.x, pos.y),
        "device" | "deviceV2" => device_v2_port_xy(node, handle_id, pos.x, pos.y),
        "micBlock" => mic_block_port_xy(node, handle_id, pos.x, pos.y),
        "speakerBlock" => speaker_block_port_xy(node, handle_id, pos.x, pos.y),
        "volumeControl" => volume_control_port_xy(node, handle_id, pos.x, pos.y),
        "flyoffNote" => flyoff_note_port_xy(node, handle_id, pos.x, pos.y),
        "wireSplit" => wire_split_port_xy(node, pos.x, pos.y),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => {
            antenna_symbol_port_xy(node, handle_id)
        }
        "groupingZone" => None,
        _ => None,
    }
}

fn read_node_width(node: &Node, fallback: f64) -> f64 {
    node.width.filter(|w| *w > 0.0).unwrap_or(fallback)
}

fn read_node_height(node: &Node, fallback: f64) -> f64 {
    node.height.filter(|h| *h > 0.0).unwrap_or(fallback)
}

fn patch_panel_port_xy(node: &Node, handle_id: &str) -> Option<PointPx> {
    let side = if handle_id.starts_with("L-") {
        Side::Left
    } else if handle_id.starts_with("R-") {
        Side::Right
    } else {
        return None;
    };
    let row_id = &handle_id[2..];
    let rows = node
        .data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let row_index = rows
        .iter()
        .position(|row| row.get("id").and_then(|v| v.as_str()) == Some(row_id))?;
    let x = match side {
        Side::Left => node.position.x,
        Side::Right => node.position.x + PATCH_PANEL_WIDTH_PX,
    };
    let y = (node.position.y
        + PATCH_BODY_TOP_PX
        + row_index as f64 * PATCH_CIRCUIT_HEIGHT_PX
        + PATCH_ROW_CENTER_Y_PX)
        .round();
    Some(PointPx { x, y })
}

fn mic_block_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    if handle_id != "S-mic" {
        return None;
    }
    let line1 = node.data.get("line1").and_then(|v| v.as_str()).unwrap_or("");
    let line2 = node.data.get("line2").and_then(|v| v.as_str()).unwrap_or("");
    let w = mic_block_outer_width_snapped_px(line1, line2);
    Some(PointPx {
        x: abs_x + w - 2.0,
        y: abs_y + MIC_SPEAKER_HANDLE_CENTER_Y_PX,
    })
}

fn speaker_block_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    let x = abs_x + SPEAKER_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX;
    match handle_id {
        "T-spk" => Some(PointPx {
            x,
            y: abs_y + SPEAKER_HANDLE_CENTER_Y_FROM_ROOT_PX,
        }),
        "S-spk-passthru" => {
            let enabled = node
                .data
                .get("passthruEnabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                return None;
            }
            Some(PointPx {
                x,
                y: abs_y + SPEAKER_PASSTHRU_HANDLE_CENTER_Y_FROM_ROOT_PX,
            })
        }
        _ => None,
    }
}

fn volume_control_port_xy(
    node: &Node,
    handle_id: &str,
    abs_x: f64,
    abs_y: f64,
) -> Option<PointPx> {
    let w = read_node_width(node, PX_PER_INCH);
    let anchor_left = (w - VC_SCHEMATIC_SVG_WIDTH_PX) / 2.0;
    let y = abs_y + VC_HANDLE_CENTER_Y_FROM_ROOT_PX;
    match handle_id {
        "T-vc" => Some(PointPx {
            x: abs_x + anchor_left + VC_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX,
            y,
        }),
        "S-vc" => Some(PointPx {
            x: abs_x + anchor_left + VC_SOURCE_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX,
            y,
        }),
        _ => None,
    }
}

fn flyoff_note_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    let w = flyoff_note_bounds_width_px(node);
    let y = abs_y + FLYOFF_TRI_H / 2.0;
    match handle_id {
        "T-fly" => Some(PointPx { x: abs_x, y }),
        "S-fly" => Some(PointPx { x: abs_x + w, y }),
        _ => None,
    }
}

fn wire_split_port_xy(node: &Node, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    Some(PointPx {
        x: abs_x + read_node_width(node, 12.0) / 2.0,
        y: abs_y + read_node_height(node, 12.0) / 2.0,
    })
}

fn av_plate_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    if let Some(caps) = regex_av_bundle(handle_id) {
        let side = caps.0;
        let first_idx: usize = caps.1.parse().ok()?;
        let offset = BUNDLE_STUB_PX + BUNDLE_FILLET_PX;
        let half_w = (offset + 2.0) / 2.0;
        let y = abs_y
            + DEVICE_V2_TITLE_HEIGHT_PX
            + first_idx as f64 * DEVICE_V2_GRID_ROW_PX
            + DEVICE_V2_ROW_CENTER_Y_PX;
        let x = if side == 'L' {
            abs_x - offset + half_w
        } else {
            abs_x + PATCH_PANEL_WIDTH_PX + offset - half_w
        };
        return Some(PointPx { x, y });
    }

    let caps = regex_av_port(handle_id)?;
    let side = caps.0;
    let group_index: usize = caps.1.parse().ok()?;
    let row_id = caps.2;
    let groups = av_plate_groups_from_data(&node.data);
    let rows = flatten_av_plate_body_rows(&groups);
    let render_row = rows.iter().position(|slot| match slot {
        AvPlateBodyRow::Port {
            group_index: gi,
            row_id: rid,
            ..
        } => *gi == group_index && rid == row_id,
        _ => false,
    })?;
    let y = abs_y
        + DEVICE_V2_TITLE_HEIGHT_PX
        + render_row as f64 * DEVICE_V2_GRID_ROW_PX
        + DEVICE_V2_ROW_CENTER_Y_PX;
    let x = if side == 'T' {
        abs_x
    } else {
        abs_x + PATCH_PANEL_WIDTH_PX
    };
    Some(PointPx { x, y })
}

fn wiretag_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    let w = read_node_width(node, WIRETAG_DEFAULT_WIDTH_PX);
    let y = abs_y + WIRETAG_BAR_HEIGHT_PX / 2.0;
    match handle_id {
        WIRETAG_CONN_SRC => Some(PointPx {
            x: abs_x + w - 1.0,
            y,
        }),
        WIRETAG_CONN_TGT => Some(PointPx {
            x: abs_x + 1.0,
            y,
        }),
        _ => None,
    }
}

fn device_v2_port_xy(node: &Node, handle_id: &str, abs_x: f64, abs_y: f64) -> Option<PointPx> {
    if let Some(caps) = regex_device_bundle(handle_id) {
        let side = caps.0;
        let group_idx: usize = caps.1.parse().ok()?;
        let bundle_idx: usize = caps.2.parse().ok()?;
        let (left, right) = device_v2_normalized_columns(&node.data);
        let column = if side == 'L' { &left } else { &right };
        let rows = flatten_device_v2_body_rows(column);
        let slots = bundled_bracket_slots(
            &rows,
            column,
            DEVICE_V2_TITLE_HEIGHT_PX,
            DEVICE_V2_GRID_ROW_PX,
        );
        let slot = slots
            .iter()
            .find(|s| s.group_index == group_idx && s.bundle_index == bundle_idx)?;
        let offset = BUNDLE_STUB_PX + BUNDLE_FILLET_PX;
        let half_w = (offset + 2.0) / 2.0;
        let x = if side == 'L' {
            abs_x - offset + half_w
        } else {
            abs_x + DEVICE_V2_WIDTH_PX + offset - half_w
        };
        return Some(PointPx {
            x,
            y: abs_y + slot.y0,
        });
    }

    let caps = regex_device_port(handle_id)?;
    let side = caps.0;
    let group_index: usize = caps.1.parse().ok()?;
    let port_id = caps.2;
    let (left, right) = device_v2_normalized_columns(&node.data);
    let column = if side == 'L' { &left } else { &right };
    let rows = flatten_device_v2_body_rows(column);
    let render_row = rows.iter().position(|slot| match slot {
        DeviceV2BodySlot::Port {
            group_index: gi,
            row,
            ..
        } => *gi == group_index && row.id == port_id,
        _ => false,
    })?;
    let x = if side == 'L' {
        abs_x
    } else {
        abs_x + DEVICE_V2_WIDTH_PX
    };
    let y = abs_y
        + DEVICE_V2_TITLE_HEIGHT_PX
        + render_row as f64 * DEVICE_V2_GRID_ROW_PX
        + DEVICE_V2_ROW_CENTER_Y_PX;
    Some(PointPx { x, y })
}

fn regex_av_port(handle_id: &str) -> Option<(char, &str, &str)> {
    // T-{groupIndex}-{rowId} or S-{groupIndex}-{rowId}
    let bytes = handle_id.as_bytes();
    if bytes.len() < 4 || (bytes[0] != b'T' && bytes[0] != b'S') || bytes[1] != b'-' {
        return None;
    }
    let rest = &handle_id[2..];
    let (group_index, row_id) = rest.split_once('-')?;
    Some((handle_id.as_bytes()[0] as char, group_index, row_id))
}

fn regex_av_bundle(handle_id: &str) -> Option<(char, &str)> {
    // L-av-left-{firstIdx}-bundle-{bundleIdx} or R-av-right-...
    let parts: Vec<&str> = handle_id.split('-').collect();
    if parts.len() != 6 {
        return None;
    }
    if parts[0] != "L" && parts[0] != "R" {
        return None;
    }
    if parts[1] != "av" {
        return None;
    }
    if parts[4] != "bundle" {
        return None;
    }
    Some((parts[0].as_bytes()[0] as char, parts[3]))
}

fn regex_device_bundle(handle_id: &str) -> Option<(char, &str, &str)> {
    // L-{groupIndex}-bundle-{bundleIndex}
    let parts: Vec<&str> = handle_id.split('-').collect();
    if parts.len() != 4 || parts[2] != "bundle" {
        return None;
    }
    if parts[0] != "L" && parts[0] != "R" {
        return None;
    }
    Some((parts[0].as_bytes()[0] as char, parts[1], parts[3]))
}

fn regex_device_port(handle_id: &str) -> Option<(char, &str, &str)> {
    // L-{groupIndex}-{portId} or R-{groupIndex}-{portId}
    let bytes = handle_id.as_bytes();
    if bytes.len() < 4 || (bytes[0] != b'L' && bytes[0] != b'R') || bytes[1] != b'-' {
        return None;
    }
    let rest = &handle_id[2..];
    let (group_index, port_id) = rest.split_once('-')?;
    Some((handle_id.as_bytes()[0] as char, group_index, port_id))
}
