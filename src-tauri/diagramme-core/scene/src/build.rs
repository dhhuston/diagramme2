//! Scene builder from diagram state.

use diagramme_geometry::RectPx;
use diagramme_schema::{active_bundle_handles, DiagramState};
use diagramme_wires::WireGeometryOptions;

use diagramme_geometry::is_patch_panel_node_type;

use crate::nodes::antenna::antenna_scene_bounds;
use crate::nodes::av_plate::av_plate_scene_bounds;
use crate::nodes::append_antenna_scene;
use crate::nodes::append_av_plate_scene;
use crate::nodes::append_device_v2_scene;
use crate::nodes::append_flyoff_note_scene;
use crate::nodes::append_grouping_zone_scene;
use crate::nodes::append_junction_scene;
use crate::nodes::append_mic_block_scene;
use crate::nodes::append_patch_panel_scene;
use crate::nodes::append_speaker_block_scene;
use crate::nodes::append_text_block_scene;
use crate::nodes::append_volume_control_scene;
use crate::nodes::append_wiretag_scene;
use crate::nodes::wiretag_scene_bounds;
use crate::nodes::device_v2::device_v2_scene_bounds;
use crate::nodes::flyoff_note::flyoff_note_scene_bounds;
use crate::nodes::grouping_zone::grouping_zone_scene_bounds;
use crate::nodes::junction::junction_scene_bounds;
use crate::nodes::mic_block::mic_block_scene_bounds;
use crate::nodes::patch_panel::patch_panel_scene_bounds;
use crate::nodes::speaker_block::speaker_block_scene_bounds;
use crate::nodes::text_block::text_block_scene_bounds;
use crate::nodes::volume_control::volume_control_scene_bounds;
use crate::scene::Scene;
use crate::wires::{build_and_append_wires, wire_extent_rect};

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

/// Options for [`build_scene_with_options`].
#[derive(Debug, Clone, Copy)]
pub struct SceneBuildOptions {
    pub include_wires: bool,
    pub wire_geometry: WireGeometryOptions,
    pub filter_bundle_brackets: bool,
}

impl Default for SceneBuildOptions {
    fn default() -> Self {
        Self {
            include_wires: true,
            wire_geometry: WireGeometryOptions::default(),
            filter_bundle_brackets: true,
        }
    }
}

/// Build the drawable scene for a diagram.
pub fn build_scene(diagram: &DiagramState) -> Scene {
    build_scene_with_options(diagram, SceneBuildOptions::default())
}

/// Build the drawable scene with optional wire geometry.
pub fn build_scene_with_options(diagram: &DiagramState, options: SceneBuildOptions) -> Scene {
    let mut scene = Scene::default();
    let mut extent_rects: Vec<RectPx> = Vec::new();

    let nodes = &diagram.nodes;
    let edges = &diagram.edges;
    let active_bundles = if options.filter_bundle_brackets {
        active_bundle_handles(diagram)
    } else {
        std::collections::HashSet::new()
    };

    for node in nodes {
        match node.node_type.as_str() {
            "deviceV2" | "device" => {
                append_device_v2_scene(&mut scene, node, &active_bundles, options.filter_bundle_brackets);
                extent_rects.push(device_v2_scene_bounds(node));
            }
            "avPlate" => {
                append_av_plate_scene(&mut scene, node, &active_bundles, options.filter_bundle_brackets);
                extent_rects.push(av_plate_scene_bounds(node));
            }
            t if is_patch_panel_node_type(t) => {
                append_patch_panel_scene(&mut scene, node, &active_bundles, options.filter_bundle_brackets);
                extent_rects.push(patch_panel_scene_bounds(node));
            }
            "junction" => {
                if append_junction_scene(&mut scene, node) {
                    extent_rects.push(junction_scene_bounds(node));
                }
            }
            "speakerBlock" => {
                append_speaker_block_scene(&mut scene, node);
                extent_rects.push(speaker_block_scene_bounds(node));
            }
            "micBlock" => {
                append_mic_block_scene(&mut scene, node);
                extent_rects.push(mic_block_scene_bounds(node));
            }
            "volumeControl" => {
                append_volume_control_scene(&mut scene, node);
                extent_rects.push(volume_control_scene_bounds(node));
            }
            "textBlock" => {
                append_text_block_scene(&mut scene, node);
                extent_rects.push(text_block_scene_bounds(node));
            }
            "flyoffNote" => {
                append_flyoff_note_scene(&mut scene, node);
                extent_rects.push(flyoff_note_scene_bounds(node));
            }
            "antennaTransmitterSymbol" | "antennaReceiverSymbol" => {
                append_antenna_scene(&mut scene, node);
                extent_rects.push(antenna_scene_bounds(node));
            }
            "groupingZone" => {
                append_grouping_zone_scene(&mut scene, node);
                extent_rects.push(grouping_zone_scene_bounds(node));
            }
            "wiretag" => {
                append_wiretag_scene(&mut scene, node, nodes, edges);
                extent_rects.push(wiretag_scene_bounds(node, nodes, edges));
            }
            _ => {}
        }
    }

    if options.include_wires {
        build_and_append_wires(&mut scene, diagram, options.wire_geometry);
        if let Some(wire_rect) = wire_extent_rect(&scene) {
            extent_rects.push(wire_rect);
        }
    }

    scene.extent = union_rects(extent_rects);
    scene
}
