use app_lib::commands::{
    apply_move_node, open_diagram_from_json, save_diagram_compact_from, test_app_state,
};
use diagramme_schema::XY;

#[test]
fn open_move_save_compact_roundtrip() {
    let json = include_str!("../../fixtures/diagrams/dxf-export-test.diagramme");
    let mut project = open_diagram_from_json(json).expect("parse dxf-export-test fixture");
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
fn app_state_open_and_get_project() {
    let json = include_str!("../../fixtures/diagrams/dxf-export-test.diagramme");
    let project = open_diagram_from_json(json).unwrap();
    let state = test_app_state(project.clone());
    let loaded = state.0.lock().unwrap().clone();
    assert_eq!(loaded.active_sheet_id, project.active_sheet_id);
    assert_eq!(
        loaded.active_sheet().state.nodes.len(),
        project.active_sheet().state.nodes.len()
    );
}
