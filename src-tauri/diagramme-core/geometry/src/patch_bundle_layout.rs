//! Flat left/right bundle bracket slots for patch panels and AV plates (v6 `flatPatchBundleSlots` / `flatBundleSlots`).

use crate::av_plate_layout::AvPlateBodyRow;
use crate::device_v2_layout::Side;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatchBundleSlot {
    pub side: Side,
    pub y0: f64,
    pub y1: f64,
    pub count: usize,
}

/// Bundle brackets for patch panel row ids (`bundledLeft` / `bundledRight` on node data).
pub fn flat_patch_bundle_slots(
    row_ids: &[String],
    bundled_left: Option<&[Vec<String>]>,
    bundled_right: Option<&[Vec<String>]>,
    body_top: f64,
    row_px: f64,
    row_center_px: f64,
) -> Vec<PatchBundleSlot> {
    let mut slots = Vec::new();

    let mut collect_bundle =
        |bundle: &[String], side: Side| collect_patch_bundle_run(row_ids, bundle, side, body_top, row_px, row_center_px, &mut slots);

    if let Some(left) = bundled_left {
        for bundle in left {
            collect_bundle(bundle, Side::Left);
        }
    }
    if let Some(right) = bundled_right {
        for bundle in right {
            collect_bundle(bundle, Side::Right);
        }
    }
    slots
}

fn collect_patch_bundle_run(
    row_ids: &[String],
    bundle: &[String],
    side: Side,
    body_top: f64,
    row_px: f64,
    row_center_px: f64,
    slots: &mut Vec<PatchBundleSlot>,
) {
    let set: std::collections::HashSet<&str> = bundle.iter().map(String::as_str).collect();
    let mut run_start: isize = -1;
    let mut run_last: isize = -1;
    let mut run_count = 0usize;

    let flush = |run_start: &mut isize,
                 run_last: &mut isize,
                 run_count: &mut usize,
                 slots: &mut Vec<PatchBundleSlot>| {
        if *run_start < 0 {
            return;
        }
        slots.push(PatchBundleSlot {
            side,
            y0: body_top + *run_start as f64 * row_px + row_center_px,
            y1: body_top + *run_last as f64 * row_px + row_center_px,
            count: *run_count,
        });
        *run_start = -1;
        *run_last = -1;
        *run_count = 0;
    };

    for (i, id) in row_ids.iter().enumerate() {
        if set.contains(id.as_str()) {
            if run_start < 0 {
                run_start = i as isize;
            }
            run_last = i as isize;
            run_count += 1;
        } else {
            flush(&mut run_start, &mut run_last, &mut run_count, slots);
        }
    }
    flush(&mut run_start, &mut run_last, &mut run_count, slots);
}

/// Bundle brackets for AV plate body rows (`bundledLeft` / `bundledRight` on node data).
pub fn flat_av_plate_bundle_slots(
    rows: &[AvPlateBodyRow],
    bundled_left: Option<&[Vec<String>]>,
    bundled_right: Option<&[Vec<String>]>,
    body_top: f64,
    row_px: f64,
    row_center_px: f64,
) -> Vec<PatchBundleSlot> {
    let mut slots = Vec::new();

    let mut collect_bundle = |bundle: &[String], side: Side| {
        collect_av_bundle_run(rows, bundle, side, body_top, row_px, row_center_px, &mut slots)
    };

    if let Some(left) = bundled_left {
        for bundle in left {
            collect_bundle(bundle, Side::Left);
        }
    }
    if let Some(right) = bundled_right {
        for bundle in right {
            collect_bundle(bundle, Side::Right);
        }
    }
    slots
}

fn collect_av_bundle_run(
    rows: &[AvPlateBodyRow],
    bundle: &[String],
    side: Side,
    body_top: f64,
    row_px: f64,
    row_center_px: f64,
    slots: &mut Vec<PatchBundleSlot>,
) {
    let set: std::collections::HashSet<&str> = bundle.iter().map(String::as_str).collect();
    let mut run_start: isize = -1;
    let mut run_last: isize = -1;
    let mut run_count = 0usize;

    let flush = |run_start: &mut isize,
                 run_last: &mut isize,
                 run_count: &mut usize,
                 slots: &mut Vec<PatchBundleSlot>| {
        if *run_start < 0 {
            return;
        }
        slots.push(PatchBundleSlot {
            side,
            y0: body_top + *run_start as f64 * row_px + row_center_px,
            y1: body_top + *run_last as f64 * row_px + row_center_px,
            count: *run_count,
        });
        *run_start = -1;
        *run_last = -1;
        *run_count = 0;
    };

    for (i, row) in rows.iter().enumerate() {
        let in_bundle = matches!(row, AvPlateBodyRow::Port { row_id, .. } if set.contains(row_id.as_str()));
        if in_bundle {
            if run_start < 0 {
                run_start = i as isize;
            }
            run_last = i as isize;
            run_count += 1;
        } else {
            flush(&mut run_start, &mut run_last, &mut run_count, slots);
        }
    }
    flush(&mut run_start, &mut run_last, &mut run_count, slots);
}
