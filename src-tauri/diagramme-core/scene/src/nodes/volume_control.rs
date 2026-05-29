//! Volume control scene primitives — ported from v6 `appendVolumeControlRevitDxf`.

use diagramme_geometry::{
    PointPx, RectPx, TextRole, PX_PER_INCH, VC_SCHEMATIC_SVG_WIDTH_PX, VC_SYMBOL_TOP_INSET_PX,
    VOLUME_CONTROL_FRAME_HEIGHT_PX, VOLUME_CONTROL_HEX_VERTEX_SPAN_PX,
};
use diagramme_schema::Node;

use crate::nodes::emit::{push_polyline, push_text, scene_text_from_role, DEFAULT_LAYER};
use crate::scene::Scene;
use crate::text::sanitize_text;

/// Scene bounds (diagram px) — matches v6 export extents (symbol cluster).
pub fn volume_control_scene_bounds(node: &Node) -> RectPx {
    RectPx::new(
        node.position.x,
        node.position.y + VC_SYMBOL_TOP_INSET_PX,
        PX_PER_INCH,
        VOLUME_CONTROL_FRAME_HEIGHT_PX,
    )
}

/// Append volume control drawable primitives to `scene`.
pub fn append_volume_control_scene(scene: &mut Scene, node: &Node) {
    let nx = node.position.x;
    let ny = node.position.y;
    let row_top = ny + VC_SYMBOL_TOP_INSET_PX;
    let anchor_left = nx + (PX_PER_INCH - VC_SCHEMATIC_SVG_WIDTH_PX) / 2.0;
    let cx = anchor_left + VC_SCHEMATIC_SVG_WIDTH_PX * 0.5;
    let cy = row_top + VOLUME_CONTROL_FRAME_HEIGHT_PX / 2.0;
    let r = VOLUME_CONTROL_HEX_VERTEX_SPAN_PX / 2.0;

    let mut pts = Vec::with_capacity(6);
    for k in 0..6 {
        let a = (k as f64) * std::f64::consts::PI / 3.0 - std::f64::consts::PI / 2.0;
        pts.push(PointPx {
            x: cx + r * a.cos(),
            y: cy + r * a.sin(),
        });
    }
    push_polyline(scene, pts, true, DEFAULT_LAYER);

    push_text(
        scene,
        scene_text_from_role(
            TextRole::VolumeControl,
            PointPx { x: cx, y: cy },
            sanitize_text("VC", 8),
            None,
            None,
        ),
    );
}
