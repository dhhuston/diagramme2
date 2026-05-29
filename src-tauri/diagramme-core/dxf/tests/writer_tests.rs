use diagramme_dxf::{
    add_line, add_lwpolyline, add_solid, add_text, create_revit_cad_document, serialize_revit_dxf,
    CadExtentInches, TextHAlign, TextVAlign,
};
use std::collections::HashMap;

fn duplicate_handles(dxf: &str) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let lines: Vec<&str> = dxf.lines().collect();
    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i].trim() == "5" {
            let handle = lines[i + 1].trim().to_string();
            *counts.entry(handle).or_insert(0) += 1;
        }
        i += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(handle, _)| handle)
        .collect()
}

#[test]
fn minimal_document_serializes_non_empty_string() {
    let doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    let dxf = serialize_revit_dxf(&doc);
    assert!(!dxf.is_empty());
    assert!(dxf.contains("ENTITIES"));
    assert!(dxf.contains("EOF"));
}

#[test]
fn output_contains_arial_narrow_style() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    add_text(
        &mut doc,
        "0",
        1.0,
        2.0,
        0.12,
        "Test",
        TextHAlign::Left,
        TextVAlign::Baseline,
    );
    let dxf = serialize_revit_dxf(&doc);
    assert!(dxf.contains("Arial Narrow"));
}

#[test]
fn output_contains_ac1015_acadver() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    add_line(&mut doc, "WIRES", 0.0, 0.0, 5.0, 5.0);
    let dxf = serialize_revit_dxf(&doc);
    assert!(dxf.contains("AC1015") || dxf.contains("AC1018"));
    let lines: Vec<&str> = dxf.lines().collect();
    let idx = lines.iter().position(|l| l.trim() == "$ACADVER").expect("$ACADVER");
    let version = lines[idx + 2].trim();
    assert!(version == "AC1015" || version == "AC1018");
}

#[test]
fn no_duplicate_handles_in_output() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 20.0,
        max_y: 20.0,
    });
    add_line(&mut doc, "WIRES", 0.0, 0.0, 10.0, 0.0);
    add_lwpolyline(
        &mut doc,
        "WIRES",
        &[(1.0, 1.0), (2.0, 2.0), (3.0, 1.0)],
        false,
    );
    add_solid(
        &mut doc,
        "FILLS",
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    );
    add_text(
        &mut doc,
        "0",
        5.0,
        5.0,
        0.15,
        "Label",
        TextHAlign::Center,
        TextVAlign::Middle,
    );
    let dxf = serialize_revit_dxf(&doc);
    assert!(duplicate_handles(&dxf).is_empty(), "duplicate handles: {:?}", duplicate_handles(&dxf));
}

#[test]
fn solid_entities_appear_before_other_entities() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    add_line(&mut doc, "WIRES", 0.0, 0.0, 5.0, 5.0);
    add_solid(
        &mut doc,
        "FILLS",
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    );
    let dxf = serialize_revit_dxf(&doc);
    let solid_idx = dxf.find("\n  0\nSOLID\n").expect("SOLID entity");
    let line_idx = dxf.find("\n  0\nLINE\n").expect("LINE entity");
    assert!(solid_idx < line_idx);
}
