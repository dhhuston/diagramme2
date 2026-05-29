//! v6-compatible persisted project types.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod document_envelope;
pub mod export_normalize;
pub mod golden_fixture;
pub mod model;
pub mod preset_format;

pub use document_envelope::{
    validate_diagram_envelope, DIAGRAMME_FORMAT, DIAGRAMME_VERSION,
};
pub use export_normalize::{
    active_bundle_handles, device_v2_bundle_handle_id, filter_bundled_side,
    is_bundle_bracket_active, is_bundle_handle_id, is_device_v2_bundle_bracket_active,
    normalize_diagram_for_export, normalize_diagram_for_persist, normalize_project_for_persist,
};
pub use preset_format::{
    node_type_from_filename, parse_preset_file_text, serialize_preset_file, strip_ephemeral_node_data,
    ParsedPresetFile, PresetNodeType, AVDEVICE_EXT, PLATE_EXT, PRESET_SCHEMA_VERSION,
};

pub use golden_fixture::{
    active_sheet, active_sheet_state, device_tag_label, device_title_label,
    find_first_node, find_node, first_device_v2, first_patch_panel, first_wiretag_end_a,
    load_dxf_export_test_fixture, load_golden_fixture, DXF_EXPORT_TEST_JSON, GOLDEN_DIAGRAM_JSON,
};
pub use model::{
    DiagramState, Edge, EmbeddedPreset, Node, NodeDimension, ProjectState, Sheet, XY,
};
