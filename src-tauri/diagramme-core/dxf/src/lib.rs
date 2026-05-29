//! Revit-safe DXF emit and sanitize.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod document;
mod primitives;
mod sanitize;
mod scene_emit;

pub use document::{create_revit_cad_document, serialize_revit_dxf, CadDocument, CadExtentInches, TextHAlign, TextVAlign};
pub use primitives::{add_line, add_lwpolyline, add_solid, add_text};
pub use sanitize::{inject_header_extents, sanitize_dxf_string};
pub use scene_emit::emit_scene_to_dxf;

use diagramme_scene::build_scene;
use diagramme_schema::DiagramState;

/// Build scene from diagram state and emit Revit-safe DXF.
pub fn build_revit_dxf_from_diagram(diagram: &DiagramState) -> String {
    let scene = build_scene(diagram);
    emit_scene_to_dxf(&scene)
}

pub use diagramme_scene;
pub use diagramme_schema;
