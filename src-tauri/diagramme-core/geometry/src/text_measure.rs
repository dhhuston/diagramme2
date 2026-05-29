//! Text cap heights and conservative width estimates for scene bounds (strict mirror).
//!
//! Scene cap height in diagram px equals DXF height in inches × 72 — no export visual scale.

use crate::paper_scale::{
    DEVICE_PORT_LABEL_FONT_PX, LABEL_FONT_PX, MIC_SPEAKER_DESC_FONT_PX, NODE_TITLE_FONT_PX,
    SNAP_GRID_PX, VOLUME_CONTROL_VC_TEXT_HEIGHT_PX, WIRETAG_BAR_HEIGHT_PX,
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

/// Text roles mirroring v6 `ExportTextRole` (strict mirror — no export visual scale).
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
        TextRole::Tag | TextRole::Title => (
            NODE_TITLE_FONT_PX,
            TextHAlign::Center,
            TextVAlign::Middle,
        ),
        TextRole::GroupingZone => {
            (LABEL_FONT_PX, TextHAlign::Center, TextVAlign::Middle)
        }
        TextRole::Cell => (
            DEVICE_PORT_LABEL_FONT_PX,
            TextHAlign::Center,
            TextVAlign::Middle,
        ),
        TextRole::RowLabel => (6.0, TextHAlign::Left, TextVAlign::Middle),
        TextRole::SpeakerPrimary => (6.75, TextHAlign::Center, TextVAlign::Middle),
        TextRole::SpeakerSecondary => (
            MIC_SPEAKER_DESC_FONT_PX,
            TextHAlign::Center,
            TextVAlign::Middle,
        ),
        TextRole::MicChannel => (5.0, TextHAlign::Center, TextVAlign::Middle),
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

/// Word-wrap schematic title lines to fit `frame_width_px` at `font_px` cap height.
///
/// Each input line is wrapped independently; empty input yields a single space line.
///
/// Line breaks use the tighter of pixel-width and character-count limits. Character width
/// is estimated from `LABEL_FONT_PX` (v6 export wrap basis) so 3/32" titles still break
/// at the same words as the on-screen / v6 Revit layout.
pub fn wrap_schematic_title_lines(
    lines: &[String],
    frame_width_px: f64,
    side_padding_px: f64,
    font_px: f64,
) -> Vec<String> {
    let normalized: Vec<String> = if lines.is_empty() {
        vec![" ".to_string()]
    } else {
        lines.to_vec()
    };
    let usable_width_px = (frame_width_px - side_padding_px * 2.0).max(8.0);
    let approx_char_width_px = LABEL_FONT_PX * 0.52;
    let max_chars_per_line =
        ((usable_width_px / approx_char_width_px).floor() as usize).max(6);
    let mut out = Vec::new();

    for line in normalized {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push(" ".to_string());
            continue;
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let mut current = String::new();
        for word in words {
            if current.is_empty() {
                current = word.to_string();
                continue;
            }
            let next = format!("{current} {word}");
            if title_line_fits(&next, font_px, usable_width_px, max_chars_per_line) {
                current = next;
            } else {
                out.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }

    if out.is_empty() {
        vec![" ".to_string()]
    } else {
        out
    }
}

fn title_line_fits(text: &str, font_px: f64, usable_width_px: f64, max_chars_per_line: usize) -> bool {
    text.chars().count() <= max_chars_per_line
        && estimate_text_width_px(text, font_px) <= usable_width_px
}

/// Vertical step between wrapped title lines at `font_px` cap height.
pub fn schematic_title_line_step_px(font_px: f64) -> f64 {
    font_px * crate::schematic_layout::SCHEMATIC_TITLE_LINE_HEIGHT
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

/// Index-column width from pair index and bar metrics (mirrors v6 `wiretagIndexColumnWidthPx`).
pub fn wiretag_index_column_width_px(pair_index: i64, bar_height_px: f64, font_size_px: f64) -> f64 {
    let floor = (bar_height_px * 1.35).round().clamp(9.0, 18.0);
    let s = pair_index.to_string();
    let computed = (s.len() as f64 * font_size_px * 0.55 + 6.0).ceil();
    floor.max(computed)
}

/// Minimum index-column width for wiretag hull sizing (pair index not yet known).
fn wiretag_index_column_min_width_px(bar_height_px: f64) -> f64 {
    wiretag_index_column_width_px(1, bar_height_px, wiretag_font_size_px(bar_height_px))
}

const WIRETAG_SPLIT_TEXT_PAD_PX: f64 = 2.0;
const WIRETAG_SPLIT_DIVIDER_WIDTH_PX: f64 = 0.5;
const WIRETAG_SPLIT_DIVIDER_MARGIN_PX: f64 = 1.0;

/// Wiretag hull width for DXF/scene export (mirrors v6 `wiretagExportApproxWidthPx`).
pub fn wiretag_export_width_px(
    pair_index: i64,
    main_text: &str,
    sheet_name: &str,
    show_sheet_name: bool,
    bar_height_px: f64,
) -> f64 {
    let bar_h = if bar_height_px > 0.0 {
        bar_height_px
    } else {
        WIRETAG_BAR_HEIGHT_PX
    };
    let font_size = wiretag_font_size_px(bar_h);
    let iw = wiretag_index_column_width_px(pair_index, bar_h, font_size);
    let tw = wiretag_tip_width_px(bar_h);
    let main = main_text.trim();
    let main_fallback = if main.is_empty() {
        format!("WT-{pair_index}")
    } else {
        main.to_string()
    };
    let text_w = if main_fallback.is_empty() {
        font_size * 2.0
    } else {
        estimate_text_width_px(&main_fallback, font_size)
    };
    let sheet = if show_sheet_name {
        sheet_name.trim()
    } else {
        ""
    };
    let sheet_w = if sheet.is_empty() {
        0.0
    } else {
        estimate_text_width_px(sheet, font_size)
    };
    let split_divider = if sheet.is_empty() {
        0.0
    } else {
        WIRETAG_SPLIT_DIVIDER_WIDTH_PX + WIRETAG_SPLIT_DIVIDER_MARGIN_PX * 2.0
    };
    let raw = iw
        + tw
        + text_w
        + WIRETAG_SPLIT_TEXT_PAD_PX
        + sheet_w
        + WIRETAG_SPLIT_TEXT_PAD_PX
        + split_divider;
    let w_min = iw + tw + 10.0;
    let w_max = 560.0;
    let clamped = raw.clamp(w_min, w_max).ceil();
    (clamped / SNAP_GRID_PX).ceil() * SNAP_GRID_PX
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
