//! Layer / entity-type DXF splits for Revit import bisection.

use crate::audit::{audit_dxf, DxfAuditReport};
use crate::document::{
    serialize_revit_dxf_with_filter, CadDocument, EntityFilter, EntityTypeFilter,
};
use crate::scene_emit::build_cad_document_from_scene;
use diagramme_scene::Scene;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DebugSplitSpec {
    pub filename: String,
    pub description: String,
    pub filter: EntityFilter,
}

#[derive(Debug, Clone)]
pub struct DebugSplitManifest {
    pub filename: String,
    pub description: String,
    pub entity_count: usize,
    pub layers: Vec<String>,
    pub kinds: Vec<String>,
    pub audit: DxfAuditReport,
}

fn entity_summary(doc: &CadDocument) -> (usize, Vec<String>, Vec<String>) {
    let mut layers = std::collections::BTreeSet::new();
    let mut kinds = std::collections::BTreeSet::new();
    for entity in &doc.entities {
        match entity {
            crate::document::EntityKind::Line { layer, .. } => {
                layers.insert(layer.clone());
                kinds.insert("LINE".to_string());
            }
            crate::document::EntityKind::LwPolyline { layer, .. } => {
                layers.insert(layer.clone());
                kinds.insert("LWPOLYLINE".to_string());
            }
            crate::document::EntityKind::Text { layer, .. } => {
                layers.insert(layer.clone());
                kinds.insert("TEXT".to_string());
            }
            crate::document::EntityKind::Solid { layer, .. } => {
                layers.insert(layer.clone());
                kinds.insert("SOLID".to_string());
            }
        }
    }
    (
        doc.entities.len(),
        layers.into_iter().collect(),
        kinds.into_iter().collect(),
    )
}

/// Preset splits for bisecting Revit import failures by layer and entity type.
pub fn debug_split_specs() -> Vec<DebugSplitSpec> {
    vec![
        DebugSplitSpec {
            filename: "00-shell-no-entities.dxf".into(),
            description: "Document shell only — no ENTITIES (tests structure vs geometry)".into(),
            filter: EntityFilter {
                layers: Some(vec![]),
                include_kinds: None,
            },
        },
        DebugSplitSpec {
            filename: "01-layer-FILLS.dxf".into(),
            description: "FILLS layer only (SOLID fill rectangles)".into(),
            filter: EntityFilter::layers(&["FILLS"]),
        },
        DebugSplitSpec {
            filename: "02-layer-WIRES.dxf".into(),
            description: "WIRES layer only (wire polylines)".into(),
            filter: EntityFilter::layers(&["WIRES"]),
        },
        DebugSplitSpec {
            filename: "03-layer-0.dxf".into(),
            description: "Layer 0 only (device outlines + text labels)".into(),
            filter: EntityFilter::layers(&["0"]),
        },
        DebugSplitSpec {
            filename: "04-type-SOLID.dxf".into(),
            description: "All SOLID entities".into(),
            filter: EntityFilter::kinds(&[EntityTypeFilter::Solid]),
        },
        DebugSplitSpec {
            filename: "05-type-LWPOLYLINE.dxf".into(),
            description: "All LWPOLYLINE entities".into(),
            filter: EntityFilter::kinds(&[EntityTypeFilter::LwPolyline]),
        },
        DebugSplitSpec {
            filename: "06-type-TEXT.dxf".into(),
            description: "All TEXT entities".into(),
            filter: EntityFilter::kinds(&[EntityTypeFilter::Text]),
        },
        DebugSplitSpec {
            filename: "07-layer-0-LWPOLYLINE.dxf".into(),
            description: "Layer 0 outlines only (no text)".into(),
            filter: EntityFilter {
                layers: Some(vec!["0".into()]),
                include_kinds: Some(vec![EntityTypeFilter::LwPolyline]),
            },
        },
        DebugSplitSpec {
            filename: "08-layer-0-TEXT.dxf".into(),
            description: "Layer 0 text labels only (no outlines)".into(),
            filter: EntityFilter {
                layers: Some(vec!["0".into()]),
                include_kinds: Some(vec![EntityTypeFilter::Text]),
            },
        },
        DebugSplitSpec {
            filename: "09-cum-FILLS+WIRES.dxf".into(),
            description: "FILLS + WIRES (no layer 0)".into(),
            filter: EntityFilter::layers(&["FILLS", "WIRES"]),
        },
        DebugSplitSpec {
            filename: "10-full.dxf".into(),
            description: "Full export (all layers)".into(),
            filter: EntityFilter::default(),
        },
    ]
}

fn manifest_to_text(manifest: &[DebugSplitManifest]) -> String {
    let mut out = String::from("Diagramme DXF layer bisection manifest\n");
    out.push_str("Import each file into Revit in order. Note the first file that fails.\n\n");
    for entry in manifest {
        out.push_str(&format!("## {}\n", entry.filename));
        out.push_str(&format!("{}\n", entry.description));
        out.push_str(&format!(
            "entities: {} | layers: {:?} | kinds: {:?}\n",
            entry.entity_count, entry.layers, entry.kinds
        ));
        out.push_str(&entry.audit.to_text());
        out.push('\n');
    }
    out
}

/// Write bisection DXF files for a scene into `out_dir`.
pub fn write_layer_debug_bundle(scene: &Scene, out_dir: &Path) -> std::io::Result<Vec<DebugSplitManifest>> {
    fs::create_dir_all(out_dir)?;
    let doc = build_cad_document_from_scene(scene);
    let mut manifest = Vec::new();

    for spec in debug_split_specs() {
        let dxf = serialize_revit_dxf_with_filter(&doc, &spec.filter);
        let path = out_dir.join(&spec.filename);
        fs::write(&path, &dxf)?;

        let filtered = spec.filter.apply(&doc.entities);
        let filtered_doc = CadDocument {
            extent: doc.extent,
            entities: filtered,
        };
        let (entity_count, layers, kinds) = entity_summary(&filtered_doc);
        let audit = audit_dxf(&dxf);
        manifest.push(DebugSplitManifest {
            filename: spec.filename,
            description: spec.description,
            entity_count,
            layers,
            kinds,
            audit,
        });
    }

    fs::write(out_dir.join("manifest.txt"), manifest_to_text(&manifest))?;
    Ok(manifest)
}

/// Default output directory relative to repo root.
pub fn default_debug_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/golden/dxf/debug/layers")
}
