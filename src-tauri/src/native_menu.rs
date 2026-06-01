//! macOS / desktop native menu bar (mirrors v6 structure).

use tauri::menu::{
    AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
};
use tauri::{App, AppHandle, Emitter};

pub fn install_native_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let pkg_info = app.package_info();
    let config = app.config();
    let display_app_name = config
        .product_name
        .clone()
        .unwrap_or_else(|| pkg_info.name.clone());
    let about_metadata = AboutMetadata {
        name: Some(display_app_name.clone()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    };

    let file_new =
        MenuItem::with_id(app, "file.new", "New Diagram", true, Some("CmdOrCtrl+N"))?;
    let file_open =
        MenuItem::with_id(app, "file.open", "Open Diagram…", true, Some("CmdOrCtrl+O"))?;
    let file_save = MenuItem::with_id(app, "file.save", "Save", true, Some("CmdOrCtrl+S"))?;
    let file_save_as =
        MenuItem::with_id(app, "file.saveAs", "Save As…", true, Some("CmdOrCtrl+Shift+S"))?;
    let file_grouped_inventory_report = MenuItem::with_id(
        app,
        "file.groupedInventoryReport",
        "Grouped Inventory Report",
        true,
        None::<&str>,
    )?;
    let file_device_tags_report = MenuItem::with_id(
        app,
        "file.deviceTagsReport",
        "Device Tags Report",
        true,
        None::<&str>,
    )?;
    let file_plate_connections_report = MenuItem::with_id(
        app,
        "file.plateConnectionsReport",
        "Plate Connections Report",
        true,
        None::<&str>,
    )?;
    let file_export_grouped_inventory = MenuItem::with_id(
        app,
        "file.exportGroupedInventoryXlsx",
        "Export Grouped Inventory (.xlsx)",
        true,
        None::<&str>,
    )?;
    let file_export_device_tags = MenuItem::with_id(
        app,
        "file.exportDeviceTagsXlsx",
        "Export Device Tags (.xlsx)",
        true,
        None::<&str>,
    )?;
    let file_export_plate_connections = MenuItem::with_id(
        app,
        "file.exportPlateConnectionsXlsx",
        "Export Plate Connections (.xlsx)",
        true,
        None::<&str>,
    )?;
    let file_export_equipment = MenuItem::with_id(
        app,
        "file.exportEquipmentXlsx",
        "Export Equipment List (.xlsx)",
        true,
        None::<&str>,
    )?;
    let file_export_revit_dxf = MenuItem::with_id(
        app,
        "file.exportRevitDxf",
        "Export DXF (Revit)",
        true,
        None::<&str>,
    )?;

    let edit_undo = MenuItem::with_id(app, "edit.undo", "Undo", true, Some("CmdOrCtrl+Z"))?;
    let redo_accelerator = if cfg!(target_os = "macos") {
        "Cmd+Shift+Z"
    } else {
        "Ctrl+Y"
    };
    let edit_redo = MenuItem::with_id(app, "edit.redo", "Redo", true, Some(redo_accelerator))?;
    let edit_copy = MenuItem::with_id(app, "edit.copy", "Copy", true, Some("CmdOrCtrl+C"))?;
    let edit_cut = MenuItem::with_id(app, "edit.cut", "Cut", true, Some("CmdOrCtrl+X"))?;
    let edit_paste = MenuItem::with_id(app, "edit.paste", "Paste", true, Some("CmdOrCtrl+V"))?;
    let edit_duplicate =
        MenuItem::with_id(app, "edit.duplicate", "Duplicate", true, Some("CmdOrCtrl+D"))?;
    let edit_select_all =
        MenuItem::with_id(app, "edit.selectAll", "Select All", true, Some("CmdOrCtrl+A"))?;
    let help_user_guide =
        MenuItem::with_id(app, "help.userGuide", "User Guide…", true, None::<&str>)?;
    let help_load_golden = MenuItem::with_id(
        app,
        "help.loadDxfExportTestDiagram",
        "Load Comp Gym (dev)…",
        true,
        None::<&str>,
    )?;
    let view_wiring_mode =
        MenuItem::with_id(app, "view.toggleWiringMode", "Wiring Mode", true, Some("W"))?;
    let view_alignment_guides = MenuItem::with_id(
        app,
        "view.toggleAlignmentGuides",
        "Alignment Guides",
        true,
        None::<&str>,
    )?;
    let view_focus_mode =
        MenuItem::with_id(app, "view.toggleFocusMode", "Focus Mode", true, Some("F"))?;
    let view_wire_geometry_overlay = MenuItem::with_id(
        app,
        "view.toggleWireGeometryOverlay",
        "Wire Geometry Overlay",
        true,
        None::<&str>,
    )?;
    let view_show_page_boundary = MenuItem::with_id(
        app,
        "view.togglePageBoundary",
        "Show Page Boundary",
        true,
        None::<&str>,
    )?;
    let view_page_boundary_settings = MenuItem::with_id(
        app,
        "view.pageBoundarySettings",
        "Page Boundary Settings…",
        true,
        None::<&str>,
    )?;
    let view_fit_page_boundary = MenuItem::with_id(
        app,
        "view.fitPageBoundary",
        "Fit View to Page",
        true,
        None::<&str>,
    )?;

    let quit_label = if cfg!(target_os = "macos") {
        format!("Quit {}", display_app_name)
    } else {
        format!("Exit {}", display_app_name)
    };
    let quit_accel = if cfg!(target_os = "macos") {
        Some("Cmd+Q")
    } else {
        Some("Ctrl+Q")
    };
    let app_quit = MenuItem::with_id(app, "app.quit", &quit_label, true, quit_accel)?;

    let file_submenu = if cfg!(target_os = "macos") {
        Submenu::with_items(
            app,
            "File",
            true,
            &[
                &file_new,
                &PredefinedMenuItem::separator(app)?,
                &file_open,
                &file_save,
                &file_save_as,
                &PredefinedMenuItem::separator(app)?,
                &file_grouped_inventory_report,
                &file_device_tags_report,
                &file_plate_connections_report,
                &file_export_grouped_inventory,
                &file_export_device_tags,
                &file_export_plate_connections,
                &file_export_equipment,
                &file_export_revit_dxf,
            ],
        )?
    } else {
        Submenu::with_items(
            app,
            "File",
            true,
            &[
                &file_new,
                &PredefinedMenuItem::separator(app)?,
                &file_open,
                &file_save,
                &file_save_as,
                &PredefinedMenuItem::separator(app)?,
                &file_grouped_inventory_report,
                &file_device_tags_report,
                &file_plate_connections_report,
                &file_export_grouped_inventory,
                &file_export_device_tags,
                &file_export_plate_connections,
                &file_export_equipment,
                &file_export_revit_dxf,
                &PredefinedMenuItem::separator(app)?,
                &app_quit,
            ],
        )?
    };
    let edit_submenu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &edit_undo,
            &edit_redo,
            &PredefinedMenuItem::separator(app)?,
            &edit_cut,
            &edit_copy,
            &edit_paste,
            &edit_duplicate,
            &edit_select_all,
        ],
    )?;

    let menu = if cfg!(target_os = "macos") {
        let app_menu = Submenu::with_items(
            app,
            display_app_name.clone(),
            true,
            &[
                &PredefinedMenuItem::about(app, None, Some(about_metadata.clone()))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::show_all(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &app_quit,
            ],
        )?;

        let view_menu = Submenu::with_items(
            app,
            "View",
            true,
            &[
                &view_wiring_mode,
                &view_alignment_guides,
                &view_focus_mode,
                &view_wire_geometry_overlay,
                &PredefinedMenuItem::separator(app)?,
                &view_show_page_boundary,
                &view_page_boundary_settings,
                &view_fit_page_boundary,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::fullscreen(app, None)?,
            ],
        )?;

        let window_menu = Submenu::with_id_and_items(
            app,
            WINDOW_SUBMENU_ID,
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(app, None)?,
                &PredefinedMenuItem::maximize(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::close_window(app, None)?,
            ],
        )?;

        let help_menu = Submenu::with_id_and_items(
            app,
            HELP_SUBMENU_ID,
            "Help",
            true,
            &[&help_user_guide, &help_load_golden],
        )?;

        Menu::with_items(
            app,
            &[
                &app_menu,
                &file_submenu,
                &edit_submenu,
                &view_menu,
                &window_menu,
                &help_menu,
            ],
        )?
    } else {
        let view_menu = Submenu::with_items(
            app,
            "View",
            true,
            &[
                &view_wiring_mode,
                &view_alignment_guides,
                &view_focus_mode,
                &view_wire_geometry_overlay,
                &PredefinedMenuItem::separator(app)?,
                &view_show_page_boundary,
                &view_page_boundary_settings,
                &view_fit_page_boundary,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::fullscreen(app, None)?,
            ],
        )?;

        let about_show =
            MenuItem::with_id(app, "about.show", "About Diagramme", true, None::<&str>)?;
        let help_menu = Submenu::with_items(
            app,
            "Help",
            true,
            &[
                &help_user_guide,
                &help_load_golden,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
                &PredefinedMenuItem::separator(app)?,
                &about_show,
            ],
        )?;

        Menu::with_items(app, &[&file_submenu, &edit_submenu, &view_menu, &help_menu])?
    };

    app.set_menu(menu)?;
    Ok(())
}

pub fn on_native_menu_event(app: &AppHandle, menu_id: &str) {
    match menu_id {
        "file.new"
        | "file.open"
        | "file.save"
        | "file.saveAs"
        | "file.groupedInventoryReport"
        | "file.deviceTagsReport"
        | "file.plateConnectionsReport"
        | "file.exportGroupedInventoryXlsx"
        | "file.exportDeviceTagsXlsx"
        | "file.exportPlateConnectionsXlsx"
        | "file.exportEquipmentXlsx"
        | "file.exportRevitDxf"
        | "edit.undo"
        | "edit.redo"
        | "edit.copy"
        | "edit.cut"
        | "edit.paste"
        | "edit.duplicate"
        | "edit.selectAll"
        | "view.toggleWiringMode"
        | "view.toggleAlignmentGuides"
        | "view.toggleFocusMode"
        | "view.toggleWireGeometryOverlay"
        | "view.togglePageBoundary"
        | "view.pageBoundarySettings"
        | "view.fitPageBoundary"
        | "about.show"
        | "help.userGuide"
        | "help.loadDxfExportTestDiagram" => {
            let _ = app.emit("app-menu-command", menu_id.to_string());
        }
        "app.quit" => {
            let _ = app.emit("diagramme-close-request", ());
        }
        _ => {}
    }
}
