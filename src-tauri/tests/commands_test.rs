use app_lib::commands::{
    apply_move_node, apply_move_node_preview_then_commit, export_revit_dxf_for_state,
    get_diagram_scene_for_state, open_diagram_from_json, save_diagram_compact_from,
    test_app_state, undo_active_sheet,
};
use diagramme_schema::{DiagramState, XY};
use diagramme_scene::ScenePrimitive;

fn wire_polyline_count(scene: &diagramme_scene::Scene) -> usize {
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
        .count()
}

#[test]
fn open_move_save_compact_roundtrip() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let mut project = open_diagram_from_json(json).expect("parse golden fixture");
    assert!(!project.sheets.is_empty());

    let node_id = project.active_sheet().state.nodes[0].id.clone();
    let original_x = project.active_sheet().state.nodes[0].position.x;
    let new_position = XY {
        x: original_x + 42.0,
        y: 999.0,
    };

    apply_move_node(&mut project, &node_id, new_position);

    let compact = save_diagram_compact_from(&project).expect("save compact");
    let roundtripped = open_diagram_from_json(&compact).expect("re-open compact save");
    let moved = roundtripped
        .active_sheet()
        .state
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .expect("node survives roundtrip");

    assert_eq!(moved.position.x, new_position.x);
    assert_eq!(moved.position.y, new_position.y);
}

#[test]
fn save_compact_omits_undo_history() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let mut project = open_diagram_from_json(json).expect("parse golden fixture");
    let node_id = project.active_sheet().state.nodes[0].id.clone();
    apply_move_node(&mut project, &node_id, XY { x: 1.0, y: 2.0 });
    project.active_sheet_mut().undo_stack.push_back(DiagramState::default());

    let compact = save_diagram_compact_from(&project).expect("save compact");
    assert!(
        !compact.contains("undo_stack"),
        "saved files must not embed undo history"
    );
}

#[test]
fn undo_restores_node_move_after_preview_commit() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let mut project = open_diagram_from_json(json).expect("parse golden fixture");
    let node_id = project.active_sheet().state.nodes[0].id.clone();
    let original = project.active_sheet().state.nodes[0].position;
    let moved = XY {
        x: original.x + 24.0,
        y: original.y + 12.0,
    };

    apply_move_node_preview_then_commit(&mut project, &node_id, moved);
    assert_eq!(project.active_sheet().state.nodes[0].position, moved);

    let restored = undo_active_sheet(&mut project);
    let node = restored
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .expect("node still present");
    assert_eq!(node.position, original);
}

#[test]
fn open_diagram_strips_embedded_undo_history() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let project = open_diagram_from_json(json).expect("parse golden fixture");
    let mut payload: serde_json::Value =
        serde_json::from_str(&save_diagram_compact_from(&project).unwrap()).unwrap();
    if let Some(sheet) = payload
        .get_mut("sheets")
        .and_then(|s| s.as_array_mut())
        .and_then(|s| s.first_mut())
    {
        sheet["undo_stack"] = serde_json::json!([{ "nodes": [], "edges": [] }]);
        sheet["redo_depth"] = serde_json::json!(1);
    }
    let with_history = serde_json::to_string(&payload).unwrap();
    assert!(with_history.contains("undo_stack"));

    let opened = open_diagram_from_json(&with_history).expect("re-open with embedded history");
    assert!(opened.active_sheet().undo_stack.is_empty());
    assert_eq!(opened.active_sheet().redo_depth, 0);
}

#[test]
fn export_uses_same_scene_wires_as_get_diagram_scene() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let project = open_diagram_from_json(json).unwrap();
    let state = test_app_state(project);
    let scene = get_diagram_scene_for_state(&state);
    let wires = wire_polyline_count(&scene);
    assert!(wires > 0, "canvas scene should include wire polylines");
    let dxf = export_revit_dxf_for_state(&state).expect("export dxf");
    assert!(
        dxf.contains("WIRES"),
        "DXF export should include WIRES layer from the same scene build"
    );
}

#[test]
fn app_state_open_and_get_project() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let project = open_diagram_from_json(json).unwrap();
    let state = test_app_state(project.clone());
    let loaded = state.0.lock().unwrap().clone();
    assert_eq!(loaded.active_sheet_id, project.active_sheet_id);
    assert_eq!(
        loaded.active_sheet().state.nodes.len(),
        project.active_sheet().state.nodes.len()
    );
}
