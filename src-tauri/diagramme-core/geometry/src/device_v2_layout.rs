//! Device v2 column flattening — must stay aligned with v6 `deviceV2ColumnLayout.ts`.

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PortRow {
    pub id: String,
    pub label: String,
    pub direction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnGroup {
    pub header: Option<String>,
    pub rows: Vec<PortRow>,
    pub bundled_row_ids: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum DeviceV2BodySlot {
    Header {
        label: String,
    },
    Port {
        group_index: usize,
        row: PortRow,
    },
    Condensed {
        group_index: usize,
        start_label: String,
        end_label: String,
    },
    Bundled {
        group_index: usize,
        count: usize,
        side: Side,
    },
    Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub fn normalize_device_side_groups(
    current: Option<&[Value]>,
    legacy: Option<&[Value]>,
    side: Side,
) -> Vec<ColumnGroup> {
    if let Some(groups) = current {
        if !groups.is_empty() {
            return groups.iter().map(parse_column_group).collect();
        }
    }
    let Some(legacy) = legacy else {
        return Vec::new();
    };
    legacy
        .iter()
        .enumerate()
        .map(|(group_index, g)| {
            let name = g.get("name").and_then(|v| v.as_str()).map(String::from);
            let ports: Vec<PortRow> = g
                .get("ports")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .enumerate()
                        .map(|(port_index, label)| PortRow {
                            id: format!(
                                "{}-{}-{}",
                                match side {
                                    Side::Left => "left",
                                    Side::Right => "right",
                                },
                                group_index,
                                port_index
                            ),
                            label: label.as_str().unwrap_or("").to_string(),
                            direction: None,
                        })
                        .collect()
                })
                .unwrap_or_default();
            ColumnGroup {
                header: name,
                rows: ports,
                bundled_row_ids: Vec::new(),
            }
        })
        .collect()
}

pub fn parse_column_group(value: &Value) -> ColumnGroup {
    let header = value
        .get("header")
        .or_else(|| value.get("title"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let rows = value
        .get("rows")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|row| PortRow {
                    id: row
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    label: row
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    direction: row
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();
    let bundled_row_ids = normalize_bundles(value.get("bundledRowIds"));
    ColumnGroup {
        header,
        rows,
        bundled_row_ids,
    }
}

fn normalize_bundles(raw: Option<&Value>) -> Vec<Vec<String>> {
    let Some(raw) = raw.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    if raw.is_empty() {
        return Vec::new();
    }
    if raw[0].is_string() {
        let ids: Vec<String> = raw
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        return vec![ids];
    }
    raw.iter()
        .filter_map(|bundle| {
            bundle.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        })
        .collect()
}

/// Flatten one I/O column into body rows (leading gap, per-group header + ports, trailing gap).
pub fn flatten_device_v2_body_rows(groups: &[ColumnGroup]) -> Vec<DeviceV2BodySlot> {
    let mut out = Vec::new();
    if !groups.is_empty() {
        out.push(DeviceV2BodySlot::Gap);
    }
    for (group_index, group) in groups.iter().enumerate() {
        if let Some(header) = group.header.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            out.push(DeviceV2BodySlot::Header {
                label: header.to_string(),
            });
        }

        let mut i = 0;
        while i < group.rows.len() {
            let row = &group.rows[i];
            let is_condensed = row_has_condensed_flag(row);
            if is_condensed {
                let mut count = 0usize;
                while i + count < group.rows.len()
                    && row_has_condensed_flag(&group.rows[i + count])
                {
                    count += 1;
                }
                if count >= 3 {
                    out.push(DeviceV2BodySlot::Port {
                        group_index,
                        row: row.clone(),
                    });
                    out.push(DeviceV2BodySlot::Condensed {
                        group_index,
                        start_label: row.label.clone(),
                        end_label: group.rows[i + count - 1].label.clone(),
                    });
                    out.push(DeviceV2BodySlot::Port {
                        group_index,
                        row: group.rows[i + count - 1].clone(),
                    });
                    i += count;
                    continue;
                }
            }
            out.push(DeviceV2BodySlot::Port {
                group_index,
                row: row.clone(),
            });
            i += 1;
        }
        out.push(DeviceV2BodySlot::Gap);
    }
    out
}

fn row_has_condensed_flag(_row: &PortRow) -> bool {
    // Condensed rows are encoded in JSON with a `condensed` flag; PortRow omits it for now.
    false
}

pub fn device_v2_normalized_columns(data: &Value) -> (Vec<ColumnGroup>, Vec<ColumnGroup>) {
    let left = normalize_device_side_groups(
        data.get("leftColumn").and_then(|v| v.as_array()).map(|a| a.as_slice()),
        data.get("leftGroups").and_then(|v| v.as_array()).map(|a| a.as_slice()),
        Side::Left,
    );
    let right = normalize_device_side_groups(
        data.get("rightColumn").and_then(|v| v.as_array()).map(|a| a.as_slice()),
        data.get("rightGroups").and_then(|v| v.as_array()).map(|a| a.as_slice()),
        Side::Right,
    );
    (left, right)
}

pub fn device_v2_body_grid_row_count(data: &Value) -> usize {
    let (left, right) = device_v2_normalized_columns(data);
    std::cmp::max(
        flatten_device_v2_body_rows(&left).len(),
        flatten_device_v2_body_rows(&right).len(),
    )
}

#[derive(Debug, Clone)]
pub struct BundleBracketSlot {
    pub y0: f64,
    pub group_index: usize,
    pub bundle_index: usize,
}

pub fn bundled_bracket_slots(
    rows: &[DeviceV2BodySlot],
    groups: &[ColumnGroup],
    body_top: f64,
    row_px: f64,
) -> Vec<BundleBracketSlot> {
    let mut result = Vec::new();
    for (group_index, group) in groups.iter().enumerate() {
        for (bundle_index, bundle) in group.bundled_row_ids.iter().enumerate() {
            if bundle.is_empty() {
                continue;
            }
            let bundled_set: std::collections::HashSet<&str> =
                bundle.iter().map(String::as_str).collect();
            let bundled_indices: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| match slot {
                    DeviceV2BodySlot::Port { group_index: gi, row, .. }
                        if *gi == group_index && bundled_set.contains(row.id.as_str()) =>
                    {
                        Some(i)
                    }
                    _ => None,
                })
                .collect();
            if bundled_indices.is_empty() {
                continue;
            }
            let first = bundled_indices[0];
            let last = *bundled_indices.last().unwrap();
            let y0 = body_top + first as f64 * row_px + row_px / 2.0;
            let y1 = body_top + last as f64 * row_px + row_px / 2.0;
            result.push(BundleBracketSlot {
                y0,
                group_index,
                bundle_index,
            });
            let _ = y1; // y1 used for bracket drawing; port geometry uses y0 only
        }
    }
    result
}
