//! Device v2 scene primitives — geometry ported from v6 `appendDeviceV2RevitDxf`.

use diagramme_geometry::{
    bundled_bracket_slots, device_v2_body_grid_row_count, device_v2_normalized_columns,
    flatten_device_v2_body_rows, text_style_for_role, DeviceV2BodySlot, PointPx, RectPx, Side,
    TextHAlign, TextRole, TextVAlign, DEVICE_CONNECTOR_COLUMN_PX, DEVICE_CONNECTOR_GUTTER_PX,
    DEVICE_V2_GRID_ROW_PX, DEVICE_V2_TITLE_HEIGHT_PX, DEVICE_V2_WIDTH_PX, SCHEMATIC_FRAME_INSET_PX,
    SCHEMATIC_TAG_BAND_PX, SCHEMATIC_TAG_TEXT_CENTER_Y_PX, SCHEMATIC_TITLE_SIDE_PADDING_PX,
    schematic_body_row_center_y, schematic_title_line_step_px, schematic_wrapped_title_line_center_y,
    wrap_schematic_title_lines,
};
use diagramme_schema::is_device_v2_bundle_bracket_active;
use diagramme_schema::Node;

use crate::breakline::{
    push_closed_inset_frame, push_closed_inset_frame_with_bottom_breakline,
};
use crate::bundle_brackets::{draw_bracket_list, BracketDrawSlot};
use crate::scene::{
    HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign,
};
use crate::text::sanitize_text;

const DEFAULT_LAYER: &str = "0";
const FILLS_LAYER: &str = "FILLS";
const HAIRLINE_STROKE_PX: f64 = 1.0;
const CONNECTOR_TEXT_SIDE_PAD_PX: f64 = 4.0;

fn device_v2_right_column_x_px() -> f64 {
    DEVICE_CONNECTOR_COLUMN_PX + DEVICE_CONNECTOR_GUTTER_PX
}

fn to_halign(align: TextHAlign) -> HAlign {
    match align {
        TextHAlign::Left => HAlign::Left,
        TextHAlign::Center => HAlign::Center,
        TextHAlign::Right => HAlign::Right,
    }
}

fn to_valign(align: TextVAlign) -> VAlign {
    match align {
        TextVAlign::Top => VAlign::Top,
        TextVAlign::Middle => VAlign::Middle,
        TextVAlign::Bottom => VAlign::Bottom,
    }
}

fn scene_text_from_role(
    role: TextRole,
    position: PointPx,
    content: String,
    halign_override: Option<TextHAlign>,
) -> SceneText {
    let style = text_style_for_role(role);
    SceneText {
        position,
        content,
        height_px: style.height_px,
        halign: to_halign(halign_override.unwrap_or(style.halign)),
        valign: to_valign(style.valign),
        font: style.font.to_string(),
        owner_node_id: None,
    }
}

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

fn push_solid(scene: &mut Scene, vertices: [PointPx; 4], node_id: &str) {
    scene.primitives.push(ScenePrimitive::Solid {
        vertices,
        layer: FILLS_LAYER.to_string(),
        node_id: Some(node_id.to_string()),
    });
}

fn push_line(scene: &mut Scene, x0: f64, y0: f64, x1: f64, y1: f64, nx: f64, ny: f64) {
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, x0, y0),
            local_to_diagram(nx, ny, x1, y1),
        ],
        false,
    );
}

fn gutter_edge_segments(
    rows: &[DeviceV2BodySlot],
    body_top: f64,
    row_px: f64,
) -> Vec<(f64, f64)> {
    let mut segs = Vec::new();
    let mut start: Option<usize> = None;
    for (i, row) in rows.iter().enumerate() {
        if matches!(row, DeviceV2BodySlot::Gap) {
            if let Some(s) = start {
                segs.push((body_top + s as f64 * row_px, body_top + i as f64 * row_px));
                start = None;
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        segs.push((
            body_top + s as f64 * row_px,
            body_top + rows.len() as f64 * row_px,
        ));
    }
    segs
}

fn cell_label(slot: Option<&DeviceV2BodySlot>) -> String {
    match slot {
        None => String::new(),
        Some(DeviceV2BodySlot::Gap) | Some(DeviceV2BodySlot::Bundled { .. }) => String::new(),
        Some(DeviceV2BodySlot::Header { label }) => label.clone(),
        Some(DeviceV2BodySlot::Condensed { .. }) => "...".to_string(),
        Some(DeviceV2BodySlot::Port { row, .. }) => row.label.clone(),
    }
}

fn device_v2_total_height_px(data: &serde_json::Value) -> f64 {
    let row_count = device_v2_body_grid_row_count(data);
    DEVICE_V2_TITLE_HEIGHT_PX + row_count as f64 * DEVICE_V2_GRID_ROW_PX
}

/// Scene bounds including tag band above the frame (diagram px).
pub fn device_v2_scene_bounds(node: &Node) -> RectPx {
    let total_height = device_v2_total_height_px(&node.data);
    RectPx::new(
        node.position.x,
        node.position.y - SCHEMATIC_TAG_BAND_PX,
        DEVICE_V2_WIDTH_PX,
        total_height + SCHEMATIC_TAG_BAND_PX,
    )
}

/// Append device v2 drawable primitives and hit targets to `scene`.
pub fn append_device_v2_scene(
    scene: &mut Scene,
    node: &Node,
    active_bundles: &std::collections::HashSet<(String, String)>,
    filter_bundle_brackets: bool,
) {
    let nx = node.position.x;
    let ny = node.position.y;
    let data = &node.data;

    let (left_groups, right_groups) = device_v2_normalized_columns(data);
    let left_rows = flatten_device_v2_body_rows(&left_groups);
    let right_rows = flatten_device_v2_body_rows(&right_groups);
    let row_count = left_rows.len().max(right_rows.len());
    let body_height = row_count as f64 * DEVICE_V2_GRID_ROW_PX;
    let total_height = DEVICE_V2_TITLE_HEIGHT_PX + body_height;

    let w = DEVICE_V2_WIDTH_PX;
    let title_h = DEVICE_V2_TITLE_HEIGHT_PX;
    let row_px = DEVICE_V2_GRID_ROW_PX;
    let col_w = DEVICE_CONNECTOR_COLUMN_PX;
    let right_col_x = device_v2_right_column_x_px();
    let inset = SCHEMATIC_FRAME_INSET_PX;
    let body_top = title_h;

    // Tag text (above frame)
    let tag_code = data
        .get("tagCode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let tag_number = data
        .get("tagNumber")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let tag_str = sanitize_text(&format!("{tag_code} / {tag_number}").trim(), 64);
    let tag_style = text_style_for_role(TextRole::Tag);
    scene.primitives.push(ScenePrimitive::Text(SceneText {
        position: local_to_diagram(nx, ny, w / 2.0, SCHEMATIC_TAG_TEXT_CENTER_Y_PX),
        content: if tag_str.is_empty() {
            " ".to_string()
        } else {
            tag_str
        },
        height_px: tag_style.height_px,
        halign: to_halign(tag_style.halign),
        valign: to_valign(tag_style.valign),
        font: tag_style.font.to_string(),
        owner_node_id: None,
    }));

    // Frame (closed inset rectangle; breakline when split instance)
    let split_instance = data.get("splitInstance");
    if split_instance.is_some() {
        push_closed_inset_frame_with_bottom_breakline(
            scene,
            nx,
            ny,
            w,
            total_height,
            inset,
            DEVICE_CONNECTOR_GUTTER_PX,
        );
    } else {
        push_closed_inset_frame(scene, nx, ny, w, total_height, inset);
    }

    // Title / body divider
    push_line(scene, inset, title_h, w - inset, title_h, nx, ny);

    // Gutter vertical lines
    let gutter_max_y = total_height - inset;
    for (y1, y2) in gutter_edge_segments(&left_rows, body_top, row_px) {
        push_line(
            scene,
            col_w,
            y1,
            col_w,
            y2.min(gutter_max_y),
            nx,
            ny,
        );
    }
    for (y1, y2) in gutter_edge_segments(&right_rows, body_top, row_px) {
        push_line(
            scene,
            right_col_x,
            y1,
            right_col_x,
            y2.min(gutter_max_y),
            nx,
            ny,
        );
    }

    // Horizontal row dividers
    for i in 0..left_rows.len().saturating_sub(1) {
        let yi = body_top + (i + 1) as f64 * row_px;
        push_line(scene, inset, yi, col_w, yi, nx, ny);
    }
    for i in 0..right_rows.len().saturating_sub(1) {
        let yi = body_top + (i + 1) as f64 * row_px;
        push_line(scene, right_col_x, yi, w - inset, yi, nx, ny);
    }

    // Title text
    let base_title = data
        .get("label")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            data.get("description")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("");
    let title_str = sanitize_text(base_title, 48);
    if !title_str.is_empty() {
        let title_style = text_style_for_role(TextRole::Title);
        let wrapped = wrap_schematic_title_lines(
            &[title_str],
            w,
            SCHEMATIC_TITLE_SIDE_PADDING_PX,
            title_style.height_px,
        );
        let line_count = wrapped.len().max(1);
        let line_step_px = schematic_title_line_step_px(title_style.height_px);
        for (i, line) in wrapped.iter().enumerate() {
            let t = sanitize_text(line.trim(), 48);
            if t.is_empty() {
                continue;
            }
            scene.primitives.push(ScenePrimitive::Text(scene_text_from_role(
                TextRole::Title,
                local_to_diagram(
                    nx,
                    ny,
                    w / 2.0,
                    schematic_wrapped_title_line_center_y(
                        0.0,
                        title_h,
                        i,
                        line_count,
                        line_step_px,
                    ),
                ),
                t,
                None,
            )));
        }
    }

    // Body rows: header fills + port labels
    let cell_style = text_style_for_role(TextRole::Cell);
    for row_index in 0..row_count {
        let row_top = body_top + row_index as f64 * row_px;
        let row_bottom = row_top + row_px;
        let row_cy = schematic_body_row_center_y(body_top, row_index, row_px);
        let left_row = left_rows.get(row_index);
        let right_row = right_rows.get(row_index);

        if matches!(left_row, Some(DeviceV2BodySlot::Header { .. })) {
            push_solid(
                scene,
                [
                    local_to_diagram(nx, ny, inset, row_top),
                    local_to_diagram(nx, ny, col_w, row_top),
                    local_to_diagram(nx, ny, col_w, row_bottom),
                    local_to_diagram(nx, ny, inset, row_bottom),
                ],
                &node.id,
            );
        }
        if matches!(right_row, Some(DeviceV2BodySlot::Header { .. })) {
            push_solid(
                scene,
                [
                    local_to_diagram(nx, ny, right_col_x, row_top),
                    local_to_diagram(nx, ny, w - inset, row_top),
                    local_to_diagram(nx, ny, w - inset, row_bottom),
                    local_to_diagram(nx, ny, right_col_x, row_bottom),
                ],
                &node.id,
            );
        }

        let left_label = sanitize_text(&cell_label(left_row), 20);
        if !left_label.is_empty() {
            let left_dir = match left_row {
                Some(DeviceV2BodySlot::Port { row, .. }) => row
                    .direction
                    .as_deref()
                    .unwrap_or("input"),
                _ => "input",
            };
            let left_align = if left_dir == "output" {
                TextHAlign::Right
            } else {
                TextHAlign::Left
            };
            let left_x = if left_align == TextHAlign::Right {
                col_w - CONNECTOR_TEXT_SIDE_PAD_PX
            } else {
                inset + CONNECTOR_TEXT_SIDE_PAD_PX
            };
            scene.primitives.push(ScenePrimitive::Text(SceneText {
                position: local_to_diagram(nx, ny, left_x, row_cy),
                content: left_label,
                height_px: cell_style.height_px,
                halign: to_halign(left_align),
                valign: to_valign(cell_style.valign),
                font: cell_style.font.to_string(),
                owner_node_id: None,
            }));
        }

        let right_label = sanitize_text(&cell_label(right_row), 20);
        if !right_label.is_empty() {
            let right_dir = match right_row {
                Some(DeviceV2BodySlot::Port { row, .. }) => row
                    .direction
                    .as_deref()
                    .unwrap_or("output"),
                _ => "output",
            };
            let right_align = if right_dir == "input" {
                TextHAlign::Left
            } else {
                TextHAlign::Right
            };
            let right_x = if right_align == TextHAlign::Left {
                right_col_x + CONNECTOR_TEXT_SIDE_PAD_PX
            } else {
                w - inset - CONNECTOR_TEXT_SIDE_PAD_PX
            };
            scene.primitives.push(ScenePrimitive::Text(SceneText {
                position: local_to_diagram(nx, ny, right_x, row_cy),
                content: right_label,
                height_px: cell_style.height_px,
                halign: to_halign(right_align),
                valign: to_valign(cell_style.valign),
                font: cell_style.font.to_string(),
                owner_node_id: None,
            }));
        }
    }

    let left_brackets: Vec<BracketDrawSlot> = bundled_bracket_slots(&left_rows, &left_groups, body_top, row_px)
        .into_iter()
        .filter(|s| {
            !filter_bundle_brackets
                || is_device_v2_bundle_bracket_active(
                    active_bundles,
                    &node.id,
                    'L',
                    s.group_index,
                    s.bundle_index,
                )
        })
        .map(|s| BracketDrawSlot {
            y0: s.y0,
            y1: s.y1,
            count: s.count,
        })
        .collect();
    let right_brackets: Vec<BracketDrawSlot> =
        bundled_bracket_slots(&right_rows, &right_groups, body_top, row_px)
            .into_iter()
            .filter(|s| {
                !filter_bundle_brackets
                    || is_device_v2_bundle_bracket_active(
                        active_bundles,
                        &node.id,
                        'R',
                        s.group_index,
                        s.bundle_index,
                    )
            })
            .map(|s| BracketDrawSlot {
                y0: s.y0,
                y1: s.y1,
                count: s.count,
            })
            .collect();
    draw_bracket_list(scene, nx, ny, &left_brackets, Side::Left, 0.0, w);
    draw_bracket_list(scene, nx, ny, &right_brackets, Side::Right, w, w);

    // Hit target: node body (full frame width)
    scene.hits.push(HitTarget {
        id: node.id.clone(),
        bounds: RectPx::new(nx, ny, w, total_height),
        node_id: Some(node.id.clone()),
        edge_id: None,
        handle_id: None,
    });

    // Port hit targets (one per port row per side)
    for (side, rows) in [("left", &left_rows), ("right", &right_rows)] {
        let side_char = if side == "left" { 'L' } else { 'R' };
        for (row_index, slot) in rows.iter().enumerate() {
            if let DeviceV2BodySlot::Port { row, group_index, .. } = slot {
                let row_top = body_top + row_index as f64 * row_px;
                let (x, width) = if side == "left" {
                    (nx, col_w)
                } else {
                    (nx + right_col_x, w - right_col_x)
                };
                let handle_id = format!("{side_char}-{group_index}-{}", row.id);
                scene.hits.push(HitTarget {
                    id: format!("{}:{}", node.id, handle_id),
                    bounds: RectPx::new(x, ny + row_top, width, row_px),
                    node_id: Some(node.id.clone()),
                    edge_id: None,
                    handle_id: Some(handle_id),
                });
            }
        }
    }
}
