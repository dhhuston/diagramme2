//! Serializable diagram types aligned with React Flow node/edge JSON (`#[serde(rename = ...)]`
//! where RF uses camelCase). `DiagramState` is the persistence unit for `save_diagram` /
//! `open_diagram`.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Diagram-space position in pixels (same convention as React Flow).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}

/// React Flow node shape: `type` and optional measured `width`/`height`/`zIndex`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: XY,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(rename = "zIndex", skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
}

/// Schematic connection; `data` holds React Flow-facing fields such as handle centers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(
        rename = "sourceHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_handle: Option<String>,
    #[serde(
        rename = "targetHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub target_handle: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
    #[serde(default = "empty_object")]
    pub data: serde_json::Value,
}

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Authoritative diagram snapshot (nodes and edges).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagramState {
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

/// A single tab/area within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub id: String,
    pub name: String,
    /// Current diagram snapshot — the only sheet payload written to `.diagramme` files.
    pub state: DiagramState,
    /// Session-only undo stack (never persisted; cleared on open/save).
    #[serde(default, skip)]
    pub undo_stack: VecDeque<DiagramState>,
    /// Session-only redo cursor into `undo_stack`.
    #[serde(default, skip)]
    pub redo_depth: usize,
    /// Pre-preview diagram captured at the start of a drag preview gesture.
    #[serde(default, skip)]
    pub preview_baseline: Option<DiagramState>,
}

/// Single device or AV-plate template stored in the project for reuse (not a canvas node).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedPreset {
    pub id: String,
    pub name: String,
    #[serde(rename = "nodeType")]
    pub node_type: String,
    pub data: serde_json::Value,
    #[serde(rename = "sourceBasename", skip_serializing_if = "Option::is_none")]
    pub source_basename: Option<String>,
}

/// Full project state containing multiple sheets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub sheets: Vec<Sheet>,
    #[serde(rename = "activeSheetId")]
    pub active_sheet_id: String,
    /// User-imported device / plate presets stored with the project file.
    #[serde(rename = "presetLibrary", default)]
    pub preset_library: Vec<EmbeddedPreset>,
}

impl Default for ProjectState {
    fn default() -> Self {
        let full_id = uuid::Uuid::new_v4().to_string();
        let main_id = format!("sheet-{}", &full_id[..8]);
        Self {
            sheets: vec![Sheet {
                id: main_id.clone(),
                name: "Main".into(),
                state: DiagramState::default(),
                undo_stack: VecDeque::new(),
                redo_depth: 0,
                preview_baseline: None,
            }],
            active_sheet_id: main_id,
            preset_library: Vec::new(),
        }
    }
}

impl ProjectState {
    pub fn active_sheet_mut(&mut self) -> &mut Sheet {
        let id = &self.active_sheet_id;
        if !self.sheets.iter().any(|s| &s.id == id) {
            self.active_sheet_id = self.sheets[0].id.clone();
        }
        let id = &self.active_sheet_id;
        self.sheets.iter_mut().find(|s| &s.id == id).unwrap()
    }

    pub fn active_sheet(&self) -> &Sheet {
        let id = &self.active_sheet_id;
        self.sheets
            .iter()
            .find(|s| &s.id == id)
            .unwrap_or_else(|| &self.sheets[0])
    }
}

#[derive(Debug, Deserialize)]
pub struct NodeDimension {
    pub id: String,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<XY>,
}
