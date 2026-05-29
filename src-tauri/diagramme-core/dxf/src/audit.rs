//! Scan DXF output for values known to crash Autodesk translation.

const PX_PER_INCH: f64 = 72.0;

fn px_to_in(px: f64) -> f64 {
    px / PX_PER_INCH
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegionPolylineAudit {
    /// Polylines on the WIRES layer intersecting the region (routed wires).
    pub wire_layer_polylines: usize,
    /// Polylines on layer 0 intersecting the region (frames, brackets, symbols).
    pub layer0_polylines: usize,
}

/// Axis-aligned region in diagram pixels (converted to inches for DXF comparison).
#[derive(Debug, Clone, Copy)]
pub struct RegionBboxPx {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl RegionBboxPx {
    pub fn to_inches(self) -> (f64, f64, f64, f64) {
        (
            px_to_in(self.x),
            px_to_in(self.y),
            px_to_in(self.x + self.width),
            px_to_in(self.y + self.height),
        )
    }
}

fn polyline_intersects_region(
    groups: &[(i32, String)],
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> bool {
    let mut pending_x: Option<f64> = None;
    for (code, value) in groups {
        match code {
            10 => pending_x = value.parse().ok(),
            20 => {
                if let Some(x) = pending_x.take() {
                    if let Ok(y) = value.parse::<f64>() {
                        if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn entities_in_section(dxf: &str) -> &str {
    dxf.split("ENTITIES")
        .nth(1)
        .and_then(|s| s.split("ENDSEC").next())
        .unwrap_or("")
}

fn parse_entities(section: &str) -> Vec<(String, Vec<(i32, String)>)> {
    let mut entities = Vec::new();
    for part in section.split("\n  0\n") {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut lines = trimmed.lines();
        let Some(kind) = lines.next() else {
            continue;
        };
        let kind = kind.trim().to_string();
        if kind == "SECTION" || kind == "ENDSEC" || kind == "EOF" {
            continue;
        }
        let mut groups = Vec::new();
        let mut iter = lines.peekable();
        while iter.peek().is_some() {
            let code_line = iter.next().unwrap().trim();
            let Ok(code) = code_line.parse::<i32>() else {
                continue;
            };
            let value = iter
                .next()
                .map(|l| l.trim().to_string())
                .unwrap_or_default();
            groups.push((code, value));
        }
        entities.push((kind, groups));
    }
    entities
}

/// Count LWPOLYLINE entities whose vertices fall inside a diagram-space region.
pub fn audit_polylines_in_region(dxf: &str, region: RegionBboxPx) -> RegionPolylineAudit {
    let (min_x, min_y, max_x, max_y) = region.to_inches();
    let mut audit = RegionPolylineAudit::default();
    for (kind, groups) in parse_entities(entities_in_section(dxf)) {
        if kind != "LWPOLYLINE" {
            continue;
        }
        if !polyline_intersects_region(&groups, min_x, min_y, max_x, max_y) {
            continue;
        }
        let layer = groups
            .iter()
            .find_map(|(code, value)| (*code == 8).then_some(value.as_str()))
            .unwrap_or("0");
        if layer == "WIRES" {
            audit.wire_layer_polylines += 1;
        } else if layer == "0" {
            audit.layer0_polylines += 1;
        }
    }
    audit
}

#[derive(Debug, Clone)]
pub struct DxfAuditIssue {
    pub code: String,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DxfAuditReport {
    pub issues: Vec<DxfAuditIssue>,
}

impl DxfAuditReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn to_text(&self) -> String {
        if self.issues.is_empty() {
            return "No issues found.\n".to_string();
        }
        let mut out = format!("{} issue(s):\n", self.issues.len());
        for issue in &self.issues {
            if let Some(line) = issue.line {
                out.push_str(&format!("- [{}] line {}: {}\n", issue.code, line, issue.message));
            } else {
                out.push_str(&format!("- [{}]: {}\n", issue.code, issue.message));
            }
        }
        out
    }
}

pub fn audit_dxf(dxf: &str) -> DxfAuditReport {
    let lines: Vec<&str> = dxf.lines().collect();
    let mut issues = Vec::new();

    for i in 0..lines.len().saturating_sub(1) {
        let code = lines[i].trim();
        let value = lines[i + 1].trim();

        if code == "420" && (i == 0 || lines[i - 1].trim() != "5") {
            issues.push(DxfAuditIssue {
                code: "true-color-420".into(),
                message: format!("true-color group 420 = {value}"),
                line: Some(i + 1),
            });
        }
        if code == "62" && value == "0" {
            issues.push(DxfAuditIssue {
                code: "byblock-62-0".into(),
                message: "BYBLOCK color 62/0".into(),
                line: Some(i + 1),
            });
        }
    }

    if !dxf.contains("AC1015") {
        issues.push(DxfAuditIssue {
            code: "acadver".into(),
            message: "missing AC1015 version".into(),
            line: None,
        });
    }

    let header_marker = "  2\nHEADER\n";
    if let Some(h_pos) = dxf.find(header_marker) {
        let header_end = dxf[h_pos..].find("\n  0\nENDSEC\n").map(|p| h_pos + p);
        if let Some(end) = header_end {
            let header = &dxf[h_pos..end];
            if !header.contains("$EXTMIN") || !header.contains("$EXTMAX") {
                issues.push(DxfAuditIssue {
                    code: "header-extents".into(),
                    message: "HEADER missing $EXTMIN/$EXTMAX".into(),
                    line: None,
                });
            }
        }
    }

    for section in ["CLASSES", "BLOCKS", "OBJECTS", "ACAD_LAYOUT", "AcDbLayout"] {
        if !dxf.contains(section) {
            issues.push(DxfAuditIssue {
                code: "missing-section".into(),
                message: format!("missing {section}"),
                line: None,
            });
        }
    }

    if dxf
        .split("ENTITIES")
        .nth(1)
        .is_some_and(|s| s.split("ENDSEC").next().unwrap_or("").contains("HATCH"))
    {
        issues.push(DxfAuditIssue {
            code: "hatch-entity".into(),
            message: "HATCH entity in ENTITIES".into(),
            line: None,
        });
    }

    issues.extend(audit_dangling_handles(dxf));
    issues.extend(audit_entity_layers(dxf));

    DxfAuditReport { issues }
}

fn audit_dangling_handles(dxf: &str) -> Vec<DxfAuditIssue> {
    let lines: Vec<&str> = dxf.lines().collect();
    let mut handles = std::collections::HashSet::new();
    let mut issues = Vec::new();

    let mut i = 0;
    while i + 1 < lines.len() {
        let code = lines[i].trim();
        let val = lines[i + 1].trim().to_uppercase();
        if code == "5" || code == "105" {
            handles.insert(val);
        }
        i += 1;
    }

    i = 0;
    while i + 1 < lines.len() {
        let code = lines[i].trim();
        let val = lines[i + 1].trim().to_uppercase();
        if matches!(code, "330" | "340" | "350") && val != "0" && !handles.contains(&val) {
            issues.push(DxfAuditIssue {
                code: "dangling-handle".into(),
                message: format!("group {code} references missing handle {val}"),
                line: Some(i + 1),
            });
        }
        i += 1;
    }
    issues
}

fn audit_entity_layers(dxf: &str) -> Vec<DxfAuditIssue> {
    let declared = ["0", "WIRES", "FILLS", "INKFILL", "GUIDES"];
    let entities = dxf
        .split("ENTITIES")
        .nth(1)
        .and_then(|s| s.split("ENDSEC").next())
        .unwrap_or("");
    let mut issues = Vec::new();
    let lines: Vec<&str> = entities.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let code = lines[i].trim();
        if code.is_empty() {
            i += 1;
            continue;
        }
        if i + 1 >= lines.len() {
            break;
        }
        if code == "8" {
            let layer = lines[i + 1].trim();
            if !declared.contains(&layer) {
                issues.push(DxfAuditIssue {
                    code: "undeclared-layer".into(),
                    message: format!("entity references undeclared layer {layer:?}"),
                    line: None,
                });
            }
        }
        i += 2;
    }
    issues
}
