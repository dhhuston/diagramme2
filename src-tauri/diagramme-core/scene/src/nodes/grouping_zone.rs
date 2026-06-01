//! Grouping zone scene primitives — ported from v6 `appendGroupingZoneRevitDxf`.

use diagramme_geometry::{
    get_label_anchor, polyline_flat_bounds, to_pairs, PointPx, Pt, RectPx, TextHAlign, TextRole,
    TextVAlign, GROUPING_ZONE_DEFAULT_H, GROUPING_ZONE_DEFAULT_W,
};
use diagramme_schema::Node;

use crate::nodes::emit::{push_dashed_line_px, push_text, scene_text_from_role, node_height, node_width};
use crate::scene::{HitTarget, Scene};
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

    push_grouping_zone_boundary_hits(scene, node, nx, ny, w, h, shape, data);
}

/// Pick band half-width along each grouping zone edge (diagram px).
const BOUNDARY_PICK_PX: f64 = 6.0;

fn push_boundary_strip(
    scene: &mut Scene,
    node_id: &str,
    index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    scene.hits.insert(
        0,
        HitTarget {
            id: format!("{node_id}:boundary:{index}"),
            bounds: RectPx::new(x, y, width, height),
            node_id: Some(node_id.to_string()),
            edge_id: None,
            handle_id: None,
            face_mask_bounds: None,
            face_mask_polygon: None,
            wire_grip_segment: None,
            wire_grip_orientation: None,
        },
    );
}

fn push_rect_boundary_hits(scene: &mut Scene, node: &Node, nx: f64, ny: f64, w: f64, h: f64) {
    let p = BOUNDARY_PICK_PX.min(w / 2.0).min(h / 2.0);
    let strips = [
        (nx, ny, w, p),
        (nx, ny + h - p, w, p),
        (nx, ny, p, h),
        (nx + w - p, ny, p, h),
    ];
    for (i, (x, y, bw, bh)) in strips.iter().enumerate() {
        push_boundary_strip(scene, &node.id, i, *x, *y, *bw, *bh);
    }
}

fn push_polyline_boundary_hits(scene: &mut Scene, node: &Node, nx: f64, ny: f64, pairs: &[Pt]) {
    let n = pairs.len();
    if n < 2 {
        return;
    }
    let p = BOUNDARY_PICK_PX;
    for i in 0..n {
        let (ax, ay) = pairs[i];
        let (bx, by) = pairs[(i + 1) % n];
        let x1 = nx + ax;
        let y1 = ny + ay;
        let x2 = nx + bx;
        let y2 = ny + by;
        let min_x = x1.min(x2) - p;
        let min_y = y1.min(y2) - p;
        let max_x = x1.max(x2) + p;
        let max_y = y1.max(y2) + p;
        push_boundary_strip(
            scene,
            &node.id,
            i,
            min_x,
            min_y,
            (max_x - min_x).max(p),
            (max_y - min_y).max(p),
        );
    }
}

fn push_grouping_zone_boundary_hits(
    scene: &mut Scene,
    node: &Node,
    nx: f64,
    ny: f64,
    w: f64,
    h: f64,
    shape: &str,
    data: &serde_json::Value,
) {
    if shape == "polyline" {
        if let Some(pts) = data.get("polylinePoints").and_then(|v| v.as_array()) {
            let flat: Vec<f64> = pts.iter().filter_map(|v| v.as_f64()).collect();
            let pairs = to_pairs(&flat);
            if pairs.len() >= 2 {
                push_polyline_boundary_hits(scene, node, nx, ny, &pairs);
                return;
            }
        }
    }
    push_rect_boundary_hits(scene, node, nx, ny, w, h);
}
