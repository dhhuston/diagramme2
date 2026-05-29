use diagramme_geometry::{wiretag_export_width_px, WIRETAG_BAR_HEIGHT_PX};
use diagramme_scene::{
    build_scene, resolve_pair_main_display_text, wiretag_scene_bounds, ScenePrimitive,
};
use diagramme_schema::{active_sheet, first_wiretag_end_a, load_golden_fixture};

#[test]
fn build_scene_produces_wiretag_hull_and_text() {
    let project = load_golden_fixture();
    let sheet = active_sheet(&project);
    let wiretag = first_wiretag_end_a(&project).expect("fixture has end-a wiretag");
    let scene = build_scene(&sheet.state);

    let closed_polylines: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline { points, closed, .. } if *closed && points.len() >= 7 => {
                Some(points.len())
            }
            _ => None,
        })
        .collect();
    assert!(
        !closed_polylines.is_empty(),
        "expected wiretag closed hull polyline (7+ vertices)"
    );

    let pair_index = wiretag
        .data
        .get("pairIndex")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .unwrap_or(0);
    let idx = pair_index.to_string();
    let main_raw = resolve_pair_main_display_text(wiretag, &sheet.state.nodes, &sheet.state.edges);
    let main = if main_raw.trim().is_empty() {
        format!("WT-{pair_index}")
    } else {
        main_raw.trim().to_uppercase()
    };

    let text_contents: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        text_contents.contains(&main.as_str()),
        "expected main tag text {main:?}, got {:?}",
        text_contents
    );
    assert!(
        text_contents.contains(&idx.as_str()),
        "expected pair index text {idx:?}, got {:?}",
        text_contents
    );

    let has_hit = scene.hits.iter().any(|h| h.id == wiretag.id);
    assert!(has_hit, "expected wiretag body hit target");

    let show_sheet = wiretag
        .data
        .get("showSheetName")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sheet_name = if show_sheet {
        wiretag
            .data
            .get("sheetName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    } else {
        ""
    };

    let bounds = wiretag_scene_bounds(wiretag, &sheet.state.nodes, &sheet.state.edges);
    assert!(
        bounds.width > 30.0,
        "wiretag hull should autosize from text, got width {}",
        bounds.width
    );
    assert!(
        (bounds.width
            - wiretag_export_width_px(
                pair_index,
                &main,
                sheet_name,
                show_sheet,
                WIRETAG_BAR_HEIGHT_PX,
            ))
        .abs()
            < 1e-9,
        "wiretag bounds width should match export autosize formula"
    );
}

#[test]
fn wiretag_emits_index_column_divider() {
    let project = load_golden_fixture();
    let scene = build_scene(&active_sheet(&project).state);

    let open_polylines: Vec<_> = scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline { points, closed, .. } if !*closed && points.len() == 2 => {
                Some(points.clone())
            }
            _ => None,
        })
        .collect();
    assert!(
        open_polylines.len() >= 1,
        "expected wiretag index divider line"
    );
}
