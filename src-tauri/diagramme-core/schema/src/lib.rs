//! v6-compatible persisted project types.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod model;

pub use model::{
    DiagramState, Edge, EmbeddedPreset, Node, NodeDimension, ProjectState, Sheet, XY,
};
