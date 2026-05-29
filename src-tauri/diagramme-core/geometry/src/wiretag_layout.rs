//! Wiretag autosize and connection ports (mirrors v6 wiretag graph + export width).

use diagramme_schema::{Edge, Node};

use crate::paper_scale::WIRETAG_BAR_HEIGHT_PX;
use crate::port_geometry::{WIRETAG_CONN_SRC, WIRETAG_CONN_TGT};
use crate::text_measure::wiretag_export_width_px;
use crate::types::PointPx;

fn is_conn_handle(id: Option<&str>) -> bool {
    matches!(id, Some(WIRETAG_CONN_SRC) | Some(WIRETAG_CONN_TGT))
}

fn read_pair_id(data: &serde_json::Value) -> Option<&str> {
    data.get("pairId").and_then(|v| v.as_str())
}

fn read_pair_index(data: &serde_json::Value) -> i64 {
    data
        .get("pairIndex")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .unwrap_or(0)
}

fn wiretag_bar_height(node: &Node) -> f64 {
    node.height
        .filter(|h| *h > 0.0)
        .unwrap_or(WIRETAG_BAR_HEIGHT_PX)
}

/// Neighbor node id connected via wiretag connection handles (not the partner wiretag).
fn find_wiretag_attached_node_id(wiretag_node_id: &str, edges: &[Edge]) -> Option<String> {
    for edge in edges {
        if edge.source == wiretag_node_id && is_conn_handle(edge.source_handle.as_deref()) {
            if edge.target != wiretag_node_id {
                return Some(edge.target.clone());
            }
        }
        if edge.target == wiretag_node_id && is_conn_handle(edge.target_handle.as_deref()) {
            if edge.source != wiretag_node_id {
                return Some(edge.source.clone());
            }
        }
    }
    None
}

fn find_wiretag_partner_node<'a>(
    self_id: &str,
    pair_id: &str,
    nodes: &'a [Node],
) -> Option<&'a Node> {
    nodes.iter().find(|n| {
        n.node_type == "wiretag"
            && n.id != self_id
            && n.data
                .get("pairId")
                .and_then(|v| v.as_str())
                .is_some_and(|pid| pid == pair_id)
    })
}

/// Display tag for equipment nodes (`tagCode / tagNumber` style).
pub fn get_device_tag_label(node: &Node) -> String {
    let d = &node.data;
    let code = d
        .get("tagCode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let num = d
        .get("tagNumber")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if !code.is_empty() || !num.is_empty() {
        if !code.is_empty() && !num.is_empty() {
            return format!("{code} / {num}");
        }
        return format!("{code}{num}").trim().to_string();
    }
    match node.node_type.as_str() {
        "micBlock" | "speakerBlock" => {
            let line1 = d
                .get("line1")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if line1.is_empty() {
                node.id.clone()
            } else {
                line1.to_string()
            }
        }
        "volumeControl" => "VC".to_string(),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => {
            let line1 = d
                .get("line1")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if line1.is_empty() {
                "ANT".to_string()
            } else {
                line1.to_string()
            }
        }
        _ => node.id.clone(),
    }
}

fn resolve_remote_tag_for_wiretag(node: &Node, nodes: &[Node], edges: &[Edge]) -> String {
    if node.node_type != "wiretag" {
        return String::new();
    }
    let Some(pair_id) = read_pair_id(&node.data) else {
        return String::new();
    };
    let Some(partner) = find_wiretag_partner_node(&node.id, pair_id, nodes) else {
        return String::new();
    };
    let Some(attach_id) = find_wiretag_attached_node_id(&partner.id, edges) else {
        return String::new();
    };
    nodes
        .iter()
        .find(|n| n.id == attach_id)
        .map(get_device_tag_label)
        .unwrap_or_default()
}

/// Main band text: explicit tag description, else the remote end's device label.
pub fn resolve_pair_main_display_text(node: &Node, nodes: &[Node], edges: &[Edge]) -> String {
    if node.node_type != "wiretag" {
        return String::new();
    }
    let desc = node
        .data
        .get("tagDescription")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if !desc.is_empty() {
        return desc.to_string();
    }
    resolve_remote_tag_for_wiretag(node, nodes, edges)
}

/// Export/autosize hull width for a wiretag (same formula as scene + DXF hull).
pub fn wiretag_export_width_for_node(node: &Node, nodes: &[Node], edges: &[Edge]) -> f64 {
    let bar_h = wiretag_bar_height(node);
    let data = &node.data;
    let pair_index = read_pair_index(data);
    let main = resolve_pair_main_display_text(node, nodes, edges);
    let show_sheet = data
        .get("showSheetName")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sheet = if show_sheet {
        data.get("sheetName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else {
        ""
    };
    wiretag_export_width_px(pair_index, &main, sheet, show_sheet, bar_h)
}

/// Connection handle center using export hull width (matches v6 live canvas autosize).
pub fn wiretag_conn_port_xy(
    node: &Node,
    handle_id: &str,
    nodes: &[Node],
    edges: &[Edge],
) -> Option<PointPx> {
    if node.node_type != "wiretag" {
        return None;
    }
    let w = wiretag_export_width_for_node(node, nodes, edges);
    let abs_x = node.position.x;
    let abs_y = node.position.y;
    let y = abs_y + wiretag_bar_height(node) / 2.0;
    match handle_id {
        WIRETAG_CONN_SRC => Some(PointPx {
            x: abs_x + w - 1.0,
            y,
        }),
        WIRETAG_CONN_TGT => Some(PointPx {
            x: abs_x + 1.0,
            y,
        }),
        _ => None,
    }
}
