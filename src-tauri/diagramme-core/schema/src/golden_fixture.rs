//! Shared regression diagram for scene, DXF, and integration tests.
//!
//! Source: `fixtures/golden/Comp Gym F102A.diagramme` (real-world gym AV schematic).

use crate::{DiagramState, Node, ProjectState, Sheet};

/// Embedded Comp Gym F102A project JSON (compile-time).
pub const GOLDEN_DIAGRAM_JSON: &str =
    include_str!("../../../../fixtures/golden/Comp Gym F102A.diagramme");

/// Synthetic palette / node-type exercise sheet (v6 `buildDxfExportTestDiagram`).
pub const DXF_EXPORT_TEST_JSON: &str =
    include_str!("../../../../fixtures/diagrams/dxf-export-test.diagramme");

/// Parse the golden fixture project.
pub fn load_golden_fixture() -> ProjectState {
    serde_json::from_str(GOLDEN_DIAGRAM_JSON).expect("parse Comp Gym F102A golden fixture")
}

/// Parse the dxf-export-test palette fixture.
pub fn load_dxf_export_test_fixture() -> ProjectState {
    serde_json::from_str(DXF_EXPORT_TEST_JSON).expect("parse dxf-export-test fixture")
}

/// Active sheet (or first sheet if `active_sheet_id` is missing).
pub fn active_sheet<'a>(project: &'a ProjectState) -> &'a Sheet {
    project
        .sheets
        .iter()
        .find(|s| s.id == project.active_sheet_id)
        .or_else(|| project.sheets.first())
        .expect("golden fixture has a sheet")
}

/// Active sheet diagram state.
pub fn active_sheet_state<'a>(project: &'a ProjectState) -> &'a DiagramState {
    &active_sheet(project).state
}

/// First node of `node_type`, if any.
pub fn find_first_node<'a>(project: &'a ProjectState, node_type: &str) -> Option<&'a Node> {
    active_sheet(project)
        .state
        .nodes
        .iter()
        .find(|n| n.node_type == node_type)
}

/// First node of `node_type` (panics when absent).
pub fn find_node<'a>(project: &'a ProjectState, node_type: &str) -> &'a Node {
    find_first_node(project, node_type)
        .unwrap_or_else(|| panic!("golden fixture missing node type {node_type}"))
}

/// First `deviceV2` / legacy `device` node.
pub fn first_device_v2<'a>(project: &'a ProjectState) -> &'a Node {
    find_first_node(project, "deviceV2")
        .or_else(|| find_first_node(project, "device"))
        .expect("golden fixture has deviceV2")
}

/// Uppercase `tagCode / tagNumber` label for device-style nodes.
pub fn device_tag_label(node: &Node) -> String {
    let d = &node.data;
    let code = d
        .get("tagCode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let num = d
        .get("tagNumber")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    format!("{code} / {num}").trim().to_uppercase()
}

/// Uppercase primary description/title line for device-style nodes.
pub fn device_title_label(node: &Node) -> String {
    node.data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase()
}

/// First patch panel node (`lppPatchPanel`, `vpbPatchPanel`, or `dppPatchPanel`).
pub fn first_patch_panel<'a>(project: &'a ProjectState) -> Option<&'a Node> {
    active_sheet(project).state.nodes.iter().find(|n| {
        matches!(
            n.node_type.as_str(),
            "lppPatchPanel" | "vpbPatchPanel" | "dppPatchPanel"
        )
    })
}

/// First wiretag with `end == "a"`.
pub fn first_wiretag_end_a<'a>(project: &'a ProjectState) -> Option<&'a Node> {
    active_sheet(project)
        .state
        .nodes
        .iter()
        .find(|n| {
            n.node_type == "wiretag"
                && n.data
                    .get("end")
                    .and_then(|v| v.as_str())
                    .is_some_and(|e| e == "a")
        })
}
