use diagramme_schema::{validate_diagram_envelope, DIAGRAMME_FORMAT, DIAGRAMME_VERSION};

#[test]
fn accepts_v6_envelope() {
    let value = serde_json::json!({
        "format": DIAGRAMME_FORMAT,
        "version": DIAGRAMME_VERSION,
        "sheets": [],
        "activeSheetId": "main",
    });
    validate_diagram_envelope(&value).expect("valid envelope");
}

#[test]
fn rejects_missing_format() {
    let value = serde_json::json!({ "version": 2, "sheets": [] });
    let err = validate_diagram_envelope(&value).unwrap_err();
    assert!(err.contains("format"), "{err}");
}

#[test]
fn rejects_wrong_format() {
    let value = serde_json::json!({
        "format": "other",
        "version": 2,
        "sheets": [],
    });
    let err = validate_diagram_envelope(&value).unwrap_err();
    assert!(err.contains("Unsupported diagram format"), "{err}");
}

#[test]
fn rejects_unknown_version() {
    let value = serde_json::json!({
        "format": DIAGRAMME_FORMAT,
        "version": 1,
        "sheets": [],
    });
    let err = validate_diagram_envelope(&value).unwrap_err();
    assert!(err.contains("Unsupported diagram version 1"), "{err}");
}

#[test]
fn golden_fixture_passes_envelope() {
    let value: serde_json::Value =
        serde_json::from_str(diagramme_schema::GOLDEN_DIAGRAM_JSON).unwrap();
    validate_diagram_envelope(&value).expect("golden Comp Gym envelope");
}
