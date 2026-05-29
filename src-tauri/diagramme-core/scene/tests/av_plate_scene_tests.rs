use diagramme_geometry::{DEVICE_PORT_LABEL_FONT_PX, PATCH_PANEL_WIDTH_PX};
use diagramme_scene::{build_scene, ScenePrimitive};
use diagramme_schema::{Node, ProjectState};

fn load_dxf_export_fixture() -> ProjectState {
    let json = include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");
    serde_json::from_str(json).expect("parse dxf-export-test fixture")
}

fn active_sheet(project: &ProjectState) -> &diagramme_schema::Sheet {
    project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .or_else(|| project.sheets.first())
        .expect("fixture has a sheet")
}

fn first_av_plate(project: &ProjectState) -> Option<&Node> {
    active_sheet(project)
        .state
        .nodes
        .iter()
        .find(|n| n.node_type == "avPlate")
}

#[test]
fn build_scene_produces_av_plate_primitives() {
    let project = load_dxf_export_fixture();
    let av = first_av_plate(&project).expect("fixture has avPlate");
    let scene = build_scene(&active_sheet(&project).state);

    let has_primitives = scene.primitives.iter().any(|p| match p {
        ScenePrimitive::Polyline { .. } | ScenePrimitive::Solid { .. } | ScenePrimitive::Text(_) => {
            true
        }
        _ => false,
    });
    assert!(has_primitives, "expected avPlate to emit scene primitives");

    let has_hit = scene.hits.iter().any(|h| h.id == av.id);
    assert!(has_hit, "expected avPlate body hit target");
}

#[test]
fn av_plate_frame_width_matches_constant() {
    let project = load_dxf_export_fixture();
    let av = first_av_plate(&project).expect("fixture has avPlate");
    let scene = build_scene(&active_sheet(&project).state);

    let body_hit = scene
        .hits
        .iter()
        .find(|h| h.id == av.id)
        .expect("avPlate body hit");
    assert!(
        (body_hit.bounds.width - PATCH_PANEL_WIDTH_PX).abs() < 1e-9,
        "frame width {} != PATCH_PANEL_WIDTH_PX {}",
        body_hit.bounds.width,
        PATCH_PANEL_WIDTH_PX
    );
}

#[test]
fn av_plate_port_label_text_height_is_device_port_label_font_px() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(&active_sheet(&project).state);

    let port_labels: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Text(t) if t.content == "In" || t.content == "Out" => Some(t),
            _ => None,
        })
        .collect();
    assert!(
        !port_labels.is_empty(),
        "expected avPlate port label SceneText (In/Out)"
    );
    for label in port_labels {
        assert!(
            (label.height_px - DEVICE_PORT_LABEL_FONT_PX).abs() < 1e-9,
            "port label height_px {} != DEVICE_PORT_LABEL_FONT_PX {}",
            label.height_px,
            DEVICE_PORT_LABEL_FONT_PX
        );
    }
}
