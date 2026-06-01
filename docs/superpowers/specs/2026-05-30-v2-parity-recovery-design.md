# Diagramme v2 — Parity Recovery Design

**Date:** 2026-05-30  
**Status:** Draft for review  
**Supersedes (for execution order):** interaction portions of `2026-05-29-diagramme-v2-design.md` / plan Tasks 17–19c  
**Does not supersede:** Rust scene/DXF strict-mirror architecture (still correct)

---

## Problem statement

v2 has strong **render/export plumbing** (Rust scene, Comp Gym golden scene, DXF emit) but **weak interaction parity**. Recent work (ScenePatch, port connect) was added as isolated features on a dev shell without v6’s interaction model, handle affordances, or acceptance tests. Users loading Comp Gym cannot wire the way they do in v6 — and the implementation plan incorrectly marked canvas/interaction items as done or “partial” while skipping the v6 shell entirely.

**Launch requirement (unchanged):** full 1:1 user-facing parity with v6 at launch (Option C in original spec).

---

## What went wrong (root causes)

### 1. Plan vs reality drift

| Area | Plan said | Reality |
|------|-----------|---------|
| Konva canvas | Task 17–18 done | **Render-only** — no v6 menus, palette, properties, wiring toggle |
| Interaction | Task 19 partial | **Ad-hoc gestures** — node drag + WIP port connect; no v6 handle contract |
| Port connect | Task 19c “done” | **Uncommitted WIP**, incomplete handle coverage, no visible handles |
| Source of truth | Rust authoritative | **Correct for save/export**, but interaction UX never ported from v6 |

The original plan ordered **core → scene → Konva renderer → interaction → shell**. Execution jumped to renderer + drag + perf (ScenePatch) while **skipping the v6 interaction map** (`handleContract`, wiring mode, palette, properties, edge editing).

### 2. Port connect case study (why “it doesn’t work”)

Observed failure modes on Comp Gym:

1. **No visible port handles** — v6 shows handles in wiring mode; v2 hit targets are invisible. Users click the device body → **node drag**, not wire start.
2. **Incomplete handle inventory** — Comp Gym golden scene has ~110 `handle_id` hits vs ~175 node-associated hits. **Speaker blocks, mic blocks, wiretags, volume controls, antennas, bundle handles** lack port hits (most Comp Gym wires attach there).
3. **No wiring mode** — v6 gates `onConnect` on `wiringMode` and disables handle pointer-events when off. v2 has no toggle and no affordance.
4. **No handle role / compatibility rules** — v6 `handleContract.ts` + Rust `rules.rs` enforce source/target pairing. v2 `add_edge` accepts any two handles with geometry; silent failures or wrong topology.
5. **No error surfacing** — failed `add_edge` IPC is not shown in UI.
6. **Work not shipped** — port connect changes were **never committed** after ScenePatch commit; running an older build shows no connect at all.

### 3. Architectural mistake (process, not stack)

The stack (Rust scene + Konva dumb renderer) is still right. The mistake was treating **interaction as small hooks** (`useDiagramInteraction` patches) instead of porting v6’s **Interaction Parity Layer** as a first-class subsystem with a gesture matrix and fixture acceptance tests.

---

## Parity truth table (v6 vs v2 today)

| Capability | v6 | v2 today | Gate |
|------------|----|-----------|----|
| Open/save `.diagramme` | ✓ | IPC exists; **no file UI** | P0 shell |
| Wiring mode toggle | ✓ | ✗ | P0 interaction |
| Visible port/bundle handles | ✓ | ✗ | P0 interaction |
| Connect ports (all node types) | ✓ | **Partial / broken** | P0 interaction |
| Handle role compatibility | ✓ | ✗ | P0 interaction |
| Drag node body | ✓ | ✓ (ScenePatch) | ✓ |
| Move wires with node (inner corners) | ✓ | ✓ Rust; preview OK | ✓ |
| Resize node (`update_dims`) | ✓ | IPC only | P1 interaction |
| Wire inner-corner drag | ✓ | ✗ | P1 interaction |
| Multi-select drag | ✓ | IPC only | P1 interaction |
| Wire split / wiretag pair UX | ✓ | Partial scene | P1 interaction |
| Properties panel | ✓ | ✗ | P1 shell |
| Palette / insert nodes | ✓ | ✗ | P1 shell |
| Undo/redo UI | ✓ | IPC only | P1 shell |
| Reports + XLSX | ✓ | Rust stubs | P2 |
| CI golden gates (scene+DXF+reports) | v6 Vitest | Partial Rust tests | P2 |
| DXF strict mirror | ✓ | ✓ Comp Gym baseline | ✓ |
| Cafeteria-scale drag perf | ✓ | ScenePatch; unprofiled | P2 perf |

---

## Strategic approaches

### A — “Shell first” (port v6 App.tsx early)

Port menus, file ops, wiring toggle, properties, palette onto Konva canvas quickly; wire existing IPC.

- **Pros:** Feels like Diagramme immediately; uses v6 UX patterns users know.
- **Cons:** Without handle overlay + contract, wiring still broken; large App.tsx port is risky in one pass.

### B — “Interaction parity layer first” (recommended)

Build **`InteractionParity`** subsystem before more shell: handle catalog from Rust scene, visible affordances, wiring mode, connect rules, gesture tests on `dxf-export-test` + Comp Gym workflows.

- **Pros:** Fixes the actual user pain; each gesture has acceptance criteria; Konva stays dumb renderer.
- **Cons:** Dev shell stays minimal a few more weeks.

### C — “Temporary React Flow bridge”

Keep Konva for view, use RF handles for wiring only.

- **Pros:** Fastest wire connect.
- **Cons:** **Violates v2 architecture** — dual geometry/hit truth; rejected.

**Recommendation: B**, with **thin shell slice from A** in parallel (file open/save + wiring toggle only).

---

## Target architecture (interaction)

```
DiagramState (Rust, authoritative)
  → build_scene → Scene { primitives, hits with handle_id, handle_role?, kind? }
  → get_diagram_scene / patch

Konva SceneRenderer (unchanged — ink only)

InteractionParity (NEW — TypeScript, v6-parity)
  ├── HandleCatalog      — from Scene.hits + node metadata; merges bundle handles
  ├── HandleOverlay      — visible ports when wiringMode; hitGraphEnabled or SVG overlay
  ├── WiringController   — connect, validate via handleContract port of rules.rs
  ├── GestureController  — dispatches drag | connect | pan | resize | corner-edit
  └── diagramStateApply  — port from v6; merge Rust IPC results into local UI state

App shell (ported slice)
  ├── useDiagramFileOps
  ├── useCanvasPreferences (wiringMode)
  └── status/error bar for IPC failures
```

### Scene hit extensions (Rust)

Extend `HitTarget` beyond current fields:

| Field | Purpose |
|-------|---------|
| `handle_id` | RF-compatible handle string (already started) |
| `handle_role` | `source` \| `target` \| `bidirectional` (mirror `rules.rs`) |
| `kind` | `body` \| `port` \| `bundle` — disambiguate pick priority without size heuristics |

Emit hits for **every v6 Handle id** including bundle handles, mic/speaker/vc/antenna/wiretag/wireSplit.

### Connect flow (correct)

1. User enables **wiring mode** (default on, like v6).
2. **Handle overlay** shows connectable ports.
3. Pointer down on port hit → rubber band (local only).
4. Pointer up on compatible port → `add_edge` IPC → **scene refresh** (full or wire patch).
5. On IPC error → status bar message; no silent failure.

Validation lives in Rust (`rules.rs` port) **and** TS mirror for instant feedback.

---

## Parity gates (definition of “done” per gesture)

No gesture is “done” without:

1. **v6 reference** — file + function in `Diagramme_v6` cited in test comment.
2. **Fixture test** — automated or scripted checklist on `dxf-export-test.diagramme` minimum.
3. **Comp Gym smoke** — manual 5-minute script in plan appendix.
4. **IPC error path** — failure visible in UI.

---

## Phasing (high level)

| Phase | Name | Outcome |
|-------|------|---------|
| **P0** | Interaction foundation | Wiring mode, handle overlay, full handle catalog, connect works on palette + Comp Gym speaker/device paths |
| **P1** | Remaining gestures + thin shell | Resize, inner-corner drag, multi-drag; file open/save; properties debounced `update_node` |
| **P2** | Full shell + reports | Menus, presets, sheets, report dialogs, XLSX |
| **P3** | Integration gates + perf | CI script, DXF/report goldens, Cafeteria profiling, CAD signoff |

**Explicitly deferred:** new perf ideas, optional render snapshot tests, wireSplit scene unless fixture gate requires it.

---

## Risks

| Risk | Mitigation |
|------|------------|
| Re-porting v6 piecemeal repeats failures | Gesture matrix + gates; no checkbox without test |
| Handle/id drift vs geometry | Single generator: scene emit reads same layout as `get_analytical_port_xy` |
| Large App.tsx port | Slice: file ops + wiring toggle first; properties second |
| Plan drift again | This doc owns execution order; old plan marked archival for Tasks 17–19 |

---

## Decisions

| Decision | Choice |
|----------|--------|
| Recover parity how | Interaction parity layer first (B) |
| Port connect fix | Redo as P0.1 — not patch on broken hits |
| Old Task 19c checkboxes | Void — replaced by P0 matrix |
| v6 reference | Mandatory for every gesture |
| Commits | Each P0 sub-task is shippable + testable |

---

## Next step

Review this spec → approve → execute **`2026-05-30-v2-parity-recovery.md`** implementation plan (task-level checkboxes, file paths, tests).
