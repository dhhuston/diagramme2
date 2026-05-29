//! AV plate body row flattening — aligned with v6 `avPlateLayout.ts`.

use crate::device_v2_layout::{parse_column_group, ColumnGroup};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum AvPlateBodyRow {
    Gap,
    Header {
        label: String,
    },
    Port {
        group_index: usize,
        row_id: String,
        label: String,
        direction: Option<String>,
    },
}

pub fn av_plate_groups_from_data(data: &Value) -> Vec<ColumnGroup> {
    data.get("groups")
        .and_then(|v| v.as_array())
        .map(|groups| groups.iter().map(parse_column_group).collect())
        .unwrap_or_default()
}

/// Flatten AV plate groups into body rows (same structure as device v2 columns).
pub fn flatten_av_plate_body_rows(groups: &[ColumnGroup]) -> Vec<AvPlateBodyRow> {
    let mut out = Vec::new();
    if !groups.is_empty() {
        out.push(AvPlateBodyRow::Gap);
    }
    for (group_index, group) in groups.iter().enumerate() {
        if let Some(header) = group.header.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            out.push(AvPlateBodyRow::Header {
                label: header.to_string(),
            });
        }
        for row in &group.rows {
            out.push(AvPlateBodyRow::Port {
                group_index,
                row_id: row.id.clone(),
                label: row.label.clone(),
                direction: row.direction.clone(),
            });
        }
        out.push(AvPlateBodyRow::Gap);
    }
    out
}

pub fn av_plate_body_grid_row_count(groups: &[ColumnGroup]) -> usize {
    flatten_av_plate_body_rows(groups).len()
}
