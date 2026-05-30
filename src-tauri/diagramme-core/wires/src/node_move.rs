//! Update edge wire geometry when a node moves (inner corners + handle centers).

use diagramme_geometry::get_analytical_port_xy;
use diagramme_schema::{DiagramState, Edge, Node, XY};

use crate::inner_corners::{
    get_inner_corners_from_edge_data, set_inner_corners_on_edge, translate_inner_corners,
};
use crate::sharp_polyline::wire_chain_stub_ends;
use crate::types::FlowXY;

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

    let node_delta = FlowXY {
        x: new_position.x - old_position.x,
        y: new_position.y - old_position.y,
    };
    let delta_s1 = if edge.source == moved_node_id {
        node_delta
    } else {
        FlowXY { x: 0.0, y: 0.0 }
    };
    let delta_t1 = if edge.target == moved_node_id {
        node_delta
    } else {
        FlowXY { x: 0.0, y: 0.0 }
    };

    let (prev_s1, prev_t1) = match wire_chain_stub_ends(edge, &old_nodes, edges) {
        Some(stubs) => stubs,
        None => (FlowXY { x: 0.0, y: 0.0 }, FlowXY { x: 0.0, y: 0.0 }),
    };

    if let Some(corners) = get_inner_corners_from_edge_data(&edge.data) {
        let translated = translate_inner_corners(
            &corners,
            delta_s1,
            delta_t1,
            Some(prev_s1),
            Some(prev_t1),
        );
        set_inner_corners_on_edge(edge, Some(translated));
    }

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
}

/// Move `node_id` to `new_position` and update wire bend/handle metadata on connected edges.
pub fn apply_node_move_geometry(
    diagram: &mut DiagramState,
    node_id: &str,
    new_position: XY,
) -> bool {
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
