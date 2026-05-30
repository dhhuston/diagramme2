//! Persisted wire bend translation (mirrors v6 `translateInnerCorners`).

use crate::sharp_polyline::{snap_coord, snap_point};
use crate::types::FlowXY;
use diagramme_schema::Edge;
use serde_json::Value;

pub fn get_inner_corners_from_edge_data(data: &Value) -> Option<Vec<FlowXY>> {
    let raw = data.get("innerCorners")?.as_array()?;
    if raw.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for p in raw {
        let x = p.get("x")?.as_f64()?;
        let y = p.get("y")?.as_f64()?;
        if x.is_finite() && y.is_finite() {
            out.push(FlowXY { x, y });
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

pub fn set_inner_corners_on_edge(edge: &mut Edge, corners: Option<Vec<FlowXY>>) {
    let mut obj = edge.data.as_object().cloned().unwrap_or_default();
    match corners {
        Some(c) if !c.is_empty() => {
            let json_corners: Vec<Value> = c
                .into_iter()
                .map(|p| serde_json::json!({ "x": p.x, "y": p.y }))
                .collect();
            obj.insert("innerCorners".into(), Value::Array(json_corners));
        }
        _ => {
            obj.remove("innerCorners");
        }
    }
    edge.data = Value::Object(obj);
}

fn stub_motion_param_for_corner(corner: FlowXY, prev_s1: FlowXY, prev_t1: FlowXY) -> f64 {
    let dx = prev_t1.x - prev_s1.x;
    let dy = prev_t1.y - prev_s1.y;
    if dx.abs() >= dy.abs() {
        if dx.abs() < 1e-6 {
            return 0.5;
        }
        return ((corner.x - prev_s1.x) / dx).clamp(0.0, 1.0);
    }
    if dy.abs() < 1e-6 {
        return 0.5;
    }
    ((corner.y - prev_s1.y) / dy).clamp(0.0, 1.0)
}

fn combine_stub_motion_for_corners(delta_s1: FlowXY, delta_t1: FlowXY) -> FlowXY {
    let same_motion =
        (delta_s1.x - delta_t1.x).abs() < 1e-3 && (delta_s1.y - delta_t1.y).abs() < 1e-3;
    if same_motion {
        return delta_s1;
    }
    FlowXY {
        x: (delta_s1.x + delta_t1.x) / 2.0,
        y: (delta_s1.y + delta_t1.y) / 2.0,
    }
}

/// Translate interior corners when stub endpoints move (preserve edited shape).
pub fn translate_inner_corners(
    corners: &[FlowXY],
    delta_s1: FlowXY,
    delta_t1: FlowXY,
    prev_s1: Option<FlowXY>,
    prev_t1: Option<FlowXY>,
) -> Vec<FlowXY> {
    if corners.is_empty() {
        return Vec::new();
    }

    let same_motion =
        (delta_s1.x - delta_t1.x).abs() < 1e-3 && (delta_s1.y - delta_t1.y).abs() < 1e-3;
    if same_motion {
        let d = delta_s1;
        return corners
            .iter()
            .map(|c| snap_point(FlowXY { x: c.x + d.x, y: c.y + d.y }))
            .collect();
    }

    let (Some(prev_s1), Some(prev_t1)) = (prev_s1, prev_t1) else {
        let d = combine_stub_motion_for_corners(delta_s1, delta_t1);
        return corners
            .iter()
            .map(|c| snap_point(FlowXY { x: c.x + d.x, y: c.y + d.y }))
            .collect();
    };

    corners
        .iter()
        .map(|c| {
            let t = stub_motion_param_for_corner(*c, prev_s1, prev_t1);
            snap_point(FlowXY {
                x: c.x + delta_s1.x * (1.0 - t) + delta_t1.x * t,
                y: c.y + delta_s1.y * (1.0 - t) + delta_t1.y * t,
            })
        })
        .collect()
}

pub fn inner_corners_equal(a: Option<&[FlowXY]>, b: Option<&[FlowXY]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) if a.len() == b.len() => a.iter().zip(b.iter()).all(|(p, q)| {
            snap_coord(p.x) == snap_coord(q.x) && snap_coord(p.y) == snap_coord(q.y)
        }),
        _ => false,
    }
}
