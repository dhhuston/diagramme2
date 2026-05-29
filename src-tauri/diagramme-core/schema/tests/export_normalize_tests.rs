use diagramme_schema::{
    active_bundle_handles, device_v2_bundle_handle_id, is_device_v2_bundle_bracket_active,
    normalize_diagram_for_export, normalize_diagram_for_persist, DiagramState, Edge, Node, XY,
};

fn device_with_orphan_bundle() -> Node {
    Node {
        id: "dev-1".into(),
        node_type: "deviceV2".into(),
        position: XY { x: 100.0, y: 100.0 },
        data: serde_json::json!({
            "tagCode": "DEV",
            "tagNumber": "1",
            "description": "Test",
            "leftColumn": [{
                "header": "Input",
                "rows": [
                    { "id": "in-a", "label": "1" },
                    { "id": "in-b", "label": "2" }
                ],
                "bundledRowIds": [["in-a", "in-b"]]
            }],
            "rightColumn": [{ "header": "Output", "rows": [] }]
        }),
        width: None,
        height: None,
        z_index: None,
    }
}

#[test]
fn active_bundle_handles_collects_bundle_edge_ports() {
    let diagram = DiagramState {
        nodes: vec![device_with_orphan_bundle()],
        edges: vec![Edge {
            id: "e-bundle".into(),
            source: "dev-1".into(),
            target: "dev-2".into(),
            source_handle: Some("R-0-bundle-0".into()),
            target_handle: Some("L-0-bundle-0".into()),
            edge_type: Some("schematic".into()),
            data: serde_json::json!({}),
        }],
    };
    let active = active_bundle_handles(&diagram);
    assert!(active.contains(&("dev-1".into(), "R-0-bundle-0".into())));
}

#[test]
fn is_device_v2_bundle_bracket_active_matches_handle_id() {
    let mut active = std::collections::HashSet::new();
    active.insert(("dev-1".into(), device_v2_bundle_handle_id('L', 0, 0)));
    assert!(is_device_v2_bundle_bracket_active(&active, "dev-1", 'L', 0, 0));
    assert!(!is_device_v2_bundle_bracket_active(&active, "dev-1", 'R', 0, 0));
}

#[test]
fn normalize_strips_orphan_bundles_and_inner_corners() {
    let mut diagram = DiagramState {
        nodes: vec![
            device_with_orphan_bundle(),
            Node {
                id: "dev-2".into(),
                node_type: "deviceV2".into(),
                position: XY { x: 300.0, y: 100.0 },
                data: serde_json::json!({
                    "tagCode": "DEV",
                    "tagNumber": "2",
                    "description": "Target",
                    "leftColumn": [{ "header": "Input", "rows": [{ "id": "in-a", "label": "1" }] }],
                    "rightColumn": [{ "header": "Output", "rows": [] }]
                }),
                width: None,
                height: None,
                z_index: None,
            },
        ],
        edges: vec![Edge {
            id: "e1".into(),
            source: "dev-1".into(),
            target: "dev-2".into(),
            source_handle: Some("R-0-out-a".into()),
            target_handle: Some("L-0-in-a".into()),
            edge_type: Some("schematic".into()),
            data: serde_json::json!({
                "innerCorners": [{ "x": 500.0, "y": 200.0 }]
            }),
        }],
    };
    normalize_diagram_for_export(&mut diagram);
    let bundles = diagram.nodes[0].data["leftColumn"][0]
        .get("bundledRowIds")
        .and_then(|v| v.as_array());
    assert!(bundles.is_none() || bundles.is_some_and(|a| a.is_empty()));
    assert!(diagram.edges[0].data.get("innerCorners").is_none());
}

#[test]
fn persist_normalize_also_strips_handle_centers() {
    let mut diagram = DiagramState {
        nodes: vec![Node {
            id: "n1".into(),
            node_type: "deviceV2".into(),
            position: XY { x: 0.0, y: 0.0 },
            data: serde_json::json!({}),
            width: None,
            height: None,
            z_index: None,
        }],
        edges: vec![Edge {
            id: "e1".into(),
            source: "n1".into(),
            target: "n1".into(),
            source_handle: None,
            target_handle: None,
            edge_type: None,
            data: serde_json::json!({
                "innerCorners": [{ "x": 1.0, "y": 2.0 }],
                "sourceHandleCenter": { "x": 10.0, "y": 20.0 },
                "targetHandleCenter": { "x": 30.0, "y": 40.0 }
            }),
        }],
    };
    normalize_diagram_for_persist(&mut diagram);
    let data = &diagram.edges[0].data;
    assert!(data.get("innerCorners").is_none());
    assert!(data.get("sourceHandleCenter").is_none());
    assert!(data.get("targetHandleCenter").is_none());
}
