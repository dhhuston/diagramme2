# Diagram fixtures

Regression and parity corpus copied from Diagramme v6. Each `.diagramme` file is valid JSON with the same on-disk envelope v2 uses:

| Field | Value |
|-------|--------|
| `format` | `"diagramme"` |
| `version` | `2` |

## v6 ↔ v2 compatibility

Diagramme v2 does **not** introduce a new file format. It reads and writes the same v6 `ProjectState` envelope. Compatibility is handled in three layers:

1. **Open gate** — `open_diagram_from_json` validates `format` and `version` before parsing. Unknown versions fail with a clear error instead of partial parse failures.
2. **Single scene** — Konva (`get_diagram_scene`) and DXF export both call `build_scene()` on the same `ProjectState`. DXF is `scene_to_cad` + emit only — no export-only wire rerouting. **Overlap fidelity:** wires and symbols the user drew on top of each other must overlap identically in DXF.
3. **Save heal** — `save_diagram` / `save_diagram_compact` run `normalize_diagram_for_persist` on the live project: strips stale `innerCorners`, orphan bundle metadata, and stale handle centers so files gradually heal on save.

Preset sidecars (`.avdevice` / `.plate`) use `schemaVersion: 1`; Rust validation lives in `diagramme_schema::parse_preset_file_text` for when preset file IPC lands.

CI opens every fixture below (envelope + parse + round-trip save + DXF smoke) in `src-tauri/tests/fixture_compatibility_test.rs`.

## Primary golden: `golden/Comp Gym F102A.diagramme`

**Source:** real-world gym AV schematic (Comp Gym F102A). Updated from `Comp Gym F102A_updated.diagramme` (v6 re-save with corrected speaker passthru routing and input-plate bundle anchor).

33 nodes, 44 edges on the active sheet: `deviceV2`, `avPlate`, `speakerBlock`, `antennaReceiverSymbol`, `flyoffNote`, `wiretag`, `groupingZone`.

The schematic-only golden omits the old palette row (junction, micBlock, etc. at y ≈ 959); use `diagrams/dxf-export-test.diagramme` for full node-type palette coverage.

**All scene, DXF, schema, and integration tests load this file** via `diagramme_schema::load_golden_fixture()`.

Golden DXF baseline: `golden/dxf/comp-gym-f102a.dxf` (regenerate with `cargo test -p diagramme-dxf write_golden_baseline -- --ignored`).

## `diagrams/dxf-export-test.diagramme`

**Source:** `Diagramme_v6/src/fixtures/buildDxfExportTestDiagram.ts`

Synthetic palette-exercise sheet (every node type in one place). Kept for manual parity checks; **not** the default test fixture.

## `diagrams/cafeteria-d104a.diagramme`

**Source:** `Diagramme_v6/troubleshooting/Cafeteria D104A.diagramme`

Real-world cafeteria AV schematic (room D104A): 52 nodes, 61 edges. Covered by the v6 compatibility test matrix (open, round-trip, DXF smoke).
