//! Wire routing and postprocess.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod obstacles;
pub mod sharp_polyline;
pub mod types;

pub use diagramme_geometry;
pub use diagramme_schema;

pub use obstacles::*;
pub use sharp_polyline::*;
pub use types::*;
