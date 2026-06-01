//! Schematic edge creation for port connect (Task 19c).

use diagramme_geometry::get_analytical_port_xy;
use diagramme_schema::{DiagramState, Edge};

use crate::wires::{resolve_port_category_for_connect, resolve_port_comparable_for_connect};

/// Build a new schematic edge with handle centers and wire category metadata.
pub fn build_schematic_edge(
    diagram: &DiagramState,
    source: &str,
    target: &str,
    source_handle: Option<&str>,
    target_handle: Option<&str>,
) -> Result<Edge, String> {
    if source == target {
        return Err("cannot connect a node to itself".into());
    }
    let src_node = diagram
        .nodes
        .iter()
        .find(|n| n.id == source)
        .ok_or_else(|| format!("missing source node {source}"))?;
    let tgt_node = diagram
        .nodes
        .iter()
        .find(|n| n.id == target)
        .ok_or_else(|| format!("missing target node {target}"))?;

    let src_center = source_handle
        .and_then(|h| get_analytical_port_xy(src_node, h))
        .ok_or_else(|| "invalid source port".to_string())?;
    let tgt_center = target_handle
        .and_then(|h| get_analytical_port_xy(tgt_node, h))
        .ok_or_else(|| "invalid target port".to_string())?;

    let src_cat = resolve_port_category_for_connect(src_node, source_handle);
    let tgt_cat = resolve_port_category_for_connect(tgt_node, target_handle);
    let src_cmp = resolve_port_comparable_for_connect(src_node, source_handle);
    let tgt_cmp = resolve_port_comparable_for_connect(tgt_node, target_handle);
    let mismatch = src_cmp.is_some() && tgt_cmp.is_some() && src_cmp != tgt_cmp;
    let category = src_cat.or(tgt_cat).unwrap_or_else(|| "default".to_string());

    let data = serde_json::json!({
        "sourceHandleCenter": { "x": src_center.x, "y": src_center.y },
        "targetHandleCenter": { "x": tgt_center.x, "y": tgt_center.y },
        "wireCategory": category,
        "wireCategoryMismatch": mismatch,
    });

    Ok(Edge {
        id: format!("e-{}", uuid::Uuid::new_v4()),
        source: source.to_string(),
        target: target.to_string(),
        source_handle: source_handle.map(str::to_string),
        target_handle: target_handle.map(str::to_string),
        edge_type: Some("schematic".to_string()),
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagramme_schema::load_dxf_export_test_fixture;

    #[test]
    fn build_edge_between_device_ports() {
        let project = load_dxf_export_test_fixture();
        let diagram = &project.sheets[0].state;

        let edge = build_schematic_edge(
            diagram,
            "dev-export",
            "av-export",
            Some("R-0-dxf-test-out"),
            Some("T-0-dxf-test-av-in"),
        )
        .expect("edge");

        assert_eq!(edge.edge_type.as_deref(), Some("schematic"));
        assert!(edge.data.get("sourceHandleCenter").is_some());
        assert!(edge.data.get("wireCategory").is_some());
    }
}
