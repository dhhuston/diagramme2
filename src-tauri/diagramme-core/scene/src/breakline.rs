//! Closed inset frame polylines, including bottom breakline for split instances (v6 `drawClosedInsetFramePx*`).

use diagramme_geometry::{BREAKLINE_OVERHANG, PointPx};

use crate::scene::{Scene, ScenePrimitive};

const DEFAULT_LAYER: &str = "0";
const HAIRLINE_STROKE_PX: f64 = 1.0;

fn local_to_diagram(nx: f64, ny: f64, lx: f64, ly: f64) -> PointPx {
    PointPx {
        x: nx + lx,
        y: ny + ly,
    }
}

fn push_polyline(scene: &mut Scene, points: Vec<PointPx>, closed: bool) {
    scene.primitives.push(ScenePrimitive::Polyline {
        points,
        stroke_px: HAIRLINE_STROKE_PX,
        layer: DEFAULT_LAYER.to_string(),
        color: 0,
        closed,
        edge_id: None,
        owner_node_id: None,
    });
}

/// Inset frame outline in node-local coordinates (closed).
pub fn inset_frame_outline_local(
    width_px: f64,
    height_px: f64,
    inset_px: f64,
    split_breakline_zone_width_px: Option<f64>,
) -> Vec<(f64, f64)> {
    let xi0 = inset_px;
    let yi0 = inset_px;
    let xi1 = width_px - inset_px;
    let yi1 = height_px - inset_px;

    if let Some(zone_width) = split_breakline_zone_width_px {
        let cx = width_px / 2.0;
        let half = zone_width / 2.0;
        let zone_left = cx - half;
        let zone_right = cx + half;
        let apex_x = cx - half / 2.0;
        let trough_x = cx + half / 2.0;
        let apex_y = yi1 - BREAKLINE_OVERHANG;
        let trough_y = yi1 + BREAKLINE_OVERHANG;
        vec![
            (xi0, yi0),
            (xi1, yi0),
            (xi1, yi1),
            (zone_right, yi1),
            (trough_x, trough_y),
            (apex_x, apex_y),
            (zone_left, yi1),
            (xi0, yi1),
        ]
    } else {
        vec![
            (xi0, yi0),
            (xi1, yi0),
            (xi1, yi1),
            (xi0, yi1),
        ]
    }
}

/// Opaque canvas face matching the inset frame (includes split breakline zigzag).
pub fn inset_frame_face_mask_polygon(
    nx: f64,
    ny: f64,
    width_px: f64,
    height_px: f64,
    inset_px: f64,
    split_breakline_zone_width_px: Option<f64>,
) -> Vec<PointPx> {
    inset_frame_outline_local(width_px, height_px, inset_px, split_breakline_zone_width_px)
        .into_iter()
        .map(|(lx, ly)| local_to_diagram(nx, ny, lx, ly))
        .collect()
}

/// Frame edges as separate open segments (same as row dividers) so lineweight matches hairlines.
fn push_outline(scene: &mut Scene, nx: f64, ny: f64, outline: &[(f64, f64)]) {
    if outline.len() < 2 {
        return;
    }
    let n = outline.len();
    for i in 0..n {
        let (ax, ay) = outline[i];
        let (bx, by) = outline[(i + 1) % n];
        push_polyline(
            scene,
            vec![
                local_to_diagram(nx, ny, ax, ay),
                local_to_diagram(nx, ny, bx, by),
            ],
            false,
        );
    }
}

/// Closed inset rectangle in node-local coordinates.
pub fn push_closed_inset_frame(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    width_px: f64,
    height_px: f64,
    inset_px: f64,
) {
    let outline = inset_frame_outline_local(width_px, height_px, inset_px, None);
    push_outline(scene, nx, ny, &outline);
}

/// Closed inset rectangle with a centered bottom breakline zigzag (split-instance frames).
pub fn push_closed_inset_frame_with_bottom_breakline(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    width_px: f64,
    height_px: f64,
    inset_px: f64,
    zone_width_px: f64,
) {
    let outline = inset_frame_outline_local(width_px, height_px, inset_px, Some(zone_width_px));
    push_outline(scene, nx, ny, &outline);
}
