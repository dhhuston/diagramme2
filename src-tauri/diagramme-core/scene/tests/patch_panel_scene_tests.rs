use diagramme_geometry::{NODE_TITLE_FONT_PX, PATCH_PANEL_WIDTH_PX};
use diagramme_scene::{build_scene, scene_to_cad, ScenePrimitive};
use diagramme_schema::{active_sheet, first_patch_panel, load_dxf_export_test_fixture};

fn palette_project() -> diagramme_schema::ProjectState {
    load_dxf_export_test_fixture()
}

#[test]
fn build_scene_produces_patch_panel_primitives() {
    let project = palette_project();
    let panel = first_patch_panel(&project).expect("fixture has patch panel");
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
    let project = palette_project();
    let panel = first_patch_panel(&project).expect("fixture has patch panel");
    let scene = build_scene(&active_sheet(&project).state);

    let body_hit = scene
        .hits
        .iter()
        .find(|h| h.id == panel.id)
        .expect("patch panel body hit");
    assert!(
        (body_hit.bounds.width - PATCH_PANEL_WIDTH_PX).abs() < 1e-9,
        "frame width {} != PATCH_PANEL_WIDTH_PX {}",
        body_hit.bounds.width,
        PATCH_PANEL_WIDTH_PX
    );
}

#[test]
fn patch_panel_title_wraps_to_two_lines() {
    let project = palette_project();
    let scene = build_scene(&active_sheet(&project).state);

    let title_lines: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Text(t) if t.content == "LOUDSPEAKER" || t.content == "PATCH" => {
                Some(t.content.as_str())
            }
            _ => None,
        })
        .collect();
    assert!(
        title_lines.contains(&"LOUDSPEAKER") && title_lines.contains(&"PATCH"),
        "expected wrapped LPP title lines, got {:?}",
        title_lines
    );
}

#[test]
fn patch_panel_title_text_height_is_three_thirty_seconds_inch() {
    let project = palette_project();
    let scene = build_scene(&active_sheet(&project).state);

    let title = scene
        .primitives
        .iter()
        .find_map(|p| match p {
            ScenePrimitive::Text(t) if t.content.contains("PATCH") => Some(t),
            _ => None,
        })
        .expect("patch panel title SceneText");
    assert!(
        (title.height_px - NODE_TITLE_FONT_PX).abs() < 1e-9,
        "title height_px {} != NODE_TITLE_FONT_PX {}",
        title.height_px,
        NODE_TITLE_FONT_PX
    );

    let cad = scene_to_cad(&scene);
    let title_cad = cad
        .primitives
        .iter()
        .find_map(|p| match p {
            diagramme_scene::CadPrimitive::Text(t) if t.content.contains("PATCH") => Some(t),
            _ => None,
        })
        .expect("patch panel title CadText");
    assert!(
        (title_cad.height_in - 3.0 / 32.0).abs() < 1e-9,
        "title height_in {} != 3/32",
        title_cad.height_in
    );
}

#[test]
fn patch_panel_row_labels_present() {
    let project = palette_project();
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
        "expected patch panel row number labels (1 and 2)"
    );
}
