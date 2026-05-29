//! Entity helpers for diagram-inch coordinates.

use crate::document::{CadDocument, EntityKind, TextHAlign, TextVAlign};

const MIN_SEGMENT_IN: f64 = 0.001;

pub fn add_line(
    doc: &mut CadDocument,
    layer_name: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) {
    if (x2 - x1).abs() < MIN_SEGMENT_IN && (y2 - y1).abs() < MIN_SEGMENT_IN {
        return;
    }
    doc.entities.push(EntityKind::Line {
        layer: layer_name.to_string(),
        x1,
        y1,
        x2,
        y2,
    });
}

pub fn add_lwpolyline(
    doc: &mut CadDocument,
    layer_name: &str,
    pts: &[(f64, f64)],
    closed: bool,
) {
    if pts.len() < 2 {
        return;
    }
    doc.entities.push(EntityKind::LwPolyline {
        layer: layer_name.to_string(),
        pts: pts.to_vec(),
        closed,
    });
}

pub fn add_solid(doc: &mut CadDocument, layer_name: &str, pts: &[(f64, f64)]) {
    if pts.len() < 3 {
        return;
    }

    let layer = layer_name.to_string();
    let push_solid = |doc: &mut CadDocument, corners: [(f64, f64); 4]| {
        doc.entities.push(EntityKind::Solid { layer: layer.clone(), corners });
    };

    match pts.len() {
        3 => push_solid(doc, [pts[0], pts[1], pts[2], pts[2]]),
        4 => {
            // SOLID vertex order is 1-2-4-3 in visual space; swap indices 3 and 4.
            push_solid(doc, [pts[0], pts[1], pts[3], pts[2]])
        }
        n => {
            for i in 1..=(n - 2) {
                push_solid(doc, [pts[0], pts[i], pts[i + 1], pts[i + 1]]);
            }
        }
    }
}

pub fn add_text(
    doc: &mut CadDocument,
    layer_name: &str,
    x: f64,
    y: f64,
    height: f64,
    text: &str,
    h_align: TextHAlign,
    v_align: TextVAlign,
) {
    doc.entities.push(EntityKind::Text {
        layer: layer_name.to_string(),
        x,
        y,
        height,
        value: text.to_string(),
        h_align,
        v_align,
    });
}
