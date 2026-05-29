# Diagram fixtures

Regression and parity corpus copied from Diagramme v6. Each `.diagramme` file is valid JSON (`format: "diagramme"`, `version: 2`).

## `diagrams/dxf-export-test.diagramme`

**Source:** `Diagramme_v6/src/fixtures/buildDxfExportTestDiagram.ts` (`DXF_EXPORT_TEST_DIAGRAM_JSON`)

Synthetic sheet that exercises the full palette and export surface in one place:

- Every schematic node type (devices, plates, speakers, mic, volume control, antennas, patch panels, junction, flyoffs, wiretags, grouping zones, text blocks)
- All wire categories (audio, video, control, network, RF, default)
- Port bundles and bundle-to-bundle wiring
- Grouped parallel edges and wire-category mismatch edges

Use for DXF golden tests, scene-builder parity, wire-geometry regression, and schema round-trip smoke tests.

## `diagrams/cafeteria-d104a.diagramme`

**Source:** `Diagramme_v6/troubleshooting/Cafeteria D104A.diagramme`

Real-world cafeteria AV schematic (room D104A): 52 nodes, 61 edges. Heavy use of `deviceV2`, `wiretag` pairs, speakers, AV plates, and antennas.

Use for save/load compatibility, layout at realistic scale, wire routing complexity, and end-to-end export/report parity against v6.
