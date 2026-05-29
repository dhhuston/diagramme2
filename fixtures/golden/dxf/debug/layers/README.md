# DXF layer bisection (Revit import debugging)

Exports use the **v6 acad-ts shell** (HEADER/CLASSES/TABLES/BLOCKS/OBJECTS) with ENTITIES emitted from Rust using the same ByLayer fields as v6.

## Baseline test first (critical)

Import these **before** the numbered splits and report which fail:

| File | Purpose |
|------|---------|
| `00b-v6-acad-ts-shell-reference.dxf` | Raw acad-ts empty export from v6 (no Rust) |
| `00c-v6-full-export-reference.dxf` | Full v6 export of the same test diagram |
| `00-shell-no-entities.dxf` | v6 shell + injected extents, no entities (Rust) |

**If all three baselines fail**, the issue is likely Revit/Forge import settings or environment — not Diagramme geometry.

**If `00c` passes but `10-full.dxf` fails**, tell us which numbered split is first to fail.

Import numbered files (`01`–`10`) in order to bisect geometry.
