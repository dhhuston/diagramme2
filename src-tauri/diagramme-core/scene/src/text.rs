//! Scene text sanitization — mirrors v6 `dxfSanitizeText`.

/// Printable ASCII for scene / DXF group 1. Output is always uppercase.
pub fn sanitize_text(s: &str, max_len: usize) -> String {
    let one_line = s.replace("\r\n", " ").replace('\n', " ").replace('\r', " ");
    let filtered: String = one_line
        .chars()
        .filter(|c| {
            let code = *c as u32;
            (32..=126).contains(&code)
        })
        .collect();
    filtered.to_uppercase().chars().take(max_len).collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_text;

    #[test]
    fn sanitize_text_uppercases_and_strips_non_printable() {
        assert_eq!(sanitize_text("Device block", 48), "DEVICE BLOCK");
        assert_eq!(sanitize_text("In", 32), "IN");
        assert_eq!(sanitize_text("loudspeaker patch", 48), "LOUDSPEAKER PATCH");
    }

    #[test]
    fn sanitize_text_preserves_space_placeholder() {
        assert_eq!(sanitize_text(" ", 48), " ");
    }
}
