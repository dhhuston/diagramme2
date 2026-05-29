//! Diagram-space geometry primitives (1 canvas px = 1/72 inch).

/// Point in diagram pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointPx {
    pub x: f64,
    pub y: f64,
}

/// Axis-aligned rectangle in diagram pixels (`x`, `y` = top-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPx {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl RectPx {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
