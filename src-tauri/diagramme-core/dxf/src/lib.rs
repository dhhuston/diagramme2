//! Revit-safe DXF emit and sanitize.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod document;
mod primitives;
mod sanitize;

pub use document::{create_revit_cad_document, serialize_revit_dxf, CadDocument, CadExtentInches, TextHAlign, TextVAlign};
pub use primitives::{add_line, add_lwpolyline, add_solid, add_text};
pub use sanitize::{inject_header_extents, sanitize_dxf_string};

pub use diagramme_scene;
pub use diagramme_schema;
