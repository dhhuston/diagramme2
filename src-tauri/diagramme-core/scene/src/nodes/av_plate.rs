//! AV plate scene primitives — geometry ported from v6 `appendAvPlateRevitDxf`.

use diagramme_geometry::{
    av_plate_groups_from_data, flat_av_plate_bundle_slots, flatten_av_plate_body_rows,
    text_style_for_role, AvPlateBodyRow, PointPx, RectPx, Side, TextHAlign, TextRole, TextVAlign,
    AV_PLATE_GRID_ROW_PX, AV_PLATE_TITLE_HEIGHT_PX, DEVICE_CONNECTOR_GUTTER_PX, PATCH_PANEL_WIDTH_PX,
    SCHEMATIC_FRAME_INSET_PX, SCHEMATIC_TAG_BAND_PX, SCHEMATIC_TAG_TEXT_CENTER_Y_PX,
    SCHEMATIC_TITLE_SIDE_PADDING_PX, schematic_body_row_center_y, schematic_title_line_step_px,
    schematic_wrapped_title_line_center_y, wrap_schematic_title_lines,
};
use diagramme_schema::{filter_bundled_side, Node};

use crate::breakline::{
    push_closed_inset_frame, push_closed_inset_frame_with_bottom_breakline,
};
use crate::bundle_brackets::{draw_bracket_list, BracketDrawSlot};
use crate::scene::{HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign};
use crate::text::sanitize_text;

const DEFAULT_LAYER: &str = "0";
const FILLS_LAYER: &str = "FILLS";
const HAIRLINE_STROKE_PX: f64 = 1.0;
const CONNECTOR_TEXT_SIDE_PAD_PX: f64 = 4.0;

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

fn with_split_suffix(base: &str, split_instance: Option<u64>) -> String {
    match split_instance {
        Some(n) => format!("{base} ({n})"),
        None => base.to_string(),
    }
}

fn parse_bundled_row_ids(data: &serde_json::Value, key: &str) -> Option<Vec<Vec<String>>> {
    let arr = data.get(key)?.as_array()?;
    Some(
        arr.iter()
            .filter_map(|bundle| {
                bundle.as_array().map(|ids| {
                    ids.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
            })
            .collect(),
    )
}

fn av_plate_total_height_px(data: &serde_json::Value) -> f64 {
    let groups = av_plate_groups_from_data(data);
    let rows = flatten_av_plate_body_rows(&groups);
    AV_PLATE_TITLE_HEIGHT_PX + rows.len() as f64 * AV_PLATE_GRID_ROW_PX
}

/// Scene bounds including tag band above the frame (diagram px).
pub fn av_plate_scene_bounds(node: &Node) -> RectPx {
    let total_height = av_plate_total_height_px(&node.data);
    RectPx::new(
        node.position.x,
        node.position.y - SCHEMATIC_TAG_BAND_PX,
        PATCH_PANEL_WIDTH_PX,
        total_height + SCHEMATIC_TAG_BAND_PX,
    )
}

/// Append AV plate drawable primitives and hit targets to `scene`.
pub fn append_av_plate_scene(
    scene: &mut Scene,
    node: &Node,
    active_bundles: &std::collections::HashSet<(String, String)>,
    filter_bundle_brackets: bool,
) {
    let nx = node.position.x;
    let ny = node.position.y;
    let data = &node.data;

    let groups = av_plate_groups_from_data(data);
    let rows = flatten_av_plate_body_rows(&groups);
    let body_height = rows.len() as f64 * AV_PLATE_GRID_ROW_PX;
    let total_height = AV_PLATE_TITLE_HEIGHT_PX + body_height;

    let w = PATCH_PANEL_WIDTH_PX;
    let title_h = AV_PLATE_TITLE_HEIGHT_PX;
    let row_px = AV_PLATE_GRID_ROW_PX;
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
    let split_instance = data.get("splitInstance").and_then(|v| v.as_u64());
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

    // Horizontal row dividers
    for i in 0..rows.len().saturating_sub(1) {
        let yi = body_top + (i + 1) as f64 * row_px;
        push_line(scene, inset, yi, w - inset, yi, nx, ny);
    }

    // Title text
    let base_title = data
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    let title_str = sanitize_text(&with_split_suffix(base_title, split_instance), 48);
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
            scene.primitives.push(ScenePrimitive::Text(SceneText {
                position: local_to_diagram(
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
                content: t,
                height_px: title_style.height_px,
                halign: to_halign(title_style.halign),
                valign: to_valign(title_style.valign),
                font: title_style.font.to_string(),
                owner_node_id: None,
            }));
        }
    }

    // Body rows: header fills + port labels
    let cell_style = text_style_for_role(TextRole::Cell);
    for (row_index, row) in rows.iter().enumerate() {
        if matches!(row, AvPlateBodyRow::Gap) {
            continue;
        }
        let row_top = body_top + row_index as f64 * row_px;
        let row_bottom = row_top + row_px;
        let row_cy = schematic_body_row_center_y(body_top, row_index, row_px);

        if matches!(row, AvPlateBodyRow::Header { .. }) {
            push_solid(
                scene,
                [
                    local_to_diagram(nx, ny, inset, row_top),
                    local_to_diagram(nx, ny, w - inset, row_top),
                    local_to_diagram(nx, ny, w - inset, row_bottom),
                    local_to_diagram(nx, ny, inset, row_bottom),
                ],
                &node.id,
            );
        }

        let label = match row {
            AvPlateBodyRow::Header { label } | AvPlateBodyRow::Port { label, .. } => {
                sanitize_text(label, 32)
            }
            AvPlateBodyRow::Gap => continue,
        };
        if label.is_empty() {
            continue;
        }

        match row {
            AvPlateBodyRow::Port { direction, .. } => {
                let is_output = direction.as_deref() == Some("output");
                let halign = if is_output {
                    TextHAlign::Right
                } else {
                    TextHAlign::Left
                };
                let tx = if is_output {
                    w - inset - CONNECTOR_TEXT_SIDE_PAD_PX
                } else {
                    inset + CONNECTOR_TEXT_SIDE_PAD_PX
                };
                scene.primitives.push(ScenePrimitive::Text(SceneText {
                    position: local_to_diagram(nx, ny, tx, row_cy),
                    content: label,
                    height_px: cell_style.height_px,
                    halign: to_halign(halign),
                    valign: to_valign(cell_style.valign),
                    font: cell_style.font.to_string(),
                owner_node_id: None,
                }));
            }
            AvPlateBodyRow::Header { .. } => {
                scene.primitives.push(ScenePrimitive::Text(SceneText {
                    position: local_to_diagram(nx, ny, w / 2.0, row_cy),
                    content: label,
                    height_px: cell_style.height_px,
                    halign: HAlign::Center,
                    valign: to_valign(cell_style.valign),
                    font: cell_style.font.to_string(),
                owner_node_id: None,
                }));
            }
            AvPlateBodyRow::Gap => {}
        }
    }

    let bundled_left = parse_bundled_row_ids(data, "bundledLeft");
    let bundled_right = parse_bundled_row_ids(data, "bundledRight");
    let bundled_left = if filter_bundle_brackets {
        filter_bundled_side(bundled_left, &node.id, 'L', active_bundles)
    } else {
        bundled_left
    };
    let bundled_right = if filter_bundle_brackets {
        filter_bundled_side(bundled_right, &node.id, 'R', active_bundles)
    } else {
        bundled_right
    };
    let row_center_px = row_px / 2.0;
    let slots = flat_av_plate_bundle_slots(
        &rows,
        bundled_left.as_deref(),
        bundled_right.as_deref(),
        body_top,
        row_px,
        row_center_px,
    );
    let left: Vec<BracketDrawSlot> = slots
        .iter()
        .filter(|s| s.side == Side::Left)
        .map(|s| BracketDrawSlot {
            y0: s.y0,
            y1: s.y1,
            count: s.count,
        })
        .collect();
    let right: Vec<BracketDrawSlot> = slots
        .iter()
        .filter(|s| s.side == Side::Right)
        .map(|s| BracketDrawSlot {
            y0: s.y0,
            y1: s.y1,
            count: s.count,
        })
        .collect();
    draw_bracket_list(scene, nx, ny, &left, Side::Left, 0.0, w);
    draw_bracket_list(scene, nx, ny, &right, Side::Right, w, w);

    // Hit target: body includes tag band; face mask is inset frame only.
    scene.hits.push(HitTarget {
        id: node.id.clone(),
        bounds: av_plate_scene_bounds(node),
        node_id: Some(node.id.clone()),
        edge_id: None,
        handle_id: None,
        face_mask_bounds: Some(RectPx::new(nx, ny, w, total_height)),
    });

    // Port hit targets — left (T) and right (S) halves per v6 handle ids
    for (row_index, row) in rows.iter().enumerate() {
        if let AvPlateBodyRow::Port {
            group_index,
            row_id,
            ..
        } = row
        {
            let row_top = body_top + row_index as f64 * row_px;
            let half_w = (w - 2.0 * inset) / 2.0;
            let t_handle = format!("T-{group_index}-{row_id}");
            let s_handle = format!("S-{group_index}-{row_id}");
            scene.hits.push(HitTarget {
                id: format!("{}:{}", node.id, t_handle),
                bounds: RectPx::new(nx + inset, ny + row_top, half_w, row_px),
                node_id: Some(node.id.clone()),
                edge_id: None,
                handle_id: Some(t_handle),
                face_mask_bounds: None,
            });
            scene.hits.push(HitTarget {
                id: format!("{}:{}", node.id, s_handle),
                bounds: RectPx::new(nx + inset + half_w, ny + row_top, half_w, row_px),
                node_id: Some(node.id.clone()),
                edge_id: None,
                handle_id: Some(s_handle),
                face_mask_bounds: None,
            });
        }
    }
}
