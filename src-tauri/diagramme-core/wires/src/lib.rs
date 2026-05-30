//! Wire routing and postprocess.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod bundle_wire;
pub mod crossings;
pub mod inner_corners;
pub mod node_move;
pub mod obstacles;
pub mod postprocess;
pub mod sharp_polyline;
pub mod types;
pub mod wire_avoidance;

pub use diagramme_geometry;
pub use diagramme_schema;

pub use bundle_wire::*;
pub use crossings::*;
pub use inner_corners::*;
pub use node_move::*;
pub use obstacles::*;
pub use postprocess::*;
pub use sharp_polyline::*;
pub use types::*;
pub use wire_avoidance::*;
