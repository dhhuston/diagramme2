//! Antenna symbol scene primitives — ported from v6 `appendAntennaSymbolRevitDxf`.

use diagramme_geometry::{
    antenna_content_row_width_px, PointPx, RectPx, TextRole, ANT_ARM_HALF_PX, ANT_FOOT_LEN_PX,
    ANT_RX_MAST_X, ANT_TX_MAST_X, ANTENNA_CONNECTOR_BAND_H_PX, ANTENNA_SCHEMATIC_SVG_WIDTH_PX,
    CONNECTOR_ROW_OUTER_HEIGHT_PX,
};
use diagramme_schema::Node;

use crate::nodes::emit::{push_line, push_node_body_hit, push_text, scene_text_from_role, node_width};
use crate::scene::Scene;
use crate::text::sanitize_text;

/// Scene bounds (diagram px).
pub fn antenna_scene_bounds(node: &Node) -> RectPx {
    let line1 = node
        .data
        .get("line1")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ANT");
    let row_w = antenna_content_row_width_px(line1, f64::INFINITY)
        .max(ANTENNA_SCHEMATIC_SVG_WIDTH_PX + 20.0);
    let w = node_width(node, row_w);
    RectPx::new(node.position.x, node.position.y, w, 20.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntennaRole {
    Transmitter,
    Receiver,
}

impl AntennaRole {
    fn from_node_type(node_type: &str) -> Option<Self> {
        match node_type {
            "antennaTransmitterSymbol" => Some(Self::Transmitter),
            "antennaReceiverSymbol" => Some(Self::Receiver),
            _ => None,
        }
    }
}

/// Append antenna symbol drawable primitives to `scene`.
pub fn append_antenna_scene(scene: &mut Scene, node: &Node) {
    let role = AntennaRole::from_node_type(&node.node_type)
        .expect("append_antenna_scene requires antenna node type");
    append_antenna_symbol_scene(scene, node, role);
}

/// Append transmitter or receiver antenna drawable primitives to `scene`.
pub fn append_antenna_symbol_scene(scene: &mut Scene, node: &Node, role: AntennaRole) {
    let data = &node.data;
    let nx = node.position.x;
    let ny = node.position.y;
    let line1 = data
        .get("line1")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("ANT");
    let row_w = antenna_content_row_width_px(line1, f64::INFINITY)
        .max(ANTENNA_SCHEMATIC_SVG_WIDTH_PX + 20.0);
    let w = node_width(node, row_w);

    let l_leg_right = role == AntennaRole::Receiver;
    let mx = if l_leg_right { ANT_RX_MAST_X } else { ANT_TX_MAST_X };
    let foot_x2 = if l_leg_right {
        mx + ANT_FOOT_LEN_PX
    } else {
        mx - ANT_FOOT_LEN_PX
    };

    let r = CONNECTOR_ROW_OUTER_HEIGHT_PX;
    let y_top = 0.0;
    let y_bot = r + r / 2.0;
    let y_junc = y_top + ANT_ARM_HALF_PX;

    let sym_left = if role == AntennaRole::Receiver {
        nx + w - ANTENNA_SCHEMATIC_SVG_WIDTH_PX
    } else {
        nx
    };
    let row_top = ny + (20.0 - ANTENNA_CONNECTOR_BAND_H_PX) / 2.0;

    push_line(scene, mx, y_top, mx, y_bot, sym_left, row_top);
    push_line(scene, mx, y_bot, foot_x2, y_bot, sym_left, row_top);
    push_line(scene, mx - ANT_ARM_HALF_PX, y_top, mx, y_junc, sym_left, row_top);
    push_line(scene, mx + ANT_ARM_HALF_PX, y_top, mx, y_junc, sym_left, row_top);

    push_text(
        scene,
        scene_text_from_role(
            TextRole::Antenna,
            PointPx {
                x: nx + w / 2.0,
                y: ny + 10.0,
            },
            sanitize_text(line1, 48),
            None,
            None,
        ),
    );

    push_node_body_hit(scene, node, antenna_scene_bounds(node));
}
