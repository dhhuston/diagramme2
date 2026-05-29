use diagramme_geometry::{
    get_analytical_port_xy, wiretag_export_width_for_node, DEVICE_V2_WIDTH_PX, PATCH_PANEL_WIDTH_PX,
};
use diagramme_schema::{active_sheet_state, load_golden_fixture, Edge, Node, XY};
use diagramme_wires::{
    build_wire_geometry_model, clip_export_polyline_endpoints, fallback_polyline_from_ports,
    wire_sharp_polyline_for_edge, WireGeometryOptions,
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
    let poly = vec![
        diagramme_wires::FlowXY {
            x: 100.0 + PATCH_PANEL_WIDTH_PX,
            y: 120.0,
        },
        diagramme_wires::FlowXY { x: 200.0, y: 120.0 },
        diagramme_wires::FlowXY { x: 260.0, y: 120.0 },
    ];
    let clipped = clip_export_polyline_endpoints(&edge, &poly, &nodes, &[]);
    assert_eq!(
        clipped[0].x,
        100.0 + PATCH_PANEL_WIDTH_PX,
        "row port wires use analytical frame edge, not inset clip"
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

    let opts = WireGeometryOptions::default();
    let edges = [orphan_edge.clone(), valid_edge.clone()];
    assert!(wire_sharp_polyline_for_edge(&orphan_edge, &nodes, &edges, opts).is_none());
    assert!(fallback_polyline_from_ports(
        &orphan_edge,
        &nodes[0],
        &nodes[1],
        &nodes,
        &edges,
        opts,
    )
    .is_none());

    let model = build_wire_geometry_model(&nodes, &[orphan_edge, valid_edge], opts);
    assert!(!model.edges.contains_key("e-orphan-als"));
    let valid = model.edges.get("e-valid-als").expect("valid edge");
    assert!(!valid.sharp_polyline.iter().any(|pt| pt.x == 1008.0));
    assert!(model.edges.contains_key("e-valid-als"));
}

#[test]
fn wire_geometry_honors_persisted_inner_corners() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let with_corners = build_wire_geometry_model(
        &state.nodes,
        &state.edges,
        WireGeometryOptions {
            use_persisted_inner_corners: true,
        },
    );
    let without_corners = build_wire_geometry_model(
        &state.nodes,
        &state.edges,
        WireGeometryOptions {
            use_persisted_inner_corners: false,
        },
    );

    let mut differ = 0usize;
    for edge in &state.edges {
        let has_corners = edge
            .data
            .get("innerCorners")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
        if !has_corners {
            continue;
        }
        let Some(with_rec) = with_corners.edges.get(&edge.id) else {
            continue;
        };
        let Some(without_rec) = without_corners.edges.get(&edge.id) else {
            continue;
        };
        if with_rec.sharp_polyline != without_rec.sharp_polyline {
            differ += 1;
        }
    }

    assert!(
        differ > 0,
        "Comp Gym edges with persisted innerCorners should affect routing when enabled"
    );
}

#[test]
fn amp_to_speaker_uses_analytical_speaker_port_with_persisted_corners() {
    let amp = node(
        "device-57ef2140",
        "deviceV2",
        919.5,
        121.5,
        serde_json::json!({
            "description": "4 ch. amplifier",
            "leftColumn": [{ "header": "Input", "rows": [] }],
            "rightColumn": [{
                "header": "Output",
                "rows": [
                    { "id": "out-5177706f", "label": "1", "wireCategory": "audio" },
                    { "id": "rightColumn-d8a58349", "label": "2", "wireCategory": "audio" }
                ]
            }]
        }),
        Some(DEVICE_V2_WIDTH_PX),
        Some(54.0),
    );
    let spk = node(
        "spk-cfe86b51",
        "speakerBlock",
        1095.0,
        117.0,
        serde_json::json!({
            "line1": "SPK 1",
            "line2": "main bleachers",
            "passthruEnabled": true,
            "symbolKind": "standard"
        }),
        Some(57.0),
        Some(50.0),
    );
    let edge = Edge {
        id: "e-amp-spk".into(),
        source: "device-57ef2140".into(),
        target: "spk-cfe86b51".into(),
        source_handle: Some("R-0-out-5177706f".into()),
        target_handle: Some("T-spk".into()),
        edge_type: Some("schematic".into()),
        data: serde_json::json!({
            "targetHandleCenter": { "x": 1096.25, "y": -12.0 },
            "innerCorners": [{ "x": 1089.0, "y": 156.0 }]
        }),
    };
    let nodes = vec![amp, spk];
    let model = build_wire_geometry_model(&nodes, &[edge], WireGeometryOptions::default());
    let rec = model.edges.get("e-amp-spk").expect("edge record");
    let last = rec.sharp_polyline.last().expect("target point");
    assert!(
        last.y > 100.0,
        "wire should use analytical speaker port near diagram y, got y={}",
        last.y
    );
    assert!(
        (last.y - 141.0).abs() < 1.5,
        "expected speaker T-spk near y=141, got y={}",
        last.y
    );
}

#[test]
fn comp_gym_antenna_wires_start_at_analytical_ant_rx_port() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let model = build_wire_geometry_model(&state.nodes, &state.edges, WireGeometryOptions::default());

    for edge in &state.edges {
        if edge.source_handle.as_deref() != Some("ant-rx") {
            continue;
        }
        let ant = state
            .nodes
            .iter()
            .find(|n| n.id == edge.source)
            .expect("antenna node");
        let port = get_analytical_port_xy(ant, "ant-rx").expect("ant-rx port");
        let rec = model.edges.get(&edge.id).expect("antenna edge");
        let start = rec.sharp_polyline.first().expect("start");
        assert!(
            (start.x - port.x).abs() < 1e-6 && (start.y - port.y).abs() < 1e-6,
            "edge {} start ({}, {}) != port ({}, {})",
            edge.id,
            start.x,
            start.y,
            port.x,
            port.y
        );
    }
}

#[test]
fn device_row_port_wire_starts_at_analytical_port_not_frame_inset() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let edge = state
        .edges
        .iter()
        .find(|e| e.id == "e-9b5a907c-e6cf-48d5-b246-f59905144472")
        .unwrap();
    let model = build_wire_geometry_model(&state.nodes, &[edge.clone()], WireGeometryOptions::default());
    let rec = model.edges.get(&edge.id).unwrap();
    let start = rec.sharp_polyline.first().unwrap();
    let amp = state
        .nodes
        .iter()
        .find(|n| n.id == "device-57ef2140")
        .unwrap();
    let port = get_analytical_port_xy(amp, "R-0-out-5177706f").expect("output port");
    assert!(
        (start.x - port.x).abs() < 1e-6,
        "row port wire should start at handle x={}, got {}",
        port.x,
        start.x
    );
    assert!(
        (start.y - port.y).abs() < 1e-6,
        "row port wire should start at handle y={}, got {}",
        port.y,
        start.y
    );
}

#[test]
fn bundle_wire_stays_on_bracket_port_not_frame_inset() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let edge = state
        .edges
        .iter()
        .find(|e| e.id == "e-287bad14-642b-4a27-9bb9-d4ecd80c52ea")
        .unwrap();
    let model = build_wire_geometry_model(&state.nodes, &[edge.clone()], WireGeometryOptions::default());
    let rec = model.edges.get(&edge.id).unwrap();
    let start = rec.sharp_polyline.first().unwrap();
    let dsp = state
        .nodes
        .iter()
        .find(|n| n.id == "device-bf36a3ea")
        .unwrap();
    let port = get_analytical_port_xy(dsp, "R-0-bundle-0").expect("bundle port");
    assert!(
        (start.x - port.x).abs() < 1e-6,
        "bundle wire should start at bracket port x={}, got {}",
        port.x,
        start.x
    );
    assert!(
        (start.y - port.y).abs() < 1e-6,
        "bundle wire should stay on bus y={}",
        port.y
    );
    assert!(
        rec.sharp_polyline.windows(2).all(|w| (w[0].y - w[1].y).abs() < 1e-6),
        "horizontal bundle link should not dog-leg: {:?}",
        rec.sharp_polyline
    );
}

#[test]
fn amp_to_speaker_still_drops_when_port_heights_differ() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let edge = state
        .edges
        .iter()
        .find(|e| e.id == "e-9b5a907c-e6cf-48d5-b246-f59905144472")
        .unwrap();
    let model = build_wire_geometry_model(&state.nodes, &[edge.clone()], WireGeometryOptions::default());
    let rec = model.edges.get(&edge.id).unwrap();
    let has_vertical = rec.sharp_polyline.windows(2).any(|w| {
        (w[0].x - w[1].x).abs() < 1e-6 && (w[0].y - w[1].y).abs() > 1e-6
    });
    assert!(has_vertical, "speaker drop should remain: {:?}", rec.sharp_polyline);
}

#[test]
fn wiretag_conn_src_uses_export_hull_width_not_persisted_node_width() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let wiretag = state
        .nodes
        .iter()
        .find(|n| n.id == "wiretag-0daac244")
        .unwrap();
    let export_w = wiretag_export_width_for_node(wiretag, &state.nodes, &state.edges);
    let persisted_w = wiretag.width.unwrap_or(0.0);
    assert!(
        export_w > persisted_w,
        "wiretag export width should exceed stale persisted width={persisted_w}, got export_w={export_w}"
    );

    let edge = state
        .edges
        .iter()
        .find(|e| e.id == "e-d319e691-964a-41ef-b42a-ceb0b687e1f5")
        .unwrap();
    let model = build_wire_geometry_model(
        &state.nodes,
        &state.edges,
        WireGeometryOptions::default(),
    );
    let rec = model.edges.get(&edge.id).unwrap();
    let hull_right_x = wiretag.position.x + export_w;
    let start = rec.sharp_polyline.first().unwrap();
    assert!(
        (start.x - hull_right_x).abs() < 1e-6,
        "wire from wiretag conn-src should start at export hull right edge x={hull_right_x}, got x={}",
        start.x
    );
    assert!(
        start.x > wiretag.position.x + wiretag.width.unwrap_or(0.0),
        "wire must not start inside persisted wiretag box width"
    );
}
