//! Scene builder from diagram state.

use diagramme_geometry::RectPx;
use diagramme_schema::DiagramState;

use diagramme_geometry::is_patch_panel_node_type;

use crate::nodes::av_plate::av_plate_scene_bounds;
use crate::nodes::append_av_plate_scene;
use crate::nodes::append_device_v2_scene;
use crate::nodes::append_patch_panel_scene;
use crate::nodes::device_v2::device_v2_scene_bounds;
use crate::nodes::patch_panel::patch_panel_scene_bounds;
use crate::scene::Scene;

fn union_rects(rects: impl IntoIterator<Item = RectPx>) -> RectPx {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut any = false;

    for rect in rects {
        any = true;
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.width);
        max_y = max_y.max(rect.y + rect.height);
    }

    if !any {
        return RectPx::new(0.0, 0.0, 0.0, 0.0);
    }

    RectPx::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Build the drawable scene for a diagram.
pub fn build_scene(diagram: &DiagramState) -> Scene {
    let mut scene = Scene::default();
    let mut extent_rects: Vec<RectPx> = Vec::new();

    for node in &diagram.nodes {
        match node.node_type.as_str() {
            "deviceV2" | "device" => {
                append_device_v2_scene(&mut scene, node);
                extent_rects.push(device_v2_scene_bounds(node));
            }
            "avPlate" => {
                append_av_plate_scene(&mut scene, node);
                extent_rects.push(av_plate_scene_bounds(node));
            }
            t if is_patch_panel_node_type(t) => {
                append_patch_panel_scene(&mut scene, node);
                extent_rects.push(patch_panel_scene_bounds(node));
            }
            _ => {}
        }
    }

    scene.extent = union_rects(extent_rects);
    scene
}
