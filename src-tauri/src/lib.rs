//! Diagramme v2 — Tauri desktop shell (scaffold).

use diagramme_dxf::VERSION as DXF_VERSION;
use diagramme_export_model::VERSION as EXPORT_MODEL_VERSION;
use diagramme_geometry::VERSION as GEOMETRY_VERSION;
use diagramme_reports::VERSION as REPORTS_VERSION;
use diagramme_scene::VERSION as SCENE_VERSION;
use diagramme_schema::VERSION as SCHEMA_VERSION;
use diagramme_wires::VERSION as WIRES_VERSION;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!(
        "diagramme-core: schema={SCHEMA_VERSION} geometry={GEOMETRY_VERSION} wires={WIRES_VERSION} scene={SCENE_VERSION} dxf={DXF_VERSION} export-model={EXPORT_MODEL_VERSION} reports={REPORTS_VERSION}"
    );

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
