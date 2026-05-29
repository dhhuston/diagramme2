use app_lib::commands::{export_revit_dxf_for_state, open_diagram_from_json, test_app_state};
#[test]
fn export_revit_dxf_returns_substantial_dxf() {
    let json = diagramme_schema::GOLDEN_DIAGRAM_JSON;
    let project = open_diagram_from_json(json).expect("parse golden fixture");
    let state = test_app_state(project);

    let dxf = export_revit_dxf_for_state(&state).expect("export revit dxf");
    assert!(
        dxf.len() > 1000,
        "expected substantial DXF output, got {} chars",
        dxf.len()
    );
}
