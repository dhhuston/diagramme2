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

#[test]
fn move_node_translates_persisted_inner_corners_on_connected_edge() {
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

    apply_node_move_geometry(&mut diagram, "a", XY { x: 130.0, y: 100.0 });

    let corners = diagram.edges[0]
        .data
        .get("innerCorners")
        .and_then(|v| v.as_array())
        .expect("corners preserved");
    assert_eq!(corners[0]["x"].as_f64(), Some(264.0));
    assert_eq!(diagram.nodes[0].position.x, 130.0);
}
