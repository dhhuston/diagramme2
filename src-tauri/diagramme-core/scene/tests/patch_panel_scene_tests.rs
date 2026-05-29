use diagramme_geometry::PATCH_PANEL_WIDTH_PX;
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

fn first_lpp_patch_panel(project: &ProjectState) -> Option<&Node> {
    active_sheet(project)
        .state
        .nodes
        .iter()
        .find(|n| n.node_type == "lppPatchPanel")
}

#[test]
fn build_scene_produces_patch_panel_primitives() {
    let project = load_dxf_export_fixture();
    let panel = first_lpp_patch_panel(&project).expect("fixture has lppPatchPanel");
    let scene = build_scene(&active_sheet(&project).state);

    let polylines = scene
        .primitives
        .iter()
        .filter(|p| matches!(p, ScenePrimitive::Polyline { .. }))
        .count();
    assert!(
        polylines > 0,
        "expected patch panel row schematic polylines"
    );

    let has_hit = scene.hits.iter().any(|h| h.id == panel.id);
    assert!(has_hit, "expected patch panel body hit target");
}

#[test]
fn patch_panel_frame_width_matches_constant() {
    let project = load_dxf_export_fixture();
    let panel = first_lpp_patch_panel(&project).expect("fixture has lppPatchPanel");
    let scene = build_scene(&active_sheet(&project).state);

    let body_hit = scene
        .hits
        .iter()
        .find(|h| h.id == panel.id)
        .expect("lppPatchPanel body hit");
    assert!(
        (body_hit.bounds.width - PATCH_PANEL_WIDTH_PX).abs() < 1e-9,
        "frame width {} != PATCH_PANEL_WIDTH_PX {}",
        body_hit.bounds.width,
        PATCH_PANEL_WIDTH_PX
    );
}

#[test]
fn patch_panel_row_labels_present() {
    let project = load_dxf_export_fixture();
    let scene = build_scene(&active_sheet(&project).state);

    let row_numbers: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Text(t) if t.content == "1" || t.content == "2" => Some(t),
            _ => None,
        })
        .collect();
    assert!(
        row_numbers.len() >= 2,
        "expected LPP row number labels (1 and 2)"
    );
}
