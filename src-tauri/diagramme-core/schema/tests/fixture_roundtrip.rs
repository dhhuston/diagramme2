use diagramme_schema::ProjectState;

#[test]
fn open_dxf_export_test_fixture() {
    let json = include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");
    let p: ProjectState = serde_json::from_str(json).expect("parse");
    assert!(!p.sheets.is_empty());
    let again = serde_json::to_string(&p).unwrap();
    let p2: ProjectState = serde_json::from_str(&again).unwrap();
    assert_eq!(p.active_sheet_id, p2.active_sheet_id);
}

#[test]
fn open_cafeteria_fixture() {
    let json = include_str!("../../../../fixtures/diagrams/cafeteria-d104a.diagramme");
    let p: ProjectState = serde_json::from_str(json).expect("parse cafeteria fixture");
    assert!(!p.sheets.is_empty());
}

#[test]
fn project_state_default() {
    let p = ProjectState::default();
    assert_eq!(p.sheets.len(), 1);
    assert_eq!(p.sheets[0].name, "Main");
}
