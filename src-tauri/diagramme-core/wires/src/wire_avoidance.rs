//! Obstacle avoidance for schematic wire inner chains (mirrors v6 `schematicWireAvoidance.ts`).

use crate::sharp_polyline::{orthogonalize_chain, sanitize_orthogonal_chain, snap_coord, snap_point};
use crate::types::{FlowXY, WireObstacleBox};

fn box_bounds(box_: &WireObstacleBox) -> (f64, f64, f64, f64) {
    (
        box_.x.min(box_.x2),
        box_.y.min(box_.y2),
        box_.x.max(box_.x2),
        box_.y.max(box_.y2),
    )
}

fn is_horizontal(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.y) == snap_coord(b.y) && snap_coord(a.x) != snap_coord(b.x)
}

fn is_vertical(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) != snap_coord(b.y)
}

fn same_snapped(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) == snap_coord(b.y)
}

/// Axis-aligned segment vs inflated obstacle (interior crossing only).
pub fn segment_intersects_box(a: FlowXY, b: FlowXY, box_: &WireObstacleBox) -> bool {
    let (x0, y0, x1, y1) = box_bounds(box_);
    if is_horizontal(a, b) {
        let y = snap_coord(a.y);
        let x_lo = a.x.min(b.x);
        let x_hi = a.x.max(b.x);
        return y > y0 && y < y1 && x_hi > x0 && x_lo < x1;
    }
    if is_vertical(a, b) {
        let x = snap_coord(a.x);
        let y_lo = a.y.min(b.y);
        let y_hi = a.y.max(b.y);
        return x > x0 && x < x1 && y_hi > y0 && y_lo < y1;
    }
    false
}

fn detour_horizontal(a: FlowXY, b: FlowXY, box_: &WireObstacleBox) -> Vec<FlowXY> {
    let (_, y0, _, y1) = box_bounds(box_);
    let bypass_top = snap_coord(y0);
    let bypass_bottom = snap_coord(y1);
    let dist_top = (bypass_top - a.y).abs() + (bypass_top - b.y).abs();
    let dist_bottom = (bypass_bottom - a.y).abs() + (bypass_bottom - b.y).abs();
    let bypass_y = if dist_top <= dist_bottom {
        bypass_top
    } else {
        bypass_bottom
    };
    let knee_a = snap_point(FlowXY {
        x: a.x,
        y: bypass_y,
    });
    let knee_b = snap_point(FlowXY {
        x: b.x,
        y: bypass_y,
    });
    let mut pts = vec![a];
    if !same_snapped(a, knee_a) {
        pts.push(knee_a);
    }
    if !same_snapped(knee_a, knee_b) && !same_snapped(a, knee_b) {
        pts.push(knee_b);
    }
    if pts.last().is_none_or(|last| !same_snapped(*last, b)) {
        pts.push(b);
    }
    pts
}

fn detour_vertical(a: FlowXY, b: FlowXY, box_: &WireObstacleBox) -> Vec<FlowXY> {
    let (x0, _, x1, _) = box_bounds(box_);
    let bypass_left = snap_coord(x0);
    let bypass_right = snap_coord(x1);
    let dist_left = (bypass_left - a.x).abs() + (bypass_left - b.x).abs();
    let dist_right = (bypass_right - a.x).abs() + (bypass_right - b.x).abs();
    let bypass_x = if dist_left <= dist_right {
        bypass_left
    } else {
        bypass_right
    };
    let knee_a = snap_point(FlowXY {
        x: bypass_x,
        y: a.y,
    });
    let knee_b = snap_point(FlowXY {
        x: bypass_x,
        y: b.y,
    });
    let mut pts = vec![a];
    if !same_snapped(a, knee_a) {
        pts.push(knee_a);
    }
    if !same_snapped(knee_a, knee_b) && !same_snapped(a, knee_b) {
        pts.push(knee_b);
    }
    if pts.last().is_none_or(|last| !same_snapped(*last, b)) {
        pts.push(b);
    }
    pts
}

fn minimal_detour_for_segment(a: FlowXY, b: FlowXY, box_: &WireObstacleBox) -> Vec<FlowXY> {
    if !segment_intersects_box(a, b, box_) {
        return vec![a, b];
    }
    if is_horizontal(a, b) {
        return detour_horizontal(a, b, box_);
    }
    if is_vertical(a, b) {
        return detour_vertical(a, b, box_);
    }
    vec![a, b]
}

pub fn chain_needs_avoidance(chain: &[FlowXY], obstacles: &[WireObstacleBox]) -> bool {
    if obstacles.is_empty() || chain.len() < 2 {
        return false;
    }
    for w in chain.windows(2) {
        let a = w[0];
        let b = w[1];
        for ob in obstacles {
            if segment_intersects_box(a, b, ob) {
                return true;
            }
        }
    }
    false
}

/// Locally nudge intersecting segments around obstacles; preserves topology when possible.
pub fn apply_wire_obstacle_avoidance(
    chain: &[FlowXY],
    obstacles: &[WireObstacleBox],
    max_iterations: usize,
) -> Vec<FlowXY> {
    if obstacles.is_empty() || chain.len() < 2 {
        return chain.to_vec();
    }

    let mut current: Vec<FlowXY> = chain.iter().copied().map(snap_point).collect();

    for _ in 0..max_iterations {
        let mut changed = false;
        'outer: for i in 0..current.len().saturating_sub(1) {
            let a = current[i];
            let b = current[i + 1];
            for ob in obstacles {
                if !segment_intersects_box(a, b, ob) {
                    continue;
                }
                let detour = minimal_detour_for_segment(a, b, ob);
                if detour.len() <= 2 && segment_intersects_box(a, b, ob) {
                    continue;
                }
                let mut next = current[..i].to_vec();
                next.extend_from_slice(&detour);
                next.extend_from_slice(&current[i + 2..]);
                current = sanitize_orthogonal_chain(&orthogonalize_chain(
                    &next.iter().copied().map(snap_point).collect::<Vec<_>>(),
                ));
                changed = true;
                break 'outer;
            }
        }
        if !changed {
            break;
        }
    }

    current
}
