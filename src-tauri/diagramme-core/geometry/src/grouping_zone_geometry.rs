//! Grouping zone polyline helpers (ported from v6 `groupingZoneGeometry.ts`).

/// Label anchor for polyline grouping zones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelAnchor {
    pub x: f64,
    pub y: f64,
}

pub type Pt = (f64, f64);

pub fn to_pairs(pts: &[f64]) -> Vec<Pt> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < pts.len() {
        out.push((pts[i], pts[i + 1]));
        i += 2;
    }
    out
}

fn seg_dir(ax: f64, ay: f64, bx: f64, by: f64) -> char {
    if (by - ay).abs() < (bx - ax).abs() {
        'H'
    } else {
        'V'
    }
}

/// Top-left-most horizontal segment start (v6 `getLabelAnchor`).
pub fn get_label_anchor(pairs: &[Pt]) -> LabelAnchor {
    let mut best: Option<(f64, f64)> = None;
    let n = pairs.len();
    if n == 0 {
        return LabelAnchor { x: 0.0, y: 0.0 };
    }

    for i in 0..n {
        let (ax, ay) = pairs[i];
        let (bx, by) = pairs[(i + 1) % n];
        if seg_dir(ax, ay, bx, by) != 'H' {
            continue;
        }
        let x = ax.min(bx);
        let y = ay;
        if best.is_none() || y < best.unwrap().1 || (y == best.unwrap().1 && x < best.unwrap().0) {
            best = Some((x, y));
        }
    }

    best.map(|(x, y)| LabelAnchor { x, y })
        .unwrap_or(LabelAnchor { x: 0.0, y: 0.0 })
}

/// Axis-aligned bounds of flat `[x0,y0,…]` polyline in node-local px.
pub fn polyline_flat_bounds(pts: &[f64]) -> Option<(f64, f64, f64, f64)> {
    let pairs = to_pairs(pts);
    if pairs.len() < 2 {
        return None;
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in pairs {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some((min_x, min_y, max_x, max_y))
}
