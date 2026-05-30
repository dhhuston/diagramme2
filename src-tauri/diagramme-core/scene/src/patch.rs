//! Partial scene rebuild for drag preview (Task 19b).

use std::collections::HashSet;

use diagramme_schema::DiagramState;

use crate::build::{append_node_to_scene, SceneBuildOptions};
use crate::scene::{Scene, ScenePatch};
use crate::wires::append_wires_for_edges;

/// When a wiretag moves, rebuild its partner too (autosize / display text depends on pair).
fn expand_patch_node_ids(diagram: &DiagramState, moved_node_id: &str) -> Vec<String> {
    let mut ids = vec![moved_node_id.to_string()];
    let Some(node) = diagram.nodes.iter().find(|n| n.id == moved_node_id) else {
        return ids;
    };
    if node.node_type != "wiretag" {
        return ids;
    }
    let Some(pair_id) = node.data.get("pairId").and_then(|v| v.as_str()) else {
        return ids;
    };
    for other in &diagram.nodes {
        if other.node_type == "wiretag"
            && other.id != moved_node_id
            && other.data.get("pairId").and_then(|v| v.as_str()) == Some(pair_id)
        {
            ids.push(other.id.clone());
        }
    }
    ids
}

fn edge_ids_for_nodes(diagram: &DiagramState, node_ids: &HashSet<String>) -> Vec<String> {
    diagram
        .edges
        .iter()
        .filter(|e| node_ids.contains(&e.source) || node_ids.contains(&e.target))
        .map(|e| e.id.clone())
        .collect()
}

/// Build a patch replacing geometry for `moved_node_id`, connected wires, and wiretag partner if any.
pub fn build_scene_patch(
    diagram: &DiagramState,
    moved_node_id: &str,
    options: SceneBuildOptions,
) -> ScenePatch {
    let node_ids = expand_patch_node_ids(diagram, moved_node_id);
    let node_id_set: HashSet<String> = node_ids.iter().cloned().collect();
    let edge_ids = edge_ids_for_nodes(diagram, &node_id_set);

    let mut scene = Scene::default();
    let active_bundles = if options.filter_bundle_brackets {
        diagramme_schema::active_bundle_handles(diagram)
    } else {
        HashSet::new()
    };

    for node in &diagram.nodes {
        if node_id_set.contains(&node.id) {
            append_node_to_scene(
                &mut scene,
                node,
                diagram,
                &active_bundles,
                options.filter_bundle_brackets,
            );
        }
    }

    if options.include_wires {
        append_wires_for_edges(&mut scene, diagram, &edge_ids, options.wire_geometry);
    }

    let hits: Vec<_> = scene
        .hits
        .iter()
        .filter(|h| h.node_id.as_ref().is_some_and(|id| node_id_set.contains(id)))
        .cloned()
        .collect();

    ScenePatch {
        node_ids,
        edge_ids,
        primitives: scene.primitives,
        hits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::ScenePrimitive;
    use diagramme_schema::{active_sheet, load_golden_fixture};

    #[test]
    fn patch_for_moved_device_is_smaller_than_full_scene() {
        let project = load_golden_fixture();
        let diagram = &active_sheet(&project).state;
        let full = crate::build_scene(diagram);
        let patch = build_scene_patch(
            diagram,
            "deviceV2-60f0f322",
            SceneBuildOptions::default(),
        );
        assert!(
            patch.primitives.len() < full.primitives.len() / 4,
            "patch {} primitives vs full {}",
            patch.primitives.len(),
            full.primitives.len()
        );
        assert!(!patch.edge_ids.is_empty());
        assert!(patch.node_ids.contains(&"deviceV2-60f0f322".to_string()));
    }

    #[test]
    fn patch_primitives_tagged_with_owner_node_id() {
        let project = load_golden_fixture();
        let diagram = &active_sheet(&project).state;
        let patch = build_scene_patch(
            diagram,
            "deviceV2-60f0f322",
            SceneBuildOptions::default(),
        );
        let owned: usize = patch
            .primitives
            .iter()
            .filter(|p| match p {
                ScenePrimitive::Polyline {
                    owner_node_id: Some(_),
                    edge_id: None,
                    ..
                } => true,
                ScenePrimitive::Text(t) => t.owner_node_id.is_some(),
                _ => false,
            })
            .count();
        assert!(
            owned > 10,
            "expected owned node primitives in patch, got {owned}"
        );
    }
}
