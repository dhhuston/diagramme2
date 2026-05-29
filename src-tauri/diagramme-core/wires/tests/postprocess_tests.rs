use diagramme_wires::{
    postprocess_dxf_wire_polylines, postprocess_dxf_wire_records_for_revit, DxfWirePolylineRecord,
    FlowXY, RevitDxfWirePiece, SCHEMATIC_BUNDLE_CORNER_RADIUS_PX,
};

fn pt(x: f64, y: f64) -> FlowXY {
    FlowXY { x, y }
}

#[test]
fn splits_horizontal_wire_at_crossings() {
    let records = vec![
        DxfWirePolylineRecord {
            edge_id: "h1".into(),
            is_schematic: true,
            is_bundle: false,
            points: vec![pt(100.0, 100.0), pt(300.0, 100.0)],
            source_node_id: None,
        },
        DxfWirePolylineRecord {
            edge_id: "v1".into(),
            is_schematic: true,
            is_bundle: false,
            points: vec![pt(200.0, 40.0), pt(200.0, 180.0)],
            source_node_id: None,
        },
    ];

    let out = postprocess_dxf_wire_polylines(&records);
    assert_eq!(out.len(), 3);
    let horizontals: Vec<_> = out
        .iter()
        .filter(|seg| seg.len() == 2 && seg[0].y == seg[1].y)
        .collect();
    assert_eq!(horizontals.len(), 2);
}

#[test]
fn splits_bundled_horizontal_wire_when_vertical_crosses() {
    let records = vec![
        DxfWirePolylineRecord {
            edge_id: "bundle-h".into(),
            is_schematic: true,
            is_bundle: true,
            points: vec![pt(0.0, 100.0), pt(300.0, 100.0), pt(300.0, 200.0)],
            source_node_id: None,
        },
        DxfWirePolylineRecord {
            edge_id: "cross-v".into(),
            is_schematic: true,
            is_bundle: false,
            points: vec![pt(150.0, 0.0), pt(150.0, 250.0)],
            source_node_id: None,
        },
    ];
    let out = postprocess_dxf_wire_records_for_revit(&records);
    let horizontals: Vec<_> = out
        .iter()
        .filter(|r| {
            matches!(
                r,
                RevitDxfWirePiece::Polyline { points, .. }
                    if points.len() == 2 && points[0].y == points[1].y
            )
        })
        .collect();
    assert!(horizontals.len() >= 2);
    assert!(horizontals.iter().all(|r| matches!(
        r,
        RevitDxfWirePiece::Polyline { is_bundle: false, .. }
    )));
    assert!(out.iter().any(|r| matches!(r, RevitDxfWirePiece::FilletArc { .. })));
}

#[test]
fn trims_bundle_vertical_legs_at_fillet_tangents_when_split_for_crossings() {
    let r = SCHEMATIC_BUNDLE_CORNER_RADIUS_PX;
    let records = vec![
        DxfWirePolylineRecord {
            edge_id: "bundle".into(),
            is_schematic: true,
            is_bundle: true,
            points: vec![pt(0.0, 99.0), pt(198.0, 99.0), pt(198.0, 51.0)],
            source_node_id: None,
        },
        DxfWirePolylineRecord {
            edge_id: "cross".into(),
            is_schematic: true,
            is_bundle: false,
            points: vec![pt(99.0, 0.0), pt(99.0, 198.0)],
            source_node_id: None,
        },
    ];
    let out = postprocess_dxf_wire_records_for_revit(&records);
    let bundle_vert = out.iter().find(|p| {
        matches!(
            p,
            RevitDxfWirePiece::Polyline { points, .. }
                if points.len() == 2
                    && points[0].x == 198.0
                    && points[0].y.min(points[1].y) == 51.0
        )
    });
    assert!(bundle_vert.is_some());
    if let Some(RevitDxfWirePiece::Polyline { points, .. }) = bundle_vert {
        let y_max = points[0].y.max(points[1].y);
        assert!((y_max - (99.0 - r)).abs() < 1e-6);
    }
}

#[test]
fn applies_rounded_corner_approximation_to_bundle_wires() {
    let records = vec![DxfWirePolylineRecord {
        edge_id: "b1".into(),
        is_schematic: true,
        is_bundle: true,
        points: vec![
            pt(100.0, 100.0),
            pt(200.0, 100.0),
            pt(200.0, 200.0),
            pt(300.0, 200.0),
        ],
        source_node_id: None,
    }];
    let out = postprocess_dxf_wire_polylines(&records);
    assert_eq!(out.len(), 1);
    assert!(out[0].len() > 4);
}
