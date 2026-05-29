//! Golden scene JSON baseline for Comp Gym F102A (Task 11).

use std::path::{Path, PathBuf};

use diagramme_scene::{build_scene, Scene};
use diagramme_schema::{active_sheet_state, load_golden_fixture};

fn golden_scene_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/golden/scene/comp-gym-f102a.json")
}

fn scene_json(scene: &Scene) -> String {
    serde_json::to_string_pretty(scene).expect("serialize scene")
}

#[test]
fn comp_gym_scene_matches_golden_baseline() {
    let golden_path = golden_scene_path();
    let expected_json = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
        panic!(
            "missing golden scene at {} ({e}); regenerate with \
             `cargo test -p diagramme-scene write_golden_scene_baseline -- --ignored`",
            golden_path.display()
        )
    });

    let project = load_golden_fixture();
    let actual = build_scene(active_sheet_state(&project));
    let actual_json = scene_json(&actual);

    if actual_json != expected_json {
        let expected: Scene = serde_json::from_str(&expected_json).expect("parse golden scene");
        eprintln!(
            "primitive count: actual {} expected {}",
            actual.primitives.len(),
            expected.primitives.len()
        );
        eprintln!(
            "hit count: actual {} expected {}",
            actual.hits.len(),
            expected.hits.len()
        );
        eprintln!(
            "extent: actual ({}, {}, {}, {}) expected ({}, {}, {}, {})",
            actual.extent.x,
            actual.extent.y,
            actual.extent.width,
            actual.extent.height,
            expected.extent.x,
            expected.extent.y,
            expected.extent.width,
            expected.extent.height
        );
    }
    assert_eq!(actual_json, expected_json);
}

#[test]
#[ignore = "run once to write golden baseline: cargo test -p diagramme-scene write_golden_scene_baseline -- --ignored"]
fn write_golden_scene_baseline() {
    let project = load_golden_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let golden = golden_scene_path();
    if let Some(parent) = golden.parent() {
        std::fs::create_dir_all(parent).expect("create golden scene dir");
    }
    std::fs::write(&golden, scene_json(&scene)).expect("write golden scene json");
    assert!(golden.exists());
    eprintln!(
        "wrote {} ({} primitives, {} hits)",
        golden.display(),
        scene.primitives.len(),
        scene.hits.len()
    );
}
