//! Text cap heights and conservative width estimates for scene bounds (strict mirror).
//!
//! Scene cap height in diagram px equals DXF height in inches × 72 — no export visual scale.

use crate::paper_scale::{
    DEVICE_PORT_LABEL_FONT_PX, LABEL_FONT_PX, SNAP_GRID_PX, VOLUME_CONTROL_VC_TEXT_HEIGHT_PX,
    WIRETAG_BAR_HEIGHT_PX,
};
use crate::types::RectPx;

/// Horizontal text alignment in diagram space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextHAlign {
    Left,
    Center,
    Right,
}

/// Vertical text alignment in diagram space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVAlign {
    Top,
    Middle,
    Bottom,
}

/// Resolved text style for scene / DXF emit (cap height is final).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font: &'static str,
    pub height_px: f64,
    pub halign: TextHAlign,
    pub valign: TextVAlign,
}

/// Text roles mirroring v6 `ExportTextRole` (without `EXPORT_TEXT_VISUAL_SCALE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Tag,
    Title,
    Cell,
    RowLabel,
    SpeakerPrimary,
    SpeakerSecondary,
    MicChannel,
    VolumeControl,
    Wiretag,
    TextBlock,
    Flyoff,
    Antenna,
    GroupingZone,
    BundleCount,
}

const ARIAL_NARROW: &str = "Arial Narrow";

/// Cap height and alignment for a fixed text role.
///
/// Dynamic roles (`Wiretag`, `TextBlock`) return `height_px = 0.0`; callers supply bar / node
/// font size via [`wiretag_font_size_px`] or an explicit override when building scene text.
pub fn text_style_for_role(role: TextRole) -> TextStyle {
    let (height_px, halign, valign) = match role {
        TextRole::Tag | TextRole::Title | TextRole::GroupingZone => {
            (LABEL_FONT_PX, TextHAlign::Center, TextVAlign::Middle)
        }
        TextRole::Cell => (
            DEVICE_PORT_LABEL_FONT_PX,
            TextHAlign::Center,
            TextVAlign::Middle,
        ),
        TextRole::RowLabel => (6.0, TextHAlign::Left, TextVAlign::Middle),
        TextRole::SpeakerPrimary => (6.75, TextHAlign::Center, TextVAlign::Middle),
        TextRole::SpeakerSecondary | TextRole::MicChannel => {
            (5.0, TextHAlign::Center, TextVAlign::Middle)
        }
        TextRole::VolumeControl => (
            VOLUME_CONTROL_VC_TEXT_HEIGHT_PX,
            TextHAlign::Center,
            TextVAlign::Middle,
        ),
        TextRole::Flyoff | TextRole::Antenna => (6.75, TextHAlign::Center, TextVAlign::Middle),
        TextRole::BundleCount => (6.0, TextHAlign::Center, TextVAlign::Middle),
        TextRole::Wiretag | TextRole::TextBlock => (0.0, TextHAlign::Center, TextVAlign::Middle),
    };

    TextStyle {
        font: ARIAL_NARROW,
        height_px,
        halign,
        valign,
    }
}

/// Wiretag band cap height from bar outer height (mirrors v6 `computeWiretagWidthPx`).
pub fn wiretag_font_size_px(bar_height_px: f64) -> f64 {
    (bar_height_px - 2.0).max(4.0)
}

/// Conservative text width estimate for autosize when font metrics are unavailable.
///
/// Uses `char_count × height_px × 0.55` (Arial Narrow bold uppercase tends to be ~0.52–0.58 em wide).
///
/// TODO: Replace with measured Arial Narrow metrics (canvas `measureText` parity from v6
/// `wiretagLayout.ts`).
pub fn estimate_text_width_px(content: &str, height_px: f64) -> f64 {
    let trimmed = content.trim();
    if trimmed.is_empty() || height_px <= 0.0 {
        return 0.0;
    }
    let char_count = trimmed.chars().count() as f64;
    char_count * height_px * 0.55
}

/// Arrow-tip horizontal extent (diagram px), from bar height only — mirrored pair ends.
pub fn wiretag_tip_width_px(bar_height_px: f64) -> f64 {
    let tip = (bar_height_px * 0.88).round();
    tip.clamp(5.0, 12.0)
}

/// Minimum index-column width for wiretag hull sizing (pair index not yet known).
fn wiretag_index_column_min_width_px(bar_height_px: f64) -> f64 {
    let floor = (bar_height_px * 1.35).round().clamp(9.0, 18.0);
    floor
}

/// Wiretag hull bounds in diagram px for autosize (origin at 0,0).
///
/// Ports v6 `computeWiretagWidthPx` without sheet-name segment; snaps width to `SNAP_GRID_PX`.
pub fn wiretag_bounds_diagram_px(content: &str, bar_height_px: f64) -> RectPx {
    let bar_h = if bar_height_px > 0.0 {
        bar_height_px
    } else {
        WIRETAG_BAR_HEIGHT_PX
    };
    let font_size = wiretag_font_size_px(bar_h);
    let index_w = wiretag_index_column_min_width_px(bar_h);
    let tip_w = wiretag_tip_width_px(bar_h);
    let text_w = estimate_text_width_px(content, font_size);
    let main_pad = 2.0;
    let raw = index_w + tip_w + text_w + main_pad;
    let w_min = index_w + tip_w + 10.0;
    let w_max = 560.0;
    let clamped = raw.clamp(w_min, w_max).ceil();
    let snapped = (clamped / SNAP_GRID_PX).ceil() * SNAP_GRID_PX;

    RectPx::new(0.0, 0.0, snapped, bar_h)
}
