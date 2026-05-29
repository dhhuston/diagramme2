use app_lib::commands::{
    export_revit_dxf_for_state, open_diagram_from_json, save_diagram_compact_from, test_app_state,
};
use diagramme_schema::{validate_diagram_envelope, active_sheet_state, load_golden_fixture};

const FIXTURES: &[(&str, &str)] = &[
    (
        "comp-gym",
        include_str!("../../fixtures/golden/Comp Gym F102A.diagramme"),
    ),
    (
        "cafeteria",
        include_str!("../../fixtures/diagrams/cafeteria-d104a.diagramme"),
    ),
    (
        "dxf-export-test",
        include_str!("../../fixtures/diagrams/dxf-export-test.diagramme"),
    ),
];

#[test]
fn v6_fixtures_open_with_valid_envelope() {
    for (name, json) in FIXTURES {
        let value: serde_json::Value = serde_json::from_str(json).expect("json parse");
        validate_diagram_envelope(&value).unwrap_or_else(|e| panic!("{name}: envelope: {e}"));
        open_diagram_from_json(json).unwrap_or_else(|e| panic!("{name}: open: {e}"));
    }
}

#[test]
fn v6_fixtures_round_trip_save_compact() {
    for (name, json) in FIXTURES {
        let project = open_diagram_from_json(json).unwrap_or_else(|e| panic!("{name}: open: {e}"));
        let compact =
            save_diagram_compact_from(&project).unwrap_or_else(|e| panic!("{name}: save: {e}"));
        let roundtripped =
            open_diagram_from_json(&compact).unwrap_or_else(|e| panic!("{name}: re-open: {e}"));
        assert_eq!(
            project.active_sheet_id, roundtripped.active_sheet_id,
            "{name}: active sheet id"
        );
        assert_eq!(
            project.active_sheet().state.nodes.len(),
            roundtripped.active_sheet().state.nodes.len(),
            "{name}: node count"
        );
        assert_eq!(
            project.active_sheet().state.edges.len(),
            roundtripped.active_sheet().state.edges.len(),
            "{name}: edge count"
        );
    }
}

#[test]
fn v6_fixtures_dxf_export_smoke() {
    for (name, json) in FIXTURES {
        let project = open_diagram_from_json(json).unwrap_or_else(|e| panic!("{name}: open: {e}"));
        let state = test_app_state(project);
        let dxf = export_revit_dxf_for_state(&state).unwrap_or_else(|e| panic!("{name}: dxf: {e}"));
        assert!(
            dxf.len() > 500,
            "{name}: expected substantial DXF, got {} chars",
            dxf.len()
        );
        assert!(dxf.contains("SECTION"), "{name}: missing DXF SECTION");
    }
}

#[test]
fn save_compact_strips_stale_inner_corners_on_comp_gym() {
    let project = load_golden_fixture();
    let before = active_sheet_state(&project)
        .edges
        .iter()
        .filter(|e| {
            e.data
                .get("innerCorners")
                .and_then(|v| v.as_array())
                .is_some_and(|a| !a.is_empty())
        })
        .count();
    assert!(before > 0, "golden fixture should carry stale innerCorners");

    let compact = save_diagram_compact_from(&project).expect("save compact");
    let roundtripped = open_diagram_from_json(&compact).expect("re-open saved");
    let with_corners = roundtripped
        .active_sheet()
        .state
        .edges
        .iter()
        .filter(|e| {
            e.data
                .get("innerCorners")
                .and_then(|v| v.as_array())
                .is_some_and(|a| !a.is_empty())
        })
        .count();
    assert_eq!(with_corners, 0, "save should strip innerCorners");
}

#[test]
fn open_rejects_wrong_version() {
    let mut value: serde_json::Value =
        serde_json::from_str(diagramme_schema::GOLDEN_DIAGRAM_JSON).unwrap();
    value["version"] = serde_json::json!(99);
    let json = value.to_string();
    let err = open_diagram_from_json(&json).unwrap_err();
    assert!(err.contains("Unsupported diagram version 99"), "{err}");
}
