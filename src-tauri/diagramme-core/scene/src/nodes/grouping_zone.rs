//! Grouping zone scene primitives — ported from v6 `appendGroupingZoneRevitDxf`.

use diagramme_geometry::{
    get_label_anchor, polyline_flat_bounds, to_pairs, PointPx, RectPx, TextHAlign, TextRole,
    TextVAlign, GROUPING_ZONE_DEFAULT_H, GROUPING_ZONE_DEFAULT_W,
};
use diagramme_schema::Node;

use crate::nodes::emit::{
    push_dashed_line_px, push_text, scene_text_from_role, node_height, node_width,
};
use crate::scene::Scene;
use crate::text::sanitize_text;

/// Scene bounds (diagram px).
pub fn grouping_zone_scene_bounds(node: &Node) -> RectPx {
    let nx = node.position.x;
    let ny = node.position.y;
    let w = node_width(node, GROUPING_ZONE_DEFAULT_W);
    let h = node_height(node, GROUPING_ZONE_DEFAULT_H);
    let shape = node
        .data
        .get("shape")
        .and_then(|v| v.as_str())
        .unwrap_or("rect");

    if shape == "polyline" {
        if let Some(pts) = node.data.get("polylinePoints").and_then(|v| v.as_array()) {
            let flat: Vec<f64> = pts
                .iter()
                .filter_map(|v| v.as_f64())
                .collect();
            if flat.len() >= 4 {
                if let Some((min_x, min_y, max_x, max_y)) = polyline_flat_bounds(&flat) {
                    return RectPx::new(nx + min_x, ny + min_y, max_x - min_x, max_y - min_y);
                }
            }
        }
    }

    RectPx::new(nx, ny, w, h)
}

/// Append grouping zone drawable primitives to `scene`.
pub fn append_grouping_zone_scene(scene: &mut Scene, node: &Node) {
    let nx = node.position.x;
    let ny = node.position.y;
    let w = node_width(node, GROUPING_ZONE_DEFAULT_W);
    let h = node_height(node, GROUPING_ZONE_DEFAULT_H);
    let data = &node.data;
    let shape = data
        .get("shape")
        .and_then(|v| v.as_str())
        .unwrap_or("rect");

    if shape == "polyline" {
        if let Some(pts) = data.get("polylinePoints").and_then(|v| v.as_array()) {
            let flat: Vec<f64> = pts.iter().filter_map(|v| v.as_f64()).collect();
            let pairs = to_pairs(&flat);
            if pairs.len() >= 2 {
                let n = pairs.len();
                for i in 0..n {
                    let (ax, ay) = pairs[i];
                    let (bx, by) = pairs[(i + 1) % n];
                    push_dashed_line_px(scene, nx, ny, ax, ay, bx, by);
                }
            }
        }
    } else {
        let inset = 0.25;
        push_dashed_line_px(scene, nx, ny, inset, inset, w - inset, inset);
        push_dashed_line_px(scene, nx, ny, inset, inset, inset, h - inset);
        push_dashed_line_px(scene, nx, ny, w - inset, inset, w - inset, h - inset);
        push_dashed_line_px(scene, nx, ny, inset, h - inset, w - inset, h - inset);
    }

    let lab = sanitize_text(
        data.get("label").and_then(|v| v.as_str()).unwrap_or(""),
        120,
    );
    if !lab.is_empty() {
        let anchor = if shape == "polyline" {
            if let Some(pts) = data.get("polylinePoints").and_then(|v| v.as_array()) {
                let flat: Vec<f64> = pts.iter().filter_map(|v| v.as_f64()).collect();
                get_label_anchor(&to_pairs(&flat))
            } else {
                diagramme_geometry::LabelAnchor { x: 0.0, y: 0.0 }
            }
        } else {
            diagramme_geometry::LabelAnchor { x: 0.0, y: 0.0 }
        };
        push_text(
            scene,
            scene_text_from_role(
                TextRole::GroupingZone,
                PointPx {
                    x: nx + anchor.x,
                    y: ny + anchor.y - 3.0,
                },
                lab,
                Some(TextHAlign::Left),
                Some(TextVAlign::Bottom),
            ),
        );
    }
}
