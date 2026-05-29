//! Sharp orthogonal wire polylines (mirrors v6 `wireSharpPolyline.ts` + core `schematicEdgePath.ts`).

use diagramme_geometry::{
    get_analytical_port_xy, schematic_layout::SCHEMATIC_FRAME_INSET_PX, wiretag_conn_port_xy,
    wiretag_export_width_for_node, DEVICE_V2_WIDTH_PX, PATCH_PANEL_WIDTH_PX, SNAP_GRID_PX,
    WIRETAG_CONN_SRC, WIRETAG_CONN_TGT,
};
use diagramme_schema::{Edge, Node};
use std::collections::HashMap;

use crate::postprocess::postprocess_dxf_wire_records_for_revit_by_edge;
use crate::types::{
    DxfWirePolylineRecord, FlowXY, HandleSide, StubEndpoints, WireGeometryEdgeRecord,
    WireGeometryModel, WireGeometryOptions, WirePolylineResult, WirePolylineSource,
};

const DEFAULT_STUB: f64 = 2.0 * SNAP_GRID_PX;

/// Snap coordinate to schematic grid (mirrors v6 `snapCoord`).
pub fn snap_coord(v: f64) -> f64 {
    (v / SNAP_GRID_PX).round() * SNAP_GRID_PX
}

pub fn snap_point(p: FlowXY) -> FlowXY {
    FlowXY {
        x: snap_coord(p.x),
        y: snap_coord(p.y),
    }
}

fn is_finite_xy(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

fn position_to_outward_delta(position: HandleSide) -> (f64, f64) {
    match position {
        HandleSide::Left => (-1.0, 0.0),
        HandleSide::Right => (1.0, 0.0),
        HandleSide::Top => (0.0, -1.0),
        HandleSide::Bottom => (0.0, 1.0),
    }
}

fn clamp_stub_length(desired: f64, distance_between_handles: f64) -> f64 {
    if !distance_between_handles.is_finite() || distance_between_handles < 1e-6 {
        return 0.0;
    }
    desired.min(distance_between_handles * 0.49)
}

pub fn compute_stub_endpoints(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: HandleSide,
    target_position: HandleSide,
) -> Option<StubEndpoints> {
    if !is_finite_xy(source_x, source_y) || !is_finite_xy(target_x, target_y) {
        return None;
    }
    let d = ((target_x - source_x).powi(2) + (target_y - source_y).powi(2)).sqrt();
    if d < 0.5 {
        return None;
    }

    let (u_dx, u_dy) = position_to_outward_delta(source_position);
    let (v_dx, v_dy) = position_to_outward_delta(target_position);
    if (u_dx == 0.0 && u_dy == 0.0) || (v_dx == 0.0 && v_dy == 0.0) {
        return None;
    }

    let stub = clamp_stub_length(DEFAULT_STUB, d);
    if stub < 1e-3 {
        return None;
    }

    let s = FlowXY {
        x: source_x,
        y: source_y,
    };
    let t = FlowXY {
        x: target_x,
        y: target_y,
    };
    let s1 = FlowXY {
        x: source_x + u_dx * stub,
        y: source_y + u_dy * stub,
    };
    let t1 = FlowXY {
        x: target_x + v_dx * stub,
        y: target_y + v_dy * stub,
    };
    Some(StubEndpoints {
        s,
        s1,
        t1,
        t,
        stub,
    })
}

fn dedupe_consecutive_points(pts: &[FlowXY]) -> Vec<FlowXY> {
    let mut out = Vec::new();
    for p in pts {
        if out
            .last()
            .is_some_and(|last: &FlowXY| last.x == p.x && last.y == p.y)
        {
            continue;
        }
        out.push(*p);
    }
    out
}

fn same_snapped_point(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) == snap_coord(b.y)
}

fn is_horizontal(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.y) == snap_coord(b.y) && snap_coord(a.x) != snap_coord(b.x)
}

fn is_vertical(a: FlowXY, b: FlowXY) -> bool {
    snap_coord(a.x) == snap_coord(b.x) && snap_coord(a.y) != snap_coord(b.y)
}

fn is_axis_aligned_segment(a: FlowXY, b: FlowXY) -> bool {
    let sx = snap_coord(a.x) == snap_coord(b.x);
    let sy = snap_coord(a.y) == snap_coord(b.y);
    sx != sy
}

fn orthogonalize_chain(chain: &[FlowXY]) -> Vec<FlowXY> {
    if chain.len() < 2 {
        return chain.iter().copied().map(snap_point).collect();
    }
    let raw: Vec<FlowXY> = chain.iter().copied().map(snap_point).collect();
    let mut out = vec![raw[0]];
    for cur in raw.iter().skip(1) {
        let prev = *out.last().unwrap();
        let cur = *cur;
        if same_snapped_point(prev, cur) {
            continue;
        }
        if is_axis_aligned_segment(prev, cur) {
            out.push(cur);
            continue;
        }
        let knee_h = snap_point(FlowXY {
            x: cur.x,
            y: prev.y,
        });
        let knee = if same_snapped_point(prev, knee_h) {
            snap_point(FlowXY {
                x: prev.x,
                y: cur.y,
            })
        } else {
            knee_h
        };
        if !same_snapped_point(prev, knee) {
            out.push(knee);
        }
        if out.last().is_none_or(|last| !same_snapped_point(*last, cur)) {
            out.push(cur);
        }
    }
    dedupe_consecutive_points(&out)
}

fn sanitize_orthogonal_chain(chain: &[FlowXY]) -> Vec<FlowXY> {
    if chain.len() <= 2 {
        return chain.to_vec();
    }
    let mut out = vec![chain[0]];
    for i in 1..chain.len() - 1 {
        let prev = *out.last().unwrap();
        let cur = chain[i];
        let next = chain[i + 1];
        let h1 = is_horizontal(prev, cur);
        let h2 = is_horizontal(cur, next);
        let v1 = is_vertical(prev, cur);
        let v2 = is_vertical(cur, next);
        let collinear = (h1 && h2 && (prev.y - cur.y).abs() < 1e-6)
            || (v1 && v2 && (prev.x - cur.x).abs() < 1e-6);
        if collinear {
            continue;
        }
        out.push(cur);
    }
    out.push(*chain.last().unwrap());
    dedupe_consecutive_points(&out)
}

/// Default Manhattan route between stub ends (offset-0 smooth-step equivalent).
pub fn default_inner_chain_points(
    s1: FlowXY,
    t1: FlowXY,
    source_position: HandleSide,
    target_position: HandleSide,
) -> Vec<FlowXY> {
    if !is_finite_xy(s1.x, s1.y) || !is_finite_xy(t1.x, t1.y) {
        return vec![
            snap_point(s1),
            snap_point(t1),
        ];
    }

    let near = |a: FlowXY, b: FlowXY| ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt() < 1.0;

    // Direct axis-aligned segment when stubs already line up.
    if snap_coord(s1.x) == snap_coord(t1.x) || snap_coord(s1.y) == snap_coord(t1.y) {
        let pts = vec![snap_point(s1), snap_point(t1)];
        return sanitize_orthogonal_chain(&pts);
    }

    let mut pts = match (source_position, target_position) {
        (HandleSide::Left | HandleSide::Right, HandleSide::Left | HandleSide::Right) => {
            let mid_x = snap_coord((s1.x + t1.x) / 2.0);
            vec![
                snap_point(s1),
                snap_point(FlowXY { x: mid_x, y: s1.y }),
                snap_point(FlowXY { x: mid_x, y: t1.y }),
                snap_point(t1),
            ]
        }
        (HandleSide::Top | HandleSide::Bottom, HandleSide::Top | HandleSide::Bottom) => {
            let mid_y = snap_coord((s1.y + t1.y) / 2.0);
            vec![
                snap_point(s1),
                snap_point(FlowXY { x: s1.x, y: mid_y }),
                snap_point(FlowXY { x: t1.x, y: mid_y }),
                snap_point(t1),
            ]
        }
        (HandleSide::Left | HandleSide::Right, HandleSide::Top | HandleSide::Bottom) => {
            vec![
                snap_point(s1),
                snap_point(FlowXY { x: t1.x, y: s1.y }),
                snap_point(t1),
            ]
        }
        (HandleSide::Top | HandleSide::Bottom, HandleSide::Left | HandleSide::Right) => {
            vec![
                snap_point(s1),
                snap_point(FlowXY { x: s1.x, y: t1.y }),
                snap_point(t1),
            ]
        }
    };

    if !near(pts[0], s1) {
        pts.insert(0, s1);
    }
    if !near(*pts.last().unwrap(), t1) {
        pts.push(t1);
    }

    let snapped: Vec<FlowXY> = pts.iter().copied().map(snap_point).collect();
    let deduped = dedupe_consecutive_points(&snapped);
    sanitize_orthogonal_chain(&orthogonalize_chain(&deduped))
}

pub fn build_inner_chain_points(
    s1: FlowXY,
    t1: FlowXY,
    source_position: HandleSide,
    target_position: HandleSide,
    inner_corners: Option<&[FlowXY]>,
) -> Vec<FlowXY> {
    let chain = if inner_corners.is_none_or(|c| c.is_empty()) {
        default_inner_chain_points(s1, t1, source_position, target_position)
    } else {
        let corners: Vec<FlowXY> = inner_corners.unwrap().iter().copied().map(snap_point).collect();
        let mut chain = vec![snap_point(s1)];
        chain.extend(corners);
        chain.push(snap_point(t1));
        let chain = dedupe_consecutive_points(&chain);
        let chain = sanitize_orthogonal_chain(&chain);
        let chain = orthogonalize_chain(&chain);
        let chain = sanitize_orthogonal_chain(&chain);
        if chain.len() < 2 {
            default_inner_chain_points(s1, t1, source_position, target_position)
        } else {
            chain
        }
    };
    chain
}

pub fn compute_schematic_wire_polyline(
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    source_position: HandleSide,
    target_position: HandleSide,
    inner_corners: Option<&[FlowXY]>,
) -> Option<Vec<FlowXY>> {
    if !is_finite_xy(source_x, source_y) || !is_finite_xy(target_x, target_y) {
        return None;
    }
    let d = ((target_x - source_x).powi(2) + (target_y - source_y).powi(2)).sqrt();
    if d < 0.5 {
        return None;
    }

    let s_exact = FlowXY {
        x: source_x,
        y: source_y,
    };
    let t_exact = FlowXY {
        x: target_x,
        y: target_y,
    };
    let mut chain = build_inner_chain_points(
        snap_point(s_exact),
        snap_point(t_exact),
        source_position,
        target_position,
        inner_corners,
    );
    if chain.len() < 2 {
        return None;
    }
    chain[0] = s_exact;
    let last = chain.len() - 1;
    chain[last] = t_exact;
    Some(chain)
}

pub fn get_inner_corners_from_edge_data(data: &serde_json::Value) -> Option<Vec<FlowXY>> {
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

fn read_handle_center(data: &serde_json::Value, key: &str) -> Option<FlowXY> {
    let obj = data.get(key)?;
    let x = obj.get("x")?.as_f64()?;
    let y = obj.get("y")?.as_f64()?;
    if x.is_finite() && y.is_finite() {
        Some(FlowXY { x, y })
    } else {
        None
    }
}

pub fn is_schematic_wire_edge_type(edge_type: Option<&str>) -> bool {
    matches!(edge_type, Some("schematic") | Some("avoidNodes") | None)
}

/// True when the edge is a grouped bundle member (not the representative).
pub fn is_bundle_member(edge: &Edge) -> bool {
    edge.data
        .get("bundledBy")
        .and_then(|v| v.as_str())
        .is_some()
}

fn should_include_for_dxf_postprocess(edge: &Edge, is_schematic: bool) -> bool {
    if !is_schematic {
        return true;
    }
    !is_bundle_member(edge)
}

/// True when the handle id is a bundle port (`…-bundle`, `…-bundle-0`, etc.).
pub fn is_bundle_handle_id(handle_id: Option<&str>) -> bool {
    let Some(handle_id) = handle_id else {
        return false;
    };
    handle_id.contains("-bundle") || handle_id.ends_with("bundle")
}

fn uses_device_port_layout(node_type: &str) -> bool {
    node_type == "device" || node_type == "deviceV2"
}

fn is_patch_like_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "lppPatchPanel" | "dppPatchPanel" | "mlpPatchPanel" | "vpbPatchPanel" | "junction"
    )
}

fn endpoint_for_export(
    port: Option<FlowXY>,
    persisted: Option<FlowXY>,
    owner_type: &str,
) -> Option<FlowXY> {
    if let Some(p) = port {
        return Some(p);
    }
    if uses_device_port_layout(owner_type) {
        return None;
    }
    persisted
}

fn known_handle_position(node_type: &str, handle_id: Option<&str>) -> Option<HandleSide> {
    let handle_id = handle_id?;
    if handle_id == WIRETAG_CONN_SRC {
        return Some(HandleSide::Right);
    }
    if handle_id == WIRETAG_CONN_TGT {
        return Some(HandleSide::Left);
    }
    if node_type == "speakerBlock"
        && (handle_id == "T-spk" || handle_id == "S-spk-passthru")
    {
        return Some(HandleSide::Left);
    }
    if handle_id == "ant-tx" {
        return Some(HandleSide::Left);
    }
    if handle_id == "ant-rx" {
        return Some(HandleSide::Right);
    }
    if handle_id.starts_with("L-") || handle_id.starts_with("T-") {
        return Some(HandleSide::Left);
    }
    if handle_id.starts_with("R-") || handle_id.starts_with("S-") {
        return Some(HandleSide::Right);
    }
    None
}

fn infer_handle_side(dx: f64, dy: f64, from_source: bool) -> HandleSide {
    if dx.abs() >= dy.abs() {
        if from_source {
            if dx >= 0.0 {
                HandleSide::Right
            } else {
                HandleSide::Left
            }
        } else if dx >= 0.0 {
            HandleSide::Left
        } else {
            HandleSide::Right
        }
    } else if from_source {
        if dy >= 0.0 {
            HandleSide::Bottom
        } else {
            HandleSide::Top
        }
    } else if dy >= 0.0 {
        HandleSide::Top
    } else {
        HandleSide::Bottom
    }
}

fn read_node_width(node: &Node, fallback: f64) -> f64 {
    node.width.filter(|w| *w > 0.0).unwrap_or(fallback)
}

/// Row/bundle schematic ports already sit on the frame edge (or bracket tip). Clipping
/// inward then emitting an outward stub retraces node chrome and looks like extra routing.
fn preserves_analytical_wire_port(node: &Node, handle_id: Option<&str>) -> bool {
    let Some(handle_id) = handle_id else {
        return false;
    };
    if is_bundle_handle_id(Some(handle_id)) {
        return true;
    }
    match node.node_type.as_str() {
        "device" | "deviceV2" => handle_id.starts_with('L') || handle_id.starts_with('R'),
        "avPlate" => handle_id.starts_with('T') || handle_id.starts_with('S'),
        t if is_patch_like_node_type(t) => {
            handle_id.starts_with('L') || handle_id.starts_with('R')
        }
        _ => false,
    }
}

fn analytical_port_xy(
    node: &Node,
    handle_id: &str,
    nodes: &[Node],
    edges: &[Edge],
) -> Option<FlowXY> {
    if node.node_type == "wiretag" {
        wiretag_conn_port_xy(node, handle_id, nodes, edges).map(FlowXY::from)
    } else {
        get_analytical_port_xy(node, handle_id).map(FlowXY::from)
    }
}

fn clip_endpoint_to_node_boundary(
    node: &Node,
    handle_id: Option<&str>,
    point: FlowXY,
    nodes: &[Node],
    edges: &[Edge],
) -> FlowXY {
    if preserves_analytical_wire_port(node, handle_id) {
        return point;
    }

    let x = node.position.x;
    if !x.is_finite() {
        return point;
    }

    if node.node_type == "avPlate" {
        let mid = x + PATCH_PANEL_WIDTH_PX / 2.0;
        return FlowXY {
            x: if point.x <= mid {
                x + SCHEMATIC_FRAME_INSET_PX
            } else {
                x + PATCH_PANEL_WIDTH_PX - SCHEMATIC_FRAME_INSET_PX
            },
            y: point.y,
        };
    }

    if node.node_type == "device" || node.node_type == "deviceV2" {
        let w = read_node_width(node, DEVICE_V2_WIDTH_PX);
        let mid = x + w / 2.0;
        return FlowXY {
            x: if point.x <= mid {
                x + SCHEMATIC_FRAME_INSET_PX
            } else {
                x + w - SCHEMATIC_FRAME_INSET_PX
            },
            y: point.y,
        };
    }

    if node.node_type == "wiretag" {
        let w = wiretag_export_width_for_node(node, nodes, edges);
        let left_x = x;
        let right_x = x + w;
        if handle_id == Some(WIRETAG_CONN_SRC) {
            return FlowXY {
                x: right_x,
                y: point.y,
            };
        }
        if handle_id == Some(WIRETAG_CONN_TGT) {
            return FlowXY {
                x: left_x,
                y: point.y,
            };
        }
        return FlowXY {
            x: if (point.x - left_x).abs() <= (point.x - right_x).abs() {
                left_x
            } else {
                right_x
            },
            y: point.y,
        };
    }

    if is_patch_like_node_type(&node.node_type) {
        let mid = x + PATCH_PANEL_WIDTH_PX / 2.0;
        return FlowXY {
            x: if point.x <= mid {
                x + SCHEMATIC_FRAME_INSET_PX
            } else {
                x + PATCH_PANEL_WIDTH_PX - SCHEMATIC_FRAME_INSET_PX
            },
            y: point.y,
        };
    }

    point
}

pub fn clip_export_polyline_endpoints(
    edge: &Edge,
    polyline: &[FlowXY],
    nodes: &[Node],
    edges: &[Edge],
) -> Vec<FlowXY> {
    if polyline.len() < 2 {
        return polyline.to_vec();
    }
    let nodes_by_id = node_lookup_for_wire_geometry(nodes);
    let Some(src) = nodes_by_id.get(&edge.source) else {
        return polyline.to_vec();
    };
    let Some(tgt) = nodes_by_id.get(&edge.target) else {
        return polyline.to_vec();
    };
    let mut next = polyline.to_vec();
    let len = next.len();
    next[0] = clip_endpoint_to_node_boundary(
        src,
        edge.source_handle.as_deref(),
        next[0],
        nodes,
        edges,
    );
    next[len - 1] = clip_endpoint_to_node_boundary(
        tgt,
        edge.target_handle.as_deref(),
        next[len - 1],
        nodes,
        edges,
    );
    next
}

pub fn node_lookup_for_wire_geometry(nodes: &[Node]) -> HashMap<String, Node> {
    nodes.iter().map(|n| (n.id.clone(), n.clone())).collect()
}

pub fn fallback_polyline_from_ports(
    edge: &Edge,
    src: &Node,
    tgt: &Node,
    nodes: &[Node],
    edges: &[Edge],
    options: WireGeometryOptions,
) -> Option<Vec<FlowXY>> {
    let sh = edge.source_handle.as_deref();
    let th = edge.target_handle.as_deref();
    let inner_corners = if options.use_persisted_inner_corners {
        get_inner_corners_from_edge_data(&edge.data)
    } else {
        None
    };
    let sc = read_handle_center(&edge.data, "sourceHandleCenter");
    let tc = read_handle_center(&edge.data, "targetHandleCenter");

    let src_port = sh.and_then(|h| analytical_port_xy(src, h, nodes, edges));
    let tgt_port = th.and_then(|h| analytical_port_xy(tgt, h, nodes, edges));
    let source = endpoint_for_export(src_port, sc, &src.node_type)?;
    let target = endpoint_for_export(tgt_port, tc, &tgt.node_type)?;

    if !is_schematic_wire_edge_type(edge.edge_type.as_deref()) {
        return Some(vec![source, target]);
    }

    let dx = target.x - source.x;
    let dy = target.y - source.y;
    let inferred_source = infer_handle_side(dx, dy, true);
    let inferred_target = infer_handle_side(dx, dy, false);

    let source_position = sh
        .and_then(|h| known_handle_position(&src.node_type, Some(h)))
        .unwrap_or(inferred_source);
    let target_position = th
        .and_then(|h| known_handle_position(&tgt.node_type, Some(h)))
        .unwrap_or(inferred_target);

    compute_schematic_wire_polyline(
        source.x,
        source.y,
        target.x,
        target.y,
        source_position,
        target_position,
        inner_corners.as_deref(),
    )
    .or_else(|| Some(vec![source, target]))
}

pub fn wire_sharp_polyline_for_edge(
    edge: &Edge,
    nodes: &[Node],
    edges: &[Edge],
    options: WireGeometryOptions,
) -> Option<WirePolylineResult> {
    let nodes_by_id = node_lookup_for_wire_geometry(nodes);
    let src = nodes_by_id.get(&edge.source)?;
    let tgt = nodes_by_id.get(&edge.target)?;

    // Resolve endpoints from analytical ports (and persisted handle centers when ports are absent).
    let fallback = fallback_polyline_from_ports(edge, src, tgt, nodes, edges, options)?;
    if fallback.len() < 2 {
        return None;
    }

    let sc = read_handle_center(&edge.data, "sourceHandleCenter");
    let tc = read_handle_center(&edge.data, "targetHandleCenter");
    let sh = edge.source_handle.as_deref();
    let th = edge.target_handle.as_deref();
    let src_port = sh.and_then(|h| analytical_port_xy(src, h, nodes, edges));
    let tgt_port = th.and_then(|h| analytical_port_xy(tgt, h, nodes, edges));
    let used_persisted = (!src_port.is_some()
        && endpoint_for_export(None, sc, &src.node_type).is_some())
        || (!tgt_port.is_some()
            && endpoint_for_export(None, tc, &tgt.node_type).is_some());

    Some(WirePolylineResult {
        polyline: clip_export_polyline_endpoints(edge, &fallback, nodes, edges),
        source: if used_persisted {
            WirePolylineSource::FallbackPersisted
        } else {
            WirePolylineSource::FallbackAnalytical
        },
    })
}

/// Build per-edge sharp polylines and DXF-ready pieces (crossing gaps + bundle fillets).
pub fn build_wire_geometry_model(
    nodes: &[Node],
    edges: &[Edge],
    options: WireGeometryOptions,
) -> WireGeometryModel {
    let mut by_edge = HashMap::new();
    let mut postprocess_input = Vec::new();

    for edge in edges {
        let is_schematic = is_schematic_wire_edge_type(edge.edge_type.as_deref());
        let Some(result) = wire_sharp_polyline_for_edge(edge, nodes, edges, options) else {
            continue;
        };
        if result.polyline.len() < 2 {
            continue;
        }

        let is_bundle = is_bundle_handle_id(edge.source_handle.as_deref())
            || is_bundle_handle_id(edge.target_handle.as_deref());

        let record = WireGeometryEdgeRecord {
            edge_id: edge.id.clone(),
            source_node_id: edge.source.clone(),
            sharp_polyline: result.polyline.clone(),
            polyline_source: result.source,
            is_schematic,
            is_bundle,
            dxf_pieces: Vec::new(),
        };
        by_edge.insert(edge.id.clone(), record);

        if !should_include_for_dxf_postprocess(edge, is_schematic) {
            continue;
        }
        postprocess_input.push(DxfWirePolylineRecord {
            edge_id: edge.id.clone(),
            points: result.polyline,
            is_schematic,
            is_bundle,
            source_node_id: Some(edge.source.clone()),
        });
    }

    let dxf_by_edge = postprocess_dxf_wire_records_for_revit_by_edge(&postprocess_input);
    for (edge_id, dxf_pieces) in dxf_by_edge.iter() {
        if let Some(edge_record) = by_edge.get_mut(edge_id) {
            edge_record.dxf_pieces = dxf_pieces.clone();
        }
    }

    WireGeometryModel {
        dxf_pieces: dxf_by_edge.into_values().flatten().collect(),
        edges: by_edge,
    }
}
