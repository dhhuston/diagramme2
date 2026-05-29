//! Schematic symbol layout constants (ported from v6 node modules for DXF/scene parity).

use diagramme_schema::Node;

use crate::grouping_zone_geometry::polyline_flat_bounds;
use crate::paper_scale::{
    CONNECTOR_ROW_OUTER_HEIGHT_PX, MIC_SPEAKER_FRAME_HEIGHT_PX, PX_PER_INCH, SNAP_GRID_PX,
    VOLUME_CONTROL_FRAME_HEIGHT_PX, VOLUME_CONTROL_HEX_VERTEX_SPAN_PX,
};
use crate::schematic_layout::{
    PATCH_GRID_ROW_PX, PATCH_PANEL_WIDTH_PX, PATCH_TITLE_HEIGHT_PX, SCHEMATIC_TAG_BAND_PX,
};
use crate::types::{PointPx, RectPx};

/// Speaker schematic SVG width (`coneRight + PAD`).
pub const SPEAKER_SCHEMATIC_SVG_WIDTH_PX: f64 = 0.25 + 13.5 + 0.25;

pub const SPEAKER_SVG_PAD: f64 = 0.25;

/// Target handle center x from node anchor (1px inset from left vertical stroke).
pub const SPEAKER_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX: f64 = SPEAKER_SVG_PAD + 1.0;

const SPEAKER_SYMBOL_MID_Y_PX: f64 = 7.0;

const SPEAKER_PASSTHRU_HANDLE_Y_PX: f64 = 9.0;

/// Symbol row top from node root (inset + 1px centering band).
pub const SPEAKER_SYMBOL_ROW_TOP_FROM_ROOT_PX: f64 = 17.0 + 1.0;

/// Wire target handle center Y from node root (snapped to placement grid).
pub const SPEAKER_HANDLE_CENTER_Y_FROM_ROOT_PX: f64 =
    crate::paper_scale::snap_placement_coord(SPEAKER_SYMBOL_ROW_TOP_FROM_ROOT_PX + SPEAKER_SYMBOL_MID_Y_PX);

/// Passthru handle center Y from node root (snapped to placement grid).
pub const SPEAKER_PASSTHRU_HANDLE_CENTER_Y_FROM_ROOT_PX: f64 =
    crate::paper_scale::snap_placement_coord(
        SPEAKER_SYMBOL_ROW_TOP_FROM_ROOT_PX + SPEAKER_PASSTHRU_HANDLE_Y_PX,
    );

/// Gap between label column and schematic (matches v6 `.mic-block` / `.speaker-block`).
pub const MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX: f64 = 6.0;

/// Vertical half-spacing between line1 and line2 labels.
pub const MIC_SPEAKER_LABEL_PAIR_HALF_SPACING_PX: f64 = 3.5;

/// Mic block outer frame height (`16 * SNAP_GRID_PX`).
pub const MIC_BLOCK_FRAME_HEIGHT_PX: f64 = 16.0 * SNAP_GRID_PX;

/// Mic schematic viewport width (circle + bus).
pub const MIC_SCHEMATIC_SVG_WIDTH_PX: f64 = 5.0 * SNAP_GRID_PX + 1.0 + 1.0;

/// Mic circle radius (3/16" diameter).
pub const MIC_SYMBOL_SIZE_PX: f64 = (3.0 / 16.0) * PX_PER_INCH;

pub const MIC_R: f64 = MIC_SYMBOL_SIZE_PX / 2.0;

pub const MIC_PAD_TOP: f64 = 0.25;

pub const MIC_CY: f64 = MIC_PAD_TOP + MIC_R;

const MIC_LAYOUT_W: f64 = 5.0 * SNAP_GRID_PX + 1.0;

pub const MIC_CX: f64 = MIC_LAYOUT_W - MIC_R;

pub const MIC_BUS_X: f64 = MIC_CX - MIC_R;

pub const MIC_SVG_H: f64 = MIC_SYMBOL_SIZE_PX + 2.0 * MIC_PAD_TOP;

/// Flyoff triangle width / height.
pub const FLYOFF_TRI_W: f64 = 8.0;

pub const FLYOFF_TRI_H: f64 = 12.0;

pub const FLYOFF_TEXT_FONT_PX: f64 = 6.75;

/// Junction body starts below title + one grid row.
pub const JUNCTION_BOX_BODY_TOP_PX: f64 = PATCH_TITLE_HEIGHT_PX + PATCH_GRID_ROW_PX;

/// Volume control schematic width (integer, √3 × circumradius).
pub const VC_SCHEMATIC_SVG_WIDTH_PX: f64 =
    ((VOLUME_CONTROL_HEX_VERTEX_SPAN_PX / 2.0) * 1.732_050_807_568_877_2).round();

/// Volume control target/source handle X from schematic anchor left.
pub const VC_TARGET_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX: f64 = 1.0;

pub const VC_SOURCE_HANDLE_CENTER_X_FROM_ANCHOR_LEFT_PX: f64 = VC_SCHEMATIC_SVG_WIDTH_PX - 1.0;

pub const VC_SYMBOL_TOP_INSET_PX: f64 = 2.25;

const VC_SYMBOL_MID_Y_PX: f64 = 6.75;

/// Volume control handle center Y from node root.
pub const VC_HANDLE_CENTER_Y_FROM_ROOT_PX: f64 = VC_SYMBOL_TOP_INSET_PX + VC_SYMBOL_MID_Y_PX;

/// Antenna schematic width (arm span + foot length).
pub const ANTENNA_SCHEMATIC_SVG_WIDTH_PX: f64 = ANT_ARM_HALF_PX * 2.0 + ANT_FOOT_LEN_PX;

pub const ANTENNA_CONNECTOR_BAND_H_PX: f64 = 2.0 * CONNECTOR_ROW_OUTER_HEIGHT_PX;

pub const ANT_FOOT_LEN_PX: f64 = 6.0;

pub const ANT_ARM_HALF_PX: f64 = 6.0;

pub const ANT_TX_MAST_X: f64 = 11.0;

pub const ANT_RX_MAST_X: f64 = 7.0;

pub const GROUPING_ZONE_DEFAULT_W: f64 = 240.0;

pub const GROUPING_ZONE_DEFAULT_H: f64 = 180.0;

/// Default collision width for speaker blocks (v6 `speakerBlockCollisionSize`).
pub const SPEAKER_BLOCK_DEFAULT_WIDTH_PX: f64 = 240.0;

/// Conservative label width when canvas measure is unavailable (v6 fallback).
fn mic_speaker_labels_max_width_px(line1: &str, line2: &str) -> f64 {
    let s1 = if line1.is_empty() { "\u{00a0}" } else { line1 };
    let s2 = if line2.is_empty() { "\u{00a0}" } else { line2 };
    (s1.len() as f64 * 3.6).max(s2.len() as f64 * 2.5).max(8.0)
}

fn mic_speaker_content_row_width_px(
    schematic_width_px: f64,
    line1: &str,
    line2: &str,
    root_w: f64,
) -> f64 {
    let labels_w = mic_speaker_labels_max_width_px(line1, line2);
    let raw = labels_w + MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX + schematic_width_px;
    let min_w = schematic_width_px + MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX;
    raw.clamp(min_w, root_w)
}

/// Mic outer width snapped so handle center lands on `SNAP_GRID_PX`.
pub fn mic_block_outer_width_snapped_px(line1: &str, line2: &str) -> f64 {
    let row_inner = mic_speaker_content_row_width_px(
        MIC_SCHEMATIC_SVG_WIDTH_PX,
        line1,
        line2,
        f64::INFINITY,
    );
    let c = row_inner.ceil();
    let r = ((c % SNAP_GRID_PX) + SNAP_GRID_PX) % SNAP_GRID_PX;
    let delta = (2.0 - r + SNAP_GRID_PX) % SNAP_GRID_PX;
    c + delta
}

/// Speaker outer width snapped to placement grid.
pub fn speaker_block_outer_width_snapped_px(line1: &str, line2: &str) -> f64 {
    let row_inner = mic_speaker_content_row_width_px(
        SPEAKER_SCHEMATIC_SVG_WIDTH_PX,
        line1,
        line2,
        f64::INFINITY,
    );
    (row_inner / SNAP_GRID_PX).ceil() * SNAP_GRID_PX
}

/// Antenna row width from label + schematic.
pub fn antenna_content_row_width_px(line1: &str, root_w: f64) -> f64 {
    let labels_w = mic_speaker_labels_max_width_px(line1, "");
    let raw = labels_w + MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX + ANTENNA_SCHEMATIC_SVG_WIDTH_PX;
    let min_w = ANTENNA_SCHEMATIC_SVG_WIDTH_PX + MIC_SPEAKER_LABEL_TO_SYMBOL_GAP_PX;
    raw.clamp(min_w, root_w)
}

/// Mic/speaker strip top within the mic block frame.
pub fn mic_block_strip_top_inset_px() -> f64 {
    (MIC_BLOCK_FRAME_HEIGHT_PX - crate::paper_scale::MIC_SPEAKER_VC_STRIP_HEIGHT_PX) / 2.0
}

/// Speaker label vertical center uses mic/speaker frame height from paper scale.
pub fn speaker_label_mid_y_from_root_px() -> f64 {
    MIC_SPEAKER_FRAME_HEIGHT_PX / 2.0
}

fn node_width(node: &Node, default_w: f64) -> f64 {
    node.width.filter(|w| *w > 0.0).unwrap_or(default_w)
}

fn node_height(node: &Node, default_h: f64) -> f64 {
    node.height.filter(|h| *h > 0.0).unwrap_or(default_h)
}

fn junction_total_height_px(row_count: usize) -> f64 {
    JUNCTION_BOX_BODY_TOP_PX + row_count as f64 * PATCH_GRID_ROW_PX + PATCH_GRID_ROW_PX
}

/// Junction frame bounds (excludes tag band in y; width is patch panel width).
pub fn junction_bounds(node: &Node) -> RectPx {
    let x = node.position.x;
    let y = node.position.y;
    let row_count = node
        .data
        .get("rowCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if row_count == 0 {
        return RectPx::new(x, y, 6.0, 6.0);
    }
    let height = junction_total_height_px(row_count);
    RectPx::new(x, y, PATCH_PANEL_WIDTH_PX, height)
}

pub fn speaker_block_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        SPEAKER_BLOCK_DEFAULT_WIDTH_PX,
        MIC_SPEAKER_FRAME_HEIGHT_PX,
    )
}

pub fn mic_block_bounds(node: &Node) -> RectPx {
    let line1 = node.data.get("line1").and_then(|v| v.as_str()).unwrap_or("");
    let line2 = node.data.get("line2").and_then(|v| v.as_str()).unwrap_or("");
    let w = mic_block_outer_width_snapped_px(line1, line2);
    RectPx::new(node.position.x, node.position.y, w, MIC_BLOCK_FRAME_HEIGHT_PX)
}

pub fn volume_control_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y + VC_SYMBOL_TOP_INSET_PX,
        PX_PER_INCH,
        VOLUME_CONTROL_FRAME_HEIGHT_PX,
    )
}

pub fn text_block_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        node_width(node, 200.0),
        node_height(node, 80.0),
    )
}

pub fn flyoff_note_bounds_width_px(node: &Node) -> f64 {
    node_width(node, 120.0)
}

pub fn flyoff_note_bounds_height_px() -> f64 {
    FLYOFF_TRI_H
}

pub fn flyoff_note_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y,
        flyoff_note_bounds_width_px(node),
        flyoff_note_bounds_height_px(),
    )
}

pub fn antenna_bounds(node: &Node) -> RectPx {
    let line1 = node
        .data
        .get("line1")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ANT");
    let row_w = antenna_content_row_width_px(line1, f64::INFINITY)
        .max(ANTENNA_SCHEMATIC_SVG_WIDTH_PX + 20.0);
    let w = node_width(node, row_w);
    RectPx::new(node.position.x, node.position.y, w, 20.0)
}

/// RF handle center for antenna wires — matches v6 `getAntennaSymbolAnalyticalPortXY`.
pub fn antenna_symbol_port_xy(node: &Node, handle_id: &str) -> Option<PointPx> {
    let (is_receiver, expected_handle) = match node.node_type.as_str() {
        "antennaTransmitterSymbol" => (false, "ant-tx"),
        "antennaReceiverSymbol" => (true, "ant-rx"),
        _ => return None,
    };
    if handle_id != expected_handle {
        return None;
    }

    let line1 = node
        .data
        .get("line1")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ANT");
    let row_w = antenna_content_row_width_px(line1, f64::INFINITY)
        .max(ANTENNA_SCHEMATIC_SVG_WIDTH_PX + 20.0);
    let w = node_width(node, row_w);

    let mx = if is_receiver {
        ANT_RX_MAST_X
    } else {
        ANT_TX_MAST_X
    };
    let tip_x = if is_receiver {
        mx + ANT_FOOT_LEN_PX
    } else {
        mx - ANT_FOOT_LEN_PX
    };
    let sym_left = if is_receiver {
        w - ANTENNA_SCHEMATIC_SVG_WIDTH_PX
    } else {
        0.0
    };
    let row_top = (20.0 - ANTENNA_CONNECTOR_BAND_H_PX) / 2.0;
    let y_bot = CONNECTOR_ROW_OUTER_HEIGHT_PX + CONNECTOR_ROW_OUTER_HEIGHT_PX / 2.0;
    let cx_offset = sym_left + tip_x + if is_receiver { -1.0 } else { 1.0 };

    Some(PointPx {
        x: node.position.x + cx_offset,
        y: node.position.y + row_top + y_bot,
    })
}

pub fn grouping_zone_bounds(node: &Node) -> RectPx {
    let nx = node.position.x;
    let ny = node.position.y;
    let w = node_width(node, GROUPING_ZONE_DEFAULT_W);
    let h = node_height(node, GROUPING_ZONE_DEFAULT_H);
    let shape = node
        .data
        .get("shape")
        .and_then(|v| v.as_str())
        .unwrap_or("rect");

    if shape == "polyline" {
        if let Some(pts) = node.data.get("polylinePoints").and_then(|v| v.as_array()) {
            let flat: Vec<f64> = pts.iter().filter_map(|v| v.as_f64()).collect();
            if let Some((min_x, min_y, max_x, max_y)) = polyline_flat_bounds(&flat) {
                return RectPx::new(nx + min_x, ny + min_y, max_x - min_x, max_y - min_y);
            }
        }
    }

    RectPx::new(nx, ny, w, h)
}

/// Junction scene/export bounds including tag band above the frame.
pub fn junction_scene_bounds_rect(node: &Node) -> RectPx {
    let row_count = node
        .data
        .get("rowCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if row_count == 0 {
        return RectPx::new(node.position.x, node.position.y, 6.0, 6.0);
    }
    let total_h = junction_total_height_px(row_count);
    RectPx::new(
        node.position.x,
        node.position.y - SCHEMATIC_TAG_BAND_PX,
        PATCH_PANEL_WIDTH_PX,
        SCHEMATIC_TAG_BAND_PX + total_h,
    )
}
