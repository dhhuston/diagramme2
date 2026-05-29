use diagramme_geometry::{
    get_analytical_port_xy, schematic_layout::SCHEMATIC_FRAME_INSET_PX, DEVICE_V2_WIDTH_PX,
    PATCH_PANEL_WIDTH_PX,
};
use diagramme_schema::{Edge, Node, XY};
use diagramme_wires::{
    build_wire_geometry_model, clip_export_polyline_endpoints, fallback_polyline_from_ports,
    node_lookup_for_wire_geometry, wire_sharp_polyline_for_edge,
};

fn node(
    id: &str,
    node_type: &str,
    x: f64,
    y: f64,
    data: serde_json::Value,
    width: Option<f64>,
    height: Option<f64>,
) -> Node {
    Node {
        id: id.to_string(),
        node_type: node_type.to_string(),
        position: XY { x, y },
        data,
        width,
        height,
        z_index: None,
    }
}

#[test]
fn clip_export_polyline_endpoints_clips_patch_panel_source() {
    let nodes = vec![
        node(
            "p1",
            "lppPatchPanel",
            100.0,
            100.0,
            serde_json::json!({ "rows": [{ "id": "1", "connected": false }] }),
            Some(PATCH_PANEL_WIDTH_PX),
            None,
        ),
        node("b", "input", 260.0, 100.0, serde_json::json!({}), None, None),
    ];
    let edge = Edge {
        id: "e1".into(),
        source: "p1".into(),
        target: "b".into(),
        source_handle: Some("R-1".into()),
        target_handle: Some("in".into()),
        edge_type: None,
        data: serde_json::json!({}),
    };
    let lookup = node_lookup_for_wire_geometry(&nodes);
    let poly = vec![
        diagramme_wires::FlowXY {
            x: 100.0 + PATCH_PANEL_WIDTH_PX,
            y: 120.0,
        },
        diagramme_wires::FlowXY { x: 200.0, y: 120.0 },
        diagramme_wires::FlowXY { x: 260.0, y: 120.0 },
    ];
    let clipped = clip_export_polyline_endpoints(&edge, &poly, &lookup);
    assert_eq!(
        clipped[0].x,
        100.0 + PATCH_PANEL_WIDTH_PX - SCHEMATIC_FRAME_INSET_PX
    );
}

#[test]
fn fallback_rejects_stale_device_handles() {
    let dsp_data = serde_json::json!({
        "description": "dsp",
        "tagCode": "DSP",
        "tagNumber": "2",
        "leftColumn": [{
            "header": "Input",
            "rows": [{ "id": "in-a", "label": "1", "wireCategory": "audio" }]
        }],
        "rightColumn": [
            {
                "header": "Output",
                "rows": [{ "id": "out-12259f97", "label": "8", "wireCategory": "audio" }]
            },
            {
                "header": "gpio",
                "rows": [{ "id": "rightColumn-3223a615", "label": "2", "wireCategory": "control" }]
            }
        ]
    });

    let nodes = vec![
        node(
            "device-bf36a3ea",
            "deviceV2",
            543.0,
            -217.5,
            dsp_data,
            Some(DEVICE_V2_WIDTH_PX),
            Some(54.0),
        ),
        node(
            "device-523e1ff3",
            "deviceV2",
            703.5,
            -217.5,
            serde_json::json!({
                "description": "assistive listening",
                "tagCode": "als",
                "tagNumber": "1",
                "leftColumn": [{
                    "header": "Input",
                    "rows": [{ "id": "in-607ebb0a", "label": "1", "wireCategory": "audio" }]
                }],
                "rightColumn": [{
                    "header": "Output",
                    "rows": [{ "id": "out-607ebb0a", "label": "1", "wireCategory": "rf" }]
                }]
            }),
            Some(DEVICE_V2_WIDTH_PX),
            Some(54.0),
        ),
    ];

    let orphan_edge = Edge {
        id: "e-orphan-als".into(),
        source: "device-bf36a3ea".into(),
        target: "device-523e1ff3".into(),
        source_handle: Some("R-0-rightColumn-96e9bfed".into()),
        target_handle: Some("L-0-in-607ebb0a".into()),
        edge_type: Some("schematic".into()),
        data: serde_json::json!({
            "sourceHandleCenter": { "x": 1002.5, "y": -141.0 },
            "targetHandleCenter": { "x": 1037.5, "y": -99.0 },
            "wireCategory": "audio"
        }),
    };

    let valid_edge = Edge {
        id: "e-valid-als".into(),
        source: "device-bf36a3ea".into(),
        target: "device-523e1ff3".into(),
        source_handle: Some("R-0-out-12259f97".into()),
        target_handle: Some("L-0-in-607ebb0a".into()),
        edge_type: Some("schematic".into()),
        data: serde_json::json!({
            "sourceHandleCenter": { "x": 668.0, "y": -177.0 },
            "targetHandleCenter": { "x": 704.5, "y": -177.0 },
            "wireCategory": "audio"
        }),
    };

    let dsp = &nodes[0];
    assert!(get_analytical_port_xy(dsp, "R-0-rightColumn-96e9bfed").is_none());

    let lookup = node_lookup_for_wire_geometry(&nodes);
    assert!(wire_sharp_polyline_for_edge(&orphan_edge, &lookup).is_none());
    assert!(fallback_polyline_from_ports(
        &orphan_edge,
        lookup.get("device-bf36a3ea").unwrap(),
        lookup.get("device-523e1ff3").unwrap(),
    )
    .is_none());

    let model = build_wire_geometry_model(&nodes, &[orphan_edge, valid_edge]);
    assert!(!model.contains_key("e-orphan-als"));
    let valid = model.get("e-valid-als").expect("valid edge");
    assert!(!valid.sharp_polyline.iter().any(|pt| pt.x == 1008.0));
    assert!(model.contains_key("e-valid-als"));
}
