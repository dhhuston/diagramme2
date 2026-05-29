//! Bundle bracket polylines (v6 `drawBracketList`).

use diagramme_geometry::{
    bundle_bracket_fillet_radius_px, text_style_for_role, PointPx, TextRole,
    BUNDLE_ARROW_LEG_PX, BUNDLE_ARROW_STEM_PX, BUNDLE_STUB_PX, Side,
};

use crate::scene::{HAlign, Scene, ScenePrimitive, SceneText, VAlign};

const DEFAULT_LAYER: &str = "0";
const HAIRLINE_STROKE_PX: f64 = 1.0;
const FILLET_ARC_STEPS: usize = 10;

#[derive(Debug, Clone, Copy)]
pub struct BracketDrawSlot {
    pub y0: f64,
    pub y1: f64,
    pub count: usize,
}

fn local_to_diagram(nx: f64, ny: f64, lx: f64, ly: f64) -> PointPx {
    PointPx {
        x: nx + lx,
        y: ny + ly,
    }
}

fn push_polyline(scene: &mut Scene, nx: f64, ny: f64, points: &[(f64, f64)], closed: bool) {
    scene.primitives.push(ScenePrimitive::Polyline {
        points: points
            .iter()
            .map(|&(lx, ly)| local_to_diagram(nx, ny, lx, ly))
            .collect(),
        stroke_px: HAIRLINE_STROKE_PX,
        layer: DEFAULT_LAYER.to_string(),
        color: 0,
        closed,
        edge_id: None,
    });
}

fn push_line(scene: &mut Scene, nx: f64, ny: f64, x0: f64, y0: f64, x1: f64, y1: f64) {
    push_polyline(scene, nx, ny, &[(x0, y0), (x1, y1)], false);
}

/// Tessellate a circular arc with known center, start angle, and sweep (radians).
fn tessellate_arc(
    center: (f64, f64),
    radius: f64,
    start_angle: f64,
    sweep_angle: f64,
    steps: usize,
) -> Vec<(f64, f64)> {
    if radius <= 1e-6 || steps == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let a = start_angle + sweep_angle * t;
        out.push((center.0 + a.cos() * radius, center.1 + a.sin() * radius));
    }
    out
}

/// Quarter-circle fillet center and sweep for a bundle bracket corner.
fn bracket_fillet_arc(
    side: Side,
    is_top: bool,
    spine_x: f64,
    y: f64,
    r: f64,
) -> ((f64, f64), f64, f64) {
    match (side, is_top) {
        (Side::Right, true) => ((spine_x + r, y + r), std::f64::consts::PI, std::f64::consts::FRAC_PI_2),
        (Side::Right, false) => ((spine_x - r, y - r), 0.0, std::f64::consts::FRAC_PI_2),
        (Side::Left, true) => ((spine_x - r, y + r), 0.0, -std::f64::consts::FRAC_PI_2),
        (Side::Left, false) => ((spine_x + r, y - r), std::f64::consts::PI, -std::f64::consts::FRAC_PI_2),
    }
}

fn push_fillet_arc(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    side: Side,
    is_top: bool,
    spine_x: f64,
    y: f64,
    r: f64,
) {
    if r <= 1e-6 {
        return;
    }
    let (center, start_angle, sweep) = bracket_fillet_arc(side, is_top, spine_x, y, r);
    let arc_pts = tessellate_arc(center, r, start_angle, sweep, FILLET_ARC_STEPS);
    for w in arc_pts.windows(2) {
        push_line(scene, nx, ny, w[0].0, w[0].1, w[1].0, w[1].1);
    }
}

/// Draw bundle brackets on one side (v6 `drawBracketList`).
pub fn draw_bracket_list(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    brackets: &[BracketDrawSlot],
    side: Side,
    x0: f64,
    _width_px: f64,
) {
    let dir = match side {
        Side::Right => 1.0,
        Side::Left => -1.0,
    };
    let label_style = text_style_for_role(TextRole::BundleCount);

    for bracket in brackets {
        let BracketDrawSlot { y0, y1, count } = *bracket;
        let r = bundle_bracket_fillet_radius_px(y0, y1);
        let spine_x = x0 + dir * BUNDLE_STUB_PX;
        let arrow_x = x0 + dir * (BUNDLE_STUB_PX / 2.0);
        let handle_x = spine_x + dir * r;
        let bot_end_x = spine_x - dir * r;
        let spine_top_y = y0 + r;
        let spine_bot_y = y1 - r;

        if spine_bot_y > spine_top_y {
            push_line(scene, nx, ny, spine_x, spine_top_y, spine_x, spine_bot_y);
        }

        push_fillet_arc(scene, nx, ny, side, true, spine_x, y0, r);
        push_line(scene, nx, ny, x0, y0, handle_x, y0);

        push_fillet_arc(scene, nx, ny, side, false, spine_x, y1, r);
        push_line(scene, nx, ny, x0, y1, bot_end_x, y1);

        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y0,
            arrow_x,
            y0 - BUNDLE_ARROW_STEM_PX,
        );
        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y0,
            arrow_x - BUNDLE_ARROW_LEG_PX,
            y0 - BUNDLE_ARROW_LEG_PX,
        );
        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y0,
            arrow_x + BUNDLE_ARROW_LEG_PX,
            y0 - BUNDLE_ARROW_LEG_PX,
        );
        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y1,
            arrow_x,
            y1 + BUNDLE_ARROW_STEM_PX,
        );
        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y1,
            arrow_x - BUNDLE_ARROW_LEG_PX,
            y1 + BUNDLE_ARROW_LEG_PX,
        );
        push_line(
            scene,
            nx,
            ny,
            arrow_x,
            y1,
            arrow_x + BUNDLE_ARROW_LEG_PX,
            y1 + BUNDLE_ARROW_LEG_PX,
        );

        if count != 2 {
            scene.primitives.push(ScenePrimitive::Text(SceneText {
                position: local_to_diagram(nx, ny, arrow_x, (y0 + y1) / 2.0),
                content: count.to_string(),
                height_px: label_style.height_px,
                halign: HAlign::Center,
                valign: VAlign::Middle,
                font: label_style.font.to_string(),
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagramme_geometry::{BUNDLE_STUB_PX, Side};

    #[test]
    fn right_bracket_fillet_endpoints_touch_spine_and_horizontals() {
        let r = 4.0;
        let x0 = 126.0;
        let y0 = 30.0;
        let y1 = 66.0;
        let spine_x = x0 + BUNDLE_STUB_PX;

        let (top_center, top_a0, top_sweep) = bracket_fillet_arc(Side::Right, true, spine_x, y0, r);
        let top_pts = tessellate_arc(top_center, r, top_a0, top_sweep, 10);
        assert!((top_pts[0].0 - spine_x).abs() < 1e-6);
        assert!((top_pts[0].1 - (y0 + r)).abs() < 1e-6);
        assert!((top_pts.last().unwrap().0 - (spine_x + r)).abs() < 1e-6);
        assert!((top_pts.last().unwrap().1 - y0).abs() < 1e-6);

        let (bot_center, bot_a0, bot_sweep) = bracket_fillet_arc(Side::Right, false, spine_x, y1, r);
        let bot_pts = tessellate_arc(bot_center, r, bot_a0, bot_sweep, 10);
        assert!((bot_pts[0].0 - spine_x).abs() < 1e-6);
        assert!((bot_pts[0].1 - (y1 - r)).abs() < 1e-6);
        assert!((bot_pts.last().unwrap().0 - (spine_x - r)).abs() < 1e-6);
        assert!((bot_pts.last().unwrap().1 - y1).abs() < 1e-6);
    }

    #[test]
    fn left_bracket_fillet_endpoints_touch_spine_and_horizontals() {
        let r = 4.0;
        let x0 = 126.0;
        let y0 = 30.0;
        let y1 = 66.0;
        let spine_x = x0 - BUNDLE_STUB_PX;

        let (top_center, top_a0, top_sweep) = bracket_fillet_arc(Side::Left, true, spine_x, y0, r);
        let top_pts = tessellate_arc(top_center, r, top_a0, top_sweep, 10);
        assert!((top_pts[0].0 - spine_x).abs() < 1e-6);
        assert!((top_pts[0].1 - (y0 + r)).abs() < 1e-6);
        assert!((top_pts.last().unwrap().0 - (spine_x - r)).abs() < 1e-6);
        assert!((top_pts.last().unwrap().1 - y0).abs() < 1e-6);

        let (bot_center, bot_a0, bot_sweep) = bracket_fillet_arc(Side::Left, false, spine_x, y1, r);
        let bot_pts = tessellate_arc(bot_center, r, bot_a0, bot_sweep, 10);
        assert!((bot_pts[0].0 - spine_x).abs() < 1e-6);
        assert!((bot_pts[0].1 - (y1 - r)).abs() < 1e-6);
        assert!((bot_pts.last().unwrap().0 - (spine_x + r)).abs() < 1e-6);
        assert!((bot_pts.last().unwrap().1 - y1).abs() < 1e-6);
    }
}
