//! Wire inner-segment midpoints and drag (mirrors v6 `schematicEdgePath.ts`).

use crate::sharp_polyline::{sanitize_orthogonal_chain, snap_coord, snap_point};
use crate::types::FlowXY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireSegmentOrientation {
    Horizontal,
    Vertical,
}

impl WireSegmentOrientation {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "h" => Some(Self::Horizontal),
            "v" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "h",
            Self::Vertical => "v",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WireSegmentGrip {
    pub segment_index: usize,
    pub mid: FlowXY,
    pub orientation: WireSegmentOrientation,
}

fn is_horizontal(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.y) == snap_coord(b.y) && snap_coord(a.x) != snap_coord(b.x)
}

fn is_vertical(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) != snap_coord(b.y)
}

/// Interior corners only (between S1 and T1, exclusive).
pub fn inner_corners_from_chain(chain: &[FlowXY]) -> Vec<FlowXY> {
    if chain.len() <= 2 {
        return Vec::new();
    }
    chain[1..chain.len() - 1].to_vec()
}

/// Midpoints for routing grips on segments strictly between S1 and T1.
pub fn inner_segment_midpoints(chain: &[FlowXY]) -> Vec<WireSegmentGrip> {
    let mut out = Vec::new();
    for i in 0..chain.len().saturating_sub(1) {
        let a = chain[i];
        let b = chain[i + 1];
        if is_horizontal(a, b) {
            out.push(WireSegmentGrip {
                segment_index: i,
                mid: snap_point(FlowXY {
                    x: (a.x + b.x) / 2.0,
                    y: a.y,
                }),
                orientation: WireSegmentOrientation::Horizontal,
            });
        } else if is_vertical(a, b) {
            out.push(WireSegmentGrip {
                segment_index: i,
                mid: snap_point(FlowXY {
                    x: a.x,
                    y: (a.y + b.y) / 2.0,
                }),
                orientation: WireSegmentOrientation::Vertical,
            });
        }
    }
    out
}

/// Drag a horizontal inner segment vertically; pins S1/T1 and inserts jogs when needed.
pub fn drag_horizontal_inner_segment(
    chain: &[FlowXY],
    segment_index: usize,
    delta_y: f64,
) -> Vec<FlowXY> {
    if segment_index >= chain.len().saturating_sub(1) {
        return chain.to_vec();
    }
    let a = chain[segment_index];
    let b = chain[segment_index + 1];
    if !is_horizontal(a, b) {
        return chain.to_vec();
    }

    let target_y = snap_coord(a.y + delta_y);
    let dy = target_y - a.y;
    if dy.abs() < 1e-6 {
        return chain.to_vec();
    }

    let s1 = chain[0];
    let t1 = chain[chain.len() - 1];

    if segment_index == 0 && a.x == s1.x && a.y == s1.y {
        let j = snap_point(FlowXY {
            x: s1.x,
            y: s1.y + dy,
        });
        let b_moved = snap_point(FlowXY {
            x: b.x,
            y: s1.y + dy,
        });
        let rest = chain.iter().skip(2).copied().collect::<Vec<_>>();
        let mut merged = vec![s1, j, b_moved];
        merged.extend(rest);
        return sanitize_orthogonal_chain(&merged);
    }

    if segment_index == chain.len() - 2 && b.x == t1.x && b.y == t1.y {
        let a_moved = snap_point(FlowXY {
            x: a.x,
            y: a.y + dy,
        });
        let j = snap_point(FlowXY {
            x: t1.x,
            y: a.y + dy,
        });
        let mut merged: Vec<FlowXY> = chain[..segment_index].to_vec();
        merged.extend([a_moved, j, t1]);
        return sanitize_orthogonal_chain(&merged);
    }

    let next: Vec<FlowXY> = chain
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            if idx == segment_index || idx == segment_index + 1 {
                if idx == 0 || idx == chain.len() - 1 {
                    *p
                } else {
                    snap_point(FlowXY {
                        x: p.x,
                        y: p.y + dy,
                    })
                }
            } else {
                *p
            }
        })
        .collect();
    sanitize_orthogonal_chain(&next)
}

/// Drag a vertical inner segment horizontally; pins S1/T1 and inserts jogs when needed.
pub fn drag_vertical_inner_segment(chain: &[FlowXY], segment_index: usize, delta_x: f64) -> Vec<FlowXY> {
    if segment_index >= chain.len().saturating_sub(1) {
        return chain.to_vec();
    }
    let a = chain[segment_index];
    let b = chain[segment_index + 1];
    if !is_vertical(a, b) {
        return chain.to_vec();
    }

    let target_x = snap_coord(a.x + delta_x);
    let dx = target_x - a.x;
    if dx.abs() < 1e-6 {
        return chain.to_vec();
    }

    let s1 = chain[0];
    let t1 = chain[chain.len() - 1];

    if segment_index == 0 && a.x == s1.x && a.y == s1.y {
        let j = snap_point(FlowXY {
            x: s1.x + dx,
            y: s1.y,
        });
        let b_moved = snap_point(FlowXY {
            x: s1.x + dx,
            y: b.y,
        });
        let rest = chain.iter().skip(2).copied().collect::<Vec<_>>();
        let mut merged = vec![s1, j, b_moved];
        merged.extend(rest);
        return sanitize_orthogonal_chain(&merged);
    }

    if segment_index == chain.len() - 2 && b.x == t1.x && b.y == t1.y {
        let a_moved = snap_point(FlowXY {
            x: a.x + dx,
            y: a.y,
        });
        let j = snap_point(FlowXY {
            x: a.x + dx,
            y: t1.y,
        });
        let mut merged: Vec<FlowXY> = chain[..segment_index].to_vec();
        merged.extend([a_moved, j, t1]);
        return sanitize_orthogonal_chain(&merged);
    }

    let next: Vec<FlowXY> = chain
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            if idx == segment_index || idx == segment_index + 1 {
                if idx == 0 || idx == chain.len() - 1 {
                    *p
                } else {
                    snap_point(FlowXY {
                        x: p.x + dx,
                        y: p.y,
                    })
                }
            } else {
                *p
            }
        })
        .collect();
    sanitize_orthogonal_chain(&next)
}

pub fn drag_inner_segment(
    chain: &[FlowXY],
    segment_index: usize,
    orientation: WireSegmentOrientation,
    delta_x: f64,
    delta_y: f64,
) -> Vec<FlowXY> {
    match orientation {
        WireSegmentOrientation::Horizontal => {
            drag_horizontal_inner_segment(chain, segment_index, delta_y)
        }
        WireSegmentOrientation::Vertical => drag_vertical_inner_segment(chain, segment_index, delta_x),
    }
}
