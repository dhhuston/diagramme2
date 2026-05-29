#[cfg(test)]
mod tests {
    use crate::paper_scale::{
        LABEL_FONT_PX, MIC_SPEAKER_DESC_FONT_PX, NODE_TITLE_FONT_PX, SNAP_GRID_PX,
        WIRETAG_BAR_HEIGHT_PX,
    };
    use crate::schematic_layout::{PATCH_PANEL_WIDTH_PX, SCHEMATIC_TITLE_SIDE_PADDING_PX};
    use crate::text_measure::{
        estimate_text_width_px, text_style_for_role, wiretag_bounds_diagram_px,
        wiretag_tip_width_px, wrap_schematic_title_lines, TextRole,
    };
    use crate::units::px_to_in;

    // Re-export private helper for minimum-width assertion (same module path via super).
    fn wiretag_index_column_min_width_px(bar_height_px: f64) -> f64 {
        let floor = (bar_height_px * 1.35).round().clamp(9.0, 18.0);
        floor
    }

    #[test]
    fn label_font_nine_px_is_one_eighth_inch_in_cad() {
        assert_eq!(LABEL_FONT_PX, 9.0);
        assert!((px_to_in(LABEL_FONT_PX) - 0.125).abs() < 1e-9);
    }

    #[test]
    fn text_style_tag_uses_node_title_font_px() {
        let style = text_style_for_role(TextRole::Tag);
        assert_eq!(style.height_px, NODE_TITLE_FONT_PX);
        assert_eq!(style.font, "Arial Narrow");
    }

    #[test]
    fn wrap_schematic_title_lines_splits_loudspeaker_patch_at_one_inch() {
        let style = text_style_for_role(TextRole::Title);
        let wrapped = wrap_schematic_title_lines(
            &["Loudspeaker patch".to_string()],
            PATCH_PANEL_WIDTH_PX,
            SCHEMATIC_TITLE_SIDE_PADDING_PX,
            style.height_px,
        );
        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0], "Loudspeaker");
        assert_eq!(wrapped[1], "patch");
    }

    #[test]
    fn node_title_font_is_three_thirty_seconds_inch() {
        assert!((px_to_in(NODE_TITLE_FONT_PX) - 3.0 / 32.0).abs() < 1e-9);
        assert!((NODE_TITLE_FONT_PX - 6.75).abs() < 1e-9);
    }

    #[test]
    fn speaker_secondary_font_is_one_sixteenth_inch() {
        assert!((MIC_SPEAKER_DESC_FONT_PX - 4.5).abs() < 1e-9);
        assert!((px_to_in(MIC_SPEAKER_DESC_FONT_PX) - 1.0 / 16.0).abs() < 1e-9);
        let style = text_style_for_role(TextRole::SpeakerSecondary);
        assert_eq!(style.height_px, MIC_SPEAKER_DESC_FONT_PX);
    }

    #[test]
    fn text_style_title_uses_node_title_font_px() {
        let style = text_style_for_role(TextRole::Title);
        assert_eq!(style.height_px, NODE_TITLE_FONT_PX);
        assert_eq!(style.font, "Arial Narrow");
    }

    #[test]
    fn estimate_text_width_scales_with_height() {
        let w9 = estimate_text_width_px("ABC", 9.0);
        let w18 = estimate_text_width_px("ABC", 18.0);
        assert!(w18 > w9);
        assert!((w18 / w9 - 2.0).abs() < 1e-9);
        assert_eq!(estimate_text_width_px("", 9.0), 0.0);
        assert_eq!(estimate_text_width_px("  ", 9.0), 0.0);
    }

    #[test]
    fn wiretag_bounds_minimum_width() {
        let bar_h = WIRETAG_BAR_HEIGHT_PX;
        let bounds = wiretag_bounds_diagram_px("", bar_h);
        let index_w = wiretag_index_column_min_width_px(bar_h);
        let tip_w = wiretag_tip_width_px(bar_h);
        let w_min = index_w + tip_w + 10.0;
        let expected = (w_min.ceil() / SNAP_GRID_PX).ceil() * SNAP_GRID_PX;
        assert_eq!(bounds.height, bar_h);
        assert!(
            bounds.width >= expected,
            "width {} should be >= snapped minimum {}",
            bounds.width,
            expected
        );
    }
}
