//! Diagramme v2 — Tauri desktop shell.

pub mod close_gate;
pub mod commands;
pub mod debug_channel;
pub mod native_menu;
pub mod state;

use close_gate::AllowNextClose;
use diagramme_schema::ProjectState;
use state::AppState;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use diagramme_dxf::VERSION as DXF_VERSION;
    use diagramme_export_model::VERSION as EXPORT_MODEL_VERSION;
    use diagramme_geometry::VERSION as GEOMETRY_VERSION;
    use diagramme_reports::VERSION as REPORTS_VERSION;
    use diagramme_scene::VERSION as SCENE_VERSION;
    use diagramme_schema::VERSION as SCHEMA_VERSION;
    use diagramme_wires::VERSION as WIRES_VERSION;

    log::info!(
        "diagramme-core: schema={SCHEMA_VERSION} geometry={GEOMETRY_VERSION} wires={WIRES_VERSION} scene={SCENE_VERSION} dxf={DXF_VERSION} export-model={EXPORT_MODEL_VERSION} reports={REPORTS_VERSION}"
    );

    tauri::Builder::default()
        .manage(AppState(Mutex::new(ProjectState::default())))
        .manage(AllowNextClose::default())
        .setup(|app| {
            native_menu::install_native_menu(app)?;

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .on_menu_event(|app, event| {
            native_menu::on_native_menu_event(app, event.id().as_ref());
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let gate = window.state::<AllowNextClose>();
                if gate.consume_if_allowed() {
                    return;
                }
                api.prevent_close();
                let _ = window.app_handle().emit("diagramme-close-request", ());
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_project,
            commands::get_state,
            commands::get_diagram_scene,
            commands::get_diagram_scene_patch,
            commands::export_revit_dxf,
            commands::set_state,
            commands::sync_state,
            commands::add_node,
            commands::update_node,
            commands::update_grouping_zone,
            commands::replace_node_type,
            commands::move_node,
            commands::add_edge,
            commands::delete_edge,
            commands::get_wire_inner_chain,
            commands::drag_wire_segment,
            commands::update_edge_inner_corners,
            commands::move_nodes,
            commands::delete_node,
            commands::delete_nodes,
            commands::update_dims,
            commands::new_diagram,
            commands::open_diagram,
            commands::set_project,
            commands::save_diagram,
            commands::save_diagram_compact,
            commands::undo,
            commands::redo,
            commands::cancel_drag_preview,
            commands::add_sheet,
            commands::remove_sheet,
            commands::rename_sheet,
            commands::set_active_sheet,
            commands::add_project_preset,
            commands::update_project_preset,
            commands::remove_project_preset,
            commands::grant_next_close,
            commands::close_window_allowing_gate,
            commands::write_recovery_snapshot,
            commands::read_recovery_snapshot,
            commands::clear_recovery_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
