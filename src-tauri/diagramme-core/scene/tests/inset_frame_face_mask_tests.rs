use diagramme_schema::{DiagramState, Node, XY};
use diagramme_scene::build_scene;

fn body_hit<'a>(scene: &'a diagramme_scene::Scene, node_id: &str) -> &'a diagramme_scene::HitTarget {
    scene
        .hits
        .iter()
        .find(|h| h.id == node_id)
        .expect("body hit")
}

fn minimal_device_v2(id: &str, x: f64, y: f64, split: Option<u64>) -> Node {
    let mut data = serde_json::json!({
        "description": "Split demo",
        "leftColumn": [{ "header": "In", "rows": [{ "id": "in-1", "label": "1", "wireCategory": "audio" }] }],
        "rightColumn": [{ "header": "Out", "rows": [{ "id": "out-1", "label": "2", "wireCategory": "audio" }] }],
        "tagCode": "DEM",
        "tagNumber": "1"
    });
    if let Some(n) = split {
        data["splitInstance"] = serde_json::json!(n);
    }
    Node {
        id: id.into(),
        node_type: "deviceV2".into(),
        position: XY { x, y },
        data,
        width: None,
        height: None,
        z_index: None,
    }
}

#[test]
fn split_device_face_mask_uses_breakline_polygon() {
    let scene = build_scene(&DiagramState {
        nodes: vec![
            minimal_device_v2("dev-split-1", 0.0, 0.0, Some(1)),
            minimal_device_v2("dev-split-2", 0.0, 120.0, Some(2)),
        ],
        edges: vec![],
    });
    let poly1 = body_hit(&scene, "dev-split-1")
        .face_mask_polygon
        .as_ref()
        .expect("mask");
    assert_eq!(poly1.len(), 8);
    let poly2 = body_hit(&scene, "dev-split-2")
        .face_mask_polygon
        .as_ref()
        .expect("mask");
    assert_eq!(poly2.len(), 8);
    assert!(body_hit(&scene, "dev-split-1").face_mask_bounds.is_none());
}
