//! Scene graph build and strict-mirror CAD transform.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod breakline;
mod build;
mod bundle_brackets;
mod cad_transform;
mod nodes;
mod patch;
mod scene;
mod text;
mod wires;

pub use build::{build_scene, build_scene_with_options, SceneBuildOptions};
pub use patch::build_scene_patch_for_node;
pub use nodes::{resolve_pair_main_display_text, wiretag_scene_bounds};
pub use wires::{append_wires_to_scene, wire_extent_rect, WireCategory};
pub use cad_transform::{
    extent_from_rects, extent_in_from_rect, px_to_in, scene_point_to_cad, scene_to_cad,
    CadHAlign, CadPrimitive, CadScene, CadText, CadVAlign, ExtentIn, PointIn, RectIn,
};
pub use scene::{
    HitTarget, HAlign, Scene, ScenePatch, ScenePrimitive, SceneText, VAlign,
};
pub use diagramme_geometry::{PointPx, RectPx};
pub use diagramme_schema;
pub use diagramme_wires;
