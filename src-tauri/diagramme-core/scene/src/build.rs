//! Scene builder from diagram state (node dispatch in Task 10).

use crate::scene::{RectPx, Scene};
use diagramme_schema::DiagramState;

/// Build the drawable scene for a diagram. Task 10 fills node/wire dispatch.
pub fn build_scene(_diagram: &DiagramState) -> Scene {
    Scene {
        primitives: Vec::new(),
        extent: RectPx::new(0.0, 0.0, 0.0, 0.0),
        hits: Vec::new(),
    }
}
