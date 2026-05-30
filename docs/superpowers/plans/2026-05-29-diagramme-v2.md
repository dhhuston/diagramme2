# Diagramme v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Diagramme v2 as a full 1:1 rebuild: v6 `.diagramme` compatibility, Konva canvas driven by Rust `Scene`, strict-mirror DXF export, and all reports in Rust.

**Architecture:** Tauri 2 + React shell; Rust workspace (`diagramme-core`) owns schema, geometry, wires, scene, DXF, reports. Konva renders scene 1:1 in diagram px; DXF uses `scene_to_cad()` only (`× 1/72`, Y mirror). Reference implementation: `../Diagramme_v6`.

**Tech Stack:** Rust 2021 workspace, Tauri 2, React 19, TypeScript, Vite, Konva/react-konva, `rust_xlsxwriter`, Vitest (UI), `cargo test` (core).

**Spec:** [`docs/superpowers/specs/2026-05-29-diagramme-v2-design.md`](../specs/2026-05-29-diagramme-v2-design.md)

**Recommended:** Use a git worktree for v2 (`diagramme2` is the target repo root).

---

## Progress snapshot (2026-05-29)

| Phase | Status | Notes |
|-------|--------|-------|
| 0–1 Bootstrap + fixtures | **Done** | Tauri/Vite workspace, Comp Gym + palette fixtures |
| 2–3 Schema + geometry | **Done** | `ProjectState`, ports, bounds, text measure |
| 4 Scene + `scene_to_cad` | **Done** | All node types except `wireSplit`; Comp Gym golden scene |
| 5 DXF export | **Done** | `export_revit_dxf`, strict-mirror emit |
| 6 Reports | **Not started** | Crates are stubs |
| **7 Konva canvas** | **Partial** | Renderer + viewport + node drag; see [Drawing pipeline — as built](#drawing-pipeline--as-built) |
| 8 React shell | **Not started** | Dev shell only (`Load Comp Gym` button) |
| 9–10 CI + perf | **Partial** | `ScenePatch` drag preview done; profiling TBD |

---

## Drawing pipeline — as built

The original plan assumed a simpler Konva layer. The implemented pipeline includes Rust wire geometry and canvas calibration not captured in Tasks 17–19.

### Rust → Scene (authoritative geometry)

```
DiagramState
  → diagramme-wires (sharp polylines, obstacle avoidance, crossing gaps, bundle fillets)
  → diagramme-scene/build_scene
       • node dispatch + bundle brackets + HitTarget per node
       • wires: category color, mismatch, fillet arcs tessellated to polylines
  → Scene JSON via get_diagram_scene
```

**Wire geometry beyond original Task 7–8 scope (implemented):**

| Module | Purpose |
|--------|---------|
| `wires/node_move.rs` | On `move_node`: translate `innerCorners`, update handle centers on connected edges |
| `wires/wire_avoidance.rs` | Obstacle-aware inner chain routing |
| `wires/bundle_wire.rs` | Bundle styling propagates through wiretag pairs |
| `scene/wires.rs` | Wire category colors, fillet arc tessellation for Konva + DXF parity |

**Scene hits (interaction):** Every draggable node type emits a body `HitTarget` via `push_node_body_hit`. Grouping zones insert at the front of the hits list so devices on top win pick order. Frame nodes also emit per-row port hits (deviceV2, avPlate, patch panels).

### Konva → display (dumb renderer + chrome)

| File | Role |
|------|------|
| `SceneRenderer.tsx` | Maps `ScenePrimitive` → Konva; **no** geometry clone during drag |
| `SceneTextNode.tsx` | Insertion-point anchors; cap height → em via `ARIAL_NARROW_CAP_TO_EM` (0.75) |
| `sceneRenderUtils.ts` | Wire keys (`polyline-{edge_id}-{index}`), schematic ink 0.5px hairline, fill colors |
| `DiagramStage.tsx` | Viewport pan/zoom; `fitRevision` fits on load/resize only |
| `useDiagramInteraction.ts` | Pointer drag/pan; 60ms throttled preview |
| `dragNode.ts` | Snap grid, `dragVisualDelta` (pointer vs last Rust scene origin) |

### Drag (interim — not final per spec)

```
pointermove → rAF: dashed drag outline (dragVisualDelta)
            → 60ms throttle: move_node(isDragPreview=true) → get_diagram_scene (full rebuild)
pointerup   → move_node(commit) → full scene
```

- **No geometry-clone overlay** — wires always come from Rust scene (fixes detached wires / ghost boxes).
- **`ScenePatch` drag preview** — throttled patch merge during drag; full scene on commit only.
- **`beginDragPreview` generation guard** — drops stale IPC responses during fast drag.

### Konva text calibration (not export scale)

Scene stores **cap height** in `height_px`. DXF uses `height_px / 72`. Konva sets `fontSize = height_px / 0.75` so rendered cap height matches scene — same calibration as v6 canvas, **not** `EXPORT_TEXT_VISUAL_SCALE` (forbidden for DXF).

### Tests covering drawing

- `scene/tests/golden_scene_tests.rs` — Comp Gym scene JSON baseline
- `scene/tests/hit_target_tests.rs` — every node has body hit; zone pick order
- `wires/tests/bundle_wire_tests.rs` — wiretag bundle propagation
- `canvas/sceneRenderUtils.test.ts`, `dragNode.test.ts`, `hitTest.test.ts`

---

## File structure (target)

```
diagramme2/
  fixtures/
    diagrams/                         # .diagramme from v6
    golden/scene/                     # JSON per fixture
    golden/dxf/                       # v2-baselined DXF (text height without 0.75)
    golden/reports/                   # row snapshots
  src/
    App.tsx                           # shell, menus (port from v6)
    canvas/
      DiagramStage.tsx                # Konva Stage + viewport
      SceneRenderer.tsx               # Scene → Konva shapes (authoritative, no drag clone)
      SceneTextNode.tsx               # Text insertion anchors + cap-height calibration
      sceneRenderUtils.ts             # Keys, stroke, text anchor, fit extent
      sceneTypes.ts                   # Mirrors Rust Scene JSON
      useDiagramScene.ts              # Scene fetch, drag preview generation guard
      useViewport.ts
      hitTest.ts
      interaction/
        useDiagramInteraction.ts      # drag + pan (replaces planned InteractionController.tsx)
        dragNode.ts                   # snap, body bounds, dragVisualDelta
    hooks/useDiagramFileOps.ts        # port from v6
    tauriIpc.ts                       # typed invoke wrappers
    components/                       # report dialogs (port from v6)
  src-tauri/
    Cargo.toml                        # workspace root
    src/main.rs
    src/commands.rs
    diagramme-core/
      Cargo.toml
      schema/src/lib.rs               # ProjectState
      geometry/src/lib.rs             # constants, ports, bounds, text measure
      wires/src/lib.rs                # routing, postprocess
      scene/src/lib.rs                # Scene, build_scene, scene_to_cad
      dxf/src/lib.rs                  # Revit DXF emit + sanitize
      export-model/src/lib.rs
      reports/src/lib.rs
  package.json
  vitest.config.ts
```

**v6 port map (keep open while working):**

| v2 crate | Primary v6 sources |
|----------|------------------|
| schema | `Diagramme_v6/src-tauri/src/model.rs`, `commands.rs` |
| geometry | `constants/paperScale.ts`, `nodes/portGeometry.ts`, `constants/dxfSchematicLayout.ts`, node layout files |
| wires | `wireGeometry/*`, `edges/schematicEdgePath.ts`, `export/dxfWirePostprocess.ts` |
| scene | `export/revitDxf/revitDxfNodeDetail.ts` (layout → scene primitives, not separate DXF path) |
| dxf | `export/revitDxf/*`, `export/dxfCadCoords.ts`, `export/dxfSanitize.ts` |
| reports | `export/deviceTagsReport.ts`, `plateConnectionsReport.ts`, `groupedInventoryReport.ts`, `equipmentListXlsx.ts`, `equipmentReportCore.ts`, `revitPrepExport.ts` |

---

## Phase 0 — Repository bootstrap

### Task 0: Scaffold Tauri + Vite + Rust workspace

**Files:**
- Create: `diagramme2/Cargo.toml`, `src-tauri/Cargo.toml`, `package.json`, `vite.config.ts`, `index.html`
- Copy/adapt: `Diagramme_v6/src-tauri/tauri.conf.json`, `Diagramme_v6/package.json` (deps: add `konva`, `react-konva`; remove `@xyflow/react` when canvas lands)

- [ ] **Step 1:** Create workspace `Cargo.toml`:

```toml
[workspace]
members = [
  "src-tauri",
  "src-tauri/diagramme-core/schema",
  "src-tauri/diagramme-core/geometry",
  "src-tauri/diagramme-core/wires",
  "src-tauri/diagramme-core/scene",
  "src-tauri/diagramme-core/dxf",
  "src-tauri/diagramme-core/export-model",
  "src-tauri/diagramme-core/reports",
]
resolver = "2"
```

- [ ] **Step 2:** Create `src-tauri/diagramme-core/schema/Cargo.toml` + empty `lib.rs`; repeat stub for each crate with path dependency chain: `schema` ← `geometry` ← `wires` ← `scene` ← `dxf`, `export-model` ← `reports`.

- [ ] **Step 3:** Copy Vite/React/Tauri bootstrap from v6; `npm install`; `cargo check` at workspace root.

- [ ] **Step 4:** Verify empty app launches: `npm run tauri dev`.

- [ ] **Step 5:** Commit: `chore: scaffold diagramme2 workspace and Tauri shell`

---

### Task 1: Copy fixture corpus

**Files:**
- Create: `fixtures/diagrams/*`
- Copy from v6: `src/fixtures/buildDxfExportTestDiagram.ts` JSON output, `troubleshooting/Cafeteria D104A.diagramme` (if present)

- [ ] **Step 1:** Copy DXF export test diagram JSON to `fixtures/diagrams/dxf-export-test.diagramme` (extract JSON from v6 fixture builder or saved file).

- [ ] **Step 2:** Copy `Cafeteria D104A.diagramme` and any v6 test `.diagramme` files into `fixtures/diagrams/`.

- [ ] **Step 3:** Add `fixtures/README.md` listing each file and what it exercises.

- [ ] **Step 4:** Commit: `chore: add diagram fixture corpus from v6`

---

## Phase 1 — Schema and persistence

### Task 2: Port `ProjectState` to `diagramme-schema`

**Files:**
- Create: `src-tauri/diagramme-core/schema/src/lib.rs`, `model.rs`, `model_tests.rs`
- Reference: `Diagramme_v6/src-tauri/src/model.rs`

- [ ] **Step 1: Write failing round-trip test**

```rust
// schema/src/model_tests.rs
use diagramme_schema::ProjectState;

#[test]
fn open_dxf_export_test_fixture() {
    let json = include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");
    let p: ProjectState = serde_json::from_str(json).expect("parse");
    assert!(!p.sheets.is_empty());
    let again = serde_json::to_string(&p).unwrap();
    let p2: ProjectState = serde_json::from_str(&again).unwrap();
    assert_eq!(p.active_sheet_id, p2.active_sheet_id);
}
```

- [ ] **Step 2:** Run `cargo test -p diagramme-schema` — expect FAIL (types missing).

- [ ] **Step 3:** Copy/adapt `model.rs` from v6 into `diagramme-schema` (`Node`, `Edge`, `DiagramState`, `Sheet`, `EmbeddedPreset`, `ProjectState`, `XY`).

- [ ] **Step 4:** Run `cargo test -p diagramme-schema` — expect PASS.

- [ ] **Step 5:** Commit: `feat(schema): v6-compatible ProjectState serde`

---

### Task 3: Port mutation commands to Tauri

**Files:**
- Create: `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`
- Reference: `Diagramme_v6/src-tauri/src/commands.rs`

- [ ] **Step 1:** Port `AppState` mutex, `get_project`, `open_diagram`, `save_diagram`, `save_diagram_compact`, `set_state`, `move_node`, `update_node`, undo/redo, sheet commands.

- [ ] **Step 2:** Add integration test `commands_tests.rs`: open fixture JSON via `open_diagram`, mutate node position, `save_diagram_compact`, assert JSON contains new position.

- [ ] **Step 3:** Create `src/tauriIpc.ts` with typed wrappers matching v6 command names.

- [ ] **Step 4:** Commit: `feat(app): port core Tauri IPC from v6`

---

## Phase 2 — Geometry and constants

### Task 4: Port paper scale constants

**Files:**
- Create: `geometry/src/paper_scale.rs`, `geometry/src/units.rs`, `geometry/src/paper_scale_tests.rs`
- Reference: `Diagramme_v6/src/constants/paperScale.ts`, `export/cadUnits.ts`

- [ ] **Step 1: Failing calibration test**

```rust
#[test]
fn connector_pitch_is_one_eighth_inch() {
    assert_eq!(PX_PER_INCH, 72.0);
    assert_eq!(CONNECTOR_LINE_PITCH_PX, 9.0);
    assert!((px_to_in(CONNECTOR_LINE_PITCH_PX) - 0.125).abs() < 1e-9);
}
```

- [ ] **Step 2:** Implement constants mirroring v6 (`PX_PER_INCH`, `CONNECTOR_LINE_PITCH_PX`, `DEVICE_V2_WIDTH_PX`, snap grid, etc.).

- [ ] **Step 3:** Implement `px_to_in`, `in_to_px` — **only** place linear scale is defined.

- [ ] **Step 4:** `cargo test -p diagramme-geometry` — PASS.

- [ ] **Step 5:** Commit: `feat(geometry): paper scale constants and px_to_in`

---

### Task 5: Port port geometry and node bounds

**Files:**
- Create: `geometry/src/port_geometry.rs`, `geometry/src/node_bounds.rs`
- Reference: `Diagramme_v6/src/nodes/portGeometry.ts`, `export/revitDxf/revitDxfNodeDetail.ts` (`detailedRevitNodeBoundsInches`)

- [ ] **Step 1:** Port `get_analytical_port_xy` for deviceV2, avPlate, patch panels, wiretag, mic/speaker, etc.

- [ ] **Step 2:** Write tests against known port positions from `dxfWireGeometry.test.ts` cases (translate TS inputs to Rust tests).

- [ ] **Step 3:** Implement `node_bounds_diagram_px(node) -> RectPx` per node type (start with `deviceV2`, `avPlate`, `wiretag`).

- [ ] **Step 4:** Commit: `feat(geometry): port positions and node bounds`

---

### Task 6: Text measurement (strict mirror prep)

**Files:**
- Create: `geometry/src/text_measure.rs`
- Reference: v6 wiretag autosize, `LABEL_FONT_PX`, `DEVICE_PORT_LABEL_FONT_PX`

- [ ] **Step 1:** Define `TextStyle { font: "Arial Narrow", height_px, halign, valign }`.

- [ ] **Step 2:** Implement width estimation for wiretag labels (port logic from v6 wiretag graph); store resulting width in scene bounds.

- [ ] **Step 3:** Test: 9px cap height label → width matches v6 fixture bounds ±1px.

- [ ] **Step 4:** Commit: `feat(geometry): rust text measurement for scene bounds`

---

## Phase 3 — Wire routing

### Task 7: Port sharp polyline builder

**Files:**
- Create: `wires/src/sharp_polyline.rs`, `wires/src/obstacles.rs`, `wires/src/types.rs`
- Reference: `wireGeometry/wireSharpPolyline.ts`, `edges/schematicEdgePath.ts`, `edges/schematicWireObstacles.ts`

- [ ] **Step 1:** Port types: `FlowXY`, edge handle centers from `edge.data`, `innerCorners`.

- [ ] **Step 2:** Port `wire_sharp_polyline_for_edge` with tests copied from `wireSharpPolyline.test.ts` (convert fixtures to Rust).

- [ ] **Step 3:** Port obstacle collection + inner chain building.

- [ ] **Step 4:** Commit: `feat(wires): sharp polyline routing`

---

### Task 8: Port DXF wire postprocess

**Files:**
- Create: `wires/src/postprocess.rs`
- Reference: `export/dxfWirePostprocess.ts`

- [ ] **Step 1:** Port crossing gaps + bundle fillet arcs; tests from `dxfWirePostprocess.test.ts`.

- [ ] **Step 2:** Expose `WireGeometryModel` equivalent: per-edge polylines + dxf-ready pieces.

- [ ] **Step 3:** Parity test: for `dxf-export-test.diagramme`, Rust wire polylines match v6 `buildWireGeometryModel` output (build TS snapshot once, check in CI or committed JSON).

- [ ] **Step 4:** Commit: `feat(wires): crossing gaps and bundle fillets`

---

## Phase 4 — Scene graph and `scene_to_cad`

### Task 9: Define Scene types

**Files:**
- Create: `scene/src/scene.rs`, `scene/src/cad_transform.rs`

- [ ] **Step 1:** Define serde types:

```rust
pub struct Scene {
    pub primitives: Vec<ScenePrimitive>,
    pub extent: RectPx,
    pub hits: Vec<HitTarget>,
}

pub enum ScenePrimitive {
    Polyline { points: Vec<PointPx>, stroke_px: f64, layer: String, color: u32, edge_id: Option<String> },
    Rect { rect: RectPx, stroke_px: f64, fill: Option<u32>, layer: String },
    Solid { vertices: [PointPx; 4], layer: String },
    Text(SceneText),
}

pub struct SceneText {
    pub position: PointPx,
    pub content: String,
    pub height_px: f64,
    pub halign: HAlign,
    pub valign: VAlign,
    pub font: String, // always "Arial Narrow"
}
```

- [ ] **Step 2:** Implement `scene_to_cad` + `scene_point_to_cad` exactly as spec (no extra text scale).

- [ ] **Step 3:** Unit test: point (0, 72) with extent height 144px → y_cad mirrors correctly.

- [ ] **Step 4:** Commit: `feat(scene): Scene types and scene_to_cad`

---

### Task 10: Scene builder — node dispatch (incremental by node type)

**Files:**
- Create: `scene/src/build.rs`, `scene/src/nodes/mod.rs`, `scene/src/nodes/device_v2.rs`, …
- Reference: `revitDxfNodeDetail.ts`, node layout TS files, `dxf-export-audit.md` matrix

Implement **one node type per commit** in this order (each with golden scene JSON assertion):

| Order | Node type | v6 source |
|-------|-----------|-----------|
| 10a | `deviceV2` | `DeviceNodeV2.tsx`, `deviceV2ColumnLayout.ts` |
| 10b | `avPlate` | `AvPlateNode.tsx`, `avPlateLayout.ts` |
| 10c | patch panels | `LppPatchPanelNode.tsx` |
| 10d | `wiretag`, `textBlock`, `flyoffNote` | respective nodes |
| 10e | mic/speaker/VC/antenna | symbol nodes |
| 10f | `groupingZone`, `wireSplit`, `junction` | |

- [ ] **Step 1 (deviceV2 example):** Failing test loads fixture, builds scene, asserts first device has frame rect width `px_to_in(DEVICE_V2_WIDTH_PX)`.

- [ ] **Step 2:** Implement `append_device_v2_scene(&mut Scene, &Node)`.

- [ ] **Step 3:** Assert `SceneText` heights equal v6 role px (9 for tag, 5 for cell) — **not** × 0.75.

- [ ] **Step 4:** Repeat for each node type in table.

- [ ] **Step 5:** Commit per node type: `feat(scene): deviceV2 scene primitives`

---

### Task 11: Assemble wires into Scene

**Files:**
- Modify: `scene/src/build.rs`

- [ ] **Step 1:** `build_scene(diagram: &DiagramState) -> Scene` calls geometry + wires + node dispatch.

- [ ] **Step 2:** Wire polylines become `ScenePrimitive::Polyline` with category layer/color.

- [ ] **Step 3:** Golden test: `fixtures/golden/scene/dxf-export-test.json` — commit first baseline from Rust builder.

- [ ] **Step 4:** Commit: `feat(scene): build_scene with wires`

---

## Phase 5 — DXF export

### Task 12: DXF document writer (Rust)

**Files:**
- Create: `dxf/src/document.rs`, `dxf/src/primitives.rs`, `dxf/src/sanitize.rs`, `dxf/src/emit.rs`
- Reference: `revitDxfDocument.ts`, `revitDxfPrimitives.ts`, `revitDxfSanitize.ts`

- [ ] **Step 1:** Implement minimal DXF writer (HEADER, TABLES: LAYER/STYLE, ENTITIES) without external crate OR evaluate `dxf` crate — must support LWPOLYLINE, TEXT, SOLID, LINE.

- [ ] **Step 2:** Port sanitize rules: SOLID-first, unique handles, AC1018, Arial Narrow STYLE, hairline, no HATCH/3DFACE.

- [ ] **Step 3:** Test: minimal scene → non-empty DXF string, contains `Arial Narrow`.

- [ ] **Step 4:** Commit: `feat(dxf): revit-safe DXF writer skeleton`

---

### Task 13: Scene → DXF emit

**Files:**
- Create: `dxf/src/scene_emit.rs`

- [ ] **Step 1:** For each `ScenePrimitive`, emit via `scene_to_cad` — **no** alternate code path.

- [ ] **Step 2:** Text height: `px_to_in(text.height_px)` only; test rejects any `EXPORT_TEXT_VISUAL_SCALE` constant in crate.

- [ ] **Step 3:** Parity test: every `SceneText` in fixture → parsed DXF TEXT entity height matches within ε.

- [ ] **Step 4:** Wire parity: scene polyline cad points match LWPOLYLINE vertices in export.

- [ ] **Step 5:** Golden DXF baseline in `fixtures/golden/dxf/` (re-baseline vs v6 for text heights).

- [ ] **Step 6:** Port autodesk-safe tests from `revitDxfExport.test.ts` (duplicate handles, HEADER vars).

- [ ] **Step 7:** Commit: `feat(dxf): emit Scene to Revit DXF with strict mirror`

---

### Task 14: Tauri `export_revit_dxf` command

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src/tauriIpc.ts`

- [ ] **Step 1:** `export_revit_dxf() -> Result<String, String>` builds scene from active sheet, emits DXF.

- [ ] **Step 2:** Vitest smoke: invoke in Tauri test env or Rust integration test only.

- [ ] **Step 3:** Commit: `feat(app): export_revit_dxf IPC`

---

## Phase 6 — Reports

### Task 15: Port export model

**Files:**
- Create: `export-model/src/lib.rs`
- Reference: `diagramExportModel.ts`

- [ ] **Step 1:** Port entity enumeration; tests from equipment list expectations.

- [ ] **Step 2:** Commit: `feat(export-model): diagram export entities`

---

### Task 16: Port reports + XLSX

**Files:**
- Create: `reports/src/device_tags.rs`, `plate_connections.rs`, `grouped_inventory.rs`, `equipment_list.rs`, `revit_prep.rs`, `xlsx.rs`
- Reference: v6 `export/*Report.ts`, `equipmentListXlsx.ts`

- [ ] **Step 1:** Port `build_device_tags_report` + test from `deviceTagsReport.test.ts`.

- [ ] **Step 2:** Port `build_plate_connections_report` + test.

- [ ] **Step 3:** Port `build_grouped_inventory_report` + test.

- [ ] **Step 4:** Implement XLSX writers with `rust_xlsxwriter`; golden byte or row snapshots in `fixtures/golden/reports/`.

- [ ] **Step 5:** Tauri commands: `build_*_report`, `export_*_xlsx`, `build_revit_prep_payload`.

- [ ] **Step 6:** Commit: `feat(reports): rust reports and XLSX exports`

---

## Phase 7 — Konva canvas

> **Revised 2026-05-29** after drag/wire/hit-target work. See [Drawing pipeline — as built](#drawing-pipeline--as-built).

### Task 17: Scene renderer — **DONE**

**Files:** `src/canvas/SceneRenderer.tsx`, `SceneTextNode.tsx`, `sceneRenderUtils.ts`, `sceneTypes.ts`

- [x] Map `ScenePrimitive` → Konva `Line`, `Rect`, `Text`, `Shape` (solid).
- [x] Text: Arial Narrow; cap height via `sceneCapHeightToFontSizePx` (see calibration note above).
- [x] Stroke: wires at `stroke_px`; schematic ink at 0.5px hairline.
- [x] Wire segment keys: `polyline-{edge_id}-{index}` (fixes stale Konva lines after crossing-gap splits).
- [x] Unit tests: `sceneRenderUtils.test.ts`.
- [ ] Optional: golden scene JSON render snapshot test.

---

### Task 18: Diagram stage + viewport — **DONE**

**Files:** `src/canvas/DiagramStage.tsx`, `useViewport.ts`, `useDiagramScene.ts`

- [x] Stage at 1 diagram px = 1 unit; wheel zoom (uniform), pan.
- [x] `get_diagram_scene` on load and after IPC mutations.
- [x] `fitRevision` — viewport fit on diagram load / resize, **not** on drag rebuilds.
- [x] Layer `clearBeforeDraw`; `hitGraphEnabled={false}` (hits from Rust `Scene.hits`, not Konva shapes).
- [ ] Document manual check: export DXF unchanged after viewport zoom (Rust-only; viewport is display-only).

---

### Task 19: Interaction controller — **PARTIAL**

**Files (as built):** `interaction/useDiagramInteraction.ts`, `dragNode.ts`, `hitTest.ts`  
**Planned but not created:** `InteractionController.tsx`, `connectPorts.ts`

#### 19a — Hit test + node drag — **DONE**

- [x] Hit test using Rust `Scene.hits` (diagram px, inverse viewport).
- [x] Body hits for all node types (`push_node_body_hit`); grouping zones behind other nodes.
- [x] Node drag: dashed outline via `dragVisualDelta`; authoritative scene from Rust.
- [x] Throttled preview: `move_node(..., isDragPreview=true)` + `get_diagram_scene_patch` every ~60ms.
- [x] Commit on pointer up: `move_node` + full scene refresh.
- [x] Rust: `apply_node_move_geometry` (inner corners + handle centers) on preview and commit.
- [x] Tests: `hitTest.test.ts`, `dragNode.test.ts`, `hit_target_tests.rs`.

#### 19b — Scene patch drag preview — **DONE** (was Task 25)

- [x] `get_diagram_scene_patch(node_id)` — moved node + wiretag partner + connected wire polylines.
- [x] Frontend merges patch into cached scene (`applyScenePatch`) instead of full rebuild during drag.
- [ ] Profile Comp Gym / Cafeteria: justify patch vs full rebuild.

#### 19c — Remaining gestures — **NOT STARTED**

- [ ] Port connect (wire creation IPC + gesture).
- [ ] Node resize (`update_dims` + handle UI).
- [ ] Wire inner-corner drag (`update_edge` / inner-corner commands).
- [ ] Multi-select drag (`move_nodes`).

---

## Phase 8 — React shell

### Task 20: File ops and menus

**Files:**
- Port: `src/hooks/useDiagramFileOps.ts`, menu sections from `App.tsx`

- [ ] **Step 1:** Port file new/open/save/dirty baseline/recovery from v6.

- [ ] **Step 2:** Wire export menu items to Rust IPC (`export_revit_dxf`, XLSX commands) with `flushCanvasToRustBeforePersist`.

- [ ] **Step 3:** Remove all `import('./export')` from App.

- [ ] **Step 4:** Commit: `feat(ui): file ops and export menus via Rust`

---

### Task 21: Properties panel, sheets, presets

**Files:**
- Port: properties panel, sheet tabs, preset toolbox from v6 components

- [ ] **Step 1:** Properties → `update_node` IPC (debounced pending queue like v6).

- [ ] **Step 2:** Presets import/export `.avdevice`/`.plate`; `presetLibrary` via Rust commands.

- [ ] **Step 3:** Multi-sheet switch → `applyRustProject` + full scene refetch.

- [ ] **Step 4:** Commit: `feat(ui): properties sheets presets`

---

### Task 22: Report dialogs

**Files:**
- Port: `DeviceTagsReportDialog.tsx`, `PlateConnectionsReportDialog.tsx`, `GroupedInventoryReportDialog.tsx`

- [ ] **Step 1:** Dialogs fetch JSON from `build_*_report` IPC instead of TS builders.

- [ ] **Step 2:** Export buttons call `export_*_xlsx` → save dialog with bytes.

- [ ] **Step 3:** Commit: `feat(ui): report dialogs from Rust`

---

## Phase 9 — Integration gates

### Task 23: CI gate script

**Files:**
- Create: `scripts/verify-gates.sh`, `.github/workflows/ci.yml` (if using GitHub)

- [ ] **Step 1:** `cargo test --workspace`

- [ ] **Step 2:** Scene golden diff for all fixtures in `fixtures/diagrams/`

- [ ] **Step 3:** DXF golden diff (normalize `\r\n`)

- [ ] **Step 4:** Report row snapshot diff

- [ ] **Step 5:** `rg EXPORT_TEXT_VISUAL_SCALE` must return no matches in v2 repo

- [ ] **Step 6:** Commit: `ci: integration gate script`

---

### Task 24: Manual CAD signoff checklist

**Files:**
- Create: `docs/cad-export-1to1-validation.md` (copy from v6, update for strict mirror text)

- [ ] **Step 1:** Run checklist on `Cafeteria D104A.diagramme` + one smaller fixture.

- [ ] **Step 2:** Record signoff date in checklist doc.

- [ ] **Step 3:** Commit: `docs: CAD validation signoff`

---

## Phase 10 — Performance pass

### Task 25: Scene patch + profiling — **MERGED INTO Task 19b**

Drag preview currently uses full `get_diagram_scene` throttled to ~60ms with generation guards. This works but is not the spec target for Cafeteria-scale diagrams.

**Files:** `scene/src/patch.rs` (new), `commands.rs`, `useDiagramScene.ts`, `SceneRenderer.tsx`

- [ ] **Step 1:** Profile Comp Gym + Cafeteria: `build_scene` wall time for full vs partial.
- [x] **Step 2:** Define `ScenePatch` JSON: replaced primitives + hits for moved nodes, wire polylines for affected `edge_id`s.
- [x] **Step 3:** `get_diagram_scene_patch` command; frontend merge without replacing unrelated primitives.
- [x] **Step 4:** Keep dashed drag outline + `dragVisualDelta` for sub-60ms pointer tracking between patches.
- [x] **Step 5:** Verify no full-scene IPC on every mousemove (only throttled patch or commit).

---

## Plan self-review

### Spec coverage

| Spec section | Task(s) |
|--------------|---------|
| Rust crate layout | Tasks 0–2, 4–16 |
| Scene model + strict mirror | Tasks 9–11, 13, 17 |
| Coordinate systems / scene_to_cad | Tasks 4, 9 |
| Konva canvas | Tasks 17–19 (17–18 done; 19 partial) |
| Save compatibility | Tasks 2–3, 23 |
| IPC contract | Tasks 3, 14, 16, 20 |
| Reports in Rust | Tasks 15–16, 22 |
| DXF Revit-safe | Tasks 12–14, 23 |
| Definition of done | Tasks 23–25 |

### Known gaps (track during execution)

- **ScenePatch IPC:** Task 19b — interim full-scene preview during drag is implemented; patch is next perf milestone.
- **Konva text:** Cap-height → em calibration in `sceneRenderUtils.ts` (`ARIAL_NARROW_CAP_TO_EM = 0.75`); not an export scale. Bundle Arial Narrow TTF if system font insufficient.
- **wireSplit node:** Not in `build_scene` (Task 10f gap).
- **Full v6 command parity:** Task 3 lists core commands; diff `commands.rs` line-by-line before UI phase complete.
- **Drag anti-patterns resolved:** No geometry-clone overlay; no duplicate wire React keys; Konva cleanup via proper `primitiveKey` disambiguation.

### Next recommended work (drawing)

1. **Task 19b** — `ScenePatch` to stop full Comp Gym rebuilds during drag.
2. **Task 19c** — connect ports (highest UX gap after drag).
3. **Task 10f** — `wireSplit` scene dispatch (if fixtures need it).

### Placeholder scan

No TBD/TODO steps in task definitions above.

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-05-29-diagramme-v2.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks, fast iteration  
2. **Inline Execution** — execute tasks in this session with checkpoints

**Which approach?**

Recommended start regardless: **Task 0 + Task 1 + Task 2** (scaffold, fixtures, schema) — everything else depends on them.
