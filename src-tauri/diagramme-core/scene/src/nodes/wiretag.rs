//! Wiretag scene primitives — geometry ported from v6 `appendWiretagRevitDxf`.

use diagramme_geometry::{
    text_style_for_role, wiretag_export_width_for_node, wiretag_font_size_px,
    wiretag_index_column_width_px, wiretag_tip_width_px, PointPx, RectPx, TextRole,
    WIRETAG_BAR_HEIGHT_PX,
};

pub use diagramme_geometry::{get_device_tag_label, resolve_pair_main_display_text};
use diagramme_schema::{Edge, Node};

use crate::scene::{HitTarget, HAlign, Scene, ScenePrimitive, SceneText, VAlign};
use crate::text::sanitize_text;

const DEFAULT_LAYER: &str = "0";
const HAIRLINE_STROKE_PX: f64 = 1.0;
const WIRETAG_SPLIT_TEXT_PAD_PX: f64 = 2.0;
const WIRETAG_SPLIT_DIVIDER_WIDTH_PX: f64 = 0.5;
const WIRETAG_SPLIT_DIVIDER_MARGIN_PX: f64 = 1.0;

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

fn push_text(scene: &mut Scene, nx: f64, ny: f64, lx: f64, ly: f64, content: String, height_px: f64) {
    let style = text_style_for_role(TextRole::Wiretag);
    scene.primitives.push(ScenePrimitive::Text(SceneText {
        position: local_to_diagram(nx, ny, lx, ly),
        content,
        height_px,
        halign: HAlign::Center,
        valign: VAlign::Middle,
        font: style.font.to_string(),
    }));
}

/// Closed hull path in wiretag-local diagram px (mirrors v6 `wiretagHullPathDiagram`).
pub fn wiretag_hull_path_diagram(
    w: f64,
    h: f64,
    iw: f64,
    tw: f64,
    end_a: bool,
) -> Vec<(f64, f64)> {
    if end_a {
        vec![
            (0.0, 0.0),
            (w - iw, 0.0),
            (w, 0.0),
            (w, h),
            (w - iw, h),
            (0.0, h),
            (tw, h * 0.5),
        ]
    } else {
        vec![
            (0.0, h),
            (0.0, 0.0),
            (iw, 0.0),
            (w - tw, 0.0),
            (w, h * 0.5),
            (w - tw, h),
            (iw, h),
            (0.0, h),
        ]
    }
}

/// Index-column width from pair index and bar metrics (mirrors v6 `wiretagIndexColumnWidthExport`).
pub fn wiretag_index_column_width_export(pair_index: i64, bar_h: f64, font_size: f64) -> f64 {
    wiretag_index_column_width_px(pair_index, bar_h, font_size)
}

/// Arrow-tip horizontal extent (mirrors v6 `wiretagTipWidthExport`).
pub fn wiretag_tip_width_export(bar_h: f64) -> f64 {
    wiretag_tip_width_px(bar_h)
}

fn read_pair_index(data: &serde_json::Value) -> i64 {
    data.get("pairIndex")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .unwrap_or(0)
}

fn read_end_a(data: &serde_json::Value) -> bool {
    data.get("end")
        .and_then(|v| v.as_str())
        .map(|s| s == "a")
        .unwrap_or(true)
}

fn wiretag_width_px(
    node: &Node,
    nodes: &[Node],
    edges: &[Edge],
    _bar_h: f64,
) -> f64 {
    wiretag_export_width_for_node(node, nodes, edges)
}

fn wiretag_height_px(node: &Node, bar_h: f64) -> f64 {
    node.height.filter(|h| *h > 0.0).unwrap_or(bar_h)
}

/// Scene bounds for a wiretag node (diagram px).
pub fn wiretag_scene_bounds(node: &Node, nodes: &[Node], edges: &[Edge]) -> RectPx {
    let bar_h = WIRETAG_BAR_HEIGHT_PX;
    let w = wiretag_width_px(node, nodes, edges, bar_h);
    let h = wiretag_height_px(node, bar_h);
    RectPx::new(node.position.x, node.position.y, w, h)
}

/// Append wiretag drawable primitives and hit targets to `scene`.
pub fn append_wiretag_scene(
    scene: &mut Scene,
    node: &Node,
    all_nodes: &[Node],
    all_edges: &[Edge],
) {
    let nx = node.position.x;
    let ny = node.position.y;
    let data = &node.data;
    let h = wiretag_height_px(node, WIRETAG_BAR_HEIGHT_PX);
    let end_a = read_end_a(data);
    let font_size = wiretag_font_size_px(h);
    let tw = wiretag_tip_width_export(h);
    let pair_index = read_pair_index(data);
    let iw = wiretag_index_column_width_export(pair_index, h, font_size);
    let w = wiretag_width_px(node, all_nodes, all_edges, h);

    let hull = wiretag_hull_path_diagram(w, h, iw, tw, end_a);
    push_polyline(
        scene,
        hull.iter()
            .map(|(lx, ly)| local_to_diagram(nx, ny, *lx, *ly))
            .collect(),
        true,
    );

    let divider_x = if end_a { w - iw } else { iw };
    push_line(scene, divider_x, 0.0, divider_x, h, nx, ny);

    let main_raw = resolve_pair_main_display_text(node, all_nodes, all_edges);
    let main = sanitize_text(
        if main_raw.trim().is_empty() {
            format!("WT-{pair_index}")
        } else {
            main_raw.trim().to_string()
        }
        .as_str(),
        120,
    );
    let show_sheet = data
        .get("showSheetName")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sheet = sanitize_text(
        if show_sheet {
            data.get("sheetName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
        } else {
            ""
        },
        120,
    );
    let idx = pair_index.to_string();
    let tip_pad = tw + 2.0;
    let inner_pad = 4.0;
    let has_sheet = !sheet.is_empty();
    let text_avail = (w - iw - tip_pad - inner_pad).max(1.0);
    let split_pad = if has_sheet {
        WIRETAG_SPLIT_TEXT_PAD_PX
    } else {
        0.0
    };
    let sheet_approx = if has_sheet {
        sheet.len() as f64 * font_size * 0.55 + split_pad
    } else {
        0.0
    };
    let main_approx = (font_size * 2.0)
        .max(main.len() as f64 * font_size * 0.55 + WIRETAG_SPLIT_TEXT_PAD_PX);
    let split_gap = if has_sheet {
        WIRETAG_SPLIT_DIVIDER_WIDTH_PX + WIRETAG_SPLIT_DIVIDER_MARGIN_PX * 2.0
    } else {
        0.0
    };
    let total_approx = if has_sheet {
        sheet_approx + split_gap + main_approx
    } else {
        main_approx
    };
    let scale = if total_approx > 0.0 {
        (text_avail / total_approx).min(1.0)
    } else {
        1.0
    };
    let sheet_w = if has_sheet {
        (sheet_approx * scale).max(4.0)
    } else {
        0.0
    };
    let main_w = if has_sheet {
        (text_avail - sheet_w - split_gap).max(4.0)
    } else {
        text_avail.max(4.0)
    };

    let draw_main_with_optional_sheet =
        |scene: &mut Scene, left: f64| {
            let mid_y = h / 2.0;
            if has_sheet {
                push_text(
                    scene,
                    nx,
                    ny,
                    left + sheet_w / 2.0,
                    mid_y,
                    sheet.clone(),
                    font_size,
                );
                let split_divider_center_x = left
                    + sheet_w
                    + WIRETAG_SPLIT_DIVIDER_MARGIN_PX
                    + WIRETAG_SPLIT_DIVIDER_WIDTH_PX / 2.0;
                push_line(
                    scene,
                    split_divider_center_x,
                    0.0,
                    split_divider_center_x,
                    h,
                    nx,
                    ny,
                );
                push_text(
                    scene,
                    nx,
                    ny,
                    left + sheet_w + split_gap + main_w / 2.0,
                    mid_y,
                    if main.is_empty() {
                        " ".to_string()
                    } else {
                        main.clone()
                    },
                    font_size,
                );
            } else {
                push_text(
                    scene,
                    nx,
                    ny,
                    left + main_w / 2.0,
                    mid_y,
                    if main.is_empty() {
                        " ".to_string()
                    } else {
                        main.clone()
                    },
                    font_size,
                );
            }
        };

    if end_a {
        let main_left = tip_pad;
        draw_main_with_optional_sheet(scene, main_left);
        push_text(scene, nx, ny, w - iw / 2.0, h / 2.0, idx, font_size);
    } else {
        let main_left = iw + inner_pad;
        push_text(scene, nx, ny, iw / 2.0, h / 2.0, idx, font_size);
        draw_main_with_optional_sheet(scene, main_left);
    }

    scene.hits.push(HitTarget {
        id: node.id.clone(),
        bounds: RectPx::new(nx, ny, w, h),
        node_id: Some(node.id.clone()),
        edge_id: None,
    });
}
