use diagramme_geometry::{DEVICE_V2_WIDTH_PX, LABEL_FONT_PX};
use diagramme_scene::{build_scene, scene_to_cad, CadPrimitive, ScenePrimitive};
use diagramme_schema::{Node, ProjectState};

fn load_dxf_export_fixture() -> ProjectState {
    let json = include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");
    serde_json::from_str(json).expect("parse dxf-export-test fixture")
}

fn first_device_v2(project: &ProjectState) -> &Node {
    let sheet = project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .or_else(|| project.sheets.first())
        .expect("fixture has a sheet");
    sheet
        .state
        .nodes
        .iter()
        .find(|n| n.node_type == "deviceV2" || n.node_type == "device")
        .expect("fixture has deviceV2")
}

#[test]
fn build_scene_produces_device_v2_primitives() {
    let project = load_dxf_export_fixture();
    let sheet = project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .unwrap();
    let scene = build_scene(&sheet.state);
    assert!(
        !scene.primitives.is_empty(),
        "expected non-empty primitives for dxf-export-test fixture"
    );
    assert!(!scene.hits.is_empty(), "expected hit targets");
}

#[test]
fn device_v2_frame_width_matches_constant() {
    let project = load_dxf_export_fixture();
    let device = first_device_v2(&project);
    let sheet = project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .unwrap();
    let scene = build_scene(&sheet.state);

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
fn device_v2_tag_text_height_is_label_font_px() {
    let project = load_dxf_export_fixture();
    let sheet = project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .unwrap();
    let scene = build_scene(&sheet.state);
    let tag = scene
        .primitives
        .iter()
        .find_map(|p| match p {
            ScenePrimitive::Text(t) if t.content.contains('/') => Some(t),
            _ => None,
        })
        .expect("tag SceneText");
    assert!(
        (tag.height_px - LABEL_FONT_PX).abs() < 1e-9,
        "tag height_px {} != LABEL_FONT_PX {}",
        tag.height_px,
        LABEL_FONT_PX
    );
}

#[test]
fn device_v2_tag_text_cad_height_is_one_eighth_inch() {
    let project = load_dxf_export_fixture();
    let sheet = project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .unwrap();
    let scene = build_scene(&sheet.state);
    let cad = scene_to_cad(&scene);
    let tag_cad = cad
        .primitives
        .iter()
        .find_map(|p| match p {
            CadPrimitive::Text(t) if t.content.contains('/') => Some(t),
            _ => None,
        })
        .expect("tag CadText");
    assert!(
        (tag_cad.height_in - 0.125).abs() < 1e-9,
        "tag height_in {} != 0.125",
        tag_cad.height_in
    );
}
