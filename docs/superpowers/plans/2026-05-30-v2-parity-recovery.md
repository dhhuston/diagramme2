2# Diagramme v2 — Parity Recovery Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development — **one P0 task per session**, verify gate before next task.  
> **Spec:** [`2026-05-30-v2-parity-recovery-design.md`](../specs/2026-05-30-v2-parity-recovery-design.md)  
> **Replaces for execution:** Tasks 17–19c and “next recommended work” in `2026-05-29-diagramme-v2.md`  
> **Keeps:** Phases 0–6 Rust work (scene, DXF, schema) — do not rewrite

**Goal:** Restore credible path to v6 **launch parity** from current state (working render + drag + DXF; broken/invisible wiring; no shell).

**Reference repo:** `../Diagramme_v6` — every task cites v6 files.

---

## Current baseline (honest)

**Works**
- Rust scene build + Comp Gym golden scene + DXF export
- Konva renderer (primitives, text calibration, wire keys)
- Node body drag + ScenePatch preview + move_node geometry
- Core IPC (open/save/move/update_dims/undo exist)

**Broken / missing**
- Port connect (WIP uncommitted; invisible hits; ~60% of Comp Gym ports lack hits)
- Wiring mode, handle overlay, handle contract
- File menu UI (only “Load Comp Gym” fetch button)
- Properties, palette, edge edit, reports, CI gates

**Immediate action:** Revert or discard uncommitted port-connect WIP **OR** land P0.1 properly — do not leave hybrid state.

---

## P0 — Interaction foundation (wire parity)

> **Exit criterion:** On `dxf-export-test.diagramme`, user can connect device↔avPlate and speaker↔speaker in wiring mode with visible handles, same as v6.

### P0.0 — Freeze and audit (1 session)

- [ ] **0.1** Archive misleading checkboxes in `2026-05-29-diagramme-v2.md` — add banner pointing to this plan.
- [ ] **0.2** Inventory uncommitted work; either commit with “WIP broken” label or `git stash` before P0.1.
- [ ] **0.3** Generate **handle inventory** script/test: for each fixture, list all v6 edge `sourceHandle`/`targetHandle` values → assert each resolves via `get_analytical_port_xy` + will have scene hit after P0.2.

**Files:** `diagramme-scene/tests/handle_inventory_tests.rs` (new)

---

### P0.1 — Scene hit catalog (Rust)

Complete v6-equivalent hits — not just deviceV2/avPlate/patch panel rows.

- [ ] **1.1** Add `HitKind` to `HitTarget`: `body | port | bundle` (serde camelCase `kind`).
- [ ] **1.2** Add `handle_role` on port/bundle hits (port `rules.rs` logic).
- [ ] **1.3** Emit port hits:
  - `deviceV2` / `device` — port rows **and** bundle handles (`L-{g}-bundle-{b}`)
  - `avPlate` — `T-` / `S-` per row
  - patch panels — `L-{rowId}` / `R-{rowId}`
  - `speakerBlock` — `T-spk`, `S-spk-passthru` (when enabled)
  - `micBlock` — `S-mic`
  - `volumeControl` — `T-vc`, `S-vc`
  - `wiretag` — `conn-src`, `conn-tgt`
  - `antennaTransmitterSymbol` / `antennaReceiverSymbol` — `ant-tx`, `ant-rx`
  - `flyoffNote` — flyoff port handle
  - `junction` — junction handles (if v6 has them)
- [ ] **1.4** Pick priority: `bundle/port` before `body` (explicit `kind` ordering, not z-index hacks).
- [ ] **1.5** Test: `handle_inventory_tests` — every handle in `dxf-export-test` + sample Comp Gym edges has exactly one scene hit with matching `handle_id`.
- [ ] **1.6** Regenerate `fixtures/golden/scene/comp-gym-f102a.json`.

**Files:** `scene/src/scene.rs`, `scene/src/nodes/*.rs`, `schema` or `geometry` `rules.rs` (port from v6)

**v6 refs:** `DeviceNodeV2.tsx`, `AvPlateNode.tsx`, `*PatchPanel*.tsx`, `handleContract.ts`, `src-tauri/src/rules.rs`

---

### P0.2 — Handle overlay + wiring mode (Konva)

Users must **see** what to click.

- [ ] **2.1** Port `useCanvasPreferences` — `wiringMode` (default **true**), persist localStorage.
- [ ] **2.2** `HandleOverlay.tsx` — render port/bundle affordances from `Scene.hits` where `kind != body`:
  - Match v6 handle colors from wire category (read from diagram state or scene metadata)
  - Hidden when `!wiringMode`
  - `listening={false}` on overlay if hits stay authoritative in controller
- [ ] **2.3** Toolbar toggle: “Wiring mode” (mirror v6 menu label).
- [ ] **2.4** When wiring off: suppress connect gesture; body drag unchanged.

**Files:** `src/hooks/useCanvasPreferences.ts` (port), `src/canvas/HandleOverlay.tsx`, `src/App.tsx`

**v6 refs:** `App.css` `.app-canvas--wiring`, `useCanvasPreferences.ts`, handle `style` in node components

---

### P0.3 — Handle contract + connect validation

- [ ] **3.1** Port `handleContract.ts` → `src/interaction/handleContract.ts` (+ Vitest vs v6 cases).
- [ ] **3.2** Port Rust `rules.rs` connect validation → reject invalid `add_edge` with clear error string.
- [ ] **3.3** Replace ad-hoc `connectPorts.ts` with `WiringController`:
  - start/end rubber band
  - validate roles before IPC
  - highlight compatible targets on drag (optional P0.3b)
- [ ] **3.4** `add_edge` returns `Result<DiagramState, String>` — already started; ensure all validation server-side.
- [ ] **3.5** App status bar shows connect errors.

**v6 refs:** `handleContract.ts`, `rules.rs`, `App.tsx` `onConnect`

---

### P0.4 — Connect gesture (redo)

- [ ] **4.1** Single `GestureController` replaces growing `useDiagramInteraction` branches:
  - modes: `pan | dragNode | connectWire`
  - pointer capture; one pointer-up path (fix double-fire)
- [ ] **4.2** Connect: port hit → rubber band → port hit → `add_edge` → **full scene refresh** (wire patch optional later).
- [ ] **4.3** Port `diagramStateApply.ts` / `diagramMerge.ts` from v6 — local diagram state stays in sync for future properties panel.
- [ ] **4.4** Integration test: `commands_test.rs` — `add_edge` on fixture nodes, scene contains new wire polyline.
- [ ] **4.5** Vitest: `wiringController.test.ts` — role rejection matrix.

**Files:** `src/canvas/interaction/GestureController.ts`, delete or shrink `useDiagramInteraction.ts`

---

### P0.5 — P0 exit gate (mandatory)

Manual script (record in PR):

1. `npm run tauri dev` → Open `dxf-export-test.diagramme` via IPC (temporary dev button OK).
2. Wiring mode **on** — handles visible on device + AV plate.
3. Connect output port → input port — wire appears, DXF export includes new edge.
4. Load Comp Gym — connect **speaker passthru → speaker** (handles visible on speakers).
5. Wiring mode **off** — drag device body, no accidental wire.

Automated:

- [ ] `cargo test handle_inventory` PASS
- [ ] `npm test` PASS
- [ ] `cargo test --workspace` PASS

---

## P1 — Remaining gestures + thin shell

### P1.1 — File ops (minimal)

- [ ] Port `useDiagramFileOps.ts` — new/open/save/save as using existing IPC.
- [ ] Remove hardcoded Comp Gym fetch as primary workflow.
- [ ] Dirty baseline + recovery (v6 parity).

**v6 ref:** `useDiagramFileOps.ts`

---

### P1.2 — Node resize

- [ ] Resize handles on selected node (Konva chrome, not scene ink).
- [ ] Throttled `update_dims` + handle attachment updates (v6 `useDragBatching` pattern).
- [ ] Scene patch for resized node + connected wires.

---

### P1.3 — Wire inner-corner drag

- [ ] Port inner-corner hit targets from wire geometry (or v6 edge interaction).
- [ ] `update_edge` IPC command for `innerCorners` mutation.
- [ ] Scene patch for affected edge only.

**v6 ref:** `docs/edge-inner-corners-audit.md`, schematic edge component

---

### P1.4 — Multi-select drag

- [ ] Selection model (shift-click / marquee — match v6).
- [ ] `move_nodes` IPC batch from v6 drag batching.

---

### P1.5 — Properties panel (debounced)

- [ ] Port `PropertiesPanel.tsx` — `update_node` IPC, debounced queue.
- [ ] Edge properties: wire category override if v6 supports.

---

## P2 — Full shell + reports

(Unchanged from original plan Tasks 20–22, but **blocked until P0 gate green**)

- Task 20: Full menus + export menu items
- Task 21: Sheets tabs, preset toolbox
- Task 22: Report dialogs → Rust `build_*_report` / `export_*_xlsx`
- Task 15–16: Implement reports crates (currently stubs)

---

## P3 — Integration gates + performance

(Unchanged from original Tasks 23–25, revised)

- [ ] `scripts/verify-gates.sh` — workspace tests + scene + DXF + report goldens
- [ ] `rg EXPORT_TEXT_VISUAL_SCALE` — zero matches
- [ ] Profile Comp Gym + Cafeteria: patch vs full scene (document; don’t block P0)
- [ ] Manual CAD signoff checklist

---

## Gesture parity matrix (living doc)

| Gesture | v6 entry | v2 owner module | Gate fixture | Status |
|---------|----------|-----------------|--------------|--------|
| Wiring toggle | `useCanvasPreferences` | `useCanvasPreferences` | — | P0.2 |
| Connect ports | `App.onConnect` | `WiringController` | dxf-export-test | **P0.4** |
| Drag node | RF drag | `GestureController` | Comp Gym | Done |
| Pan/zoom | RF viewport | `useViewport` | — | Done |
| Resize node | RF resize | `GestureController` | dxf-export-test | P1.2 |
| Inner corner | SchematicEdge | `GestureController` | edge in fixture | P1.3 |
| Multi-drag | `move_nodes` | `GestureController` | — | P1.4 |
| Delete edge | context menu | shell | — | P2 |
| Wire split | context menu | shell + IPC | — | P2 |

---

## What we stop doing

1. **Marking interaction tasks done** without Comp Gym manual script.
2. **Adding IPC commands** without UI error surfacing + test.
3. **Invisible hit targets** as the only connect affordance.
4. **Bolting gestures onto** `useDiagramInteraction` — migrate to `GestureController`.
5. **Optimizing ScenePatch** before wiring works (perf is P3).

---

## Suggested first sprint (2–3 sessions)

1. P0.0 audit + handle inventory test (expect RED)
2. P0.1 complete scene hits (GREEN inventory)
3. P0.2 handle overlay + wiring toggle
4. P0.3 + P0.4 connect redo + P0.5 gate

---

## Appendix: Comp Gym 5-minute smoke (after P0)

1. Open Comp Gym, wiring on.
2. Connect amplifier output to speaker input (visible handles).
3. Drag speaker — wire follows (ScenePatch).
4. Export DXF — new wire at same path as canvas.
5. Wiring off — drag grouping zone / device without new wires.

---

## Plan self-review

- [x] No TBD task definitions
- [x] Explains why port connect failed
- [x] Phased with explicit gates
- [x] Preserves Rust scene/DXF investment
- [x] Defers reports/shell until interaction credible

**Review this plan before implementation.** Do not continue old Task 19c checkboxes.
