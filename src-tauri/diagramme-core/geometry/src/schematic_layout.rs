//! Schematic layout numbers shared by port geometry and node bounds (aligned with v6 `dxfSchematicLayout.ts`).

use crate::paper_scale::{CONNECTOR_LINE_PITCH_PX, CONNECTOR_ROW_OUTER_HEIGHT_PX, PX_PER_INCH};

/// Vertical padding reserved above the frame for the tag strip.
pub const SCHEMATIC_TAG_BAND_PX: f64 = 15.0;

/// Tag text anchor (center/middle) in node-local px; frame top is y = 0.
pub const SCHEMATIC_TAG_TEXT_CENTER_Y_PX: f64 = -SCHEMATIC_TAG_BAND_PX / 2.0;

/// Geometric center Y of wrapped title line `line_index` in a title band.
pub fn schematic_wrapped_title_line_center_y(
    title_top_px: f64,
    title_height_px: f64,
    line_index: usize,
    line_count: usize,
    line_step_px: f64,
) -> f64 {
    let line_count = line_count.max(1);
    let first_center_y = title_top_px + title_height_px / 2.0
        - ((line_count - 1) as f64 * line_step_px) / 2.0;
    first_center_y + line_index as f64 * line_step_px
}

/// Geometric center Y of a single-line title band of height `title_height_px` (top at y = 0).
pub fn schematic_title_band_center_y(title_height_px: f64) -> f64 {
    schematic_wrapped_title_line_center_y(0.0, title_height_px, 0, 1, 0.0)
}

/// Geometric center Y of body row `row_index` (0-based) given body top and row height.
pub fn schematic_body_row_center_y(body_top_px: f64, row_index: usize, row_height_px: f64) -> f64 {
    body_top_px + (row_index as f64 + 0.5) * row_height_px
}

/// Vertical padding inside schematic title bands for wrapped header text.
pub const SCHEMATIC_TITLE_SIDE_PADDING_PX: f64 = 4.0;

/// Line spacing multiplier for wrapped schematic titles (matches v6 patch header CSS).
pub const SCHEMATIC_TITLE_LINE_HEIGHT: f64 = 1.15;

/// Inset from schematic chrome for border rectangles.
pub const SCHEMATIC_FRAME_INSET_PX: f64 = 0.25;

/// Device v2 / AV plate body row pitch (center-to-center of connector rules).
pub const DEVICE_V2_GRID_ROW_PX: f64 = CONNECTOR_LINE_PITCH_PX;

/// Row handle center offset within a grid row.
pub const DEVICE_V2_ROW_CENTER_Y_PX: f64 = DEVICE_V2_GRID_ROW_PX / 2.0;

/// Device v2 title band height (two grid rows).
pub const DEVICE_V2_TITLE_HEIGHT_PX: f64 = 2.0 * DEVICE_V2_GRID_ROW_PX;

/// AV plate uses the same row pitch as device v2.
pub const AV_PLATE_GRID_ROW_PX: f64 = CONNECTOR_LINE_PITCH_PX;

pub const AV_PLATE_TITLE_HEIGHT_PX: f64 = 2.0 * AV_PLATE_GRID_ROW_PX;

/// Patch panel nominal width (1").
pub const PATCH_PANEL_WIDTH_PX: f64 = PX_PER_INCH;

/// Patch panel bordered row band height.
pub const PATCH_GRID_ROW_PX: f64 = CONNECTOR_ROW_OUTER_HEIGHT_PX;

/// Patch title band: three connector-row bands.
pub const PATCH_TITLE_HEIGHT_PX: f64 = 3.0 * PATCH_GRID_ROW_PX;

/// Patch circuit row pitch (1/8" line centers).
pub const PATCH_CIRCUIT_HEIGHT_PX: f64 = CONNECTOR_LINE_PITCH_PX;

/// Circuit row line/handle center offset from row top.
pub const PATCH_ROW_CENTER_Y_PX: f64 = 6.0;

pub const PATCH_BODY_SPACER_PX: f64 = PATCH_CIRCUIT_HEIGHT_PX;

pub const PATCH_FOOTER_HEIGHT_PX: f64 = PATCH_CIRCUIT_HEIGHT_PX;

/// Spacer between title divider and first circuit row.
pub const PATCH_BODY_TOP_PX: f64 = PATCH_TITLE_HEIGHT_PX + PATCH_BODY_SPACER_PX;

/// Bundle bracket stub + fillet (outward handle placement).
pub const BUNDLE_STUB_PX: f64 = 18.0;

pub const BUNDLE_FILLET_PX: f64 = 6.0;

/// Breakline zigzag vertical extent at the frame bottom (half a patch grid row).
pub const BREAKLINE_OVERHANG: f64 = PATCH_GRID_ROW_PX / 2.0;

/// Breakline flat zone width at device / patch frame center (1/4").
pub const BREAKLINE_DEVICE_ZONE_WIDTH_PX: f64 = PX_PER_INCH / 4.0;

pub const BUNDLE_ARROW_STEM_PX: f64 = 6.0;

pub const BUNDLE_ARROW_LEG_PX: f64 = 3.0;

/// Fillet radius for a bundle bracket; shrinks when bundled rows are closer than 2× fillet.
pub fn bundle_bracket_fillet_radius_px(y0: f64, y1: f64) -> f64 {
    let span = (y1 - y0).max(0.0);
    BUNDLE_FILLET_PX.min(span / 2.0)
}

/// Patch panel types that share L/R row handle geometry.
pub const PATCH_PANEL_NODE_TYPES: &[&str] = &[
    "lppPatchPanel",
    "dppPatchPanel",
    "mlpPatchPanel",
    "vpbPatchPanel",
];

pub fn is_patch_panel_node_type(node_type: &str) -> bool {
    PATCH_PANEL_NODE_TYPES.contains(&node_type)
}

/// Total patch panel frame height for `row_count` circuit rows.
pub fn patch_panel_total_height_px(row_count: usize) -> f64 {
    PATCH_BODY_TOP_PX + row_count as f64 * PATCH_CIRCUIT_HEIGHT_PX + PATCH_FOOTER_HEIGHT_PX
}
