use diagramme_geometry::DEVICE_V2_WIDTH_PX;
use diagramme_schema::{DiagramState, Edge, Node, XY};
use diagramme_wires::apply_node_move_geometry;

fn node(id: &str, node_type: &str, x: f64, y: f64) -> Node {
    Node {
        id: id.into(),
        node_type: node_type.into(),
        position: XY { x, y },
        data: serde_json::json!({}),
        width: None,
        height: None,
        z_index: None,
    }
}

fn device(id: &str, x: f64, y: f64) -> Node {
    Node {
        id: id.into(),
        node_type: "deviceV2".into(),
        position: XY { x, y },
        data: serde_json::json!({
            "description": "amp",
            "tagCode": "AMP",
            "tagNumber": "1",
            "leftColumn": [{
                "header": "Input",
                "rows": [{ "id": "in1", "label": "1", "wireCategory": "audio" }]
            }],
            "rightColumn": [{
                "header": "Output",
                "rows": [{ "id": "out1", "label": "1", "wireCategory": "audio" }]
            }]
        }),
        width: Some(DEVICE_V2_WIDTH_PX),
        height: Some(54.0),
        z_index: None,
    }
}

#[test]
fn move_node_keeps_corners_when_stub_endpoints_unchanged() {
    let mut diagram = DiagramState {
        nodes: vec![
            node("a", "flyoffNote", 100.0, 100.0),
            node("b", "flyoffNote", 400.0, 100.0),
        ],
        edges: vec![Edge {
            id: "e1".into(),
            source: "a".into(),
            target: "b".into(),
            source_handle: None,
            target_handle: None,
            edge_type: Some("schematic".into()),
            data: serde_json::json!({
                "innerCorners": [{ "x": 250.0, "y": 100.0 }],
                "sourceHandleCenter": { "x": 150.0, "y": 110.0 },
                "targetHandleCenter": { "x": 350.0, "y": 110.0 }
            }),
        }],
    };

    apply_node_move_geometry(&mut diagram, "a", XY { x: 132.0, y: 100.0 });

    let corners = diagram.edges[0]
        .data
        .get("innerCorners")
        .and_then(|v| v.as_array())
        .expect("corners preserved");
    assert_eq!(corners[0]["x"].as_f64(), Some(250.0));
    assert_eq!(diagram.nodes[0].position.x, 132.0);
}

#[test]
fn move_node_rebuilds_inner_corners_when_stubs_shift() {
    let amp = device("a", 100.0, 100.0);
    let spk = Node {
        id: "spk".into(),
        node_type: "speakerBlock".into(),
        position: XY { x: 400.0, y: 100.0 },
        data: serde_json::json!({
            "line1": "SPK",
            "passthruEnabled": false,
            "symbolKind": "standard"
        }),
        width: Some(57.0),
        height: Some(50.0),
        z_index: None,
    };
    let mut diagram = DiagramState {
        nodes: vec![amp, spk],
        edges: vec![Edge {
            id: "e1".into(),
            source: "a".into(),
            target: "spk".into(),
            source_handle: Some("R-0-out1".into()),
            target_handle: Some("T-spk".into()),
            edge_type: Some("schematic".into()),
            data: serde_json::json!({
                "innerCorners": [{ "x": 280.0, "y": 130.0 }]
            }),
        }],
    };

    apply_node_move_geometry(&mut diagram, "a", XY { x: 130.0, y: 100.0 });

    assert!(diagram.nodes[0].position.x > 100.0);

    let corner_x = diagram.edges[0]
        .data
        .get("innerCorners")
        .and_then(|v| v.as_array())
        .and_then(|c| c.first())
        .and_then(|p| p.get("x"))
        .and_then(|x| x.as_f64());
    assert!(
        corner_x.is_none_or(|x| (x - 280.0).abs() > 0.5),
        "rebuilt corner should follow stub motion or simplify away, got {corner_x:?}"
    );
}

#[test]
fn move_node_snaps_unaligned_position_to_placement_grid() {
    let mut diagram = DiagramState {
        nodes: vec![node("a", "flyoffNote", 100.0, 100.0)],
        edges: vec![],
    };

    apply_node_move_geometry(&mut diagram, "a", XY { x: 107.0, y: 199.0 });

    assert_eq!(diagram.nodes[0].position.x, 106.5);
    assert_eq!(diagram.nodes[0].position.y, 199.5);
}
