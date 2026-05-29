//! Layout constants, port geometry, bounds, text measure.

/// Crate version (startup logging only).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod av_plate_layout;
pub mod device_v2_layout;
pub mod node_bounds;
pub mod paper_scale;
pub mod port_geometry;
pub mod schematic_layout;
pub mod text_measure;
pub mod types;
pub mod units;

#[cfg(test)]
mod paper_scale_tests;

#[cfg(test)]
mod port_geometry_tests;

#[cfg(test)]
mod text_measure_tests;

pub use av_plate_layout::*;
pub use device_v2_layout::*;
pub use node_bounds::*;
pub use paper_scale::*;
pub use port_geometry::*;
pub use schematic_layout::*;
pub use text_measure::*;
pub use types::*;
pub use units::*;
