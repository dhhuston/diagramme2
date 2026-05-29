//! Layout constants, port geometry, bounds, text measure.

/// Crate version (startup logging only).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod paper_scale;
pub mod units;

#[cfg(test)]
mod paper_scale_tests;

pub use paper_scale::*;
pub use units::*;
