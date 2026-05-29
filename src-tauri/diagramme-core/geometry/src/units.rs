//! Diagram-space px ↔ CAD inches (single linear scale).

use crate::paper_scale::PX_PER_INCH;

/// Export calibration: 1 canvas px = 1/72 inch (`PX_PER_INCH` = 72).
pub const EXPORT_PX_PER_INCH: f64 = PX_PER_INCH;

/// Single source of truth for diagram-space px → CAD inches conversion.
pub const DIAGRAM_PX_TO_INCH: f64 = 1.0 / EXPORT_PX_PER_INCH;

/// Convert diagram pixels to inches.
#[inline]
pub fn px_to_in(px: f64) -> f64 {
    px * DIAGRAM_PX_TO_INCH
}

/// Convert inches to diagram pixels.
#[inline]
pub fn in_to_px(inches: f64) -> f64 {
    inches * EXPORT_PX_PER_INCH
}
