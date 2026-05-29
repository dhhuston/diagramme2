# Diagramme v2 — Design Spec

**Date:** 2026-05-29  
**Status:** Approved  
**Reference codebase:** Diagramme_v6 (`/Users/davinhuston/dev/Diagramme_v6`)

---

## Summary

Diagramme v2 is a **1:1 user-facing rebuild** of Diagramme CAD with a new internal architecture optimized for **exportable, Revit-safe DXF** of AV functional schematics. Users keep the same workflows, menus, file formats, and on-screen fidelity. Implementation may change completely between UI and export—as long as **what they see on canvas is what DXF contains**.

**Non-negotiables:**

- Full parity at launch (no phased “save-only” or “export-only” beta)
- `.diagramme` save/load compatibility with v6
- `.avdevice` / `.plate` preset compatibility
- Konva canvas renders **only** from a Rust-built scene graph
- DXF and all reports generated **only** from Rust (same persisted state + scene pipeline)
- **Strict mirror:** Konva and DXF consume the same scene primitives with **one** linear scale and **one** Y-flip at CAD emit—no renderer-specific corrections (including text)

---

## Problem

Diagramme v6 couples a React Flow DOM canvas with TypeScript export/report pipelines. Wire routing, node layout, and DXF emission can diverge from on-screen geometry—the wire geometry overlay exists to surface that drift. Reports and DXF read from mixed sources (live React nodes vs flushed Rust state). v2 eliminates dual truth.

---

## Goals

| Goal | Success criterion |
|------|-------------------|
| WYSIWYG export | Every scene primitive (points, bounds, text height, stroke width, alignment) maps to DXF via `scene_to_cad()` only—no export-only adjustments |
| Save compatibility | Fixture corpus: open v6 `.diagramme` → edit → save → reopen with semantic equality |
| UX parity | File ops, sheets, presets, properties, reports menus behave like v6 |
| Performance | Konva + incremental scene updates; no per-node DOM at scale |
| Revit-safe DXF | Pass v6 golden tests + `cad-export-1to1-validation.md` manual signoff |

---

## Non-goals (v2 launch)

- New diagram features not in v6
- Web-only deployment (desktop Tauri remains)
- DWG export (DXF only unless trivially added later)
- Rewriting user guide copy unless behavior changes

**Intentional v6 behavior change:** v2 drops `EXPORT_TEXT_VISUAL_SCALE` (0.75). Text cap height in DXF equals on-screen cap height × `1/72`. v6 golden DXF files are re-baselined for text height only.

---

## Architecture

### Layer diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  React shell (menus, dialogs, properties panel, file ops)       │
│  Konva Stage — renders Scene only; handles hit targets / drag   │
└────────────────────────────┬────────────────────────────────────┘
                             │ Tauri IPC
┌────────────────────────────▼────────────────────────────────────┐
│  diagramme-app (Tauri commands)                                 │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│  diagramme-core (Rust workspace)                                │
│                                                                 │
│  schema ──► geometry ──► wires ──► scene ──┬──► dxf            │
│                                            └──► reports         │
│                                                                 │
│  ProjectState (v6-compatible serde) = persistence authority     │
└─────────────────────────────────────────────────────────────────┘
```

### Hard rules

1. **Rust `ProjectState` is authoritative** for save, undo, DXF, reports, and scene build.
2. **Rust `Scene` is authoritative** for canvas drawing and DXF geometry (two renderers, one scene).
3. **Konva never computes** schematic wire polylines, node CAD layout, crossing gaps, or bundle fillets.
4. **TypeScript never exports** DXF, XLSX, or report data from local canvas state.
5. **Flush before persist/export:** UI runs one `flushCanvasToRust` path before save, dirty checks, reports, and DXF—identical to v6 semantics.

---

## Rust crate layout

| Crate | Responsibility | Ports from v6 |
|-------|----------------|---------------|
| `diagramme-schema` | `ProjectState`, `DiagramState`, nodes, edges, sheets, presets | `src-tauri/src/model.rs` |
| `diagramme-geometry` | Port positions, node bounds, layout constants (inches + diagram px) | `portGeometry.ts`, `dxfSchematicLayout.ts`, node layout modules |
| `diagramme-wires` | Sharp polylines, obstacles, crossings, bundle fillets | `wireGeometry/*`, `dxfWirePostprocess.ts`, schematic edge path logic |
| `diagramme-scene` | `build_scene(sheet) → Scene` — unified drawable + exportable geometry | New; consolidates layout + wires + node dispatch |
| `diagramme-dxf` | Revit-safe DXF from `Scene` | `revitDxf/*` (native Rust writer, same entity rules as `acad-ts` path) |
| `diagramme-export-model` | Entity enumeration for equipment list / Revit-prep | `diagramExportModel.ts` |
| `diagramme-reports` | Report structs, XLSX (`rust_xlsxwriter`), Revit-prep JSON | `deviceTagsReport.ts`, `plateConnectionsReport.ts`, `groupedInventoryReport.ts`, `equipmentListXlsx.ts`, `revitPrepExport.ts`, `equipmentReportCore.ts` |
| `diagramme-app` | Tauri command wiring, app mutex, file/recovery hooks | `commands.rs` |

---

## Scene model

### Purpose

`Scene` is the **single geometric truth** for the active sheet. Konva draws it; DXF transforms it.

### Contents (diagram-space pixels unless noted)

- **Nodes:** frames, title bands, port rows, symbols as rects, polylines, solids with explicit `stroke_px`
- **Text:** `SceneText { position, content, height_px, halign, valign, font: "Arial Narrow" }` — cap height is final; no role indirection at render time
- **Wires:** polylines with crossing gaps, bundle arcs, `layer`/`color` from category (same in Konva and DXF)
- **Metadata per primitive:** node id, edge id, hit target ids for interaction
- **Extents:** bounding box for zoom-fit and DXF header

### Build pipeline

```
ProjectState.active_sheet
  → diagramme-geometry (node bounds, port XY)
  → diagramme-wires (edge polylines + postprocess)
  → diagramme-scene (assemble Scene)
```

DXF path:

```
Scene (diagram px, Y-down)
  → scene_to_cad() — px × (1/72), Y mirror
  → diagramme-dxf emit → sanitize → String
```

See [Coordinate systems and scaling](#coordinate-systems-and-scaling).

### Incremental updates (performance)

Full scene rebuild on every mousemove is forbidden.

| Phase | Behavior |
|-------|----------|
| Drag preview | Konva moves node group locally OR Rust returns lightweight `ScenePatch` (moved node ids + affected wire ids) throttled ~60ms |
| Drag commit | `move_node` → full scene for active sheet |
| Wire edit | `update_edge` / inner-corner commands → patch wires touching that edge |
| Sheet switch | Full scene replace |

### Visual parity contract (strict mirror)

**Rule:** If it appears in the Konva scene, it appears in DXF at the corresponding CAD coordinate. No export-only geometry, scale factors, or text corrections.

| Primitive | Scene field(s) | Konva | DXF |
|-----------|----------------|-------|-----|
| Points / polylines | `diagram px`, Y-down | Draw 1:1 | `× (1/72)`, then Y mirror |
| Rects / solids | bounds in diagram px | Fill/stroke 1:1 | Same bounds in inches |
| Stroke width | `stroke_px` (e.g. 1 px hairline) | `strokeWidth = stroke_px` | Hairline / 0 width per Revit policy at `stroke_px / 72"` |
| Text insertion | `(x, y)` diagram px | Same point | `(x/72, y_cad)` |
| Text cap height | `height_px` | `fontSize = height_px` | `height_in = height_px / 72` |
| Text font | `"Arial Narrow"` | Load same family in app | STYLE table entry (required install) |
| Text alignment | `halign`, `valign` | Konva `align` / offset | DXF TEXT alignment flags (same semantics) |
| Text content | sanitized string | Same string | Same string (`dxfSanitizeText` rules in Rust) |
| Wire color | `layer` / `color` in scene | Konva stroke from scene | DXF layer/color from scene |
| Fills | polygon vertices | Same vertices | SOLID / equivalent |

**Forbidden:** Any multiplier (e.g. v6 `EXPORT_TEXT_VISUAL_SCALE`), export-only font sizes, DXF-only inset tweaks, or Konva-only layout nudges not stored in the scene.

**Residual risk (minimize, test):** Arial Narrow glyph advance may differ slightly between Konva/canvas metrics and Revit’s TrueType renderer. Cap height and insertion point must still match; string width is computed once in Rust for autosizing (wiretags) and stored in scene bounds both renderers use.

---

## Coordinate systems and scaling

Diagramme uses **three distinct spaces**. Mixing them is the most common cause of “export surprise.” v2 keeps them explicit and tested.

### 1. Diagram space (authoritative)

**All persisted geometry and the Rust `Scene` live here.**

| Property | Value | Source (v6) |
|----------|-------|-------------|
| Unit | **Diagram pixel** (1 px = 1 PDF point) | `PX_PER_INCH = 72` in `paperScale.ts` |
| Scale | **72 diagram px = 1 inch** | `DIAGRAM_PX_TO_INCH = 1/72` in `cadUnits.ts` |
| Origin | Top-left of the sheet canvas | Same as React Flow / node `position` |
| Y axis | **Down** (+Y = down) | Canvas convention |
| Node positions | `position.x`, `position.y` in diagram px | `ProjectState` |
| Layout constants | Defined in inches × 72, e.g. connector pitch 9 px = 1/8" | `CONNECTOR_LINE_PITCH_PX`, device widths, etc. |

**Calibration anchor (must not change without a migration):**

```
CONNECTOR_LINE_PITCH_PX = 9
DIAGRAM_PX_TO_INCH      = 1/72
→ connector row pitch   = 0.125" in DXF
```

Rust `diagramme-geometry` owns `PX_PER_INCH`, all inch-derived layout constants, and `px_to_in()` / `in_to_px()`. Konva and DXF both consume **diagram px from the scene**—neither re-derives scale from CSS or viewport zoom.

### 2. Konva viewport space (display only)

Pan and zoom are **purely cosmetic**. They must never mutate scene coordinates or export scale.

```
screenX = stage.x + diagramX × stage.scaleX
screenY = stage.y + diagramY × stage.scaleY
```

| Rule | Detail |
|------|--------|
| Scene → Stage | 1 diagram px = 1 Konva unit at `scale = 1` |
| Zoom | `stage.scale({ x: zoom, y: zoom })` — uniform only (no anisotropic stretch) |
| Pan | `stage.position({ x, y })` — translation only |
| Hit testing | Inverse viewport transform → diagram px → scene hit targets |
| Export | **Ignored entirely** — DXF never reads stage scale/position |

During drag preview, Konva may translate a node **group** in diagram space (or apply a temporary offset in diagram px). On commit, Rust returns authoritative positions; preview offsets are discarded.

### 3. CAD / DXF space (export)

DXF is emitted in **inches**, Y **up** (standard CAD), from the same scene points:

```rust
// Linear scale (all X and Y coordinates)
x_in = x_diagram_px * DIAGRAM_PX_TO_INCH
y_in = y_diagram_px * DIAGRAM_PX_TO_INCH

// Y mirror (diagram Y-down → CAD Y-up), after computing content extent in inches
y_cad = ext.min_y_in + ext.max_y_in - y_in
```

Port from v6: `dxfYMirrorInches(ext)` in `dxfCadCoords.ts`. Extent (`minX`, `maxX`, `minY`, `maxY`) is accumulated from scene bounds in inches before mirror is applied.

| DXF header | Value |
|------------|-------|
| `INSUNITS` | 1 (inches) |
| `$MEASUREMENT` | 1 (metric off) |
| Extents | Scene bounds + padding (v6 uses ~1.15" padding) |

### Transform pipeline (single path)

```
ProjectState
    → build_scene()          [diagram px, Y-down]
    → Konva Layer            [diagram px, Y-down, 1:1; viewport zoom separate]
    → scene_to_cad()         [inches, Y-up mirror]
    → diagramme-dxf emit
```

There is **no** second scale factor in Konva and **no** “fit to page” resize at export. Sheet size (e.g. E1 30"×42" = 2160×3024 diagram px) is a reference frame, not a transform applied at export.

### Strict mirror transform (`scene_to_cad`)

Rust exposes **one** conversion used by DXF emit and by parity tests. Konva reads diagram px directly (identity in scene space).

```rust
/// Linear scale — applies to ALL coordinates and sizes (including text height, stroke width).
fn px_to_in(v: f64) -> f64 {
    v * DIAGRAM_PX_TO_INCH  // 1/72
}

/// Point: diagram px (Y-down) → CAD inches (Y-up).
fn scene_point_to_cad(p: PointPx, ext: ExtentIn) -> PointIn {
    PointIn {
        x: px_to_in(p.x),
        y: ext.min_y + ext.max_y - px_to_in(p.y),
    }
}
```

| Quantity | Scene (Konva) | CAD (DXF) |
|----------|---------------|-----------|
| Position | `(x_px, y_px)` | `scene_point_to_cad` |
| Width / height | `w_px`, `h_px` | `px_to_in(w_px)`, `px_to_in(h_px)` |
| Text cap height | `height_px` | `px_to_in(height_px)` — **no extra factor** |
| Polyline vertex | `(x_px, y_px)` | each vertex via `scene_point_to_cad` |

Scene builder resolves layout roles (`tag`, `cell`, `wiretag`, etc.) to concrete `height_px` **once**. Neither Konva nor DXF re-interpret roles or apply optical corrections.

**Font:** Scene text uses **Arial Narrow** everywhere. Konva loads the same face (bundled or system); DXF references the TrueType in the STYLE table. Text measurement for autosize (wiretags, text blocks) runs in Rust (`diagramme-geometry`) and writes resulting bounds into the scene so both renderers share the same box.

### Snap grid (diagram space)

| Constant | Diagram px | Inches |
|----------|------------|--------|
| `SNAP_GRID_PX` | 3 | 1/24" |
| `CONNECTOR_LINE_PITCH_PX` | 9 | 1/8" |

Placement, port centers, and wire routing snap in **diagram px** in Rust. Konva does not apply its own snap scale.

### Scaling tests (CI)

| Test | Assert |
|------|--------|
| Constant calibration | `CONNECTOR_LINE_PITCH_PX * DIAGRAM_PX_TO_INCH == 0.125` |
| Scene ↔ DXF wires | Sample polyline: each `(x_px, y_px)` → `(x_in, y_cad)` within ε |
| Viewport isolation | Export same before/after arbitrary Konva zoom/pan |
| Known segment | Horizontal wire of N diagram px → `N/72` inches in DXF |
| Device width | `DEVICE_V2_WIDTH_PX * DIAGRAM_PX_TO_INCH == 1.75"` |
| Text cap height | Scene `height_px == 9` → DXF TEXT height `0.125"` exactly |
| Text parity sweep | Every `SceneText` in fixture → DXF entity height equals `px_to_in(height_px)` |
| No visual scale | Grep / lint: `EXPORT_TEXT_VISUAL_SCALE` and export-only multipliers absent from codebase |

Manual signoff: v6 `docs/cad-export-1to1-validation.md` checklist on ≥2 real projects.

### Anti-patterns (explicitly forbidden)

- Applying `DIAGRAM_PX_TO_INCH` in the Konva renderer (scene is already diagram px)
- Export-only scale factors (`EXPORT_TEXT_VISUAL_SCALE`, DXF-only text height, export-only insets)
- Exporting from screen/pixel coordinates after viewport transform
- Different layout constants in TS vs Rust
- Anisotropic stage scale (different X/Y zoom)
- Recomputing node bounds in DXF that are not in the scene
- Konva choosing a different font or fontSize than `SceneText.height_px`

---

## Canvas: Konva

### Why Konva (not React Flow)

- Batched canvas drawing scales better for large AV schematics
- “Dumb renderer” fits scene-driven WYSIWYG
- Avoids per-device DOM + independent SVG edge routing

### Konva responsibilities

- Render `Scene` layers (nodes, wires, selection chrome) — **all** stroke, fill, text, and layer from scene primitives
- Hit testing → map to node id / port id / wire segment index
- Forward gestures to IPC (drag, connect, resize, polyline edit)
- Load **Arial Narrow** for canvas text (same face as DXF STYLE table)

### Konva must NOT

- Run `buildInnerChainPoints`, obstacle avoidance, or DXF postprocess
- Cache authoritative geometry that differs from Rust scene
- Port `SchematicEdge.tsx` routing logic

### Interaction flow

```
User gesture → intent IPC command → Rust mutates ProjectState
  → returns { project, scene | scene_patch }
  → Konva redraw
```

---

## Persistence and compatibility

### File formats (unchanged)

| Format | Contract |
|--------|----------|
| `.diagramme` | v6 `ProjectState` JSON; serde field names and defaults preserved |
| `.avdevice` / `.plate` | `schemaVersion: 1` preset envelope |
| Recovery snapshot | Same behavior as v6 |

### Compatibility gates (CI)

1. **Open gate:** Load N fixture `.diagramme` files (v6 test diagram, `Cafeteria D104A`, edge-case sheets)
2. **Round-trip gate:** Mutate → save → reload → semantic JSON equality on nodes/edges/presets/sheets
3. **Scene gate:** `build_scene` golden JSON per fixture
4. **DXF gate:** Export vs v6 golden DXF (normalize line endings; document allowed deltas)
5. **Report gate:** Report row snapshots vs v6 Vitest fixtures
6. **Scene↔DXF gate:** Scene wire segments in inches match DXF wire entities within tolerance

---

## IPC command contract

### State mutations (mirror v6)

| Command | Notes |
|---------|-------|
| `get_project` | Full project |
| `set_state` / `sync_state` | Bulk replace; sync without undo for flush |
| `open_diagram` / `save_diagram` / `save_diagram_compact` | File round-trip |
| `move_node` / `move_nodes` | Optional `EdgeHandleAttachmentUpdate[]`; `no_history` for preview |
| `update_dims` / `update_node` | Properties + resize |
| Node/edge/sheet/preset commands | Parity with v6 `commands.rs` |
| `undo` / `redo` | Per active sheet |

### Scene and geometry

| Command | Returns |
|---------|---------|
| `get_diagram_scene` | Full `Scene` for active sheet |
| `get_diagram_scene_patch` | Optional partial update after drag preview |
| `get_wire_geometry_model` | Debug overlay (optional; should match scene wires) |

### Export and reports (all Rust)

| Command | Returns |
|---------|---------|
| `export_revit_dxf` | DXF string or writes via path |
| `build_device_tags_report` | JSON for dialog |
| `export_device_tags_xlsx` | `Vec<u8>` |
| `build_plate_connections_report` | JSON |
| `export_plate_connections_xlsx` | `Vec<u8>` |
| `build_grouped_inventory_report` | JSON |
| `export_grouped_inventory_xlsx` | `Vec<u8>` |
| `export_equipment_list_xlsx` | `Vec<u8>` |
| `build_revit_prep_payload` | JSON string |

All export/report commands require caller to flush canvas to Rust first (same helper as save).

---

## Frontend (TypeScript) responsibilities

| Keep | Remove / replace |
|------|------------------|
| Menu structure, shortcuts, dialogs | `src/export/*` builders |
| Properties panel → `update_node` IPC | `wireGeometry` as geometry authority |
| `useDiagramFileOps`, dirty baseline | `SchematicEdge` routing |
| Report dialog components (data from IPC) | `buildRevitDxf(nodes, edges)` |
| Konva renderer + interaction controller | React Flow canvas (except optional dev tooling) |
| Wire category **display** colors (if not in scene metadata) | `diagramExportModel` in TS |

### Flush pipeline (single entry)

```typescript
async function flushCanvasToRustBeforePersist(): Promise<void> {
  if (pendingDragMove) return
  await flushPendingNodeDataToRust()
  await rustSyncState({ nodes: authoritativeKonvaSnapshot, edges: ... })
}
```

Note: In v2, “nodes/edges” for flush may be slim interaction records derived from Konva state, or scene hit-map updates—design detail for implementation plan. Persisted shape remains v6 `ProjectState`.

### Ephemeral UI state (local only)

- Selection, hover, marquee
- In-progress text edit buffer (commit → `update_node`)
- Dialog open state
- Viewport pan/zoom (not persisted)

---

## DXF implementation notes

- Port v6 Revit path entity rules: SOLID for fills, no `3DFACE`, SOLID-first ordering, unique handles, Arial Narrow STYLE table, hairline policy, `sanitizeDxfString` equivalents
- Source: `Scene` → `scene_to_cad()` → entities. **No** export-only node detail path.
- Text height: `px_to_in(scene_text.height_px)` only—do **not** port v6 `EXPORT_TEXT_VISUAL_SCALE`
- Node-type matrix from v6 `dxf-export-audit.md` must be complete at launch
- v6 DXF golden files: re-baseline expected TEXT heights; wire/frame geometry should match

---

## Reports implementation notes

- All builders read `ProjectState` from mutex after flush
- XLSX via `rust_xlsxwriter`; port column layouts and aggregation from v6 report modules
- TypeScript dialog types mirror Rust `serde` JSON shapes
- Equipment list and Revit-prep share `diagramme-export-model`

---

## Repository layout (`diagramme2/`)

```
diagramme2/
  docs/
    superpowers/specs/2026-05-29-diagramme-v2-design.md
  fixtures/                    # copied from v6 + golden outputs
  src/                         # React + Konva UI
  src-tauri/
    src/commands.rs
    diagramme-core/            # Cargo workspace
      schema/
      geometry/
      wires/
      scene/
      dxf/
      export-model/
      reports/
  package.json
  Cargo.toml                   # workspace root
```

**Bootstrap strategy:** Copy v6 Tauri/Vite shell; replace React Flow canvas with Konva; wire IPC to new Rust workspace; port tests with fixtures first.

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Rust port of wire routing is large | Port incrementally with v6 Vitest → Rust test translation; scene↔DXF parity tests |
| Rust DXF writer (no acad-ts) | Golden DXF tests from day one; port sanitize rules exactly |
| Konva text editing UX | Match v6 properties panel for heavy edits; inline label edit where v6 had it |
| IPC scene payload size | Scene patches + binary encoding if JSON too heavy |
| Layout drift during port | One `diagramme-scene` dispatch table; DXF reads scene, not separate node detail |
| Full parity (C) extends timeline | Strict fixture gates; no feature additions until gates green |

---

## Definition of done (v2 launch)

- [ ] All v6 file menu actions work (new, open, save, save as, exports, reports, presets)
- [ ] Fixture corpus passes save round-trip
- [ ] Fixture corpus passes scene golden + DXF golden + report golden
- [ ] Manual CAD validation checklist on ≥2 real project files
- [ ] No TypeScript geometry authority for wires or export layout
- [ ] Konva canvas matches scene; DXF matches scene via `scene_to_cad()` (automated primitive parity tests)
- [ ] Text cap heights in DXF equal scene `height_px / 72` for all fixture labels
- [ ] Performance: smooth pan/zoom and drag on Cafeteria-scale fixture (subjective + no full-scene IPC per frame)

---

## Decisions log

| Decision | Choice |
|----------|--------|
| Launch scope | C — full 1:1 at launch |
| Stack | Tauri 2 + React shell; Rust owns geometry, scene, DXF, reports |
| Canvas | Konva, scene-driven (A) |
| Export surprise | Eliminated via single Rust `Scene` |
| Konva ↔ DXF | Strict mirror: one scale (`1/72`), one Y-flip at CAD emit; no renderer-specific tweaks |
| Text height | Same as on screen (`height_px / 72` in DXF); v6 `EXPORT_TEXT_VISUAL_SCALE` removed |
| Reporting | All in Rust including XLSX |
| Save format | v6 `.diagramme` compatible |

---

## Next step

After spec approval: invoke **writing-plans** skill to produce implementation plan with phased **internal** work order (schema → geometry → wires → scene → dxf/reports → Konva UI → integration gates).
