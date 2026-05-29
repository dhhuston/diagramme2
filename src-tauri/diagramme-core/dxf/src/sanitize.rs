//! Revit-safe DXF post-processing: extents, SOLID-first reorder, unique handles.

use crate::document::{CadExtentInches, PAD_IN};

const ACADVER: &str = "AC1015";

/// Inserts `$EXTMIN` / `$EXTMAX` / `$LIMMIN` / `$LIMMAX` into HEADER when the writer
/// omitted them. If HEADER already contains `$EXTMIN`, returns `dxf` unchanged.
pub fn inject_header_extents(dxf: &str, ext: CadExtentInches) -> String {
    let header_marker = "  2\nHEADER\n";
    let h_pos = dxf.find(header_marker);
    let h_pos = match h_pos {
        Some(p) => p,
        None => return dxf.to_string(),
    };

    let endsec_marker = "\n  0\nENDSEC\n";
    let endsec_pos = dxf[h_pos..].find(endsec_marker);
    let endsec_pos = match endsec_pos {
        Some(p) => h_pos + p,
        None => return dxf.to_string(),
    };

    let header_body = &dxf[h_pos..endsec_pos];
    if header_body.contains("$EXTMIN") {
        return dxf.to_string();
    }

    let min_x = ext.min_x - PAD_IN;
    let min_y = ext.min_y - PAD_IN;
    let max_x = ext.max_x + PAD_IN;
    let max_y = ext.max_y + PAD_IN;

    let vars = format!(
        "  9\n$EXTMIN\n 10\n{min_x}\n 20\n{min_y}\n 30\n0.0\n\
           9\n$EXTMAX\n 10\n{max_x}\n 20\n{max_y}\n 30\n0.0\n\
           9\n$LIMMIN\n 10\n{min_x}\n 20\n{min_y}\n\
           9\n$LIMMAX\n 10\n{max_x}\n 20\n{max_y}"
    );

    let mut out = String::with_capacity(dxf.len() + vars.len() + 1);
    out.push_str(&dxf[..endsec_pos]);
    out.push('\n');
    out.push_str(&vars);
    out.push_str(&dxf[endsec_pos..]);
    out
}

pub fn sanitize_dxf_string(dxf: &str) -> String {
    let lines: Vec<&str> = dxf.split('\n').collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed == "420" && i > 0 && lines[i - 1].trim() != "5" {
            i += 2;
            continue;
        }

        if trimmed == "62" && i + 1 < lines.len() && lines[i + 1].trim() == "0" {
            i += 2;
            continue;
        }

        out.push(line);
        i += 1;
    }

    for j in 0..out.len().saturating_sub(2) {
        if out[j].trim() == "$ACADVER" && out[j + 2].trim() != ACADVER {
            out[j + 2] = ACADVER;
        }
    }

    let last_non_empty = out.iter().rev().find(|l| !l.trim().is_empty());
    if last_non_empty.map(|l| l.trim()) != Some("EOF") {
        out.push("  0");
        out.push("EOF");
        out.push("");
    }

    let joined = out.join("\n");
    let reordered = reorder_solids_first(&joined);
    ensure_unique_handles(&reordered)
}

fn reorder_solids_first(dxf: &str) -> String {
    let entities_marker = "  2\nENTITIES\n";
    let endsec_marker = "  0\nENDSEC\n";

    let Some(ent_start) = dxf.find(entities_marker) else {
        return dxf.to_string();
    };
    let body_start = ent_start + entities_marker.len();
    let Some(rel_end) = dxf[body_start..].find(endsec_marker) else {
        return dxf.to_string();
    };
    let body_end = body_start + rel_end;

    let before = &dxf[..body_start];
    let body = &dxf[body_start..body_end];
    let after = &dxf[body_end..];

    let mut solids = Vec::new();
    let mut others = Vec::new();

    for part in body.split("\n  0\n") {
        if part.trim().is_empty() {
            continue;
        }
        let chunk = if part.starts_with("  0\n") {
            part.to_string()
        } else {
            format!("  0\n{part}")
        };
        if chunk.starts_with("  0\nSOLID\n") {
            solids.push(chunk);
        } else {
            others.push(chunk);
        }
    }

    format!("{}{}{}{}", before, solids.join(""), others.join(""), after)
}

fn ensure_unique_handles(dxf: &str) -> String {
    let mut lines: Vec<String> = dxf.lines().map(str::to_string).collect();
    let mut seen = std::collections::HashSet::new();
    let mut next_id: u32 = 0x1000;

    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i].trim() == "5" {
            let handle = lines[i + 1].trim().to_string();
            if !seen.insert(handle.clone()) {
                let new_handle = loop {
                    let candidate = format!("{:X}", next_id);
                    next_id += 1;
                    if seen.insert(candidate.clone()) {
                        break candidate;
                    }
                };
                lines[i + 1] = new_handle;
            }
        }
        i += 1;
    }

    lines.join("\n")
}
