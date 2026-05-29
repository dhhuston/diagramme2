//! Schematic wire crossing detection (mirrors v6 `schematicWireCrossings.ts`).

use diagramme_geometry::SNAP_GRID_PX;

use crate::sharp_polyline::snap_coord;
use crate::types::FlowXY;

/// Half-width of the horizontal gap on each side of a crossing vertical (flow px).
pub const SCHEMATIC_CROSSING_GAP_HALF_PX: f64 = 4.0;

/// Same-source verticals within this X distance of the source handle are treated as a shared output bus (no gap).
pub const SOURCE_SHARED_BUS_X_TOLERANCE_PX: f64 = SNAP_GRID_PX * 6.0;

const EPS: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
pub struct AxisHorizontalRun {
    pub edge_id: String,
    pub y: f64,
    pub x_min: f64,
    pub x_max: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisVerticalRun {
    pub edge_id: String,
    pub x: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// One edge of a schematic wire polyline (axis-aligned segment).
#[derive(Debug, Clone, PartialEq)]
pub struct WireSegment {
    pub edge_id: String,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub source_node_id: Option<String>,
}

fn segment_is_horizontal(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.y) == snap_coord(b.y) && snap_coord(a.x) != snap_coord(b.x)
}

fn segment_is_vertical(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) != snap_coord(b.y)
}

/// Split a sharp orthogonal polyline into axis-aligned runs (corners only).
pub fn polyline_to_axis_runs(points: &[FlowXY], edge_id: &str) -> (Vec<AxisHorizontalRun>, Vec<AxisVerticalRun>) {
    let mut h = Vec::new();
    let mut v = Vec::new();
    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        let x_lo = a.x.min(b.x);
        let x_hi = a.x.max(b.x);
        let y_lo = a.y.min(b.y);
        let y_hi = a.y.max(b.y);
        if segment_is_horizontal(a, b) {
            h.push(AxisHorizontalRun {
                edge_id: edge_id.to_string(),
                y: snap_coord(a.y),
                x_min: x_lo,
                x_max: x_hi,
            });
        } else if segment_is_vertical(a, b) {
            v.push(AxisVerticalRun {
                edge_id: edge_id.to_string(),
                x: snap_coord(a.x),
                y_min: y_lo,
                y_max: y_hi,
            });
        }
    }
    (h, v)
}

pub fn crosses_interior(h: &AxisHorizontalRun, v: &AxisVerticalRun) -> bool {
    if v.edge_id == h.edge_id {
        return false;
    }
    let x_lo = h.x_min.min(h.x_max);
    let x_hi = h.x_min.max(h.x_max);
    let xv = snap_coord(v.x);
    if xv <= snap_coord(x_lo) || xv >= snap_coord(x_hi) {
        return false;
    }
    let y_lo = v.y_min.min(v.y_max);
    let y_hi = v.y_min.max(v.y_max);
    let hy = snap_coord(h.y);
    hy > snap_coord(y_lo) && hy < snap_coord(y_hi)
}

pub fn polyline_to_wire_segments(points: &[FlowXY], edge_id: &str) -> Vec<WireSegment> {
    let mut segs = Vec::new();
    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        if snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) == snap_coord(b.y) {
            continue;
        }
        segs.push(WireSegment {
            edge_id: edge_id.to_string(),
            x0: a.x,
            y0: a.y,
            x1: b.x,
            y1: b.y,
            source_node_id: None,
        });
    }
    segs
}

/// Segments from other edges for crossing-gap tests (excludes same-source device outputs).
pub fn wire_segments_for_crossing_gaps(
    segments: &[WireSegment],
    exclude_source_node_id: Option<&str>,
    source_handle_x: Option<f64>,
) -> Vec<WireSegment> {
    let Some(exclude_source_node_id) = exclude_source_node_id else {
        return segments.to_vec();
    };
    segments
        .iter()
        .filter(|s| {
            if s.source_node_id.as_deref() != Some(exclude_source_node_id) {
                return true;
            }
            let is_vertical = snap_coord(s.x0) == snap_coord(s.x1);
            if !is_vertical {
                return false;
            }
            let Some(source_handle_x) = source_handle_x else {
                return false;
            };
            (snap_coord(s.x0) - snap_coord(source_handle_x)).abs()
                > SOURCE_SHARED_BUS_X_TOLERANCE_PX
        })
        .cloned()
        .collect()
}

pub fn horizontal_runs_cross_any_vertical(
    horizontal_runs: &[AxisHorizontalRun],
    vertical_obstacles: &[AxisVerticalRun],
) -> bool {
    for h in horizontal_runs {
        for v in vertical_obstacles {
            if crosses_interior(h, v) {
                return true;
            }
        }
    }
    false
}

pub fn merge_intervals(intervals: &mut Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    if intervals.is_empty() {
        return Vec::new();
    }
    intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut out = Vec::new();
    let mut cur = intervals[0];
    for next in intervals.iter().copied().skip(1) {
        if next.0 <= cur.1 + EPS {
            cur.1 = cur.1.max(next.1);
        } else {
            out.push(cur);
            cur = next;
        }
    }
    out.push(cur);
    out
}

/// One horizontal run as solid subpaths with gaps over crossing vertical obstacles.
pub fn segments_for_gapped_horizontal_run(
    h: &AxisHorizontalRun,
    vertical_obstacles: &[AxisVerticalRun],
) -> Vec<Vec<FlowXY>> {
    let x_lo = h.x_min.min(h.x_max);
    let x_hi = h.x_min.max(h.x_max);
    let mut raw_gaps = Vec::new();
    for v in vertical_obstacles {
        if !crosses_interior(h, v) {
            continue;
        }
        let gx_lo = x_lo.max(v.x - SCHEMATIC_CROSSING_GAP_HALF_PX);
        let gx_hi = x_hi.min(v.x + SCHEMATIC_CROSSING_GAP_HALF_PX);
        if gx_lo + EPS < gx_hi {
            raw_gaps.push((gx_lo, gx_hi));
        }
    }
    let gaps = merge_intervals(&mut raw_gaps);
    if gaps.is_empty() {
        return vec![vec![
            FlowXY { x: x_lo, y: h.y },
            FlowXY { x: x_hi, y: h.y },
        ]];
    }

    let mut out = Vec::new();
    let mut cursor = x_lo;
    for (ga, gb) in gaps {
        if ga > cursor + EPS {
            out.push(vec![
                FlowXY { x: cursor, y: h.y },
                FlowXY { x: ga, y: h.y },
            ]);
        }
        cursor = cursor.max(gb);
    }
    if x_hi > cursor + EPS {
        out.push(vec![
            FlowXY { x: cursor, y: h.y },
            FlowXY { x: x_hi, y: h.y },
        ]);
    }
    out
}
