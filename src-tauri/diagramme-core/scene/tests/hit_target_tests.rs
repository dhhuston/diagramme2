use diagramme_scene::build_scene;
use diagramme_schema::{active_sheet, load_dxf_export_test_fixture, load_golden_fixture};

fn node_ids_with_body_hits(project: &diagramme_schema::ProjectState) -> std::collections::HashSet<String> {
    let scene = build_scene(&active_sheet(project).state);
    scene
        .hits
        .iter()
        .filter(|h| h.id == h.node_id.as_deref().unwrap_or(""))
        .filter_map(|h| h.node_id.clone())
        .collect()
}

fn assert_all_nodes_draggable(project: &diagramme_schema::ProjectState) {
    let diagram = &active_sheet(project).state;
    let covered = node_ids_with_body_hits(project);

    let missing: Vec<String> = diagram
        .nodes
        .iter()
        .filter(|n| {
            if n.node_type == "groupingZone" {
                return false;
            }
            if n.node_type == "junction" {
                return n.data.get("rowCount").and_then(|v| v.as_u64()).unwrap_or(0) == 0;
            }
            true
        })
        .filter(|n| !covered.contains(&n.id))
        .map(|n| format!("{} ({})", n.id, n.node_type))
        .collect();

    assert!(
        missing.is_empty(),
        "nodes missing body hit targets: {missing:?}"
    );
}

#[test]
fn palette_fixture_every_node_has_draggable_body_hit() {
    assert_all_nodes_draggable(&load_dxf_export_test_fixture());
}

#[test]
fn comp_gym_every_node_has_draggable_body_hit() {
    assert_all_nodes_draggable(&load_golden_fixture());
}

#[test]
fn grouping_zone_hits_sit_behind_other_nodes() {
    let project = load_dxf_export_test_fixture();
    let scene = build_scene(&active_sheet(&project).state);
    let zone_ids: std::collections::HashSet<_> = active_sheet(&project)
        .state
        .nodes
        .iter()
        .filter(|n| n.node_type == "groupingZone")
        .map(|n| n.id.as_str())
        .collect();

    let first_zone_hit = scene.hits.iter().position(|h| {
        h.node_id
            .as_deref()
            .is_some_and(|id| zone_ids.contains(id) && h.id.contains(":boundary:"))
    });
    let first_device_hit = scene
        .hits
        .iter()
        .position(|h| {
            h.node_id
                .as_deref()
                .is_some_and(|id| id.starts_with("dev-") || id.contains("device"))
        });

    if let (Some(zi), Some(di)) = (first_zone_hit, first_device_hit) {
        assert!(
            zi < di,
            "grouping zone hits should precede device hits for lower pick priority"
        );
    }
}
