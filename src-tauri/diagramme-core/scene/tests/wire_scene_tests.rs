use diagramme_geometry::units::px_to_in;
use diagramme_scene::{build_scene, scene_to_cad, CadPrimitive, ScenePrimitive};
use diagramme_schema::ProjectState;

fn load_dxf_export_fixture() -> ProjectState {
    let json = include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");
    serde_json::from_str(json).expect("parse dxf-export-test fixture")
}

fn active_sheet_state(project: &ProjectState) -> &diagramme_schema::DiagramState {
    &project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .or_else(|| project.sheets.first())
        .expect("fixture has a sheet")
        .state
}

fn wire_polylines(scene: &diagramme_scene::Scene) -> Vec<&ScenePrimitive> {
    scene
        .primitives
        .iter()
        .filter(|p| {
            matches!(
                p,
                ScenePrimitive::Polyline {
                    edge_id: Some(_),
                    ..
                }
            )
        })
        .collect()
}

#[test]
fn dxf_export_test_wire_polyline_count() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let count = wire_polylines(&scene).len();
    assert_eq!(count, 42, "dxf-export-test wire polyline count");
}

#[test]
fn build_scene_includes_wire_polylines_with_edge_ids() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let wires = wire_polylines(&scene);
    assert!(
        !wires.is_empty(),
        "expected wire polylines for dxf-export-test fixture"
    );
    for prim in &wires {
        let ScenePrimitive::Polyline { edge_id, stroke_px, .. } = prim else {
            continue;
        };
        assert!(edge_id.as_ref().unwrap().starts_with('e'));
        assert!((*stroke_px - 1.0).abs() < 1e-9);
    }
}

#[test]
fn first_wire_polyline_has_at_least_two_points() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let first = wire_polylines(&scene)
        .into_iter()
        .find_map(|p| match p {
            ScenePrimitive::Polyline { points, .. } if points.len() >= 2 => Some(points),
            _ => None,
        })
        .expect("wire polyline with >= 2 points");
    assert!(first.len() >= 2);
}

#[test]
fn scene_to_cad_wire_segment_length_matches_px_scale() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let cad = scene_to_cad(&scene);
    let wire = cad
        .primitives
        .iter()
        .find_map(|p| match p {
            CadPrimitive::Polyline { points, edge_id, .. } if edge_id.is_some() && points.len() >= 2 => {
                Some(points)
            }
            _ => None,
        })
        .expect("cad wire polyline");
    let a = wire[0];
    let b = wire[1];
    let cad_len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    let scene_wire = wire_polylines(&scene)
        .into_iter()
        .find_map(|p| match p {
            ScenePrimitive::Polyline { points, .. } if points.len() >= 2 => Some(points),
            _ => None,
        })
        .expect("scene wire");
    let px_len = ((scene_wire[1].x - scene_wire[0].x).powi(2)
        + (scene_wire[1].y - scene_wire[0].y).powi(2))
    .sqrt();
    let expected_in = px_to_in(px_len);
    assert!(
        (cad_len - expected_in).abs() < 1e-9,
        "cad segment length {cad_len} != px/72 {expected_in}"
    );
}
