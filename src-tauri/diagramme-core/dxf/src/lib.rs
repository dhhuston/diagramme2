//! Revit-safe DXF emit and sanitize.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod audit;
mod debug_split;
mod document;
mod primitives;
mod sanitize;
mod scene_emit;
mod template;

pub use audit::{audit_dxf, audit_polylines_in_region, DxfAuditIssue, DxfAuditReport, RegionBboxPx, RegionPolylineAudit};
pub use debug_split::{debug_split_specs, default_debug_out_dir, write_layer_debug_bundle, DebugSplitManifest, DebugSplitSpec};
pub use document::{
    create_revit_cad_document, serialize_revit_dxf, serialize_revit_dxf_with_filter,
    CadDocument, CadExtentInches, EntityFilter, EntityTypeFilter, TextHAlign, TextVAlign,
};
pub use primitives::{add_line, add_lwpolyline, add_solid, add_text};
pub use sanitize::{inject_header_extents, sanitize_dxf_string};
pub use scene_emit::emit_scene_to_dxf;

use diagramme_scene::{build_scene, build_scene_with_options, SceneBuildOptions};
use diagramme_schema::DiagramState;

/// Build scene from diagram state and emit Revit-safe DXF.
///
/// Uses the same [`build_scene`] path as Konva (`get_diagram_scene`) — no export-only
/// wire rerouting or diagram normalization.
pub fn build_revit_dxf_from_diagram(diagram: &DiagramState) -> String {
    let scene = build_scene(diagram);
    emit_scene_to_dxf(&scene)
}

/// Emit Revit-safe DXF with explicit scene build options (tests / debug only).
pub fn build_revit_dxf_from_diagram_with_options(
    diagram: &DiagramState,
    options: SceneBuildOptions,
) -> String {
    let scene = build_scene_with_options(diagram, options);
    emit_scene_to_dxf(&scene)
}

pub use diagramme_scene;
pub use diagramme_schema;
