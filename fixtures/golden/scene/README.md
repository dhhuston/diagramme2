# Golden scene JSON

Baselines for `build_scene()` output on the Comp Gym F102A fixture.

| File | Regenerate |
|------|------------|
| `comp-gym-f102a.json` | `cargo test -p diagramme-scene write_golden_scene_baseline -- --ignored` |

Parity gate: `cargo test -p diagramme-scene comp_gym_scene_matches_golden_baseline`

Scene JSON is diagram px (Y-down), the same payload Konva will consume via `get_diagram_scene`.
