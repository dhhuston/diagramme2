use diagramme_schema::{DiagramState, Node, XY};
use diagramme_scene::build_scene;

fn grouping_zone_node(x: f64, y: f64, w: f64, h: f64) -> Node {
    Node {
        id: "zone-a".into(),
        node_type: "groupingZone".into(),
        position: XY { x, y },
        data: serde_json::json!({ "label": "Gym", "shape": "rect" }),
        width: Some(w),
        height: Some(h),
        z_index: None,
    }
}

#[test]
fn grouping_zone_uses_boundary_hits_not_fill_rect() {
    let diagram = DiagramState {
        nodes: vec![grouping_zone_node(100.0, 100.0, 200.0, 120.0)],
        edges: vec![],
    };
    let scene = build_scene(&diagram);
    assert!(
        scene.hits.iter().all(|h| h.id.contains(":boundary:")),
        "expected only boundary strip hits, got {:?}",
        scene.hits.iter().map(|h| &h.id).collect::<Vec<_>>()
    );
    let interior = (150.0, 150.0);
    let inside = scene
        .hits
        .iter()
        .any(|h| {
            let b = &h.bounds;
            interior.0 >= b.x
                && interior.0 <= b.x + b.width
                && interior.1 >= b.y
                && interior.1 <= b.y + b.height
        });
    assert!(!inside, "interior point must not hit grouping zone");
}
