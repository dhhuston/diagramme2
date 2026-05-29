//! Merge Rust-emitted ENTITIES into the acad-ts shell template from Diagramme v6.

const SHELL_TEMPLATE: &str = include_str!("../assets/acad_shell_template.dxf");
const ENTITIES_EMPTY: &str = "  0\nSECTION\n  2\nENTITIES\n  0\nENDSEC\n";

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub model_space_owner: String,
    pub next_entity_handle: u32,
}

pub fn shell_template() -> &'static str {
    SHELL_TEMPLATE
}

pub fn parse_shell_info(template: &str) -> ShellInfo {
    let model_space_owner = parse_model_space_owner(template).unwrap_or_else(|| "1D".to_string());
    let next_entity_handle = max_handle(template).map(|h| h + 1).unwrap_or(0x331);
    ShellInfo {
        model_space_owner,
        next_entity_handle,
    }
}

pub fn merge_entities_into_shell(template: &str, entities_body: &str) -> String {
    let Some(start) = template.find(ENTITIES_EMPTY) else {
        return template.to_string();
    };
    let end = start + ENTITIES_EMPTY.len();
    let mut out = String::with_capacity(template.len() + entities_body.len() + 32);
    out.push_str(&template[..start]);
    out.push_str("  0\nSECTION\n  2\nENTITIES\n");
    out.push_str(entities_body);
    if !entities_body.is_empty() && !entities_body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("  0\nENDSEC\n");
    out.push_str(&template[end..]);
    update_handseed(&mut out);
    out
}

pub fn write_dxf_from_template(entities_body: &str) -> String {
    merge_entities_into_shell(SHELL_TEMPLATE, entities_body)
}

fn parse_model_space_owner(template: &str) -> Option<String> {
    let lines: Vec<&str> = template.lines().collect();
    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i].trim() == "0" && lines[i + 1].trim() == "BLOCK_RECORD" {
            let mut handle = None;
            let mut name = None;
            let mut j = i + 2;
            while j + 1 < lines.len() {
                let code = lines[j].trim();
                let val = lines[j + 1].trim();
                if code == "0" {
                    break;
                }
                if code == "5" {
                    handle = Some(val.to_string());
                }
                if code == "2" {
                    name = Some(val.to_string());
                }
                j += 2;
            }
            if name.as_deref() == Some("*Model_Space") {
                return handle;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    None
}

fn max_handle(template: &str) -> Option<u32> {
    let lines: Vec<&str> = template.lines().collect();
    let mut max = 0u32;
    let mut found = false;
    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i].trim() == "5" || lines[i].trim() == "105" {
            if let Ok(value) = u32::from_str_radix(lines[i + 1].trim(), 16) {
                max = max.max(value);
                found = true;
            }
        }
        i += 1;
    }
    found.then_some(max)
}

fn update_handseed(dxf: &mut String) {
    let max = max_handle(dxf).unwrap_or(0x2D);
    let seed = format!("{:X}", max + 1);
    let lines: Vec<&str> = dxf.lines().collect();
    let mut out = String::with_capacity(dxf.len());
    let mut i = 0;
    while i < lines.len() {
        out.push_str(lines[i]);
        out.push('\n');
        if lines[i].trim() == "$HANDSEED" && i + 2 < lines.len() {
            i += 1;
            out.push_str(lines[i]);
            out.push('\n');
            i += 1;
            out.push_str(&seed);
            out.push('\n');
            i += 1;
            continue;
        }
        i += 1;
    }
    *dxf = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_parses_model_space_owner() {
        let info = parse_shell_info(SHELL_TEMPLATE);
        assert_eq!(info.model_space_owner, "1D");
        assert!(info.next_entity_handle > 0x10);
    }

    #[test]
    fn merge_preserves_shell_sections() {
        let merged = merge_entities_into_shell(SHELL_TEMPLATE, "");
        for section in ["HEADER", "CLASSES", "TABLES", "BLOCKS", "OBJECTS"] {
            assert!(merged.contains(section), "missing {section}");
        }
        assert!(merged.contains("EOF"));
    }

    #[test]
    fn merge_inserts_entities_between_section_markers() {
        let body = "  0\nLINE\n  5\n331\n";
        let merged = merge_entities_into_shell(SHELL_TEMPLATE, body);
        assert!(merged.contains("  0\nLINE\n"));
        let entities = merged.split("ENTITIES").nth(1).unwrap().split("ENDSEC").next().unwrap();
        assert!(entities.contains("LINE"));
    }

    #[test]
    fn handseed_updated_after_merge() {
        let body = "  0\nLINE\n  5\n999\n";
        let merged = merge_entities_into_shell(SHELL_TEMPLATE, body);
        let seed_line = merged
            .lines()
            .skip_while(|l| l.trim() != "$HANDSEED")
            .nth(2)
            .unwrap()
            .trim();
        assert_eq!(seed_line, "99A");
    }
}
