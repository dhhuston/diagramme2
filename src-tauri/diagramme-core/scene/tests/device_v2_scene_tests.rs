use diagramme_geometry::{DEVICE_V2_WIDTH_PX, NODE_TITLE_FONT_PX};
use diagramme_scene::{build_scene, scene_to_cad, CadPrimitive, ScenePrimitive};
use diagramme_schema::{
    active_sheet, device_tag_label, device_title_label, first_device_v2, load_golden_fixture,
};

#[test]
fn build_scene_produces_device_v2_primitives() {
    let project = load_golden_fixture();
    let scene = build_scene(&active_sheet(&project).state);
    assert!(
        !scene.primitives.is_empty(),
        "expected non-empty primitives for golden fixture"
    );
    assert!(!scene.hits.is_empty(), "expected hit targets");
}

#[test]
fn device_v2_frame_width_matches_constant() {
    let project = load_golden_fixture();
    let device = first_device_v2(&project);
    let scene = build_scene(&active_sheet(&project).state);

    let body_hit = scene
        .hits
        .iter()
        .find(|h| h.id == device.id)
        .expect("node body hit");
    assert!(
        (body_hit.bounds.width - DEVICE_V2_WIDTH_PX).abs() < 1e-9,
        "frame width {} != DEVICE_V2_WIDTH_PX {}",
        body_hit.bounds.width,
        DEVICE_V2_WIDTH_PX
    );
}

#[test]
fn device_v2_tag_text_height_is_node_title_font_px() {
    let project = load_golden_fixture();
    let device = first_device_v2(&project);
    let expected_tag = device_tag_label(device);
    let scene = build_scene(&active_sheet(&project).state);
    let tag = scene
        .primitives
        .iter()
        .find_map(|p| match p {
            ScenePrimitive::Text(t) if t.content == expected_tag => Some(t),
            _ => None,
        })
        .expect("device tag SceneText");
    assert!(
        (tag.height_px - NODE_TITLE_FONT_PX).abs() < 1e-9,
        "tag height_px {} != NODE_TITLE_FONT_PX {}",
        tag.height_px,
        NODE_TITLE_FONT_PX
    );
}

#[test]
fn device_v2_inset_frame_polyline_is_closed() {
    let project = load_golden_fixture();
    let scene = build_scene(&active_sheet(&project).state);
    let has_closed_frame = scene.primitives.iter().any(|p| {
        matches!(
            p,
            ScenePrimitive::Polyline {
                closed: true,
                points,
                edge_id: None,
                ..
            } if points.len() == 4
        )
    });
    assert!(has_closed_frame, "device frame should be a closed 4-point polyline");
}

#[test]
fn device_v2_title_text_is_uppercase() {
    let project = load_golden_fixture();
    let device = first_device_v2(&project);
    let expected_title = device_title_label(device);
    let scene = build_scene(&active_sheet(&project).state);
    let title = scene
        .primitives
        .iter()
        .find_map(|p| match p {
            ScenePrimitive::Text(t) if t.content == expected_title => Some(t),
            _ => None,
        })
        .expect("device title SceneText");
    assert_eq!(title.content, expected_title);
}

#[test]
fn device_v2_tag_text_cad_height_is_three_thirty_seconds_inch() {
    let project = load_golden_fixture();
    let device = first_device_v2(&project);
    let expected_tag = device_tag_label(device);
    let scene = build_scene(&active_sheet(&project).state);
    let cad = scene_to_cad(&scene);
    let tag_cad = cad
        .primitives
        .iter()
        .find_map(|p| match p {
            CadPrimitive::Text(t) if t.content == expected_tag => Some(t),
            _ => None,
        })
        .expect("device tag CadText");
    assert!(
        (tag_cad.height_in - 3.0 / 32.0).abs() < 1e-9,
        "tag height_in {} != 3/32",
        tag_cad.height_in
    );
}
