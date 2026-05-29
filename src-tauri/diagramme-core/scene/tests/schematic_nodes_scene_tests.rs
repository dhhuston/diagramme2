use diagramme_geometry::{
    FLYOFF_TRI_H, MIC_SPEAKER_FRAME_HEIGHT_PX, PATCH_PANEL_WIDTH_PX,
    SPEAKER_BLOCK_DEFAULT_WIDTH_PX, VC_SYMBOL_TOP_INSET_PX, VOLUME_CONTROL_FRAME_HEIGHT_PX,
};
use diagramme_scene::{build_scene, ScenePrimitive};
use diagramme_schema::{
    active_sheet, find_first_node, find_node, load_dxf_export_test_fixture, load_golden_fixture,
};

fn palette_project() -> diagramme_schema::ProjectState {
    load_dxf_export_test_fixture()
}

fn scene_for(project: &diagramme_schema::ProjectState) -> diagramme_scene::Scene {
    build_scene(&active_sheet(project).state)
}

#[test]
fn build_scene_emits_junction_row_lines() {
    let project = palette_project();
    let scene = scene_for(&project);
    let junction = find_node(&project, "junction");
    let row_count = junction
        .data
        .get("rowCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let row_top_y = junction.position.y + diagramme_geometry::JUNCTION_BOX_BODY_TOP_PX;
    let row_bottom_y = row_top_y + row_count as f64 * diagramme_geometry::PATCH_GRID_ROW_PX;
    let footer_y = row_bottom_y;
    let inset = diagramme_geometry::SCHEMATIC_FRAME_INSET_PX;
    let inner_w = PATCH_PANEL_WIDTH_PX - 2.0 * inset;
    let ys: Vec<f64> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline {
                points,
                closed: false,
                ..
            } if points.len() == 2
                && (points[0].y - points[1].y).abs() < 1e-9
                && (points[1].x - points[0].x - inner_w).abs() < 1e-9
                && (points[0].x - (junction.position.x + inset)).abs() < 1e-9
                && points[0].y >= row_top_y
                && points[0].y < row_bottom_y =>
            {
                Some(points[0].y)
            }
            _ => None,
        })
        .collect();
    assert!(
        !ys.iter().any(|y| (*y - footer_y).abs() < 1e-9),
        "junction should not emit footer spacer line at y={footer_y}, got ys={ys:?}"
    );
    assert_eq!(
        ys.len(),
        row_count,
        "expected exactly {row_count} junction row lines, got {} at ys={ys:?}",
        ys.len()
    );
}

#[test]
fn build_scene_emits_speaker_symbol_polyline() {
    let project = load_golden_fixture();
    let scene = scene_for(&project);
    let has_closed = scene.primitives.iter().any(|p| {
        matches!(
            p,
            ScenePrimitive::Polyline {
                closed: true,
                points,
                ..
            } if points.len() == 6
        )
    });
    assert!(has_closed, "speaker block should emit closed 6-point outline");
}

#[test]
fn build_scene_emits_mic_circle_polyline() {
    let project = palette_project();
    let scene = scene_for(&project);
    let _mic = find_node(&project, "micBlock");
    let has_circle = scene.primitives.iter().any(|p| {
        matches!(
            p,
            ScenePrimitive::Polyline {
                closed: true,
                points,
                ..
            } if points.len() == 32
        )
    });
    assert!(has_circle, "mic block should emit 32-segment circle polyline");
}

#[test]
fn build_scene_emits_volume_control_hex_and_vc_text() {
    let project = palette_project();
    let scene = scene_for(&project);
    let has_hex = scene.primitives.iter().any(|p| {
        matches!(
            p,
            ScenePrimitive::Polyline {
                closed: true,
                points,
                ..
            } if points.len() == 6
        )
    });
    let has_vc = scene
        .primitives
        .iter()
        .any(|p| matches!(p, ScenePrimitive::Text(t) if t.content == "VC"));
    assert!(has_hex, "volume control hex");
    assert!(has_vc, "volume control VC label");
}

#[test]
fn build_scene_emits_text_block_frame_and_body() {
    let project = palette_project();
    let scene = scene_for(&project);
    let frame_lines = scene
        .primitives
        .iter()
        .filter(|p| matches!(p, ScenePrimitive::Polyline { points, closed: false, .. } if points.len() == 2))
        .count();
    let body = scene
        .primitives
        .iter()
        .any(|p| matches!(p, ScenePrimitive::Text(t) if t.content.contains("ANNOTATION")));
    assert!(frame_lines >= 4, "text block inset frame");
    assert!(body, "text block body text");
}

#[test]
fn build_scene_emits_flyoff_inkfill_and_uppercase_text() {
    let project = load_golden_fixture();
    let flyoff = find_first_node(&project, "flyoffNote").expect("fixture has flyoffNote");
    let expected = flyoff
        .data
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    let scene = scene_for(&project);
    let has_fill = scene.primitives.iter().any(|p| {
        matches!(p, ScenePrimitive::Solid { layer, .. } if layer == "INKFILL")
    });
    let has_flyoff = scene
        .primitives
        .iter()
        .any(|p| matches!(p, ScenePrimitive::Text(t) if t.content == expected));
    assert!(has_fill, "flyoff triangle solid");
    assert!(has_flyoff, "flyoff uppercase text {expected:?}");
}

#[test]
fn build_scene_emits_antenna_lines_and_label() {
    let project = load_golden_fixture();
    let scene = scene_for(&project);
    let ant = find_first_node(&project, "antennaReceiverSymbol")
        .or_else(|| find_first_node(&project, "antennaTransmitterSymbol"))
        .expect("fixture has antenna symbol");
    let expected_label = ant
        .data
        .get("line1")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    let nx = ant.position.x;
    let ny = ant.position.y;
    let w = ant.width.unwrap_or(43.0);
    let h = ant.height.unwrap_or(20.0);
    let lines_at_ant = scene
        .primitives
        .iter()
        .filter(|p| {
            matches!(
                p,
                ScenePrimitive::Polyline { points, closed: false, .. }
                if points.len() == 2
                    && points.iter().any(|pt| {
                        pt.x >= nx && pt.x <= nx + w && pt.y >= ny && pt.y <= ny + h
                    })
            )
        })
        .count();
    let label = scene
        .primitives
        .iter()
        .any(|p| matches!(p, ScenePrimitive::Text(t) if t.content == expected_label));
    assert!(lines_at_ant >= 4, "antenna mast + arms");
    assert!(label, "antenna label {expected_label:?}");
}

#[test]
fn build_scene_emits_grouping_zone_guides_dashes() {
    let project = load_golden_fixture();
    let scene = scene_for(&project);
    let guides = scene
        .primitives
        .iter()
        .filter(|p| {
            matches!(
                p,
                ScenePrimitive::Polyline { layer, closed: false, .. } if layer == "GUIDES"
            )
        })
        .count();
    assert!(guides >= 4, "grouping zones should emit GUIDES dashes");
}

#[test]
fn schematic_node_bounds_constants_match_scene() {
    let project = load_golden_fixture();
    let sheet = active_sheet(&project);

    let speaker = find_node(&project, "speakerBlock");
    let bounds = diagramme_geometry::speaker_block_bounds(speaker);
    assert!((bounds.width - SPEAKER_BLOCK_DEFAULT_WIDTH_PX).abs() < 1e-9);
    assert!((bounds.height - MIC_SPEAKER_FRAME_HEIGHT_PX).abs() < 1e-9);

    if find_first_node(&project, "junction").is_some() {
        let junction = find_node(&project, "junction");
        let j_bounds = diagramme_geometry::junction_bounds(junction);
        assert!((j_bounds.width - PATCH_PANEL_WIDTH_PX).abs() < 1e-9);
    }

    if find_first_node(&project, "volumeControl").is_some() {
        let vc = find_node(&project, "volumeControl");
        let vc_bounds = diagramme_geometry::volume_control_bounds(vc);
        assert!((vc_bounds.y - (vc.position.y + VC_SYMBOL_TOP_INSET_PX)).abs() < 1e-9);
        assert!((vc_bounds.height - VOLUME_CONTROL_FRAME_HEIGHT_PX).abs() < 1e-9);
    }

    let flyoff = find_node(&project, "flyoffNote");
    let f_bounds = diagramme_geometry::flyoff_note_bounds(flyoff);
    assert!((f_bounds.height - FLYOFF_TRI_H).abs() < 1e-9);

    let _ = sheet.state.nodes.len();
}
