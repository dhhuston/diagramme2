//! Scene builder from diagram state.

use diagramme_geometry::RectPx;
use diagramme_schema::{active_bundle_handles, DiagramState, Node};
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
use crate::scene::{Scene, ScenePrimitive, SceneText};
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

/// Tag node-owned primitives emitted since `start` (skips wire polylines with `edge_id`).
pub fn tag_node_primitives(scene: &mut Scene, start: usize, node_id: &str) {
    for prim in &mut scene.primitives[start..] {
        match prim {
            ScenePrimitive::Polyline {
                owner_node_id,
                edge_id: None,
                ..
            } => {
                *owner_node_id = Some(node_id.to_string());
            }
            ScenePrimitive::Polyline { edge_id: Some(_), .. } => {}
            ScenePrimitive::Text(SceneText { owner_node_id, .. }) => {
                *owner_node_id = Some(node_id.to_string());
            }
            ScenePrimitive::Rect { node_id: id, .. } | ScenePrimitive::Solid { node_id: id, .. } => {
                if id.is_none() {
                    *id = Some(node_id.to_string());
                }
            }
        }
    }
}

/// Append one node's drawable primitives and hits to `scene`.
pub fn append_node_to_scene(
    scene: &mut Scene,
    node: &Node,
    diagram: &DiagramState,
    active_bundles: &std::collections::HashSet<(String, String)>,
    filter_bundle_brackets: bool,
) {
    let start = scene.primitives.len();
    let nodes = &diagram.nodes;
    let edges = &diagram.edges;

    match node.node_type.as_str() {
        "deviceV2" | "device" => {
            append_device_v2_scene(scene, node, active_bundles, filter_bundle_brackets);
        }
        "avPlate" => {
            append_av_plate_scene(scene, node, active_bundles, filter_bundle_brackets);
        }
        t if is_patch_panel_node_type(t) => {
            append_patch_panel_scene(scene, node, active_bundles, filter_bundle_brackets);
        }
        "junction" => {
            if !append_junction_scene(scene, node) {
                return;
            }
        }
        "speakerBlock" => append_speaker_block_scene(scene, node),
        "micBlock" => append_mic_block_scene(scene, node),
        "volumeControl" => append_volume_control_scene(scene, node),
        "textBlock" => append_text_block_scene(scene, node),
        "flyoffNote" => append_flyoff_note_scene(scene, node),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => {
            append_antenna_scene(scene, node);
        }
        "groupingZone" => append_grouping_zone_scene(scene, node),
        "wiretag" => append_wiretag_scene(scene, node, nodes, edges),
        _ => return,
    }

    tag_node_primitives(scene, start, &node.id);
}

fn node_extent_rect(node: &Node, diagram: &DiagramState) -> Option<RectPx> {
    let nodes = &diagram.nodes;
    let edges = &diagram.edges;
    Some(match node.node_type.as_str() {
        "deviceV2" | "device" => device_v2_scene_bounds(node),
        "avPlate" => av_plate_scene_bounds(node),
        t if is_patch_panel_node_type(t) => patch_panel_scene_bounds(node),
        "junction" => junction_scene_bounds(node),
        "speakerBlock" => speaker_block_scene_bounds(node),
        "micBlock" => mic_block_scene_bounds(node),
        "volumeControl" => volume_control_scene_bounds(node),
        "textBlock" => text_block_scene_bounds(node),
        "flyoffNote" => flyoff_note_scene_bounds(node),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => antenna_scene_bounds(node),
        "groupingZone" => grouping_zone_scene_bounds(node),
        "wiretag" => wiretag_scene_bounds(node, nodes, edges),
        _ => return None,
    })
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

    let active_bundles = if options.filter_bundle_brackets {
        active_bundle_handles(diagram)
    } else {
        std::collections::HashSet::new()
    };

    for node in &diagram.nodes {
        append_node_to_scene(
            &mut scene,
            node,
            diagram,
            &active_bundles,
            options.filter_bundle_brackets,
        );
        if let Some(rect) = node_extent_rect(node, diagram) {
            extent_rects.push(rect);
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
