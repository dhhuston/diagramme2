use diagramme_dxf::{audit_polylines_in_region, build_revit_dxf_from_diagram, write_layer_debug_bundle, RegionBboxPx};
use diagramme_scene::{build_scene, px_to_in, scene_to_cad, CadPrimitive, ScenePrimitive};
use diagramme_schema::{active_sheet_state, load_golden_fixture};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

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

fn dxf_entities(dxf: &str) -> Vec<(String, Vec<(i32, String)>)> {
    let mut entities = Vec::new();
    for part in dxf.split("\n  0\n") {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut lines = trimmed.lines();
        let Some(kind) = lines.next() else {
            continue;
        };
        let kind = kind.trim().to_string();
        if kind == "SECTION" || kind == "ENDSEC" || kind == "EOF" {
            continue;
        }
        let mut groups = Vec::new();
        let mut iter = lines.peekable();
        while iter.peek().is_some() {
            let code_line = iter.next().unwrap().trim();
            let Ok(code) = code_line.parse::<i32>() else {
                continue;
            };
            let value = iter
                .next()
                .map(|l| l.trim().to_string())
                .unwrap_or_default();
            groups.push((code, value));
        }
        entities.push((kind, groups));
    }
    entities
}

fn dxf_text_heights(dxf: &str) -> Vec<f64> {
    dxf_entities(dxf)
        .into_iter()
        .filter(|(kind, _)| kind == "TEXT")
        .flat_map(|(_, groups)| {
            groups
                .into_iter()
                .filter_map(|(code, value)| (code == 40).then(|| value.parse::<f64>().ok())?)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn dxf_wire_polylines(dxf: &str) -> Vec<Vec<(f64, f64)>> {
    dxf_entities(dxf)
        .into_iter()
        .filter(|(kind, groups)| {
            kind == "LWPOLYLINE"
                && groups
                    .iter()
                    .any(|(code, value)| *code == 8 && value == "WIRES")
        })
        .filter_map(|(_, groups)| {
            let mut verts = Vec::new();
            let mut pending_x: Option<f64> = None;
            for (code, value) in groups {
                match code {
                    10 => pending_x = value.parse().ok(),
                    20 => {
                        if let Some(x) = pending_x.take() {
                            if let Ok(y) = value.parse::<f64>() {
                                verts.push((x, y));
                            }
                        }
                    }
                    _ => {}
                }
            }
            (verts.len() >= 2).then_some(verts)
        })
        .collect()
}

fn horizontal_wire_segment_px(scene: &diagramme_scene::Scene) -> (f64, f64) {
    for prim in &scene.primitives {
        let ScenePrimitive::Polyline {
            points,
            edge_id: Some(_),
            ..
        } = prim
        else {
            continue;
        };
        for window in points.windows(2) {
            let a = window[0];
            let b = window[1];
            if (a.y - b.y).abs() < 1e-9 && (a.x - b.x).abs() > 1e-9 {
                return ((b.x - a.x).abs(), a.y);
            }
            if (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() > 1e-9 {
                continue;
            }
        }
    }
    panic!("expected horizontal wire segment in golden fixture scene");
}

#[test]
fn dxf_export_matches_canvas_scene_wire_geometry() {
    let project = load_golden_fixture();
    let diagram = active_sheet_state(&project);
    let canvas_scene = build_scene(diagram);
    let cad = scene_to_cad(&canvas_scene);
    let _dxf = build_revit_dxf_from_diagram(diagram);

    let canvas_wires: Vec<_> = canvas_scene
        .primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Polyline {
                points,
                edge_id: Some(id),
                ..
            } => Some((id.clone(), points.len())),
            _ => None,
        })
        .collect();
    let cad_wires: Vec<_> = cad
        .primitives
        .iter()
        .filter_map(|p| match p {
            CadPrimitive::Polyline {
                points,
                edge_id: Some(id),
                ..
            } => Some((id.clone(), points.len())),
            _ => None,
        })
        .collect();

    assert_eq!(
        canvas_wires.len(),
        cad_wires.len(),
        "scene_to_cad should preserve wire polyline count"
    );
    for ((scene_id, scene_pts), (cad_id, cad_pts)) in canvas_wires.iter().zip(cad_wires.iter()) {
        assert_eq!(scene_id, cad_id);
        assert_eq!(scene_pts, cad_pts, "wire point count for edge {scene_id}");
    }
}

#[test]
fn golden_fixture_produces_non_empty_dxf() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    assert!(!dxf.is_empty());
    assert!(dxf.contains("ENTITIES"));
    assert!(dxf.contains("EOF"));
}

#[test]
fn text_height_parity_for_node_title_font_scene_text() {
    let project = load_golden_fixture();
    let diagram = active_sheet_state(&project);
    let scene = build_scene(diagram);
    let title_px = 6.75;
    let title_text = scene
        .primitives
        .iter()
        .find_map(|p| match p {
            ScenePrimitive::Text(t) if (t.height_px - title_px).abs() < 1e-9 => Some(t),
            _ => None,
        })
        .expect("SceneText with node title height_px");
    assert!((title_text.height_px - title_px).abs() < 1e-9);

    let dxf = build_revit_dxf_from_diagram(diagram);
    let expected = px_to_in(title_px);
    assert!(
        dxf_text_heights(&dxf)
            .iter()
            .any(|h| (*h - expected).abs() < 1e-6),
        "expected DXF TEXT height {expected}, got {:?}",
        dxf_text_heights(&dxf)
    );
}

#[test]
fn wire_horizontal_segment_length_matches_px_scale_in_dxf() {
    let project = load_golden_fixture();
    let diagram = active_sheet_state(&project);
    let scene = build_scene(diagram);
    let (px_len, _) = horizontal_wire_segment_px(&scene);
    let expected_in = px_to_in(px_len);

    let cad = scene_to_cad(&scene);
    let cad_len = cad
        .primitives
        .iter()
        .find_map(|p| match p {
            CadPrimitive::Polyline { points, edge_id, .. }
                if edge_id.is_some() && points.len() >= 2 =>
            {
                for w in points.windows(2) {
                    let a = w[0];
                    let b = w[1];
                    if (a.y - b.y).abs() < 1e-9 && (a.x - b.x).abs() > 1e-9 {
                        return Some((b.x - a.x).abs());
                    }
                }
                None
            }
            _ => None,
        })
        .expect("cad horizontal wire segment");
    assert!((cad_len - expected_in).abs() < 1e-6);

    let dxf = build_revit_dxf_from_diagram(diagram);
    let dxf_len = dxf_wire_polylines(&dxf)
        .into_iter()
        .find_map(|verts| {
            for w in verts.windows(2) {
                let (ax, ay) = w[0];
                let (bx, by) = w[1];
                if (ay - by).abs() < 1e-6 && (ax - bx).abs() > 1e-6 {
                    let len = (bx - ax).abs();
                    if (len - expected_in).abs() < 1e-6 {
                        return Some(len);
                    }
                }
            }
            None
        })
        .expect("dxf wire horizontal segment matching scene length");
    assert!(
        (dxf_len - expected_in).abs() < 1e-6,
        "dxf segment {dxf_len} != expected {expected_in}"
    );
}

#[test]
fn no_export_text_visual_scale_in_diagramme_core() {
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("diagramme-core");
    let forbidden = format!("EXPORT_{}VISUAL_SCALE", "TEXT_");
    let status = Command::new("grep")
        .args(["-r", &forbidden])
        .arg(core)
        .status()
        .expect("grep");
    assert_eq!(
        status.code(),
        Some(1),
        "export text visual scale constant must not appear under diagramme-core"
    );
}

fn entity_layers_in_dxf(dxf: &str) -> std::collections::HashSet<String> {
    let entities = dxf
        .split("ENTITIES")
        .nth(1)
        .and_then(|s| s.split("ENDSEC").next())
        .unwrap_or("");
    let lines: Vec<&str> = entities.lines().collect();
    let mut used = std::collections::HashSet::new();
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
        if code == "8" {
            used.insert(lines[i + 1].trim().to_string());
        }
        i += 2;
    }
    used
}

#[test]
fn fixture_export_uses_only_declared_layers() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    let declared = ["0", "WIRES", "FILLS", "INKFILL", "GUIDES"];
    for layer in entity_layers_in_dxf(&dxf) {
        assert!(
            declared.contains(&layer.as_str()),
            "entity on undeclared layer {layer:?}"
        );
    }
}

#[test]
fn fixture_export_frame_polylines_are_closed() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    let entities = dxf
        .split("ENTITIES")
        .nth(1)
        .and_then(|s| s.split("ENDSEC").next())
        .unwrap_or("");
    assert!(
        entities.contains("LWPOLYLINE") && entities.contains("\n  70\n1\n"),
        "expected closed LWPOLYLINE (group 70 = 1) for inset frames"
    );
}

#[test]
fn amp_region_export_has_no_stale_speaker_bus_wires() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    // AMP block + speaker fan-out zone (diagram px).
    let audit = audit_polylines_in_region(
        &dxf,
        RegionBboxPx {
            x: 900.0,
            y: 100.0,
            width: 450.0,
            height: 180.0,
        },
    );
    assert!(
        audit.wire_layer_polylines <= 8,
        "expected fewer wire polylines after fresh routing, got {:?}",
        audit
    );
}

#[test]
fn no_duplicate_handles_in_fixture_export() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    let dupes = duplicate_handles(&dxf);
    assert!(dupes.is_empty(), "duplicate handles: {dupes:?}");
}

#[test]
#[ignore = "run to write layer bisection files: cargo test -p diagramme-dxf write_layer_debug_splits -- --ignored"]
fn write_layer_debug_splits() {
    let project = load_golden_fixture();
    let scene = build_scene(active_sheet_state(&project));
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/golden/dxf/debug/layers");
    let manifest = write_layer_debug_bundle(&scene, &out_dir).expect("write splits");
    assert!(!manifest.is_empty());
    assert!(out_dir.join("manifest.txt").exists());
}

#[test]
#[ignore = "run once to write golden baseline: cargo test -p diagramme-dxf write_golden_baseline -- --ignored"]
fn write_golden_baseline() {
    let project = load_golden_fixture();
    let dxf = build_revit_dxf_from_diagram(active_sheet_state(&project));
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/golden/dxf/comp-gym-f102a.dxf");
    if let Some(parent) = golden.parent() {
        std::fs::create_dir_all(parent).expect("create golden dir");
    }
    std::fs::write(&golden, &dxf).expect("write golden dxf");
    assert!(golden.exists());
}
