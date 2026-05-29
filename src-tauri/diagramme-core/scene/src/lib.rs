//! Scene graph build and strict-mirror CAD transform.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod build;
mod cad_transform;
mod scene;

pub use build::build_scene;
pub use cad_transform::{
    extent_from_rects, extent_in_from_rect, px_to_in, scene_point_to_cad, scene_to_cad,
    CadHAlign, CadPrimitive, CadScene, CadText, CadVAlign, ExtentIn, PointIn, RectIn,
};
pub use scene::{
    HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign,
};
pub use diagramme_geometry::{PointPx, RectPx};
pub use diagramme_schema;
pub use diagramme_wires;
