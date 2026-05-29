//! Obstacle collection for schematic wire routing (mirrors v6 `schematicWireObstacles.ts`).

use diagramme_geometry::{node_bounds_diagram_px, schematic_layout::SCHEMATIC_TAG_BAND_PX, SNAP_GRID_PX};
use diagramme_schema::Node;
use std::collections::HashMap;

use crate::sharp_polyline::snap_coord;
use crate::types::WireObstacleBox;

/// Clearance between wire segments and obstacle boxes (one snap cell).
pub const WIRE_OBSTACLE_CLEARANCE_PX: f64 = SNAP_GRID_PX;

const GROUPING_ZONE_TYPE: &str = "groupingZone";

/// Node types whose schematic bounds include a tag band above the frame.
const TAG_BAND_NODE_TYPES: &[&str] = &[
    "device",
    "deviceV2",
    "avPlate",
    "lppPatchPanel",
    "dppPatchPanel",
    "mlpPatchPanel",
    "vpbPatchPanel",
    "junction",
];

fn snap_box(x: f64, y: f64, x2: f64, y2: f64) -> (f64, f64, f64, f64) {
    let lo_x = x.min(x2);
    let hi_x = x.max(x2);
    let lo_y = y.min(y2);
    let hi_y = y.max(y2);
    (
        snap_coord(lo_x),
        snap_coord(lo_y),
        snap_coord(hi_x),
        snap_coord(hi_y),
    )
}

fn inflate_box(box_: WireObstacleBox, margin: f64) -> WireObstacleBox {
    let (x, y, x2, y2) = snap_box(box_.x - margin, box_.y - margin, box_.x2 + margin, box_.y2 + margin);
    WireObstacleBox {
        id: box_.id,
        x,
        y,
        x2,
        y2,
    }
}

fn obstacle_from_node(node: &Node, margin: f64) -> Option<WireObstacleBox> {
    if node.node_type == GROUPING_ZONE_TYPE {
        return None;
    }

    let mut bounds = node_bounds_diagram_px(node)?;
    if TAG_BAND_NODE_TYPES.contains(&node.node_type.as_str()) {
        bounds.y -= SCHEMATIC_TAG_BAND_PX;
    }

    if bounds.width < 1e-3 && bounds.height < 1e-3 {
        return None;
    }

    let (x, y, x2, y2) = snap_box(bounds.x, bounds.y, bounds.x + bounds.width, bounds.y + bounds.height);
    Some(inflate_box(
        WireObstacleBox {
            id: node.id.clone(),
            x,
            y,
            x2,
            y2,
        },
        margin,
    ))
}

/// Obstacle rectangles for wire routing: all nodes except grouping zones and this edge's endpoints.
pub fn collect_wire_obstacles(
    nodes_by_id: &HashMap<String, Node>,
    source_id: &str,
    target_id: &str,
    margin: f64,
) -> Vec<WireObstacleBox> {
    let mut out = Vec::new();
    for node in nodes_by_id.values() {
        if node.id == source_id || node.id == target_id {
            continue;
        }
        if let Some(box_) = obstacle_from_node(node, margin) {
            out.push(box_);
        }
    }
    out
}

/// Obstacles whose boxes overlap a wire axis-aligned bounding box (cheap cull for live drag).
pub fn obstacles_near_wire_aabb(
    obstacles: &[WireObstacleBox],
    x_lo: f64,
    y_lo: f64,
    x_hi: f64,
    y_hi: f64,
) -> Vec<WireObstacleBox> {
    obstacles
        .iter()
        .cloned()
        .filter(|ob| {
            let ox0 = ob.x.min(ob.x2);
            let ox1 = ob.x.max(ob.x2);
            let oy0 = ob.y.min(ob.y2);
            let oy1 = ob.y.max(ob.y2);
            x_hi >= ox0 && x_lo <= ox1 && y_hi >= oy0 && y_lo <= oy1
        })
        .collect()
}
