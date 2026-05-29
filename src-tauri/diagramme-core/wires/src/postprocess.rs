//! DXF wire postprocess — crossing gaps and bundle fillet arcs (mirrors v6 `dxfWirePostprocess.ts`).

use std::collections::HashMap;

use diagramme_geometry::CONNECTOR_LINE_PITCH_PX;

use crate::crossings::{
    horizontal_runs_cross_any_vertical, polyline_to_axis_runs, polyline_to_wire_segments,
    segments_for_gapped_horizontal_run, wire_segments_for_crossing_gaps, AxisHorizontalRun,
    AxisVerticalRun,
};
use crate::sharp_polyline::snap_coord;
use crate::types::{DxfWirePolylineRecord, FlowXY, RevitDxfWirePiece, SchematicFilletCorner};

/// Bundle corner fillet radius — 1/8 inch in flow space (mirrors v6 `SCHEMATIC_BUNDLE_CORNER_RADIUS_PX`).
pub const SCHEMATIC_BUNDLE_CORNER_RADIUS_PX: f64 = CONNECTOR_LINE_PITCH_PX;

const EPS: f64 = 1e-6;

fn dist_xy(a: FlowXY, b: FlowXY) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

fn is_horizontal(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.y) == snap_coord(b.y) && snap_coord(a.x) != snap_coord(b.x)
}

fn is_vertical(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) != snap_coord(b.y)
}

/// True when path turns 90° at B (axis-aligned incoming/outgoing).
fn is_ortho90_corner(a: FlowXY, b: FlowXY, c: FlowXY) -> bool {
    let dx1 = snap_coord(b.x - a.x);
    let dy1 = snap_coord(b.y - a.y);
    let dx2 = snap_coord(c.x - b.x);
    let dy2 = snap_coord(c.y - b.y);
    if dx1 == 0.0 && dy1 == 0.0 {
        return false;
    }
    if dx2 == 0.0 && dy2 == 0.0 {
        return false;
    }
    let horiz1 = dy1 == 0.0 && dx1 != 0.0;
    let vert1 = dx1 == 0.0 && dy1 != 0.0;
    let horiz2 = dy2 == 0.0 && dx2 != 0.0;
    let vert2 = dx2 == 0.0 && dy2 != 0.0;
    (horiz1 && vert2) || (vert1 && horiz2)
}

fn push_horizontal_run(runs: &mut Vec<AxisHorizontalRun>, edge_id: &str, a: FlowXY, b: FlowXY) {
    if !is_horizontal(a, b) {
        return;
    }
    runs.push(AxisHorizontalRun {
        edge_id: edge_id.to_string(),
        y: snap_coord(a.y),
        x_min: a.x.min(b.x),
        x_max: a.x.max(b.x),
    });
}

fn push_vertical_run(runs: &mut Vec<AxisVerticalRun>, edge_id: &str, a: FlowXY, b: FlowXY) {
    if !is_vertical(a, b) {
        return;
    }
    runs.push(AxisVerticalRun {
        edge_id: edge_id.to_string(),
        x: snap_coord(a.x),
        y_min: a.y.min(b.y),
        y_max: a.y.max(b.y),
    });
}

/// Horizontal axis runs used for crossing gaps. With bundle fillets, endpoints match trimmed legs.
pub fn collect_horizontal_runs_for_crossing_gaps(
    points: &[FlowXY],
    edge_id: &str,
    fillet_radius: f64,
) -> Vec<AxisHorizontalRun> {
    let n = points.len();
    if n < 2 {
        return Vec::new();
    }
    let r = if fillet_radius.is_finite() && fillet_radius > 0.0 {
        fillet_radius
    } else {
        0.0
    };
    let mut runs = Vec::new();
    let mut pen = points[0];
    let mut i = 0;
    while i < n - 1 {
        if r > 0.0 && i + 2 < n {
            let a = points[i];
            let b = points[i + 1];
            let c = points[i + 2];
            if is_ortho90_corner(a, b, c) {
                let ab = dist_xy(a, b);
                let bc = dist_xy(b, c);
                let r_eff = r.min(ab).min(bc);
                if r_eff > 1e-4 {
                    let v_in_x = (b.x - a.x) / ab;
                    let v_in_y = (b.y - a.y) / ab;
                    let v_out_x = (c.x - b.x) / bc;
                    let v_out_y = (c.y - b.y) / bc;
                    let p_tan = FlowXY {
                        x: b.x - v_in_x * r_eff,
                        y: b.y - v_in_y * r_eff,
                    };
                    let q = FlowXY {
                        x: b.x + v_out_x * r_eff,
                        y: b.y + v_out_y * r_eff,
                    };
                    if is_horizontal(a, b) {
                        push_horizontal_run(&mut runs, edge_id, pen, p_tan);
                    }
                    pen = q;
                    i += 1;
                    continue;
                }
            }
        }
        let next = points[i + 1];
        if is_horizontal(pen, next) {
            push_horizontal_run(&mut runs, edge_id, pen, next);
        }
        pen = next;
        i += 1;
    }
    runs
}

/// Vertical axis runs for DXF / crossing gaps. With bundle fillets, endpoints match trimmed legs.
pub fn collect_vertical_runs_for_crossing_gaps(
    points: &[FlowXY],
    edge_id: &str,
    fillet_radius: f64,
) -> Vec<AxisVerticalRun> {
    let n = points.len();
    if n < 2 {
        return Vec::new();
    }
    let r = if fillet_radius.is_finite() && fillet_radius > 0.0 {
        fillet_radius
    } else {
        0.0
    };
    let mut runs = Vec::new();
    let mut pen = points[0];
    let mut i = 0;
    while i < n - 1 {
        if r > 0.0 && i + 2 < n {
            let a = points[i];
            let b = points[i + 1];
            let c = points[i + 2];
            if is_ortho90_corner(a, b, c) {
                let ab = dist_xy(a, b);
                let bc = dist_xy(b, c);
                let r_eff = r.min(ab).min(bc);
                if r_eff > 1e-4 {
                    let v_in_x = (b.x - a.x) / ab;
                    let v_in_y = (b.y - a.y) / ab;
                    let v_out_x = (c.x - b.x) / bc;
                    let v_out_y = (c.y - b.y) / bc;
                    let p_tan = FlowXY {
                        x: b.x - v_in_x * r_eff,
                        y: b.y - v_in_y * r_eff,
                    };
                    let q = FlowXY {
                        x: b.x + v_out_x * r_eff,
                        y: b.y + v_out_y * r_eff,
                    };
                    if is_vertical(pen, p_tan) {
                        push_vertical_run(&mut runs, edge_id, pen, p_tan);
                    }
                    pen = q;
                    i += 1;
                    continue;
                }
            }
        }
        let next = points[i + 1];
        if is_vertical(pen, next) {
            push_vertical_run(&mut runs, edge_id, pen, next);
        }
        pen = next;
        i += 1;
    }
    runs
}

/// Quarter-circle fillet corners on a sharp orthogonal polyline (diagram px, y-down).
pub fn collect_fillet_corner_arcs(
    points: &[FlowXY],
    fillet_radius: f64,
) -> Vec<SchematicFilletCorner> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    let r = if fillet_radius.is_finite() && fillet_radius > 0.0 {
        fillet_radius
    } else {
        0.0
    };
    if r <= 0.0 {
        return Vec::new();
    }
    let mut arcs = Vec::new();
    for i in 1..n - 1 {
        let a = points[i - 1];
        let b = points[i];
        let c = points[i + 1];
        if !is_ortho90_corner(a, b, c) {
            continue;
        }
        let ab = dist_xy(a, b);
        let bc = dist_xy(b, c);
        let r_eff = r.min(ab).min(bc);
        if r_eff <= 1e-4 {
            continue;
        }
        let v_in_x = (b.x - a.x) / ab;
        let v_in_y = (b.y - a.y) / ab;
        let v_out_x = (c.x - b.x) / bc;
        let v_out_y = (c.y - b.y) / bc;
        let p = FlowXY {
            x: b.x - v_in_x * r_eff,
            y: b.y - v_in_y * r_eff,
        };
        let q = FlowXY {
            x: b.x + v_out_x * r_eff,
            y: b.y + v_out_y * r_eff,
        };
        let sweep = if v_in_x * v_out_y - v_in_y * v_out_x >= 0.0 {
            1
        } else {
            0
        };
        arcs.push(SchematicFilletCorner {
            corner: b,
            p,
            q,
            sweep,
        });
    }
    arcs
}

fn to_step(v: f64) -> i8 {
    if v > EPS {
        1
    } else if v < -EPS {
        -1
    } else {
        0
    }
}

fn append_arc_points(
    out: &mut Vec<FlowXY>,
    center: FlowXY,
    start: FlowXY,
    turn: i8,
    steps: usize,
) {
    let a0 = (start.y - center.y).atan2(start.x - center.x);
    let sweep = if turn > 0 {
        std::f64::consts::FRAC_PI_2
    } else {
        -std::f64::consts::FRAC_PI_2
    };
    let r = dist_xy(center, start);
    for i in 1..steps {
        let a = a0 + (sweep * i as f64) / steps as f64;
        out.push(FlowXY {
            x: center.x + a.cos() * r,
            y: center.y + a.sin() * r,
        });
    }
}

fn fillet_orthogonal_polyline(points: &[FlowXY], radius: f64, steps_per_corner: usize) -> Vec<FlowXY> {
    if points.len() < 3 || radius <= 0.0 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    for i in 1..points.len() - 1 {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];
        let in_dx = to_step(curr.x - prev.x);
        let in_dy = to_step(curr.y - prev.y);
        let out_dx = to_step(next.x - curr.x);
        let out_dy = to_step(next.y - curr.y);
        let is_turn = (in_dx != 0 || in_dy != 0)
            && (out_dx != 0 || out_dy != 0)
            && (in_dx != out_dx || in_dy != out_dy);
        if !is_turn {
            out.push(curr);
            continue;
        }
        let in_len = dist_xy(prev, curr);
        let out_len = dist_xy(curr, next);
        let r = radius.min(in_len * 0.5).min(out_len * 0.5);
        if r <= EPS {
            out.push(curr);
            continue;
        }
        let p_in = FlowXY {
            x: curr.x - in_dx as f64 * r,
            y: curr.y - in_dy as f64 * r,
        };
        let p_out = FlowXY {
            x: curr.x + out_dx as f64 * r,
            y: curr.y + out_dy as f64 * r,
        };
        let center = FlowXY {
            x: curr.x - in_dx as f64 * r + out_dx as f64 * r,
            y: curr.y - in_dy as f64 * r + out_dy as f64 * r,
        };
        let turn = in_dx * out_dy - in_dy * out_dx;
        out.push(p_in);
        append_arc_points(&mut out, center, p_in, turn, steps_per_corner);
        out.push(p_out);
    }
    out.push(*points.last().unwrap());
    out
}

fn vertical_obstacles_for_record(
    records: &[DxfWirePolylineRecord],
    exclude_source_node_id: Option<&str>,
    source_handle_x: Option<f64>,
) -> Vec<AxisVerticalRun> {
    let segments: Vec<_> = records
        .iter()
        .filter(|rec| rec.is_schematic)
        .flat_map(|rec| {
            polyline_to_wire_segments(&rec.points, &rec.edge_id)
                .into_iter()
                .map(|mut s| {
                    s.source_node_id = rec.source_node_id.clone();
                    s
                })
        })
        .collect();
    wire_segments_for_crossing_gaps(&segments, exclude_source_node_id, source_handle_x)
        .into_iter()
        .filter(|s| snap_coord(s.x0) == snap_coord(s.x1))
        .map(|s| AxisVerticalRun {
            edge_id: s.edge_id,
            x: snap_coord(s.x0),
            y_min: s.y0.min(s.y1),
            y_max: s.y0.max(s.y1),
        })
        .collect()
}

pub fn postprocess_dxf_wire_polylines(records: &[DxfWirePolylineRecord]) -> Vec<Vec<FlowXY>> {
    let mut out = Vec::new();
    for r in records {
        if !r.is_schematic {
            out.push(r.points.clone());
            continue;
        }
        if r.is_bundle {
            out.push(fillet_orthogonal_polyline(
                &r.points,
                SCHEMATIC_BUNDLE_CORNER_RADIUS_PX,
                4,
            ));
            continue;
        }
        let vertical_obstacles = vertical_obstacles_for_record(
            records,
            r.source_node_id.as_deref(),
            r.points.first().map(|p| p.x),
        );
        let (runs_h, runs_v) = polyline_to_axis_runs(&r.points, &r.edge_id);
        let h_segments: Vec<_> = runs_h
            .iter()
            .flat_map(|h| segments_for_gapped_horizontal_run(h, &vertical_obstacles))
            .collect();
        let v_segments: Vec<Vec<FlowXY>> = runs_v
            .iter()
            .map(|v| {
                vec![
                    FlowXY { x: v.x, y: v.y_min },
                    FlowXY { x: v.x, y: v.y_max },
                ]
            })
            .collect();
        let mut split: Vec<Vec<FlowXY>> = v_segments
            .into_iter()
            .chain(h_segments)
            .filter(|seg| seg.len() >= 2)
            .collect();
        if !split.is_empty() {
            out.append(&mut split);
        } else {
            out.push(r.points.clone());
        }
    }
    out
}

/// Revit DXF: crossing gaps on schematic wires; bundle corners stay sharp for fillet emit.
pub fn postprocess_single_dxf_wire_record_for_revit(
    r: &DxfWirePolylineRecord,
    all_records: &[DxfWirePolylineRecord],
) -> Vec<RevitDxfWirePiece> {
    if !r.is_schematic {
        return vec![RevitDxfWirePiece::Polyline {
            points: r.points.clone(),
            is_bundle: false,
        }];
    }
    let mut out = Vec::new();
    let vertical_obstacles = vertical_obstacles_for_record(
        all_records,
        r.source_node_id.as_deref(),
        r.points.first().map(|p| p.x),
    );
    let fillet_r = if r.is_bundle {
        SCHEMATIC_BUNDLE_CORNER_RADIUS_PX
    } else {
        0.0
    };
    let h_runs = collect_horizontal_runs_for_crossing_gaps(&r.points, &r.edge_id, fillet_r);
    let needs_crossing_gaps = horizontal_runs_cross_any_vertical(&h_runs, &vertical_obstacles);
    if r.is_bundle && !needs_crossing_gaps {
        return vec![RevitDxfWirePiece::Polyline {
            points: r.points.clone(),
            is_bundle: true,
        }];
    }
    let v_runs = if fillet_r > 0.0 {
        collect_vertical_runs_for_crossing_gaps(&r.points, &r.edge_id, fillet_r)
    } else {
        polyline_to_axis_runs(&r.points, &r.edge_id).1
    };
    let h_segments: Vec<_> = h_runs
        .iter()
        .flat_map(|h| segments_for_gapped_horizontal_run(h, &vertical_obstacles))
        .collect();
    let v_segments: Vec<Vec<FlowXY>> = v_runs
        .iter()
        .map(|v| {
            vec![
                FlowXY { x: v.x, y: v.y_min },
                FlowXY { x: v.x, y: v.y_max },
            ]
        })
        .collect();
    let split: Vec<Vec<FlowXY>> = v_segments
        .into_iter()
        .chain(h_segments)
        .filter(|seg| seg.len() >= 2)
        .collect();
    if !split.is_empty() {
        for seg in split {
            out.push(RevitDxfWirePiece::Polyline {
                points: seg,
                is_bundle: false,
            });
        }
        if fillet_r > 0.0 {
            for arc in collect_fillet_corner_arcs(&r.points, fillet_r) {
                out.push(RevitDxfWirePiece::FilletArc { arc });
            }
        }
    } else {
        out.push(RevitDxfWirePiece::Polyline {
            points: r.points.clone(),
            is_bundle: r.is_bundle,
        });
    }
    out
}

pub fn postprocess_dxf_wire_records_for_revit_by_edge(
    records: &[DxfWirePolylineRecord],
) -> HashMap<String, Vec<RevitDxfWirePiece>> {
    records
        .iter()
        .map(|record| {
            (
                record.edge_id.clone(),
                postprocess_single_dxf_wire_record_for_revit(record, records),
            )
        })
        .collect()
}

pub fn postprocess_dxf_wire_records_for_revit(
    records: &[DxfWirePolylineRecord],
) -> Vec<RevitDxfWirePiece> {
    postprocess_dxf_wire_records_for_revit_by_edge(records)
        .into_values()
        .flatten()
        .collect()
}
