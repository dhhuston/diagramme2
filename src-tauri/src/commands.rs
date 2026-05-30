//! Tauri IPC handlers: read/write [`DiagramState`] and return the full state after each
//! mutation so the UI can replace local nodes/edges via merge helpers.
//!
//! `move_node` and `update_dims` accept optional `EdgeHandleAttachmentUpdate` payloads so the
//! frontend can persist handle centers used by straight schematic edges.

use crate::close_gate::AllowNextClose;
use crate::debug_channel;
use crate::state::AppState;
use diagramme_dxf::build_revit_dxf_from_diagram;
use diagramme_wires::apply_node_move_geometry;
use diagramme_scene::{build_scene, Scene};
use diagramme_schema::{
    validate_diagram_envelope, DiagramState, EmbeddedPreset, Node, NodeDimension, ProjectState,
    Sheet, XY, normalize_project_for_persist,
};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

const HISTORY_LIMIT: usize = 50;

fn emit_debug(app: &AppHandle) {
    let msgs = debug_channel::drain();
    if !msgs.is_empty() {
        let _ = app.emit("diagramme://debug", msgs);
    }
}

fn push_history(sheet: &mut Sheet) {
    let snapshot = sheet.state.clone();
    if sheet.redo_depth > 0 {
        let keep = sheet.undo_stack.len() - sheet.redo_depth;
        sheet.undo_stack.truncate(keep);
        sheet.redo_depth = 0;
    }
    sheet.undo_stack.push_back(snapshot);
    if sheet.undo_stack.len() > HISTORY_LIMIT {
        sheet.undo_stack.pop_front();
    }
}

fn mutate_active_diagram(
    state: &State<'_, AppState>,
    app: &AppHandle,
    mutate: impl FnOnce(&mut DiagramState),
) -> DiagramState {
    let mut project = state.0.lock().unwrap();
    let sheet = project.active_sheet_mut();
    push_history(sheet);
    mutate(&mut sheet.state);
    let result = sheet.state.clone();
    drop(project);
    emit_debug(app);
    result
}

/// Like `mutate_active_diagram` but does not record a history entry.
fn mutate_active_diagram_no_history(
    state: &State<'_, AppState>,
    app: &AppHandle,
    mutate: impl FnOnce(&mut DiagramState),
) -> DiagramState {
    let mut project = state.0.lock().unwrap();
    let diagram = &mut project.active_sheet_mut().state;
    mutate(diagram);
    let result = diagram.clone();
    drop(project);
    emit_debug(app);
    result
}

fn maybe_apply_edge_handle_attachments(
    state: &mut DiagramState,
    updates: Option<Vec<EdgeHandleAttachmentUpdate>>,
) {
    if let Some(updates) = updates.as_ref() {
        apply_edge_handle_attachments(state, updates);
    }
}

/// Per-edge UI snapshot of handle centers in diagram space (camelCase keys match TS).
#[derive(Debug, serde::Deserialize)]
pub struct EdgeHandleAttachmentUpdate {
    #[serde(rename = "edgeId")]
    pub edge_id: String,
    #[serde(rename = "sourceHandleCenter")]
    pub source_handle_center: Option<XY>,
    #[serde(rename = "targetHandleCenter")]
    pub target_handle_center: Option<XY>,
}

fn apply_edge_handle_attachments(s: &mut DiagramState, updates: &[EdgeHandleAttachmentUpdate]) {
    for u in updates {
        let Some(edge) = s.edges.iter_mut().find(|e| e.id == u.edge_id) else {
            continue;
        };
        let mut obj = edge.data.as_object().cloned().unwrap_or_default();
        if let Some(c) = u.source_handle_center {
            obj.insert(
                "sourceHandleCenter".into(),
                serde_json::to_value(c).unwrap_or(serde_json::Value::Null),
            );
        }
        if let Some(c) = u.target_handle_center {
            obj.insert(
                "targetHandleCenter".into(),
                serde_json::to_value(c).unwrap_or(serde_json::Value::Null),
            );
        }
        edge.data = serde_json::Value::Object(obj);
    }
}

/// Parse project JSON (`.diagramme` document) and clear undo stacks (same as [`open_diagram`]).
pub fn open_diagram_from_json(json: &str) -> Result<ProjectState, String> {
    let parsed: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    validate_diagram_envelope(&parsed)?;

    let mut project: ProjectState =
        serde_json::from_value(parsed).map_err(|e| format!("project: {e}"))?;
    for sheet in &mut project.sheets {
        sheet.undo_stack.clear();
        sheet.redo_depth = 0;
    }
    Ok(project)
}

/// Compact JSON for dirty checks and baselines (same schema as [`save_diagram`]).
pub fn save_diagram_compact_from(project: &ProjectState) -> Result<String, String> {
    let mut normalized = project.clone();
    normalize_project_for_persist(&mut normalized);
    let payload = serde_json::json!({
        "format": "diagramme",
        "version": 2,
        "sheets": normalized.sheets,
        "activeSheetId": normalized.active_sheet_id,
        "presetLibrary": normalized.preset_library,
    });
    serde_json::to_string(&payload).map_err(|e| e.to_string())
}

// ─── State query ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> ProjectState {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> DiagramState {
    state.0.lock().unwrap().active_sheet().state.clone()
}

#[tauri::command]
pub fn get_diagram_scene(state: State<'_, AppState>) -> Scene {
    get_diagram_scene_for_state(state.inner())
}

#[tauri::command]
pub fn export_revit_dxf(state: State<'_, AppState>) -> Result<String, String> {
    export_revit_dxf_for_state(state.inner())
}

#[tauri::command]
pub fn set_state(app: AppHandle, state: State<'_, AppState>, next: DiagramState) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        *diagram = next;
    })
}

#[tauri::command]
pub fn sync_state(app: AppHandle, state: State<'_, AppState>, next: DiagramState) -> DiagramState {
    mutate_active_diagram_no_history(&state, &app, |diagram| {
        *diagram = next;
    })
}

// ─── Node mutations ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_node(app: AppHandle, state: State<'_, AppState>, node: Node) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        diagram.nodes.retain(|n| n.id != node.id);
        diagram.nodes.push(node);
    })
}

#[tauri::command]
pub fn update_node(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    data: serde_json::Value,
) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        if let Some(node) = diagram.nodes.iter_mut().find(|n| n.id == node_id) {
            node.data = data;
        }
    })
}

#[tauri::command]
pub fn replace_node_type(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    node_type: String,
    data: serde_json::Value,
) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        if let Some(node) = diagram.nodes.iter_mut().find(|n| n.id == node_id) {
            node.node_type = node_type;
            node.data = data;
        }
    })
}

#[tauri::command]
pub fn move_node(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: String,
    position: XY,
    handle_attachment_updates: Option<Vec<EdgeHandleAttachmentUpdate>>,
    is_drag_preview: Option<bool>,
) -> DiagramState {
    if is_drag_preview == Some(true) {
        let mut project = state.0.lock().unwrap();
        let diagram = &mut project.active_sheet_mut().state;
        apply_node_move_geometry(diagram, &node_id, position);
        maybe_apply_edge_handle_attachments(diagram, handle_attachment_updates);
        return diagram.clone();
    }
    mutate_active_diagram(&state, &app, |diagram| {
        apply_node_move_geometry(diagram, &node_id, position);
        maybe_apply_edge_handle_attachments(diagram, handle_attachment_updates);
    })
}

#[tauri::command]
pub fn delete_node(app: AppHandle, state: State<'_, AppState>, node_id: String) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        diagram.nodes.retain(|n| n.id != node_id);
        diagram
            .edges
            .retain(|e| e.source != node_id && e.target != node_id);
    })
}

#[derive(serde::Deserialize)]
pub struct NodeMove {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub position: XY,
}

#[tauri::command]
pub fn move_nodes(
    app: AppHandle,
    state: State<'_, AppState>,
    moves: Vec<NodeMove>,
    handle_attachment_updates: Option<Vec<EdgeHandleAttachmentUpdate>>,
) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        for m in &moves {
            apply_node_move_geometry(diagram, &m.node_id, m.position);
        }
        maybe_apply_edge_handle_attachments(diagram, handle_attachment_updates);
    })
}

#[tauri::command]
pub fn delete_nodes(
    app: AppHandle,
    state: State<'_, AppState>,
    node_ids: Vec<String>,
) -> DiagramState {
    mutate_active_diagram(&state, &app, |diagram| {
        diagram.nodes.retain(|n| !node_ids.contains(&n.id));
        diagram
            .edges
            .retain(|e| !node_ids.contains(&e.source) && !node_ids.contains(&e.target));
    })
}

#[tauri::command]
pub fn update_dims(
    app: AppHandle,
    state: State<'_, AppState>,
    dims: Vec<NodeDimension>,
    handle_attachment_updates: Option<Vec<EdgeHandleAttachmentUpdate>>,
) -> DiagramState {
    mutate_active_diagram_no_history(&state, &app, |diagram| {
        for dim in &dims {
            if let Some(node) = diagram.nodes.iter_mut().find(|n| n.id == dim.id) {
                node.width = Some(dim.width);
                node.height = Some(dim.height);
                if let Some(pos) = dim.position {
                    node.position = pos;
                }
            }
        }
        maybe_apply_edge_handle_attachments(diagram, handle_attachment_updates);
    })
}

// ─── File I/O ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn new_diagram(state: State<'_, AppState>) -> ProjectState {
    let project = ProjectState::default();
    let mut s = state.0.lock().unwrap();
    *s = project.clone();
    project
}

#[tauri::command]
pub fn open_diagram(state: State<'_, AppState>, json: String) -> Result<ProjectState, String> {
    let project = open_diagram_from_json(&json)?;
    let mut s = state.0.lock().unwrap();
    *s = project.clone();
    Ok(project)
}

#[tauri::command]
pub fn set_project(state: State<'_, AppState>, project: ProjectState) -> ProjectState {
    let mut s = state.0.lock().unwrap();
    *s = project.clone();
    project
}

#[tauri::command]
pub fn save_diagram(state: State<'_, AppState>) -> Result<String, String> {
    let mut s = state.0.lock().unwrap();
    normalize_project_for_persist(&mut s);
    let payload = serde_json::json!({
        "format": "diagramme",
        "version": 2,
        "sheets": s.sheets,
        "activeSheetId": s.active_sheet_id,
        "presetLibrary": s.preset_library,
    });
    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_diagram_compact(state: State<'_, AppState>) -> Result<String, String> {
    let mut project = state.0.lock().unwrap();
    normalize_project_for_persist(&mut project);
    save_diagram_compact_from(&project)
}

// ─── Undo / Redo ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn undo(state: State<'_, AppState>) -> DiagramState {
    let mut project = state.0.lock().unwrap();
    let sheet = project.active_sheet_mut();
    let available = sheet.undo_stack.len().saturating_sub(sheet.redo_depth);
    if available == 0 {
        return sheet.state.clone();
    }
    sheet.redo_depth += 1;
    let target_index = sheet.undo_stack.len() - sheet.redo_depth;
    sheet.state = sheet.undo_stack[target_index].clone();
    sheet.state.clone()
}

#[tauri::command]
pub fn redo(state: State<'_, AppState>) -> DiagramState {
    let mut project = state.0.lock().unwrap();
    let sheet = project.active_sheet_mut();
    if sheet.redo_depth == 0 {
        return sheet.state.clone();
    }
    sheet.redo_depth -= 1;
    let target_index = sheet.undo_stack.len() - sheet.redo_depth - 1;
    sheet.state = sheet.undo_stack[target_index].clone();
    sheet.state.clone()
}

// ─── Sheet management ───────────────────────────────────────────────────────

#[tauri::command]
pub fn add_sheet(state: State<'_, AppState>, name: String) -> ProjectState {
    let mut p = state.0.lock().unwrap();
    let full_id = uuid::Uuid::new_v4().to_string();
    let new_id = format!("sheet-{}", &full_id[..8]);
    p.sheets.push(Sheet {
        id: new_id.clone(),
        name,
        state: DiagramState::default(),
        undo_stack: std::collections::VecDeque::new(),
        redo_depth: 0,
    });
    p.active_sheet_id = new_id;
    p.clone()
}

#[tauri::command]
pub fn remove_sheet(state: State<'_, AppState>, id: String) -> Result<ProjectState, String> {
    let mut p = state.0.lock().unwrap();
    if p.sheets.len() <= 1 {
        return Err("Cannot remove the last sheet.".into());
    }
    p.sheets.retain(|s| s.id != id);
    if p.active_sheet_id == id {
        p.active_sheet_id = p.sheets[0].id.clone();
    }
    Ok(p.clone())
}

#[tauri::command]
pub fn rename_sheet(state: State<'_, AppState>, id: String, name: String) -> ProjectState {
    let mut p = state.0.lock().unwrap();
    if let Some(s) = p.sheets.iter_mut().find(|s| s.id == id) {
        s.name = name;
    }
    p.clone()
}

#[tauri::command]
pub fn set_active_sheet(state: State<'_, AppState>, id: String) -> ProjectState {
    let mut p = state.0.lock().unwrap();
    if p.sheets.iter().any(|s| s.id == id) {
        p.active_sheet_id = id;
    }
    p.clone()
}

// ─── Project presets (embedded in .diagramme) ────────────────────────────────

#[tauri::command]
pub fn add_project_preset(state: State<'_, AppState>, preset: EmbeddedPreset) -> ProjectState {
    let mut p = state.0.lock().unwrap();
    p.preset_library.retain(|e| e.id != preset.id);
    p.preset_library.push(preset);
    p.clone()
}

#[tauri::command]
pub fn update_project_preset(
    state: State<'_, AppState>,
    id: String,
    name: String,
    node_type: String,
    data: serde_json::Value,
    source_basename: Option<String>,
) -> Result<ProjectState, String> {
    let mut p = state.0.lock().unwrap();
    let Some(entry) = p.preset_library.iter_mut().find(|e| e.id == id) else {
        return Err(format!("No preset with id {id}"));
    };
    entry.name = name;
    entry.node_type = node_type;
    entry.data = data;
    entry.source_basename = source_basename;
    Ok(p.clone())
}

#[tauri::command]
pub fn remove_project_preset(state: State<'_, AppState>, id: String) -> ProjectState {
    let mut p = state.0.lock().unwrap();
    p.preset_library.retain(|e| e.id != id);
    p.clone()
}

// ─── Close / recovery (desktop shell) ─────────────────────────────────────────

#[tauri::command]
pub fn grant_next_close(gate: State<'_, AllowNextClose>) {
    gate.grant();
}

#[tauri::command]
pub fn close_window_allowing_gate(
    window: tauri::Window,
    gate: State<'_, AllowNextClose>,
) -> Result<(), String> {
    gate.grant();
    window.close().map_err(|e| e.to_string())
}

const RECOVERY_FILENAME: &str = "recovery.diagramme";

#[tauri::command]
pub fn write_recovery_snapshot(app: AppHandle, json: String) -> Result<(), String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let p = dir.join(RECOVERY_FILENAME);
    let tmp = dir.join(format!("{RECOVERY_FILENAME}.tmp"));
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_recovery_snapshot(app: AppHandle) -> Result<Option<String>, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let p = dir.join(RECOVERY_FILENAME);
    if !p.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(&p).map(Some).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_recovery_snapshot(app: AppHandle) -> Result<(), String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let p = dir.join(RECOVERY_FILENAME);
    if p.exists() {
        std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Build scene for the active sheet (used by integration tests).
pub fn get_diagram_scene_for_state(state: &AppState) -> Scene {
    let project = state.0.lock().unwrap();
    build_scene(&project.active_sheet().state)
}

/// Export Revit DXF for the active sheet (used by integration tests).
pub fn export_revit_dxf_for_state(state: &AppState) -> Result<String, String> {
    let project = state.0.lock().unwrap();
    Ok(build_revit_dxf_from_diagram(&project.active_sheet().state))
}

/// Apply a node move on in-memory project state (used by integration tests).
pub fn apply_move_node(project: &mut ProjectState, node_id: &str, position: XY) {
    apply_node_move_geometry(
        &mut project.active_sheet_mut().state,
        node_id,
        position,
    );
}

/// Shared test harness wrapping [`AppState`].
pub fn test_app_state(project: ProjectState) -> AppState {
    AppState(Mutex::new(project))
}
