//! Patch panel scene primitives — geometry ported from v6 patch panel appenders.

use diagramme_geometry::{
    flat_patch_bundle_slots, text_style_for_role, PointPx, RectPx, Side, TextHAlign, TextRole,
    TextVAlign, BREAKLINE_DEVICE_ZONE_WIDTH_PX, PATCH_BODY_TOP_PX, PATCH_CIRCUIT_HEIGHT_PX,
    PATCH_PANEL_WIDTH_PX, PATCH_ROW_CENTER_Y_PX, PATCH_TITLE_HEIGHT_PX, SCHEMATIC_FRAME_INSET_PX,
    SCHEMATIC_TAG_BAND_PX, SCHEMATIC_TAG_TEXT_CENTER_Y_PX, SCHEMATIC_TITLE_SIDE_PADDING_PX,
    patch_panel_total_height_px, schematic_title_line_step_px, schematic_wrapped_title_line_center_y,
    wrap_schematic_title_lines,
};
use diagramme_schema::{filter_bundled_side, Node};

use crate::breakline::{
    inset_frame_face_mask_polygon, push_closed_inset_frame,
    push_closed_inset_frame_with_bottom_breakline,
};
use crate::bundle_brackets::{draw_bracket_list, BracketDrawSlot};
use crate::scene::{HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign};
use crate::text::sanitize_text;

const DEFAULT_LAYER: &str = "0";
const HAIRLINE_STROKE_PX: f64 = 1.0;

const DPP_ROW_NORM_W: f64 = 100.0;
const DPP_WING_INSET: f64 = 4.5;
const DPP_EDGE_GAP: f64 = (1.0 / 32.0) * 100.0;

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

fn append_split_instance_to_lines(lines: &[String], split_instance: Option<u64>) -> Vec<String> {
    match split_instance {
        Some(n) => {
            let mut out = lines.to_vec();
            if let Some(last) = out.last_mut() {
                *last = format!("{} ({n})", last.trim());
            }
            out
        }
        None => lines.to_vec(),
    }
}

fn patch_row_label_center_y(row_top_px: f64) -> f64 {
    row_top_px + PATCH_ROW_CENTER_Y_PX
}

fn map_norm_x(x_norm: f64) -> f64 {
    (x_norm / DPP_ROW_NORM_W) * PATCH_PANEL_WIDTH_PX
}

fn push_lpp_row_schematic(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    row_top_px: f64,
    has_left: bool,
    connected: bool,
) {
    let w_norm = 100.0;
    let rh = PATCH_CIRCUIT_HEIGHT_PX;
    let y_line = row_top_px + PATCH_ROW_CENTER_Y_PX;
    let left_apex = map_norm_x(w_norm / 3.0);
    let right_apex = map_norm_x(2.0 * w_norm / 3.0);
    let wing_inset = map_norm_x(DPP_WING_INSET);
    let left_inner = left_apex + wing_inset;
    let right_inner = right_apex - wing_inset;
    let xw = PATCH_PANEL_WIDTH_PX;
    let inset = SCHEMATIC_FRAME_INSET_PX;
    let x_inner_l = inset;
    let x_inner_r = xw - inset;

    if connected {
        push_line(scene, x_inner_l, y_line, x_inner_r, y_line, nx, ny);
    } else if has_left {
        push_line(scene, x_inner_l, y_line, left_apex, y_line, nx, ny);
        push_line(scene, right_apex, y_line, x_inner_r, y_line, nx, ny);
    } else {
        push_line(scene, right_apex, y_line, x_inner_r, y_line, nx, ny);
    }

    let wing_half = (4.5 / 16.0) * rh;
    let wing_top = y_line - wing_half;
    let wing_bot = y_line + wing_half;

    if has_left {
        push_polyline(
            scene,
            vec![
                local_to_diagram(nx, ny, left_inner, wing_top),
                local_to_diagram(nx, ny, left_apex, y_line),
                local_to_diagram(nx, ny, left_inner, wing_bot),
            ],
            false,
        );
    }
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, right_inner, wing_top),
            local_to_diagram(nx, ny, right_apex, y_line),
            local_to_diagram(nx, ny, right_inner, wing_bot),
        ],
        false,
    );
}

fn push_dpp_row_schematic(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    row_top_px: f64,
    direction: &str,
) {
    let is_output = direction == "output";
    let apex = map_norm_x(if is_output {
        DPP_ROW_NORM_W - DPP_EDGE_GAP
    } else {
        DPP_EDGE_GAP
    });
    let base = if is_output {
        apex - map_norm_x(DPP_WING_INSET)
    } else {
        apex + map_norm_x(DPP_WING_INSET)
    };
    let rh = PATCH_CIRCUIT_HEIGHT_PX;
    let cy = row_top_px + PATCH_ROW_CENTER_Y_PX;
    let xw = PATCH_PANEL_WIDTH_PX;
    let inset = SCHEMATIC_FRAME_INSET_PX;
    let x1 = if is_output { inset } else { apex };
    let x2 = if is_output { apex } else { xw - inset };
    let wing_half = (4.5 / 16.0) * rh;

    push_line(scene, x1, cy, x2, cy, nx, ny);
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, base, cy - wing_half),
            local_to_diagram(nx, ny, apex, cy),
            local_to_diagram(nx, ny, base, cy + wing_half),
        ],
        false,
    );
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

fn append_patch_panel_bundle_brackets(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    data: &serde_json::Value,
    row_ids: &[String],
    node_id: &str,
    active_bundles: &std::collections::HashSet<(String, String)>,
    filter_bundle_brackets: bool,
) {
    let bundled_left = parse_bundled_row_ids(data, "bundledLeft");
    let bundled_right = parse_bundled_row_ids(data, "bundledRight");
    let bundled_left = if filter_bundle_brackets {
        filter_bundled_side(bundled_left, node_id, 'L', active_bundles)
    } else {
        bundled_left
    };
    let bundled_right = if filter_bundle_brackets {
        filter_bundled_side(bundled_right, node_id, 'R', active_bundles)
    } else {
        bundled_right
    };
    let body_top = PATCH_BODY_TOP_PX;
    let slots = flat_patch_bundle_slots(
        row_ids,
        bundled_left.as_deref(),
        bundled_right.as_deref(),
        body_top,
        PATCH_CIRCUIT_HEIGHT_PX,
        PATCH_ROW_CENTER_Y_PX,
    );
    let w = PATCH_PANEL_WIDTH_PX;
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
}

/// Shared patch-panel chrome (tag, frame, title). Returns `f_top` (always 0).
pub(crate) fn push_patch_panel_frame(
    scene: &mut Scene,
    node: &Node,
    total_height: f64,
    tag_text: &str,
    title_lines: &[String],
    show_breakline: bool,
) -> f64 {
    let nx = node.position.x;
    let ny = node.position.y;
    let w = PATCH_PANEL_WIDTH_PX;
    let inset = SCHEMATIC_FRAME_INSET_PX;
    let f_top = 0.0;

    let tag_str = sanitize_text(tag_text, 64);
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

    if show_breakline {
        push_closed_inset_frame_with_bottom_breakline(
            scene,
            nx,
            ny,
            w,
            total_height,
            inset,
            BREAKLINE_DEVICE_ZONE_WIDTH_PX,
        );
    } else {
        push_closed_inset_frame(scene, nx, ny, w, total_height, inset);
    }
    push_line(
        scene,
        inset,
        f_top + PATCH_TITLE_HEIGHT_PX,
        w - inset,
        f_top + PATCH_TITLE_HEIGHT_PX,
        nx,
        ny,
    );

    let title_style = text_style_for_role(TextRole::Title);
    let wrapped = wrap_schematic_title_lines(
        title_lines,
        w,
        SCHEMATIC_TITLE_SIDE_PADDING_PX,
        title_style.height_px,
    );
    let line_count = wrapped.len().max(1);
    let line_step_px = schematic_title_line_step_px(title_style.height_px);

    for (i, line) in wrapped.iter().enumerate() {
        let t = sanitize_text(
            if line.trim().is_empty() {
                " "
            } else {
                line.trim()
            },
            48,
        );
        let cy = schematic_wrapped_title_line_center_y(
            f_top,
            PATCH_TITLE_HEIGHT_PX,
            i,
            line_count,
            line_step_px,
        );
        scene.primitives.push(ScenePrimitive::Text(SceneText {
            position: local_to_diagram(nx, ny, w / 2.0, cy),
            content: t,
            height_px: title_style.height_px,
            halign: to_halign(title_style.halign),
            valign: to_valign(title_style.valign),
            font: title_style.font.to_string(),
            owner_node_id: None,
        }));
    }
    f_top
}

fn push_row_label_text(
    scene: &mut Scene,
    nx: f64,
    ny: f64,
    x: f64,
    y: f64,
    content: String,
    halign: HAlign,
) {
    let style = text_style_for_role(TextRole::RowLabel);
    scene.primitives.push(ScenePrimitive::Text(SceneText {
        position: local_to_diagram(nx, ny, x, y),
        content,
        height_px: style.height_px,
        halign,
        valign: VAlign::Bottom,
        font: style.font.to_string(),
        owner_node_id: None,
    }));
}

fn description_lines_from_data(data: &serde_json::Value, default: &[&str]) -> Vec<String> {
    data.get("descriptionLines")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::trim).filter(|s| !s.is_empty()))
                .map(|s| s.to_string())
                .collect()
        })
        .filter(|lines: &Vec<String>| !lines.is_empty())
        .unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect())
}

fn patch_panel_row_count(data: &serde_json::Value) -> usize {
    data.get("rows")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Scene bounds including tag band above the frame (diagram px).
pub fn patch_panel_scene_bounds(node: &Node) -> RectPx {
    let row_count = patch_panel_row_count(&node.data);
    let total_height = patch_panel_total_height_px(row_count);
    RectPx::new(
        node.position.x,
        node.position.y - SCHEMATIC_TAG_BAND_PX,
        PATCH_PANEL_WIDTH_PX,
        total_height + SCHEMATIC_TAG_BAND_PX,
    )
}

/// Append patch panel drawable primitives and hit targets to `scene`.
///
/// Handles `lppPatchPanel`, `dppPatchPanel`, `mlpPatchPanel`, and `vpbPatchPanel`.
pub fn append_patch_panel_scene(
    scene: &mut Scene,
    node: &Node,
    active_bundles: &std::collections::HashSet<(String, String)>,
    filter_bundle_brackets: bool,
) {
    let nx = node.position.x;
    let ny = node.position.y;
    let data = &node.data;
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let row_count = rows.len();
    let total_height = patch_panel_total_height_px(row_count);

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
    let tag = format!("{tag_code} / {tag_number}").trim().to_string();
    let split_instance = data.get("splitInstance").and_then(|v| v.as_u64());

    let default_lines: &[&str] = match node.node_type.as_str() {
        "dppPatchPanel" => &["Data patch panel"],
        "mlpPatchPanel" => &["Mic/line", "Patch panel"],
        "vpbPatchPanel" => &["Video", "Patch panel"],
        _ => &["Loudspeaker patch panel"],
    };
    let header_lines = description_lines_from_data(data, default_lines);
    let display_lines = append_split_instance_to_lines(&header_lines, split_instance);

    push_patch_panel_frame(
        scene,
        node,
        total_height,
        &tag,
        &display_lines,
        split_instance.is_some(),
    );

    let row_ids: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            row.get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .collect();
    append_patch_panel_bundle_brackets(
        scene,
        nx,
        ny,
        data,
        &row_ids,
        &node.id,
        active_bundles,
        filter_bundle_brackets,
    );

    let row_label_style = text_style_for_role(TextRole::RowLabel);
    let w = PATCH_PANEL_WIDTH_PX;
    let inset = SCHEMATIC_FRAME_INSET_PX;

    for (i, row) in rows.iter().enumerate() {
        let row_top = PATCH_BODY_TOP_PX + i as f64 * PATCH_CIRCUIT_HEIGHT_PX;
        let row_label_cy = patch_row_label_center_y(row_top);

        match node.node_type.as_str() {
            "dppPatchPanel" => {
                let direction = row
                    .get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("input");
                push_dpp_row_schematic(scene, nx, ny, row_top, direction);
                let lab = sanitize_text(
                    row.get("label").and_then(|v| v.as_str()).unwrap_or(""),
                    24,
                );
                if !lab.is_empty() {
                    if direction == "output" {
                        push_row_label_text(scene, nx, ny, 4.0, row_label_cy, lab, HAlign::Left);
                    } else {
                        push_row_label_text(scene, nx, ny, w - 4.0, row_label_cy, lab, HAlign::Right);
                    }
                }
            }
            "mlpPatchPanel" | "vpbPatchPanel" => {
                push_lpp_row_schematic(scene, nx, ny, row_top, true, false);
                let max_norm_len = if node.node_type == "mlpPatchPanel" { 4 } else { 2 };
                let norm = row
                    .get("normalling")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| sanitize_text(s, max_norm_len))
                    .unwrap_or_default();
                if !norm.is_empty() {
                    scene.primitives.push(ScenePrimitive::Text(SceneText {
                        position: local_to_diagram(nx, ny, w / 2.0, row_label_cy),
                        content: norm,
                        height_px: row_label_style.height_px,
                        halign: HAlign::Center,
                        valign: VAlign::Middle,
                        font: row_label_style.font.to_string(),
                        owner_node_id: None,
                    }));
                }
                let num = (i + 1).to_string();
                push_row_label_text(scene, nx, ny, 4.0, row_label_cy, num.clone(), HAlign::Left);
                push_row_label_text(scene, nx, ny, w - 4.0, row_label_cy, num, HAlign::Right);
            }
            _ => {
                // lppPatchPanel (default)
                let connected = row
                    .get("connected")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                push_lpp_row_schematic(scene, nx, ny, row_top, true, connected);
                let num = (i + 1).to_string();
                push_row_label_text(scene, nx, ny, 4.0, row_label_cy, num.clone(), HAlign::Left);
                push_row_label_text(scene, nx, ny, w - 4.0, row_label_cy, num, HAlign::Right);
            }
        }

        if let Some(row_id) = row.get("id").and_then(|v| v.as_str()) {
            let row_bounds = RectPx::new(
                nx + inset,
                ny + row_top,
                w - 2.0 * inset,
                PATCH_CIRCUIT_HEIGHT_PX,
            );
            let half_w = row_bounds.width / 2.0;
            let l_handle = format!("L-{row_id}");
            let r_handle = format!("R-{row_id}");
            scene.hits.push(HitTarget {
                id: format!("{}:{}", node.id, l_handle),
                bounds: RectPx::new(row_bounds.x, row_bounds.y, half_w, row_bounds.height),
                node_id: Some(node.id.clone()),
                edge_id: None,
                handle_id: Some(l_handle),
                face_mask_bounds: None,
                face_mask_polygon: None,
                wire_grip_segment: None,
                wire_grip_orientation: None,
            });
            scene.hits.push(HitTarget {
                id: format!("{}:{}", node.id, r_handle),
                bounds: RectPx::new(row_bounds.x + half_w, row_bounds.y, half_w, row_bounds.height),
                node_id: Some(node.id.clone()),
                edge_id: None,
                handle_id: Some(r_handle),
                face_mask_bounds: None,
                face_mask_polygon: None,
                wire_grip_segment: None,
                wire_grip_orientation: None,
            });
        }
    }

    let split_zone = split_instance.map(|_| BREAKLINE_DEVICE_ZONE_WIDTH_PX);
    scene.hits.push(HitTarget {
        id: node.id.clone(),
        bounds: patch_panel_scene_bounds(node),
        node_id: Some(node.id.clone()),
        edge_id: None,
        handle_id: None,
        face_mask_bounds: None,
        face_mask_polygon: Some(inset_frame_face_mask_polygon(
            nx,
            ny,
            w,
            total_height,
            inset,
            split_zone,
        )),
        wire_grip_segment: None,
        wire_grip_orientation: None,
    });
}
