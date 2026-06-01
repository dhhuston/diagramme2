//! Wire polylines in the scene — geometry from `WireGeometryModel`, colors from v6 wire categories.

use diagramme_geometry::{PointPx, RectPx};
use diagramme_schema::{DiagramState, Edge, Node};
use diagramme_wires::{
    build_wire_geometry_model, node_lookup_for_wire_geometry, FlowXY, RevitDxfWirePiece,
    SchematicFilletCorner, WireGeometryModel, WireGeometryOptions,
};

use crate::scene::{HitTarget, Scene, ScenePrimitive};

const WIRE_STROKE_PX: f64 = 1.0;
/// Pick band around wire centerline (not full routing corridor).
const WIRE_PICK_PX: f64 = 4.0;
const FILLET_ARC_STEPS: usize = 8;

/// Wire signal categories (mirrors v6 `WireCategory` / `SchematicEdge` `CATEGORY_COLOR`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireCategory {
    Audio,
    Video,
    Control,
    Network,
    Rf,
    Power,
    Default,
    Mismatch,
}

impl WireCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Control => "control",
            Self::Network => "network",
            Self::Rf => "rf",
            Self::Power => "power",
            Self::Default => "default",
            Self::Mismatch => "mismatch",
        }
    }

    /// sRGB `0xRRGGBB` from v6 `index.css` `--clr-wire-*` (oklch; see `SchematicEdge.tsx`).
    pub fn color_rgb(self) -> u32 {
        match self {
            Self::Audio => 0xAE3700,
            Self::Video => 0x0057C0,
            Self::Control => 0x006F00,
            Self::Network => 0x6F20BD,
            Self::Rf => 0xB30300,
            Self::Power => 0x9B0096,
            Self::Default => 0x494F4B,
            Self::Mismatch => 0xBA0000,
        }
    }
}

fn parse_wire_category(value: &serde_json::Value) -> Option<WireCategory> {
    let s = value.as_str()?.trim().to_lowercase();
    match s.as_str() {
        "audio" => Some(WireCategory::Audio),
        "video" => Some(WireCategory::Video),
        "control" => Some(WireCategory::Control),
        "network" => Some(WireCategory::Network),
        "rf" => Some(WireCategory::Rf),
        "power" => Some(WireCategory::Power),
        "default" | "" => Some(WireCategory::Default),
        _ => None,
    }
}

fn comparable_category_from_stored(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        if s.trim() == "-" {
            return Some("-".to_string());
        }
    }
    parse_wire_category(value).map(|c| c.as_str().to_string())
}

fn device_v2_handle_parts(handle_id: &str) -> Option<(&str, usize, &str)> {
    let (side, rest) = if let Some(r) = handle_id.strip_prefix('L') {
        ("L", r)
    } else if let Some(r) = handle_id.strip_prefix('R') {
        ("R", r)
    } else {
        return None;
    };
    let rest = rest.strip_prefix('-')?;
    let (group_s, tail) = rest.split_once('-')?;
    let group_index: usize = group_s.parse().ok()?;
    Some((side, group_index, tail))
}

fn device_v2_row_category(node: &Node, handle_id: &str) -> Option<WireCategory> {
    let (side, group_index, row_id) = device_v2_handle_parts(handle_id)?;
    if row_id == "bundle" {
        return None;
    }
    let col = if side == "L" { "leftColumn" } else { "rightColumn" };
    let group = node.data.get(col)?.get(group_index)?;
    let rows = group.get("rows")?.as_array()?;
    for row in rows {
        if row.get("id")?.as_str() == Some(row_id) {
            return row
                .get("wireCategory")
                .and_then(parse_wire_category)
                .or(Some(WireCategory::Default));
        }
    }
    None
}

fn device_v2_bundle_category(node: &Node, handle_id: &str) -> Option<WireCategory> {
    let (side, group_index, tail) = device_v2_handle_parts(handle_id)?;
    let tail = tail.strip_prefix("bundle")?;
    let bundle_index = if tail.is_empty() {
        0usize
    } else {
        tail.strip_prefix('-')?.parse().ok()?
    };
    let col = if side == "L" { "leftColumn" } else { "rightColumn" };
    let group = node.data.get(col)?.get(group_index)?;
    let bundled = group.get("bundledRowIds")?.as_array()?;
    if bundled.is_empty() || bundle_index >= bundled.len() {
        return None;
    }
    let row_ids = bundled[bundle_index].as_array()?;
    let rows = group.get("rows")?.as_array()?;
    let mut cats = std::collections::HashSet::new();
    for row_id in row_ids {
        let id = row_id.as_str()?;
        for row in rows {
            if row.get("id")?.as_str() == Some(id) {
                cats.insert(
                    row.get("wireCategory")
                        .and_then(parse_wire_category)
                        .unwrap_or(WireCategory::Default),
                );
            }
        }
    }
    if cats.len() == 1 {
        cats.into_iter().next()
    } else {
        None
    }
}

fn device_v2_row_comparable(node: &Node, handle_id: &str) -> Option<String> {
    let (_, group_index, row_id) = device_v2_handle_parts(handle_id)?;
    if row_id == "bundle" {
        return None;
    }
    let side = if handle_id.starts_with('L') { "L" } else { "R" };
    let col = if side == "L" { "leftColumn" } else { "rightColumn" };
    let group = node.data.get(col)?.get(group_index)?;
    let rows = group.get("rows")?.as_array()?;
    for row in rows {
        if row.get("id")?.as_str() == Some(row_id) {
            return row
                .get("wireCategory")
                .and_then(comparable_category_from_stored);
        }
    }
    None
}

fn av_plate_handle_row(handle_id: &str) -> Option<&str> {
    let rest = handle_id
        .strip_prefix('T')
        .or_else(|| handle_id.strip_prefix('S'))?;
    let rest = rest.strip_prefix('-')?;
    let (_gi, tail) = rest.split_once('-')?;
    Some(tail)
}

fn av_plate_row_category(node: &Node, handle_id: &str) -> Option<WireCategory> {
    let row_id = av_plate_handle_row(handle_id)?;
    let groups = node.data.get("groups")?.as_array()?;
    for group in groups {
        let rows = group.get("rows")?.as_array()?;
        for row in rows {
            if row.get("id")?.as_str() == Some(row_id) {
                return row
                    .get("wireCategory")
                    .and_then(parse_wire_category)
                    .or(Some(WireCategory::Default));
            }
        }
    }
    None
}

fn av_plate_row_comparable(node: &Node, handle_id: &str) -> Option<String> {
    let row_id = av_plate_handle_row(handle_id)?;
    let groups = node.data.get("groups")?.as_array()?;
    for group in groups {
        let rows = group.get("rows")?.as_array()?;
        for row in rows {
            if row.get("id")?.as_str() == Some(row_id) {
                return row
                    .get("wireCategory")
                    .and_then(comparable_category_from_stored);
            }
        }
    }
    None
}

fn resolve_port_category(node: &Node, handle_id: Option<&str>) -> Option<WireCategory> {
    let handle_id = handle_id?;
    match node.node_type.as_str() {
        "deviceV2" | "device" => device_v2_bundle_category(node, handle_id)
            .or_else(|| device_v2_row_category(node, handle_id)),
        "avPlate" => av_plate_row_category(node, handle_id),
        "lppPatchPanel" | "mlpPatchPanel" => Some(WireCategory::Audio),
        "dppPatchPanel" => Some(WireCategory::Network),
        "vpbPatchPanel" => Some(WireCategory::Video),
        "micBlock" | "speakerBlock" | "volumeControl" => node
            .data
            .get("wireCategory")
            .and_then(parse_wire_category)
            .or(Some(WireCategory::Audio)),
        "flyoffNote" => node
            .data
            .get("wireCategory")
            .and_then(parse_wire_category)
            .or(Some(WireCategory::Default)),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => Some(WireCategory::Rf),
        _ => None,
    }
}

fn resolve_port_comparable(node: &Node, handle_id: Option<&str>) -> Option<String> {
    let handle_id = handle_id?;
    match node.node_type.as_str() {
        "deviceV2" | "device" => device_v2_row_comparable(node, handle_id),
        "avPlate" => av_plate_row_comparable(node, handle_id),
        "lppPatchPanel" | "mlpPatchPanel" => Some("audio".to_string()),
        "dppPatchPanel" => Some("network".to_string()),
        "vpbPatchPanel" => Some("video".to_string()),
        "micBlock" | "speakerBlock" | "volumeControl" => Some(
            node.data
                .get("wireCategory")
                .and_then(parse_wire_category)
                .map(|c| c.as_str().to_string())
                .unwrap_or_else(|| "audio".to_string()),
        ),
        "flyoffNote" => node
            .data
            .get("wireCategory")
            .and_then(comparable_category_from_stored),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => Some("rf".to_string()),
        _ => None,
    }
}

fn wire_style_for_edge(edge: &Edge, nodes: &std::collections::HashMap<String, Node>) -> (String, u32) {
    let src_node = nodes.get(&edge.source);
    let tgt_node = nodes.get(&edge.target);
    let src_cat = src_node.and_then(|n| resolve_port_category(n, edge.source_handle.as_deref()));
    let tgt_cat = tgt_node.and_then(|n| resolve_port_category(n, edge.target_handle.as_deref()));
    let src_cmp = src_node.and_then(|n| resolve_port_comparable(n, edge.source_handle.as_deref()));
    let tgt_cmp = tgt_node.and_then(|n| resolve_port_comparable(n, edge.target_handle.as_deref()));
    let resolved_mismatch = src_cmp.is_some() && tgt_cmp.is_some() && src_cmp != tgt_cmp;
    let mismatch = resolved_mismatch
        || edge
            .data
            .get("wireCategoryMismatch")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    let category = src_cat
        .or(tgt_cat)
        .or_else(|| edge.data.get("wireCategory").and_then(parse_wire_category))
        .unwrap_or(WireCategory::Default);
    let display = if mismatch {
        WireCategory::Mismatch
    } else {
        category
    };
    (display.as_str().to_string(), display.color_rgb())
}

fn flow_to_px(p: FlowXY) -> PointPx {
    PointPx { x: p.x, y: p.y }
}

fn dist_xy(a: FlowXY, b: FlowXY) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

fn tessellate_fillet_arc(arc: &SchematicFilletCorner, steps: usize) -> Vec<PointPx> {
    let r = dist_xy(arc.corner, arc.p);
    if r <= 1e-6 {
        return vec![flow_to_px(arc.p), flow_to_px(arc.q)];
    }
    let v_in_x = (arc.corner.x - arc.p.x) / r;
    let v_in_y = (arc.corner.y - arc.p.y) / r;
    let v_out_x = (arc.q.x - arc.corner.x) / r;
    let v_out_y = (arc.q.y - arc.corner.y) / r;
    let center = FlowXY {
        x: arc.corner.x - v_in_x * r + v_out_x * r,
        y: arc.corner.y - v_in_y * r + v_out_y * r,
    };
    let turn: i8 = if arc.sweep == 1 { 1 } else { -1 };
    let a0 = (arc.p.y - center.y).atan2(arc.p.x - center.x);
    let sweep = if turn > 0 {
        std::f64::consts::FRAC_PI_2
    } else {
        -std::f64::consts::FRAC_PI_2
    };
    let mut out = vec![flow_to_px(arc.p)];
    for i in 1..steps {
        let a = a0 + (sweep * i as f64) / steps as f64;
        out.push(PointPx {
            x: center.x + a.cos() * r,
            y: center.y + a.sin() * r,
        });
    }
    out.push(flow_to_px(arc.q));
    out
}

fn push_wire_segment_hits(scene: &mut Scene, points: &[PointPx], edge_id: &str) {
    let p = WIRE_PICK_PX;
    for i in 0..points.len().saturating_sub(1) {
        let p0 = &points[i];
        let p1 = &points[i + 1];
        let min_x = p0.x.min(p1.x) - p;
        let min_y = p0.y.min(p1.y) - p;
        let max_x = p0.x.max(p1.x) + p;
        let max_y = p0.y.max(p1.y) + p;
        scene.hits.push(HitTarget {
            id: format!("{edge_id}:seg:{i}"),
            bounds: RectPx::new(min_x, min_y, (max_x - min_x).max(p), (max_y - min_y).max(p)),
            node_id: None,
            edge_id: Some(edge_id.to_string()),
            handle_id: None,
            face_mask_bounds: None,
        });
    }
}

fn push_wire_polyline(
    scene: &mut Scene,
    points: Vec<PointPx>,
    layer: &str,
    color: u32,
    edge_id: &str,
) {
    if points.len() < 2 {
        return;
    }
    push_wire_segment_hits(scene, &points, edge_id);
    scene.primitives.push(ScenePrimitive::Polyline {
        points,
        stroke_px: WIRE_STROKE_PX,
        layer: layer.to_string(),
        color,
        closed: false,
        edge_id: Some(edge_id.to_string()),
        owner_node_id: None,
    });
}

fn polylines_from_dxf_pieces(pieces: &[RevitDxfWirePiece]) -> Vec<Vec<PointPx>> {
    let mut out = Vec::new();
    for piece in pieces {
        match piece {
            RevitDxfWirePiece::Polyline { points, .. } if points.len() >= 2 => {
                out.push(points.iter().map(|&p| flow_to_px(p)).collect());
            }
            RevitDxfWirePiece::FilletArc { arc } => {
                out.push(tessellate_fillet_arc(arc, FILLET_ARC_STEPS));
            }
            _ => {}
        }
    }
    out
}

/// Append wire `ScenePrimitive::Polyline` entries from a wire geometry model.
pub fn append_wires_to_scene(scene: &mut Scene, model: &WireGeometryModel, diagram: &DiagramState) {
    let node_lookup = node_lookup_for_wire_geometry(&diagram.nodes);
    for edge in &diagram.edges {
        let Some(record) = model.edges.get(&edge.id) else {
            continue;
        };
        let (layer, color) = wire_style_for_edge(edge, &node_lookup);
        let polylines = if !record.dxf_pieces.is_empty() {
            polylines_from_dxf_pieces(&record.dxf_pieces)
        } else if record.sharp_polyline.len() >= 2 {
            vec![record
                .sharp_polyline
                .iter()
                .map(|&p| flow_to_px(p))
                .collect()]
        } else {
            Vec::new()
        };
        for points in polylines {
            push_wire_polyline(scene, points, &layer, color, &edge.id);
        }
    }
}

/// Bounding rect covering wire polylines (primitives with `edge_id` set).
pub fn wire_extent_rect(scene: &Scene) -> Option<RectPx> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut any = false;
    for prim in &scene.primitives {
        let ScenePrimitive::Polyline {
            points,
            edge_id: Some(_),
            ..
        } = prim
        else {
            continue;
        };
        for p in points {
            any = true;
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    if !any {
        return None;
    }
    Some(RectPx::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

/// Build wire geometry and append to scene (used by `build_scene`).
pub fn build_and_append_wires(
    scene: &mut Scene,
    diagram: &DiagramState,
    options: WireGeometryOptions,
) {
    let model = build_wire_geometry_model(&diagram.nodes, &diagram.edges, options);
    append_wires_to_scene(scene, &model, diagram);
}

/// Append wire polylines for a subset of edges (drag preview patches).
pub fn append_wires_for_edges(
    scene: &mut Scene,
    diagram: &DiagramState,
    edge_ids: &[String],
    options: WireGeometryOptions,
) {
    if edge_ids.is_empty() {
        return;
    }
    let edge_set: std::collections::HashSet<&str> =
        edge_ids.iter().map(String::as_str).collect();
    let model = build_wire_geometry_model(&diagram.nodes, &diagram.edges, options);
    let node_lookup = node_lookup_for_wire_geometry(&diagram.nodes);
    for edge in &diagram.edges {
        if !edge_set.contains(edge.id.as_str()) {
            continue;
        }
        let Some(record) = model.edges.get(&edge.id) else {
            continue;
        };
        let (layer, color) = wire_style_for_edge(edge, &node_lookup);
        let polylines = if !record.dxf_pieces.is_empty() {
            polylines_from_dxf_pieces(&record.dxf_pieces)
        } else if record.sharp_polyline.len() >= 2 {
            vec![record
                .sharp_polyline
                .iter()
                .map(|&p| flow_to_px(p))
                .collect()]
        } else {
            Vec::new()
        };
        for points in polylines {
            push_wire_polyline(scene, points, &layer, color, &edge.id);
        }
    }
}

/// Wire category string for a new schematic connection.
pub fn resolve_port_category_for_connect(node: &Node, handle_id: Option<&str>) -> Option<String> {
    resolve_port_category(node, handle_id).map(|c| c.as_str().to_string())
}

/// Comparable category token for mismatch detection on new connections.
pub fn resolve_port_comparable_for_connect(node: &Node, handle_id: Option<&str>) -> Option<String> {
    resolve_port_comparable(node, handle_id)
}
