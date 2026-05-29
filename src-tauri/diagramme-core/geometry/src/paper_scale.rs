//! Canvas coordinate system: 1 canvas px = 1 PDF point = 1/72 inch.
//!
//! Reference sheet: E1 (30" × 42" landscape) = 2160 × 3024 canvas units.

/// 72 dpi diagram space — 1 canvas px = 1/72 inch.
pub const PX_PER_INCH: f64 = 72.0;

/// 1/4 inch — standard AV functional diagram connector row height.
pub const ROW_HEIGHT_PX: f64 = 18.0;

/// 1/8 inch — standard CAD plotted text height for tag strip above node frames.
pub const LABEL_FONT_PX: f64 = 9.0;

/// 3/32 inch — node title band cap height (patch panel header, device description, AV plate title).
pub const NODE_TITLE_FONT_PX: f64 = (3.0 / 32.0) * PX_PER_INCH;

/// Device block: each input/output connector column width (3/4").
pub const DEVICE_CONNECTOR_COLUMN_PX: f64 = (3.0 / 4.0) * PX_PER_INCH;

/// Device block: open gutter between connector columns (1/4").
pub const DEVICE_CONNECTOR_GUTTER_PX: f64 = PX_PER_INCH / 4.0;

/// Device v2: gutter between L/R connector columns (1/2") — layout only, on 1/8" snap grid.
pub const DEVICE_V2_CONNECTOR_GUTTER_PX: f64 = (1.0 / 2.0) * PX_PER_INCH;

/// Device v2 inner connector grid width (two 3/4" columns + 1/2" gutter) = 144 px.
pub const DEVICE_V2_CONNECTOR_GRID_WIDTH_PX: f64 =
    2.0 * DEVICE_CONNECTOR_COLUMN_PX + DEVICE_V2_CONNECTOR_GUTTER_PX;

/// Device shell content width (two columns + gutter), excluding outer 1px borders.
pub const DEVICE_CONNECTOR_GRID_WIDTH_PX: f64 =
    2.0 * DEVICE_CONNECTOR_COLUMN_PX + DEVICE_CONNECTOR_GUTTER_PX;

/// Outer device block width: connector grid + 1px left/right shell border.
pub const DEVICE_SHELL_OUTER_WIDTH_PX: f64 = DEVICE_CONNECTOR_GRID_WIDTH_PX + 2.0;

/// 1/8" vertical pitch for connector drafting: distance **center-to-center** of adjacent 1px
/// horizontal rules (72dpi diagram space).
pub const CONNECTOR_LINE_PITCH_PX: f64 = PX_PER_INCH / 8.0;

/// Border-box height of a port row or grey group-header cell: `pitch + 1px` so the two horizontal
/// rules bounding the cell have centers `CONNECTOR_LINE_PITCH_PX` apart.
pub const CONNECTOR_ROW_OUTER_HEIGHT_PX: f64 = CONNECTOR_LINE_PITCH_PX + 1.0;

/// Outer-edge clearance between bordered stacks so rule centers are `CONNECTOR_LINE_PITCH_PX`
/// apart when both sides contribute a 1px edge at the boundary.
pub const CONNECTOR_RULE_OUTER_CLEARANCE_PX: f64 = CONNECTOR_LINE_PITCH_PX - 1.0;

/// Port / grey-header row: CSS height (see `CONNECTOR_ROW_OUTER_HEIGHT_PX`).
pub const DEVICE_ROW_HEIGHT_PX: f64 = CONNECTOR_ROW_OUTER_HEIGHT_PX;

/// Stacked schematic blocks: one connector row band (`CONNECTOR_ROW_OUTER_HEIGHT_PX`).
pub const SCHEMATIC_BLOCK_ROW_PX: f64 = CONNECTOR_ROW_OUTER_HEIGHT_PX;

/// Mic/speaker block root height — five block rows, aligns handle band with device port grid.
pub const MIC_SPEAKER_FRAME_HEIGHT_PX: f64 = 5.0 * SCHEMATIC_BLOCK_ROW_PX;

/// Mic/speaker/VC schematic strip height (symbol row — unchanged vs block row pitch).
pub const MIC_SPEAKER_VC_STRIP_HEIGHT_PX: f64 = 14.0;

/// Shell connector padding + continuation inter-group margin — pitch between rule centers.
pub const CONNECTOR_GROUP_GAP_PX: f64 = CONNECTOR_RULE_OUTER_CLEARANCE_PX;

/// Node drag / placement snap grid — aligns with 1/8" pitch (`CONNECTOR_LINE_PITCH_PX` / 3).
pub const SNAP_GRID_PX: f64 = 3.0;

/// Visible canvas grid line spacing (`Background` gap only).
pub const VISIBLE_GRID_GAP_PX: f64 = SNAP_GRID_PX;

/// Half of `SNAP_GRID_PX` — React Flow `snapGrid` / handle nudge use this; grid lines stay on full spacing.
pub const SNAP_PLACEMENT_HALF_STEP_PX: f64 = SNAP_GRID_PX / 2.0;

/// Mic / speaker / volume-control: pull RF handles inward from the schematic stroke.
pub const SCHEMATIC_AUDIO_HANDLE_INSET_PX: f64 = SNAP_GRID_PX;

/// Device v2 outer block width (1.75").
pub const DEVICE_V2_WIDTH_PX: f64 = 1.75 * PX_PER_INCH;

/// Snap to full grid (multiples of `SNAP_GRID_PX`).
pub const fn snap_placement_coord(v: f64) -> f64 {
    (v / SNAP_GRID_PX).round() * SNAP_GRID_PX
}

/// Snap to invisible half grid (multiples of `SNAP_PLACEMENT_HALF_STEP_PX`).
pub const fn snap_placement_half_grid(v: f64) -> f64 {
    ((v * 2.0) / SNAP_GRID_PX).round() * (SNAP_GRID_PX / 2.0)
}

/// Mic/speaker handle band center Y — snapped so port centers fall on `SNAP_GRID_PX`.
pub const MIC_SPEAKER_HANDLE_CENTER_Y_PX: f64 =
    snap_placement_coord(MIC_SPEAKER_FRAME_HEIGHT_PX / 2.0);

/// Strip (`MIC_SPEAKER_VC_STRIP_HEIGHT_PX`) is vertically centered in the mic/speaker frame.
pub const MIC_SPEAKER_STRIP_TOP_INSET_PX: f64 =
    (MIC_SPEAKER_FRAME_HEIGHT_PX - MIC_SPEAKER_VC_STRIP_HEIGHT_PX) / 2.0;

/// Mic/speaker handle center Y inside the 14px symbol strip.
pub const MIC_SPEAKER_HANDLE_Y_IN_STRIP_PX: f64 =
    MIC_SPEAKER_HANDLE_CENTER_Y_PX - MIC_SPEAKER_STRIP_TOP_INSET_PX;

/// VC hex: top vertex to bottom vertex (3/16" @ 72dpi).
pub const VOLUME_CONTROL_HEX_VERTEX_SPAN_PX: f64 = (3.0 / 16.0) * PX_PER_INCH;

/// VC node / schematic height — matches hex vertical vertex span.
pub const VOLUME_CONTROL_FRAME_HEIGHT_PX: f64 = VOLUME_CONTROL_HEX_VERTEX_SPAN_PX;

/// Volume control handle center Y — snapped (`13.5/2` is fractional).
pub const VOLUME_CONTROL_HANDLE_CENTER_Y_PX: f64 =
    snap_placement_coord(VOLUME_CONTROL_FRAME_HEIGHT_PX / 2.0);

/// Volume control “VC” label cap height (1/16" @ 72dpi).
pub const VOLUME_CONTROL_VC_TEXT_HEIGHT_PX: f64 = PX_PER_INCH / 16.0;

/// Mic/speaker line2 (description) cap height (1/16" @ 72dpi).
pub const MIC_SPEAKER_DESC_FONT_PX: f64 = PX_PER_INCH / 16.0;

/// Device block: short description band total height (1/4").
pub const DEVICE_DESCRIPTION_BAND_PX: f64 = PX_PER_INCH / 4.0;

/// Device block: short description cap / nominal plot height (3/32").
pub const DEVICE_DESCRIPTION_FONT_PX: f64 = (3.0 / 32.0) * PX_PER_INCH;

/// Wiretag pair banner row height — 2 × `SNAP_GRID_PX` so H/2 lands on the snap grid.
pub const WIRETAG_BAR_HEIGHT_PX: f64 = 2.0 * SNAP_GRID_PX;

/// Fallback width when node width is missing (overlay cursors, etc.).
pub const WIRETAG_DEFAULT_WIDTH_PX: f64 = 160.0;

/// Initial / empty wiretag width — tight box until autosize runs.
pub const WIRETAG_INITIAL_WIDTH_PX: f64 = 40.0;

/// Port and group-header labels inside 1/8" rows.
pub const DEVICE_PORT_LABEL_FONT_PX: f64 = 5.0;
