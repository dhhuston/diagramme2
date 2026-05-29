//! `.avdevice` / `.plate` preset file parsing (v6-compatible).
//!
//! Wire into Tauri preset IPC when file import/export commands land.

use serde_json::{Map, Value};

pub const PRESET_SCHEMA_VERSION: i64 = 1;
pub const AVDEVICE_EXT: &str = "avdevice";
pub const PLATE_EXT: &str = "plate";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetNodeType {
    DeviceV2,
    AvPlate,
}

impl PresetNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeviceV2 => "deviceV2",
            Self::AvPlate => "avPlate",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPresetFile {
    pub schema_version: i64,
    pub name: String,
    pub node_type: PresetNodeType,
    pub data: Map<String, Value>,
}

fn basename(path_or_name: &str) -> String {
    path_or_name
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or(path_or_name)
        .to_string()
}

fn strip_extension(base: &str) -> String {
    let lower = base.to_lowercase();
    let av_suffix = format!(".{AVDEVICE_EXT}");
    let plate_suffix = format!(".{PLATE_EXT}");
    if lower.ends_with(&av_suffix) {
        return base[..base.len() - av_suffix.len()].to_string();
    }
    if lower.ends_with(&plate_suffix) {
        return base[..base.len() - plate_suffix.len()].to_string();
    }
    base.to_string()
}

/// Infer preset node type from filename extension (`.avdevice` or `.plate`).
pub fn node_type_from_filename(filename: &str) -> Result<PresetNodeType, String> {
    let base = basename(filename).to_lowercase();
    if base.ends_with(&format!(".{AVDEVICE_EXT}")) {
        return Ok(PresetNodeType::DeviceV2);
    }
    if base.ends_with(&format!(".{PLATE_EXT}")) {
        return Ok(PresetNodeType::AvPlate);
    }
    Err(format!(
        "Expected .{AVDEVICE_EXT} or .{PLATE_EXT} file: {filename}"
    ))
}

/// Strip runtime-only node data fields before embedding or serializing a preset.
pub fn strip_ephemeral_node_data(data: Map<String, Value>) -> Map<String, Value> {
    const EPHEMERAL: &[&str] = &[
        "wireCategory",
        "wireCategoryMismatch",
        "proximityViolation",
        "violationReason",
        "bundledLeft",
        "bundledRight",
        "embeddedPresetId",
        "presetLibraryId",
        "presetId",
        "sourceBasename",
    ];

    let mut out: Map<String, Value> = data
        .into_iter()
        .filter(|(k, _)| !EPHEMERAL.contains(&k.as_str()))
        .collect();

    for key in ["leftColumn", "rightColumn", "groups"] {
        if let Some(Value::Array(groups)) = out.get_mut(key) {
            for group in groups.iter_mut() {
                if let Some(obj) = group.as_object_mut() {
                    obj.remove("bundledRowIds");
                }
            }
        }
    }

    out
}

/// Parse preset JSON text (v6 [`avPresetFormat.ts`](https://github.com) parity).
pub fn parse_preset_file_text(text: &str, filename_for_type: &str) -> Result<ParsedPresetFile, String> {
    let node_type = node_type_from_filename(filename_for_type)?;

    let parsed: Value = serde_json::from_str(text)
        .map_err(|e| format!("Invalid JSON in preset file: {e}"))?;

    let obj = parsed
        .as_object()
        .ok_or_else(|| "Preset file must be a JSON object".to_string())?;

    let schema_version = obj
        .get("schemaVersion")
        .and_then(|v| v.as_i64())
        .filter(|v| *v >= 1)
        .ok_or_else(|| "Preset file must include a positive \"schemaVersion\"".to_string())?;
    if schema_version > PRESET_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported preset schemaVersion {schema_version} (max {PRESET_SCHEMA_VERSION})"
        ));
    }

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| strip_extension(&basename(filename_for_type)));

    let data_raw = obj
        .get("data")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "Preset file must include an object \"data\" field".to_string())?;

    if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
        if (kind == "deviceV2" || kind == "avPlate") && kind != node_type.as_str() {
            return Err(format!(
                "File extension implies {} but \"kind\" is {kind}",
                node_type.as_str()
            ));
        }
    }

    let data = strip_ephemeral_node_data(data_raw.clone());

    Ok(ParsedPresetFile {
        schema_version,
        name,
        node_type,
        data,
    })
}

/// Serialize a preset file (pretty JSON, trailing newline).
pub fn serialize_preset_file(
    name: &str,
    _node_type: PresetNodeType,
    data: Map<String, Value>,
) -> Result<String, String> {
    let cleaned = strip_ephemeral_node_data(data);
    let payload = serde_json::json!({
        "schemaVersion": PRESET_SCHEMA_VERSION,
        "name": name,
        "data": cleaned,
    });
    serde_json::to_string_pretty(&payload)
        .map(|s| format!("{s}\n"))
        .map_err(|e| e.to_string())
}
