//! AV plate scene primitives — geometry ported from v6 `appendAvPlateRevitDxf`.

use diagramme_geometry::{
    av_plate_groups_from_data, flatten_av_plate_body_rows, text_style_for_role, AvPlateBodyRow,
    PointPx, RectPx, TextHAlign, TextRole, TextVAlign, AV_PLATE_GRID_ROW_PX, AV_PLATE_TITLE_HEIGHT_PX,
    PATCH_PANEL_WIDTH_PX, SCHEMATIC_FRAME_INSET_PX, SCHEMATIC_TAG_BAND_PX,
    SCHEMATIC_TAG_TEXT_CENTER_Y_PX, schematic_body_row_center_y, schematic_title_band_center_y,
};
use diagramme_schema::Node;

use crate::scene::{HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign};

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

fn push_polyline(scene: &mut Scene, points: Vec<PointPx>) {
    scene.primitives.push(ScenePrimitive::Polyline {
        points,
        stroke_px: HAIRLINE_STROKE_PX,
        layer: DEFAULT_LAYER.to_string(),
        color: 0,
        edge_id: None,
    });
}

fn push_solid(scene: &mut Scene, vertices: [PointPx; 4], node_id: &str) {
    scene.primitives.push(ScenePrimitive::Solid {
        vertices,
        layer: FILLS_LAYER.to_string(),
        node_id: Some(node_id.to_string()),
    });
}

fn push_closed_inset_frame(
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
    );
}

fn push_line(scene: &mut Scene, x0: f64, y0: f64, x1: f64, y1: f64, nx: f64, ny: f64) {
    push_polyline(
        scene,
        vec![
            local_to_diagram(nx, ny, x0, y0),
            local_to_diagram(nx, ny, x1, y1),
        ],
    );
}

fn sanitize_text(raw: &str, max_len: usize) -> String {
    let trimmed: String = raw.chars().filter(|c| *c != '\r' && *c != '\n').collect();
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed.chars().take(max_len).collect()
}

fn with_split_suffix(base: &str, split_instance: Option<u64>) -> String {
    match split_instance {
        Some(n) => format!("{base} ({n})"),
        None => base.to_string(),
    }
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
pub fn append_av_plate_scene(scene: &mut Scene, node: &Node) {
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

    // Title band fill
    push_solid(
        scene,
        [
            local_to_diagram(nx, ny, 0.0, 0.0),
            local_to_diagram(nx, ny, w, 0.0),
            local_to_diagram(nx, ny, w, title_h),
            local_to_diagram(nx, ny, 0.0, title_h),
        ],
        &node.id,
    );

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
    }));

    // Frame (closed inset rectangle)
    push_closed_inset_frame(scene, nx, ny, w, total_height, inset);

    // Title / body divider
    push_line(scene, inset, title_h, w - inset, title_h, nx, ny);

    // Horizontal row dividers
    for i in 0..rows.len().saturating_sub(1) {
        let yi = body_top + (i + 1) as f64 * row_px;
        push_line(scene, inset, yi, w - inset, yi, nx, ny);
    }

    // Title text
    let split_instance = data.get("splitInstance").and_then(|v| v.as_u64());
    let base_title = data
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    let title_str = sanitize_text(&with_split_suffix(base_title, split_instance), 48);
    if !title_str.is_empty() {
        let title_style = text_style_for_role(TextRole::Title);
        scene.primitives.push(ScenePrimitive::Text(SceneText {
            position: local_to_diagram(nx, ny, w / 2.0, schematic_title_band_center_y(title_h)),
            content: title_str,
            height_px: title_style.height_px,
            halign: to_halign(title_style.halign),
            valign: to_valign(title_style.valign),
            font: title_style.font.to_string(),
        }));
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
                }));
            }
            AvPlateBodyRow::Gap => {}
        }
    }

    // Hit target: node body
    scene.hits.push(HitTarget {
        id: node.id.clone(),
        bounds: RectPx::new(nx, ny, w, total_height),
        node_id: Some(node.id.clone()),
        edge_id: None,
    });

    // Port hit targets
    for (row_index, row) in rows.iter().enumerate() {
        if let AvPlateBodyRow::Port { row_id, .. } = row {
            let row_top = body_top + row_index as f64 * row_px;
            scene.hits.push(HitTarget {
                id: format!("{}:{}", node.id, row_id),
                bounds: RectPx::new(nx + inset, ny + row_top, w - 2.0 * inset, row_px),
                node_id: Some(node.id.clone()),
                edge_id: None,
            });
        }
    }
}
