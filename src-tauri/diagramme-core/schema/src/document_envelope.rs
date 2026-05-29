//! On-disk `.diagramme` document envelope (`format` + `version`).

pub const DIAGRAMME_FORMAT: &str = "diagramme";
pub const DIAGRAMME_VERSION: i64 = 2;

/// Validate the top-level envelope before deserializing into [`ProjectState`](crate::ProjectState).
pub fn validate_diagram_envelope(value: &serde_json::Value) -> Result<(), String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "Diagram file must be a JSON object".to_string())?;

    let format = obj
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Diagram file must include a string \"format\" field".to_string())?;
    if format != DIAGRAMME_FORMAT {
        return Err(format!(
            "Unsupported diagram format \"{format}\" (expected \"{DIAGRAMME_FORMAT}\")"
        ));
    }

    let version = obj
        .get("version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "Diagram file must include a numeric \"version\" field".to_string())?;
    if version != DIAGRAMME_VERSION {
        return Err(format!(
            "Unsupported diagram version {version} (expected {DIAGRAMME_VERSION})"
        ));
    }

    Ok(())
}
