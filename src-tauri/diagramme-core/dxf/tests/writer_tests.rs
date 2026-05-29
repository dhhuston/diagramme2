use diagramme_dxf::{
    add_line, add_lwpolyline, add_solid, add_text, create_revit_cad_document, serialize_revit_dxf,
    CadExtentInches, TextHAlign, TextVAlign,
};
use std::collections::HashMap;

fn entities_section(dxf: &str) -> &str {
    let start = dxf.find("  2\nENTITIES\n").map(|i| i + "  2\nENTITIES\n".len());
    let Some(start) = start else {
        return "";
    };
    let end = dxf[start..].find("\n  0\nENDSEC\n").map(|i| start + i);
    match end {
        Some(end) => &dxf[start..end],
        None => &dxf[start..],
    }
}

fn duplicate_handles(dxf: &str) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let lines: Vec<&str> = entities_section(dxf).lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let code = lines[i].trim();
        if code.is_empty() {
            i += 1;
            continue;
        }
        if i + 1 >= lines.len() {
            break;
        }
        if code == "5" {
            let handle = lines[i + 1].trim().to_string();
            *counts.entry(handle).or_insert(0) += 1;
        }
        i += 2;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(handle, _)| handle)
        .collect()
}

fn dangling_handle_refs(dxf: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = dxf.lines().collect();
    let mut handles = std::collections::HashSet::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        let code = lines[i].trim();
        let val = lines[i + 1].trim().to_uppercase();
        if code == "5" || code == "105" {
            handles.insert(val);
        }
        i += 1;
    }

    let mut missing = Vec::new();
    i = 0;
    while i + 1 < lines.len() {
        let code = lines[i].trim();
        let val = lines[i + 1].trim().to_uppercase();
        if matches!(code, "330" | "340" | "350") && val != "0" && !handles.contains(&val) {
            missing.push((code.to_string(), val));
        }
        i += 1;
    }
    missing
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
fn output_includes_blocks_and_objects_for_autodesk_translation() {
    let doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    let dxf = serialize_revit_dxf(&doc);
    assert!(dxf.contains("  2\nCLASSES\n"), "missing CLASSES section");
    assert!(dxf.contains("  2\nBLOCKS\n"), "missing BLOCKS section");
    assert!(dxf.contains("  2\nOBJECTS\n"), "missing OBJECTS section");
    assert!(dxf.contains("ACAD_LAYOUT"), "missing ACAD_LAYOUT dictionary");
    assert!(dxf.contains("*Model_Space"), "missing model space block");
    assert!(dxf.contains("AcDbLayout"), "missing LAYOUT objects");
}

#[test]
fn no_dangling_handle_refs_in_output() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 20.0,
        max_y: 20.0,
    });
    add_line(&mut doc, "WIRES", 0.0, 0.0, 10.0, 0.0);
    add_solid(
        &mut doc,
        "FILLS",
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    );
    let dxf = serialize_revit_dxf(&doc);
    let dangling = dangling_handle_refs(&dxf);
    assert!(dangling.is_empty(), "dangling handle refs: {dangling:?}");
}

#[test]
fn solid_entities_appear_before_other_entities() {
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 10.0,
        max_y: 10.0,
    });
    add_lwpolyline(
        &mut doc,
        "WIRES",
        &[(0.0, 0.0), (5.0, 5.0)],
        false,
    );
    add_solid(
        &mut doc,
        "FILLS",
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    );
    let dxf = serialize_revit_dxf(&doc);
    let solid_idx = dxf.find("\n  0\nSOLID\n").expect("SOLID entity");
    let line_idx = dxf.find("\n  0\nLWPOLYLINE\n").expect("LWPOLYLINE entity");
    assert!(solid_idx < line_idx);
}
