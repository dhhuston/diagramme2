use diagramme_schema::{DiagramState, Node, XY};
use diagramme_scene::build_scene;

fn body_hit<'a>(scene: &'a diagramme_scene::Scene, node_id: &str) -> &'a diagramme_scene::HitTarget {
    scene
        .hits
        .iter()
        .find(|h| h.id == node_id)
        .expect("body hit")
}

#[test]
fn speaker_face_mask_uses_symbol_hull_polygon() {
    let node = Node {
        id: "spk-1".into(),
        node_type: "speakerBlock".into(),
        position: XY { x: 100.0, y: 200.0 },
        data: serde_json::json!({ "line1": "SPK", "line2": "Lobby" }),
        width: None,
        height: None,
        z_index: None,
    };
    let scene = build_scene(&DiagramState {
        nodes: vec![node],
        edges: vec![],
    });
    let hit = body_hit(&scene, "spk-1");
    let poly = hit.face_mask_polygon.as_ref().expect("speaker hull mask");
    assert_eq!(poly.len(), 6);
    assert!(hit.face_mask_bounds.is_none());
}

#[test]
fn mic_face_mask_uses_circle_polygon() {
    let node = Node {
        id: "mic-1".into(),
        node_type: "micBlock".into(),
        position: XY { x: 50.0, y: 80.0 },
        data: serde_json::json!({ "line1": "MIC", "line2": "01" }),
        width: None,
        height: None,
        z_index: None,
    };
    let scene = build_scene(&DiagramState {
        nodes: vec![node],
        edges: vec![],
    });
    let hit = body_hit(&scene, "mic-1");
    let poly = hit.face_mask_polygon.as_ref().expect("mic circle mask");
    assert!(poly.len() >= 16);
    assert!(hit.face_mask_bounds.is_none());
}

#[test]
fn antenna_has_no_opaque_face_mask() {
    let node = Node {
        id: "ant-1".into(),
        node_type: "antennaReceiverSymbol".into(),
        position: XY { x: 10.0, y: 10.0 },
        data: serde_json::json!({ "line1": "ANT-1" }),
        width: None,
        height: None,
        z_index: None,
    };
    let scene = build_scene(&DiagramState {
        nodes: vec![node],
        edges: vec![],
    });
    let hit = body_hit(&scene, "ant-1");
    assert!(hit.face_mask_bounds.is_none());
    assert!(hit.face_mask_polygon.is_none());
}

#[test]
fn volume_control_face_mask_uses_hex_polygon() {
    let node = Node {
        id: "vc-1".into(),
        node_type: "volumeControl".into(),
        position: XY { x: 100.0, y: 200.0 },
        data: serde_json::json!({}),
        width: None,
        height: None,
        z_index: None,
    };
    let scene = build_scene(&DiagramState {
        nodes: vec![node],
        edges: vec![],
    });
    let hit = body_hit(&scene, "vc-1");
    let poly = hit.face_mask_polygon.as_ref().expect("vc hex mask");
    assert_eq!(poly.len(), 6);
    assert!(hit.face_mask_bounds.is_none());
}

#[test]
fn wiretag_face_mask_uses_hull_polygon() {
    let node = Node {
        id: "wt-1".into(),
        node_type: "wiretag".into(),
        position: XY { x: 10.0, y: 10.0 },
        data: serde_json::json!({ "pairIndex": 1, "end": "a" }),
        width: None,
        height: None,
        z_index: None,
    };
    let scene = build_scene(&DiagramState {
        nodes: vec![node],
        edges: vec![],
    });
    let hit = body_hit(&scene, "wt-1");
    let poly = hit.face_mask_polygon.as_ref().expect("wiretag hull mask");
    assert!(poly.len() >= 4);
    assert!(hit.face_mask_bounds.is_none());
}
