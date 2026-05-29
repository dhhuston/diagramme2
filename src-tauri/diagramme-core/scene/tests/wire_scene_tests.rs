use diagramme_geometry::units::px_to_in;
use diagramme_scene::{build_scene, scene_to_cad, CadPrimitive, ScenePrimitive};
use diagramme_schema::{active_sheet_state, load_golden_fixture};

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
fn golden_fixture_wire_polyline_count() {
    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let scene = build_scene(state);
    let edge_ids: std::collections::HashSet<_> = wire_polylines(&scene)
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline {
                edge_id: Some(id),
                ..
            } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        edge_ids.len(),
        state.edges.len(),
        "each diagram edge should emit at least one wire polyline segment"
    );
    assert!(
        !edge_ids.is_empty(),
        "Comp Gym F102A golden fixture should include routed wires"
    );
}

#[test]
fn build_scene_includes_wire_polylines_with_edge_ids() {
    let project = load_golden_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let wires = wire_polylines(&scene);
    assert!(
        !wires.is_empty(),
        "expected wire polylines for golden fixture"
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
    let project = load_golden_fixture();
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
    let project = load_golden_fixture();
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

const DSP_AMP_BUNDLE_EDGE: &str = "e-287bad14-642b-4a27-9bb9-d4ecd80c52ea";

fn scene_wire_points_for_edge(scene: &diagramme_scene::Scene, edge_id: &str) -> Vec<diagramme_geometry::PointPx> {
    scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline {
                points,
                edge_id: Some(id),
                ..
            } if id == edge_id => Some(points.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn cad_wire_points_for_edge(
    scene: &diagramme_scene::Scene,
    edge_id: &str,
) -> Vec<diagramme_scene::PointIn> {
    let cad = scene_to_cad(scene);
    cad.primitives
        .iter()
        .filter_map(|p| match p {
            CadPrimitive::Polyline {
                points,
                edge_id: Some(id),
                ..
            } if id == edge_id => Some(points.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn overlapping_bundle_bus_scene_matches_cad_topology() {
    use diagramme_scene::scene_point_to_cad;

    let project = load_golden_fixture();
    let state = active_sheet_state(&project);
    let scene = build_scene(state);
    let cad = scene_to_cad(&scene);
    let scene_pts = scene_wire_points_for_edge(&scene, DSP_AMP_BUNDLE_EDGE);
    let cad_pts = cad_wire_points_for_edge(&scene, DSP_AMP_BUNDLE_EDGE);

    assert!(scene_pts.len() >= 2, "missing scene wire: {:?}", scene_pts);
    assert_eq!(
        scene_pts.len(),
        cad_pts.len(),
        "overlap fidelity: scene and CAD must emit the same wire vertices"
    );
    assert!(
        scene_pts.windows(2).all(|w| (w[0].y - w[1].y).abs() < 1e-6),
        "user-drawn horizontal bus must stay collinear in scene: {:?}",
        scene_pts
    );
    for (s, c) in scene_pts.iter().zip(cad_pts.iter()) {
        let expected = scene_point_to_cad(*s, cad.extent);
        assert!((expected.x - c.x).abs() < 1e-9, "x drift");
        assert!((expected.y - c.y).abs() < 1e-9, "y drift");
    }
}
