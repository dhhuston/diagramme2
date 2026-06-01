//! Update edge wire geometry when a node moves (inner corners + handle centers).

use diagramme_geometry::{get_analytical_port_xy, snap_placement_half_grid};
use diagramme_schema::{DiagramState, Edge, Node, XY};

use crate::inner_corners::{
    get_inner_corners_from_edge_data, inner_corners_equal, inner_corners_for_stub_move,
    set_inner_corners_on_edge,
};
use crate::inner_segment::inner_corners_from_chain;
use crate::sharp_polyline::{
    build_inner_chain_points, is_bundle_member, is_schematic_wire_edge_type,
    wire_chain_stub_ends, wire_inner_routing_for_edge,
};
use crate::types::FlowXY;
use crate::wire_avoidance::chain_needs_avoidance;

fn nodes_with_position(nodes: &[Node], node_id: &str, position: XY) -> Vec<Node> {
    nodes
        .iter()
        .map(|n| {
            if n.id == node_id {
                let mut copy = n.clone();
                copy.position = position;
                return copy;
            }
            n.clone()
        })
        .collect()
}

fn write_handle_center(edge: &mut Edge, key: &str, center: FlowXY) {
    let mut obj = edge.data.as_object().cloned().unwrap_or_default();
    obj.insert(
        key.into(),
        serde_json::json!({ "x": center.x, "y": center.y }),
    );
    edge.data = serde_json::Value::Object(obj);
}

fn analytical_handle_center(node: &Node, handle_id: &str) -> Option<FlowXY> {
    get_analytical_port_xy(node, handle_id).map(|p| FlowXY { x: p.x, y: p.y })
}

/// Rebuild and persist simplified inner corners when stub endpoints move (v6 `SchematicEdge` layout effect).
fn sync_inner_corners_on_stub_move(
    edge: &mut Edge,
    nodes: &[Node],
    edges: &[Edge],
    prev_s1: FlowXY,
    prev_t1: FlowXY,
) {
    if is_bundle_member(edge) || !is_schematic_wire_edge_type(edge.edge_type.as_deref()) {
        return;
    }

    let Some(routing) =
        wire_inner_routing_for_edge(edge, nodes, edges, Some((prev_s1, prev_t1)))
    else {
        return;
    };

    let delta_s1 = FlowXY {
        x: routing.s1.x - prev_s1.x,
        y: routing.s1.y - prev_s1.y,
    };
    let delta_t1 = FlowXY {
        x: routing.t1.x - prev_t1.x,
        y: routing.t1.y - prev_t1.y,
    };
    if delta_s1.x.abs() < 1e-6
        && delta_s1.y.abs() < 1e-6
        && delta_t1.x.abs() < 1e-6
        && delta_t1.y.abs() < 1e-6
    {
        return;
    }

    let inner_corners = get_inner_corners_from_edge_data(&edge.data);
    let has_corners = inner_corners.is_some();

    let corner_input = inner_corners.as_ref().map(|corners| {
        inner_corners_for_stub_move(
            corners,
            delta_s1,
            delta_t1,
            prev_s1,
            prev_t1,
            routing.s1,
            routing.t1,
        )
    });

    let chain = build_inner_chain_points(
        routing.s1,
        routing.t1,
        routing.source_position,
        routing.target_position,
        corner_input.as_deref().filter(|c| !c.is_empty()),
        Some(&routing.nearby_obstacles),
    );
    let next_corners = inner_corners_from_chain(&chain);
    let corners_arg = if next_corners.is_empty() {
        None
    } else {
        Some(next_corners)
    };

    if inner_corners_equal(corners_arg.as_deref(), inner_corners.as_deref()) {
        if has_corners || routing.nearby_obstacles.is_empty() {
            return;
        }
        let plain_chain = build_inner_chain_points(
            routing.s1,
            routing.t1,
            routing.source_position,
            routing.target_position,
            None,
            None,
        );
        if !chain_needs_avoidance(&plain_chain, &routing.nearby_obstacles) {
            return;
        }
        let plain_corners = inner_corners_from_chain(&plain_chain);
        let plain_arg = if plain_corners.is_empty() {
            None
        } else {
            Some(plain_corners)
        };
        if inner_corners_equal(corners_arg.as_deref(), plain_arg.as_deref()) {
            return;
        }
    }

    set_inner_corners_on_edge(edge, corners_arg);
}

fn update_edge_for_node_move(
    edge: &mut Edge,
    nodes: &[Node],
    edges: &[Edge],
    moved_node_id: &str,
    old_position: XY,
    new_position: XY,
) {
    if edge.source != moved_node_id && edge.target != moved_node_id {
        return;
    }

    let old_nodes = nodes_with_position(nodes, moved_node_id, old_position);
    let new_nodes = nodes_with_position(nodes, moved_node_id, new_position);

    let (prev_s1, prev_t1) = match wire_chain_stub_ends(edge, &old_nodes, edges) {
        Some(stubs) => stubs,
        None => return,
    };

    if edge.source == moved_node_id {
        if let Some(handle_id) = edge.source_handle.as_deref() {
            if let Some(src) = new_nodes.iter().find(|n| n.id == edge.source) {
                if let Some(center) = analytical_handle_center(src, handle_id) {
                    write_handle_center(edge, "sourceHandleCenter", center);
                }
            }
        }
    }
    if edge.target == moved_node_id {
        if let Some(handle_id) = edge.target_handle.as_deref() {
            if let Some(tgt) = new_nodes.iter().find(|n| n.id == edge.target) {
                if let Some(center) = analytical_handle_center(tgt, handle_id) {
                    write_handle_center(edge, "targetHandleCenter", center);
                }
            }
        }
    }

    sync_inner_corners_on_stub_move(edge, &new_nodes, edges, prev_s1, prev_t1);
}

/// Move `node_id` to `new_position` and update wire bend/handle metadata on connected edges.
pub fn apply_node_move_geometry(
    diagram: &mut DiagramState,
    node_id: &str,
    new_position: XY,
) -> bool {
    let new_position = XY {
        x: snap_placement_half_grid(new_position.x),
        y: snap_placement_half_grid(new_position.y),
    };
    let Some(old_position) = diagram
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(|n| n.position)
    else {
        return false;
    };

    if old_position.x == new_position.x && old_position.y == new_position.y {
        return false;
    }

    let nodes_snapshot = diagram.nodes.clone();
    let edges_snapshot = diagram.edges.clone();
    let connected: Vec<usize> = diagram
        .edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.source == node_id || e.target == node_id)
        .map(|(i, _)| i)
        .collect();

    if let Some(node) = diagram.nodes.iter_mut().find(|n| n.id == node_id) {
        node.position = new_position;
    }

    for idx in connected {
        update_edge_for_node_move(
            &mut diagram.edges[idx],
            &nodes_snapshot,
            &edges_snapshot,
            node_id,
            old_position,
            new_position,
        );
    }

    true
}

/// Batch move with per-node old positions captured before mutation.
pub fn apply_node_moves_geometry(diagram: &mut DiagramState, moves: &[(String, XY)]) {
    for (node_id, new_position) in moves {
        apply_node_move_geometry(diagram, node_id, *new_position);
    }
}
