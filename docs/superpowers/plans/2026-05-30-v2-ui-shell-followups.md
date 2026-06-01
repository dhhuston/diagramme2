# v2 UI shell — follow-ups

Chrome port from v6 is in place (`AppShell`, layout CSS, disabled palette/presets, thin browser menu). Remaining work:

## Native menu (Tauri)

- macOS / desktop: native menu is installed in `src-tauri/src/native_menu.rs`; commands route via `tauriMenuBridge` + `useNativeAppMenu`.
- Browser `AppMenuBar` remains hidden when `isTauriApp()` (use the system menu bar on desktop).
- Optional: macOS Services / hide-others helpers from v6 if needed later.

## File operations

- Wire File menu: New, Open, Save, Save As via `useDiagramFileOps` + dialogs (not ported).
- Reports and XLSX exports remain disabled until report crates exist.

## Properties panel

- Replace `PropertiesPlaceholder` with real `PropertiesPanel` + forms when node data binding is ready.
- Enable palette insert actions via `useNodeCreation` (or Konva-specific creation path).

## Preset toolbox

- Restore full `PresetToolbox` with `avPresetFormat` and project preset IPC when node insertion exists.

## View menu

- Page boundary, wire geometry overlay, alignment guides: wire to canvas/Konva behavior (currently UI-only toggles where disabled).

## Interaction (parallel track)

- P0 handles, wiring mode behavior on Konva, connect gesture polish — unchanged by shell work; see parity recovery plan.
