//! Export-time diagram hygiene and bundle-handle indexing.

use std::collections::HashSet;

use crate::{DiagramState, Edge, Node, ProjectState};

/// True when the handle id is a bundle port (`…-bundle`, `…-bundle-0`, etc.).
pub fn is_bundle_handle_id(handle_id: Option<&str>) -> bool {
    let Some(handle_id) = handle_id else {
        return false;
    };
    handle_id.contains("-bundle") || handle_id.ends_with("bundle")
}

/// Device v2 bundle handle (`L-{group}-bundle-{index}` / `R-…`).
pub fn device_v2_bundle_handle_id(side: char, group_index: usize, bundle_index: usize) -> String {
    format!("{side}-{group_index}-bundle-{bundle_index}")
}

/// `(node_id, handle_id)` pairs for every bundle port referenced by a live edge.
pub fn active_bundle_handles(diagram: &DiagramState) -> HashSet<(String, String)> {
    let mut out = HashSet::new();
    for edge in &diagram.edges {
        if let Some(handle) = edge.source_handle.as_deref() {
            if is_bundle_handle_id(Some(handle)) {
                out.insert((edge.source.clone(), handle.to_string()));
            }
        }
        if let Some(handle) = edge.target_handle.as_deref() {
            if is_bundle_handle_id(Some(handle)) {
                out.insert((edge.target.clone(), handle.to_string()));
            }
        }
    }
    out
}

/// Whether a bundle bracket should be drawn for `node_id` on `side` (`L`/`R`) at `bundle_index`.
pub fn is_bundle_bracket_active(
    active: &HashSet<(String, String)>,
    node_id: &str,
    side: char,
    bundle_index: usize,
) -> bool {
    let suffix = format!("-bundle-{bundle_index}");
    active.iter().any(|(nid, handle)| {
        nid == node_id
            && is_bundle_handle_id(Some(handle))
            && handle.starts_with(side)
            && handle.ends_with(&suffix)
    })
}

/// Whether a device v2 bundle bracket slot has a live bundle edge.
pub fn is_device_v2_bundle_bracket_active(
    active: &HashSet<(String, String)>,
    node_id: &str,
    side: char,
    group_index: usize,
    bundle_index: usize,
) -> bool {
    active.contains(&(
        node_id.to_string(),
        device_v2_bundle_handle_id(side, group_index, bundle_index),
    ))
}

/// Filter patch/av `bundledLeft` / `bundledRight` arrays to bundles with live edges.
pub fn filter_bundled_side(
    bundles: Option<Vec<Vec<String>>>,
    node_id: &str,
    side: char,
    active: &HashSet<(String, String)>,
) -> Option<Vec<Vec<String>>> {
    let list = bundles?;
    let kept: Vec<Vec<String>> = list
        .into_iter()
        .enumerate()
        .filter(|(bi, bundle)| {
            !bundle.is_empty() && is_bundle_bracket_active(active, node_id, side, *bi)
        })
        .map(|(_, bundle)| bundle)
        .collect();
    if kept.is_empty() {
        None
    } else {
        Some(kept)
    }
}

fn prune_device_v2_bundles(node: &mut Node, active: &HashSet<(String, String)>) {
    let Some(data) = node.data.as_object_mut() else {
        return;
    };
    for (col_key, side) in [("leftColumn", 'L'), ("rightColumn", 'R')] {
        let Some(groups) = data.get_mut(col_key).and_then(|v| v.as_array_mut()) else {
            continue;
        };
        for (group_index, group) in groups.iter_mut().enumerate() {
            let Some(bundles) = group.get_mut("bundledRowIds") else {
                continue;
            };
            let Some(arr) = bundles.as_array_mut() else {
                continue;
            };
            if arr.is_empty() {
                continue;
            }

            let normalized: Vec<Vec<String>> = if arr[0].is_string() {
                vec![arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()]
            } else {
                arr.iter()
                    .filter_map(|bundle| {
                        bundle.as_array().map(|ids| {
                            ids.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                    })
                    .collect()
            };

            let kept: Vec<serde_json::Value> = normalized
                .into_iter()
                .enumerate()
                .filter(|(bi, bundle)| {
                    !bundle.is_empty()
                        && is_device_v2_bundle_bracket_active(
                            active,
                            &node.id,
                            side,
                            group_index,
                            *bi,
                        )
                })
                .map(|(_, bundle)| {
                    serde_json::Value::Array(bundle.into_iter().map(serde_json::Value::from).collect())
                })
                .collect();

            if kept.is_empty() {
                group.as_object_mut().map(|o| o.remove("bundledRowIds"));
            } else {
                *bundles = serde_json::Value::Array(kept);
            }
        }
    }
}

fn prune_side_bundles(
    data: &mut serde_json::Value,
    key: &str,
    node_id: &str,
    side: char,
    active: &HashSet<(String, String)>,
) {
    let Some(bundles) = data.get_mut(key) else {
        return;
    };
    let Some(arr) = bundles.as_array_mut() else {
        return;
    };
    if arr.is_empty() {
        return;
    }

    let normalized: Vec<Vec<String>> = arr
        .iter()
        .filter_map(|bundle| {
            bundle.as_array().map(|ids| {
                ids.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        })
        .collect();

    let kept: Vec<serde_json::Value> = normalized
        .into_iter()
        .enumerate()
        .filter(|(bi, bundle)| {
            !bundle.is_empty() && is_bundle_bracket_active(active, node_id, side, *bi)
        })
        .map(|(_, bundle)| {
            serde_json::Value::Array(bundle.into_iter().map(serde_json::Value::from).collect())
        })
        .collect();

    if kept.is_empty() {
        data.as_object_mut().map(|o| o.remove(key));
    } else {
        *bundles = serde_json::Value::Array(kept);
    }
}

fn prune_node_bundles(node: &mut Node, active: &HashSet<(String, String)>) {
    match node.node_type.as_str() {
        "deviceV2" | "device" => prune_device_v2_bundles(node, active),
        "avPlate" | "lppPatchPanel" | "dppPatchPanel" | "mlpPatchPanel" | "vpbPatchPanel" => {
            let id = node.id.clone();
            let mut data = node.data.clone();
            prune_side_bundles(&mut data, "bundledLeft", &id, 'L', active);
            prune_side_bundles(&mut data, "bundledRight", &id, 'R', active);
            node.data = data;
        }
        _ => {}
    }
}

fn strip_inner_corners(edge: &mut Edge) {
    if let Some(obj) = edge.data.as_object_mut() {
        obj.remove("innerCorners");
    }
}

fn strip_stale_handle_attachments(edge: &mut Edge) {
    if let Some(obj) = edge.data.as_object_mut() {
        obj.remove("sourceHandleCenter");
        obj.remove("targetHandleCenter");
    }
}

fn edge_endpoints_valid(edge: &Edge, nodes: &std::collections::HashMap<&str, &Node>) -> bool {
    nodes.contains_key(edge.source.as_str()) && nodes.contains_key(edge.target.as_str())
}

/// Prepare a diagram clone for DXF export: drop stale routing and orphan bundle metadata.
pub fn normalize_diagram_for_export(diagram: &mut DiagramState) {
    let active = active_bundle_handles(diagram);

    let node_map: std::collections::HashMap<&str, &Node> = diagram
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();

    diagram.edges.retain(|e| edge_endpoints_valid(e, &node_map));
    for edge in &mut diagram.edges {
        strip_inner_corners(edge);
    }

    for node in &mut diagram.nodes {
        prune_node_bundles(node, &active);
    }
}

/// Save-time hygiene: export normalize plus strip stale handle centers so v6/v2 re-derive ports.
pub fn normalize_diagram_for_persist(diagram: &mut DiagramState) {
    normalize_diagram_for_export(diagram);
    for edge in &mut diagram.edges {
        strip_stale_handle_attachments(edge);
    }
}

/// Apply [`normalize_diagram_for_persist`] to every sheet in a project.
pub fn normalize_project_for_persist(project: &mut ProjectState) {
    for sheet in &mut project.sheets {
        normalize_diagram_for_persist(&mut sheet.state);
    }
}
