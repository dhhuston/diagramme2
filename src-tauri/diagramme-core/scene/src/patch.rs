//! Incremental scene updates for drag preview — wires + hit targets for moved nodes.

use std::collections::HashSet;

use diagramme_schema::{active_bundle_handles, DiagramState};
use diagramme_wires::build_wire_geometry_model;

use diagramme_geometry::is_patch_panel_node_type;

use crate::build::SceneBuildOptions;
use crate::nodes::{
    append_antenna_scene, append_av_plate_scene, append_device_v2_scene, append_flyoff_note_scene,
    append_grouping_zone_scene, append_junction_scene, append_mic_block_scene,
    append_patch_panel_scene, append_speaker_block_scene, append_text_block_scene,
    append_volume_control_scene, append_wiretag_scene,
};
use crate::scene::{HitTarget, Scene, ScenePatch};
use crate::wires::append_wires_to_scene_for_edges;

fn connected_edge_ids(diagram: &DiagramState, node_id: &str) -> Vec<String> {
    diagram
        .edges
        .iter()
        .filter(|e| e.source == node_id || e.target == node_id)
        .map(|e| e.id.clone())
        .collect()
}

/// Append drawable primitives and hits for a single node (no wires).
pub fn append_node_scene(
    scene: &mut Scene,
    diagram: &DiagramState,
    node: &diagramme_schema::Node,
    options: SceneBuildOptions,
) {
    let nodes = &diagram.nodes;
    let edges = &diagram.edges;
    let active_bundles = if options.filter_bundle_brackets {
        active_bundle_handles(diagram)
    } else {
        HashSet::new()
    };

    match node.node_type.as_str() {
        "deviceV2" | "device" => {
            append_device_v2_scene(
                scene,
                node,
                &active_bundles,
                options.filter_bundle_brackets,
            );
        }
        "avPlate" => {
            append_av_plate_scene(
                scene,
                node,
                &active_bundles,
                options.filter_bundle_brackets,
            );
        }
        t if is_patch_panel_node_type(t) => {
            append_patch_panel_scene(
                scene,
                node,
                &active_bundles,
                options.filter_bundle_brackets,
            );
        }
        "junction" => {
            let _ = append_junction_scene(scene, node);
        }
        "speakerBlock" => append_speaker_block_scene(scene, node),
        "micBlock" => append_mic_block_scene(scene, node),
        "volumeControl" => append_volume_control_scene(scene, node),
        "textBlock" => append_text_block_scene(scene, node),
        "flyoffNote" => append_flyoff_note_scene(scene, node),
        "antennaTransmitterSymbol" | "antennaReceiverSymbol" => append_antenna_scene(scene, node),
        "groupingZone" => append_grouping_zone_scene(scene, node),
        "wiretag" => append_wiretag_scene(scene, node, nodes, edges),
        _ => {}
    }
}

fn hits_for_node(diagram: &DiagramState, node_id: &str, options: SceneBuildOptions) -> Vec<HitTarget> {
    let Some(node) = diagram.nodes.iter().find(|n| n.id == node_id) else {
        return Vec::new();
    };
    let mut scene = Scene::default();
    append_node_scene(&mut scene, diagram, node, options);
    scene
        .hits
        .into_iter()
        .filter(|h| h.node_id.as_deref() == Some(node_id))
        .collect()
}

/// Lightweight scene patch after `move_node` preview — affected wire polylines + moved node hits.
pub fn build_scene_patch_for_node(
    diagram: &DiagramState,
    node_id: &str,
    options: SceneBuildOptions,
) -> ScenePatch {
    let edge_ids = connected_edge_ids(diagram, node_id);
    let edge_set: HashSet<String> = edge_ids.iter().cloned().collect();

    let model = build_wire_geometry_model(
        &diagram.nodes,
        &diagram.edges,
        options.wire_geometry,
    );
    let mut wire_scene = Scene::default();
    append_wires_to_scene_for_edges(&mut wire_scene, &model, diagram, &edge_set);

    ScenePatch {
        node_ids: vec![node_id.to_string()],
        edge_ids,
        primitives: wire_scene.primitives,
        hits: hits_for_node(diagram, node_id, options),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagramme_schema::{active_sheet, load_golden_fixture, XY};

    #[test]
    fn patch_includes_connected_edges_for_moved_mixer() {
        let mut project = load_golden_fixture();
        let node_id = "deviceV2-60f0f322".to_string();
        project
            .active_sheet_mut()
            .state
            .nodes
            .iter_mut()
            .find(|n| n.id == node_id)
            .expect("mixer")
            .position = XY { x: 500.0, y: 200.0 };
        let diagram = &project.active_sheet().state;

        let patch = build_scene_patch_for_node(diagram, &node_id, SceneBuildOptions::default());
        assert!(
            !patch.edge_ids.is_empty(),
            "mixer should have connected edges in patch"
        );
        assert!(
            patch.primitives.iter().any(|p| matches!(
                p,
                crate::scene::ScenePrimitive::Polyline { edge_id: Some(_), .. }
            )),
            "patch should include wire polylines"
        );
        assert!(
            !patch.hits.is_empty(),
            "patch should refresh hit targets for moved node"
        );
    }

    #[test]
    fn patch_wire_count_matches_edge_segments() {
        let project = load_golden_fixture();
        let diagram = &active_sheet(&project).state;
        let node_id = "wiretag-950b7690";
        let patch = build_scene_patch_for_node(diagram, node_id, SceneBuildOptions::default());
        assert_eq!(patch.node_ids, vec![node_id.to_string()]);
        assert!(
            patch.edge_ids.len() >= 1,
            "wiretag should have at least one attached edge"
        );
    }
}
