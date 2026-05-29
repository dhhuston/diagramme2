use diagramme_schema::{
    node_type_from_filename, parse_preset_file_text, serialize_preset_file, PresetNodeType,
    PRESET_SCHEMA_VERSION,
};
use serde_json::json;

#[test]
fn parses_avdevice_preset() {
    let text = r#"{
  "schemaVersion": 1,
  "name": "My Amp",
  "data": { "description": "Amplifier", "tagCode": "AMP" }
}"#;
    let parsed = parse_preset_file_text(text, "rack.avdevice").expect("parse avdevice");
    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.name, "My Amp");
    assert_eq!(parsed.node_type, PresetNodeType::DeviceV2);
    assert_eq!(
        parsed.data.get("description").and_then(|v| v.as_str()),
        Some("Amplifier")
    );
}

#[test]
fn parses_plate_preset_with_default_name() {
    let text = r#"{
  "schemaVersion": 1,
  "data": { "plateLabel": "HDMI" }
}"#;
    let parsed = parse_preset_file_text(text, "hdmi-input.plate").expect("parse plate");
    assert_eq!(parsed.name, "hdmi-input");
    assert_eq!(parsed.node_type, PresetNodeType::AvPlate);
}

#[test]
fn rejects_unsupported_schema_version() {
    let text = r#"{
  "schemaVersion": 99,
  "data": {}
}"#;
    let err = parse_preset_file_text(text, "x.avdevice").unwrap_err();
    assert!(err.contains("Unsupported preset schemaVersion 99"), "{err}");
}

#[test]
fn rejects_missing_schema_version() {
    let text = r#"{ "data": {} }"#;
    let err = parse_preset_file_text(text, "x.avdevice").unwrap_err();
    assert!(err.contains("schemaVersion"), "{err}");
}

#[test]
fn rejects_kind_extension_mismatch() {
    let text = r#"{
  "schemaVersion": 1,
  "kind": "avPlate",
  "data": {}
}"#;
    let err = parse_preset_file_text(text, "x.avdevice").unwrap_err();
    assert!(err.contains("extension implies deviceV2"), "{err}");
}

#[test]
fn strips_ephemeral_fields_on_parse() {
    let text = r#"{
  "schemaVersion": 1,
  "data": {
    "description": "Keep",
    "wireCategory": "audio",
    "presetId": "ghost"
  }
}"#;
    let parsed = parse_preset_file_text(text, "dev.avdevice").unwrap();
    assert!(parsed.data.get("description").is_some());
    assert!(parsed.data.get("wireCategory").is_none());
    assert!(parsed.data.get("presetId").is_none());
}

#[test]
fn node_type_from_filename_matches_extensions() {
    assert_eq!(
        node_type_from_filename("path/to/foo.avdevice").unwrap(),
        PresetNodeType::DeviceV2
    );
    assert_eq!(
        node_type_from_filename("bar.plate").unwrap(),
        PresetNodeType::AvPlate
    );
    assert!(node_type_from_filename("readme.txt").is_err());
}

#[test]
fn serialize_roundtrip() {
    let mut data = serde_json::Map::new();
    data.insert("description".into(), json!("Test device"));
    let text = serialize_preset_file("Test", PresetNodeType::DeviceV2, data).unwrap();
    let parsed = parse_preset_file_text(&text, "test.avdevice").unwrap();
    assert_eq!(parsed.name, "Test");
    assert_eq!(parsed.schema_version, PRESET_SCHEMA_VERSION);
}
