//! Write layer-bisection DXF files for Revit import debugging.
//!
//! Usage:
//!   cargo run -p diagramme-dxf --bin split-dxf-debug
//!   cargo run -p diagramme-dxf --bin split-dxf-debug -- /path/to/output-dir

use diagramme_dxf::{
    audit_dxf, build_revit_dxf_from_diagram, default_debug_out_dir, write_layer_debug_bundle,
};
use diagramme_scene::{build_scene_with_options, SceneBuildOptions};
use diagramme_schema::{active_sheet_state, load_golden_fixture};
use std::env;
use std::path::PathBuf;

fn load_fixture_sheet_state() -> diagramme_schema::DiagramState {
    active_sheet_state(&load_golden_fixture()).clone()
}

fn main() {
    let out_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_debug_out_dir);

    let diagram = load_fixture_sheet_state();
    let scene = build_scene_with_options(&diagram, SceneBuildOptions::default());
    let manifest = write_layer_debug_bundle(&scene, &out_dir).expect("write debug splits");

    println!("Wrote {} bisection DXF files to {}", manifest.len(), out_dir.display());
    for entry in &manifest {
        println!(
            "  {} — {} entities ({:?}) audit={}",
            entry.filename,
            entry.entity_count,
            entry.kinds,
            if entry.audit.is_clean() { "clean" } else { "ISSUES" }
        );
    }

    let full = build_revit_dxf_from_diagram(&diagram);
    let audit = audit_dxf(&full);
    println!("\nFull export audit:\n{}", audit.to_text());
}
