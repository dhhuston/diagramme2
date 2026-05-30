//! Bundle wire detection — direct bundle ports plus propagation through wiretag pairs.

use diagramme_geometry::{WIRETAG_CONN_SRC, WIRETAG_CONN_TGT};
use diagramme_schema::{is_bundle_handle_id, Edge, Node};

fn is_wiretag_conn_handle(handle_id: Option<&str>) -> bool {
    matches!(handle_id, Some(WIRETAG_CONN_SRC) | Some(WIRETAG_CONN_TGT))
}

fn node_is_wiretag(node: &Node) -> bool {
    node.node_type == "wiretag"
}

fn wiretag_pair_id(node: &Node) -> Option<&str> {
    node.data.get("pairId")?.as_str()
}

/// Wiretag node id when `edge` attaches via conn-src/conn-tgt.
fn wiretag_conn_endpoint(edge: &Edge, nodes: &[Node]) -> Option<String> {
    if is_wiretag_conn_handle(edge.source_handle.as_deref()) {
        let n = nodes.iter().find(|n| n.id == edge.source)?;
        if node_is_wiretag(n) {
            return Some(edge.source.clone());
        }
    }
    if is_wiretag_conn_handle(edge.target_handle.as_deref()) {
        let n = nodes.iter().find(|n| n.id == edge.target)?;
        if node_is_wiretag(n) {
            return Some(edge.target.clone());
        }
    }
    None
}

/// True when `edge` connects `wiretag_id` to non-wiretag equipment via a conn handle.
fn edge_connects_wiretag_to_equipment(edge: &Edge, wiretag_id: &str, nodes: &[Node]) -> bool {
    let other_id = if edge.source == wiretag_id && is_wiretag_conn_handle(edge.source_handle.as_deref())
    {
        &edge.target
    } else if edge.target == wiretag_id && is_wiretag_conn_handle(edge.target_handle.as_deref()) {
        &edge.source
    } else {
        return false;
    };
    if other_id == wiretag_id {
        return false;
    }
    nodes
        .iter()
        .find(|n| n.id == *other_id)
        .is_none_or(|n| !node_is_wiretag(n))
}

/// Any equipment attachment on either tag in the pair uses a bundle port.
fn wiretag_pair_has_bundle_equipment_attachment(
    pair_id: &str,
    nodes: &[Node],
    edges: &[Edge],
) -> bool {
    let wiretag_ids: Vec<&str> = nodes
        .iter()
        .filter(|n| node_is_wiretag(n) && wiretag_pair_id(n) == Some(pair_id))
        .map(|n| n.id.as_str())
        .collect();

    for wt_id in wiretag_ids {
        for edge in edges {
            if !edge_connects_wiretag_to_equipment(edge, wt_id, nodes) {
                continue;
            }
            if is_bundle_handle_id(edge.source_handle.as_deref())
                || is_bundle_handle_id(edge.target_handle.as_deref())
            {
                return true;
            }
        }
    }
    false
}

/// True when the edge should render/export as a bundle wire (rounded corners, bundle fillets).
pub fn is_bundle_wire_edge(edge: &Edge, nodes: &[Node], edges: &[Edge]) -> bool {
    if is_bundle_handle_id(edge.source_handle.as_deref())
        || is_bundle_handle_id(edge.target_handle.as_deref())
    {
        return true;
    }
    let Some(wt_id) = wiretag_conn_endpoint(edge, nodes) else {
        return false;
    };
    let Some(wt) = nodes.iter().find(|n| n.id == wt_id) else {
        return false;
    };
    let Some(pair_id) = wiretag_pair_id(wt) else {
        return false;
    };
    wiretag_pair_has_bundle_equipment_attachment(pair_id, nodes, edges)
}
