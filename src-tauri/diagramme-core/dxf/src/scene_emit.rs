//! Emit a built `Scene` to Revit-safe DXF via `scene_to_cad` only.

use diagramme_scene::{scene_to_cad, CadHAlign, CadPrimitive, CadVAlign, Scene};

use crate::document::{create_revit_cad_document, serialize_revit_dxf, CadDocument, CadExtentInches, TextHAlign, TextVAlign};
use crate::primitives::{add_lwpolyline, add_solid, add_text};

fn cad_halign(h: CadHAlign) -> TextHAlign {
    match h {
        CadHAlign::Left => TextHAlign::Left,
        CadHAlign::Center => TextHAlign::Center,
        CadHAlign::Right => TextHAlign::Right,
    }
}

fn cad_valign(v: CadVAlign) -> TextVAlign {
    match v {
        CadVAlign::Top => TextVAlign::Top,
        CadVAlign::Middle => TextVAlign::Middle,
        CadVAlign::Bottom => TextVAlign::Bottom,
    }
}

fn point_in_pair(p: diagramme_scene::PointIn) -> (f64, f64) {
    (p.x, p.y)
}

fn emit_cad_primitive(doc: &mut CadDocument, prim: &CadPrimitive) {
    match prim {
        CadPrimitive::Polyline { points, layer, .. } => {
            let pts: Vec<(f64, f64)> = points.iter().copied().map(point_in_pair).collect();
            add_lwpolyline(doc, layer, &pts, false);
        }
        CadPrimitive::Rect {
            rect,
            fill,
            layer,
            ..
        } => {
            let x0 = rect.origin.x;
            let y0 = rect.origin.y;
            let x1 = x0 + rect.width_in;
            let y1 = y0 - rect.height_in;
            let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
            if fill.is_some() {
                add_solid(doc, "FILLS", &corners);
            }
            add_lwpolyline(doc, layer, &corners, true);
        }
        CadPrimitive::Solid { vertices, layer, .. } => {
            let pts: Vec<(f64, f64)> = vertices.iter().copied().map(point_in_pair).collect();
            add_solid(doc, layer, &pts);
        }
        CadPrimitive::Text(text) => {
            add_text(
                doc,
                "0",
                text.position.x,
                text.position.y,
                text.height_in,
                &text.content,
                cad_halign(text.halign),
                cad_valign(text.valign),
            );
        }
    }
}

/// Transform `scene` through `scene_to_cad` and serialize Revit-safe DXF.
pub fn emit_scene_to_dxf(scene: &Scene) -> String {
    let cad = scene_to_cad(scene);
    let mut doc = create_revit_cad_document(CadExtentInches {
        min_x: cad.extent.min_x,
        min_y: cad.extent.min_y,
        max_x: cad.extent.max_x,
        max_y: cad.extent.max_y,
    });

    for prim in &cad.primitives {
        emit_cad_primitive(&mut doc, prim);
    }

    serialize_revit_dxf(&doc)
}
