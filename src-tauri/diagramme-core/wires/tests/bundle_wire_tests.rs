use diagramme_geometry::{WIRETAG_CONN_SRC, WIRETAG_CONN_TGT};
use diagramme_schema::{active_sheet_state, load_golden_fixture, Edge, Node, XY};
use diagramme_wires::{build_wire_geometry_model, is_bundle_wire_edge, WireGeometryOptions};

fn node(id: &str, node_type: &str, x: f64, y: f64, data: serde_json::Value) -> Node {
    Node {
        id: id.to_string(),
        node_type: node_type.to_string(),
        position: XY { x, y },
        data,
        width: None,
        height: None,
        z_index: None,
    }
}

fn edge(
    id: &str,
    source: &str,
    target: &str,
    source_handle: &str,
    target_handle: &str,
) -> Edge {
    Edge {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        source_handle: Some(source_handle.to_string()),
        target_handle: Some(target_handle.to_string()),
        edge_type: Some("schematic".to_string()),
        data: serde_json::json!({}),
    }
}

#[test]
fn wiretag_pair_propagates_bundle_to_receiver_edge() {
    let pair_id = "pair-1";
    let nodes = vec![
        node(
            "dev-src",
            "deviceV2",
            0.0,
            0.0,
            serde_json::json!({
                "rightColumn": [{
                    "rows": [{ "id": "out-a", "wireCategory": "audio" }, { "id": "out-b", "wireCategory": "audio" }],
                    "bundledRowIds": [["out-a", "out-b"]]
                }]
            }),
        ),
        node(
            "wt-b",
            "wiretag",
            100.0,
            0.0,
            serde_json::json!({ "pairId": pair_id, "end": "b", "pairIndex": 1 }),
        ),
        node(
            "wt-a",
            "wiretag",
            200.0,
            0.0,
            serde_json::json!({ "pairId": pair_id, "end": "a", "pairIndex": 1 }),
        ),
        node(
            "dev-rcv",
            "deviceV2",
            300.0,
            0.0,
            serde_json::json!({
                "leftColumn": [{
                    "rows": [{ "id": "in-a", "wireCategory": "audio" }]
                }]
            }),
        ),
    ];
    let edges = vec![
        edge(
            "e-bundle-in",
            "dev-src",
            "wt-b",
            "R-0-bundle-0",
            WIRETAG_CONN_TGT,
        ),
        edge(
            "e-receiver-out",
            "wt-a",
            "dev-rcv",
            WIRETAG_CONN_SRC,
            "L-0-in-a",
        ),
    ];

    assert!(is_bundle_wire_edge(&edges[0], &nodes, &edges));
    assert!(
        is_bundle_wire_edge(&edges[1], &nodes, &edges),
        "receiver wiretag edge should inherit bundle from partner"
    );
}

#[test]
fn wiretag_pair_without_bundle_stays_regular() {
    let pair_id = "pair-2";
    let nodes = vec![
        node("wt-a", "wiretag", 0.0, 0.0, serde_json::json!({ "pairId": pair_id, "end": "a" })),
        node("wt-b", "wiretag", 100.0, 0.0, serde_json::json!({ "pairId": pair_id, "end": "b" })),
        node(
            "dev-a",
            "deviceV2",
            200.0,
            0.0,
            serde_json::json!({
                "leftColumn": [{ "rows": [{ "id": "in-a", "wireCategory": "audio" }] }]
            }),
        ),
        node(
            "dev-b",
            "deviceV2",
            300.0,
            0.0,
            serde_json::json!({
                "rightColumn": [{ "rows": [{ "id": "out-a", "wireCategory": "audio" }] }]
            }),
        ),
    ];
    let edges = vec![
        edge("e-a", "dev-a", "wt-a", "L-0-in-a", WIRETAG_CONN_TGT),
        edge("e-b", "wt-b", "dev-b", WIRETAG_CONN_SRC, "R-0-out-a"),
    ];

    assert!(!is_bundle_wire_edge(&edges[0], &nodes, &edges));
    assert!(!is_bundle_wire_edge(&edges[1], &nodes, &edges));
}

#[test]
fn comp_gym_wiretag_receiver_inherits_bundle_from_partner() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let nodes = &state.nodes;
    let edges = &state.edges;

    let bundle_in = edges
        .iter()
        .find(|e| e.id == "e-8cf079de-e0de-4cff-9794-6f561b84d49d")
        .expect("bundle → wiretag edge");
    assert!(
        is_bundle_wire_edge(bundle_in, nodes, edges),
        "bundle handle side should be bundle"
    );

    let receiver_out = edges
        .iter()
        .find(|e| e.id == "e-492fc5d5-889a-4846-a81d-c22ab0f017b4")
        .expect("receiver wiretag → avPlate edge");
    assert!(
        is_bundle_wire_edge(receiver_out, nodes, edges),
        "partner wiretag receiver edge should inherit bundle styling"
    );

    let model = build_wire_geometry_model(nodes, edges, WireGeometryOptions::default());
    let record = model
        .edges
        .get("e-492fc5d5-889a-4846-a81d-c22ab0f017b4")
        .expect("geometry record");
    assert!(
        record.is_bundle,
        "wire geometry model should mark receiver edge as bundle"
    );
}
