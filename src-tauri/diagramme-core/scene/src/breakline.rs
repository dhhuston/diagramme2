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
    });
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
    let xi0 = inset_px;
    let yi0 = inset_px;
    let xi1 = width_px - inset_px;
    let yi1 = height_px - inset_px;
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, xi0, yi0),
            local_to_diagram(nx, ny, xi1, yi0),
            local_to_diagram(nx, ny, xi1, yi1),
            local_to_diagram(nx, ny, xi0, yi1),
        ],
        true,
    );
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
    let xi0 = inset_px;
    let yi0 = inset_px;
    let xi1 = width_px - inset_px;
    let yi1 = height_px - inset_px;
    let cx = width_px / 2.0;
    let half = zone_width_px / 2.0;
    let zone_left = cx - half;
    let zone_right = cx + half;
    let apex_x = cx - half / 2.0;
    let trough_x = cx + half / 2.0;
    let apex_y = yi1 - BREAKLINE_OVERHANG;
    let trough_y = yi1 + BREAKLINE_OVERHANG;

    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, xi0, yi0),
            local_to_diagram(nx, ny, xi1, yi0),
            local_to_diagram(nx, ny, xi1, yi1),
            local_to_diagram(nx, ny, zone_right, yi1),
            local_to_diagram(nx, ny, trough_x, trough_y),
            local_to_diagram(nx, ny, apex_x, apex_y),
            local_to_diagram(nx, ny, zone_left, yi1),
            local_to_diagram(nx, ny, xi0, yi1),
        ],
        true,
    );
}
