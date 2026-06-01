use diagramme_schema::{active_sheet_state, validate_diagram_envelope, ProjectState};

#[test]
fn open_golden_fixture() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    validate_diagram_envelope(&value).expect("golden envelope");
    let p = diagramme_schema::load_golden_fixture();
    assert!(!p.sheets.is_empty());
    let again = serde_json::to_string(&p).unwrap();
    let p2: ProjectState = serde_json::from_str(&again).unwrap();
    assert_eq!(p.active_sheet_id, p2.active_sheet_id);
}

#[test]
fn open_cafeteria_fixture() {
    let json = include_str!("../../../../fixtures/diagrams/cafeteria-d104a.diagramme");
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    validate_diagram_envelope(&value).expect("cafeteria envelope");
    let p: ProjectState = serde_json::from_str(json).expect("parse cafeteria fixture");
    assert_eq!(p.sheets.len(), 2);
    assert!(
        p.sheets
            .iter()
            .any(|s| s.id == "sheet-split-face-demo" && s.name == "Split face demo")
    );
}

#[test]
fn open_split_face_demo_fixture() {
    let json = include_str!("../../../../fixtures/diagrams/split-face-demo.diagramme");
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    validate_diagram_envelope(&value).expect("split face demo envelope");
    let p: ProjectState = serde_json::from_str(json).expect("parse split face demo fixture");
    assert_eq!(p.sheets.len(), 1);
    assert_eq!(p.active_sheet_id, "sheet-split-face-demo");
    assert_eq!(active_sheet_state(&p).nodes.len(), 5);
}

#[test]
fn project_state_default() {
    let p = ProjectState::default();
    assert_eq!(p.sheets.len(), 1);
    assert_eq!(p.sheets[0].name, "Main");
}
