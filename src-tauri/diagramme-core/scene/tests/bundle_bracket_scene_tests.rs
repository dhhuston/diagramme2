use diagramme_geometry::DEVICE_V2_WIDTH_PX;
use diagramme_scene::{build_scene_with_options, SceneBuildOptions, ScenePrimitive};
use diagramme_schema::{DiagramState, Edge, Node, XY};

fn layer0_polylines(scene: &diagramme_scene::Scene) -> usize {
    scene
        .primitives
        .iter()
        .filter(|p| {
            matches!(
                p,
                ScenePrimitive::Polyline {
                    edge_id: None,
                    layer,
                    ..
                } if layer == "0"
            )
        })
        .count()
}

fn device_v2(left_bundled: bool) -> Node {
    let mut left_column = serde_json::json!({
        "header": "Input",
        "rows": [
            { "id": "in-a", "label": "1" },
            { "id": "in-b", "label": "2" }
        ]
    });
    if left_bundled {
        left_column
            .as_object_mut()
            .unwrap()
            .insert(
                "bundledRowIds".into(),
                serde_json::json!([["in-a", "in-b"]]),
            );
    }

    Node {
        id: "dev-1".into(),
        node_type: "deviceV2".into(),
        position: XY { x: 100.0, y: 100.0 },
        data: serde_json::json!({
            "tagCode": "DEV",
            "tagNumber": "1",
            "description": "Test device",
            "leftColumn": [left_column],
            "rightColumn": [{ "header": "Output", "rows": [] }]
        }),
        width: Some(DEVICE_V2_WIDTH_PX),
        height: Some(54.0),
        z_index: None,
    }
}

#[test]
fn orphan_bundle_metadata_does_not_emit_bracket_polylines() {
    let plain = DiagramState {
        nodes: vec![device_v2(false)],
        edges: vec![],
    };
    let orphan = DiagramState {
        nodes: vec![device_v2(true)],
        edges: vec![],
    };
    let opts = SceneBuildOptions::default();
    let plain_scene = build_scene_with_options(&plain, opts);
    let orphan_scene = build_scene_with_options(&orphan, opts);
    assert_eq!(
        layer0_polylines(&plain_scene),
        layer0_polylines(&orphan_scene),
        "orphan bundle metadata must not add bracket polylines"
    );
}

#[test]
fn live_bundle_edge_emits_bracket_polylines() {
    let source = Node {
        id: "dev-1".into(),
        node_type: "deviceV2".into(),
        position: XY { x: 100.0, y: 100.0 },
        data: serde_json::json!({
            "tagCode": "DEV",
            "tagNumber": "1",
            "description": "Source",
            "leftColumn": [{ "header": "Input", "rows": [] }],
            "rightColumn": [{
                "header": "Output",
                "rows": [
                    { "id": "out-a", "label": "1" },
                    { "id": "out-b", "label": "2" }
                ],
                "bundledRowIds": [["out-a", "out-b"]]
            }]
        }),
        width: Some(DEVICE_V2_WIDTH_PX),
        height: Some(54.0),
        z_index: None,
    };
    let target = Node {
        id: "dev-2".into(),
        node_type: "deviceV2".into(),
        position: XY { x: 300.0, y: 100.0 },
        data: serde_json::json!({
            "tagCode": "DEV",
            "tagNumber": "2",
            "description": "Target",
            "leftColumn": [{
                "header": "Input",
                "rows": [{ "id": "in-x", "label": "1" }],
                "bundledRowIds": [["in-x"]]
            }],
            "rightColumn": [{ "header": "Output", "rows": [] }]
        }),
        width: Some(DEVICE_V2_WIDTH_PX),
        height: Some(54.0),
        z_index: None,
    };
    let orphan = DiagramState {
        nodes: vec![source.clone(), target.clone()],
        edges: vec![],
    };
    let wired = DiagramState {
        nodes: vec![source, target],
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
    let opts = SceneBuildOptions::default();
    let orphan_scene = build_scene_with_options(&orphan, opts);
    let wired_scene = build_scene_with_options(&wired, opts);
    assert!(
        layer0_polylines(&wired_scene) > layer0_polylines(&orphan_scene),
        "expected extra bracket polylines when bundle edge is present"
    );
}
