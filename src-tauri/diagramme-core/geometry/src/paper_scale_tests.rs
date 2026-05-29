#[cfg(test)]
mod tests {
    use crate::paper_scale::{
        snap_placement_coord, snap_placement_half_grid, CONNECTOR_LINE_PITCH_PX,
        DEVICE_V2_WIDTH_PX, MIC_SPEAKER_HANDLE_CENTER_Y_PX, MIC_SPEAKER_FRAME_HEIGHT_PX,
        SNAP_GRID_PX, VOLUME_CONTROL_HANDLE_CENTER_Y_PX, VOLUME_CONTROL_FRAME_HEIGHT_PX,
        PX_PER_INCH,
    };
    use crate::units::px_to_in;

    #[test]
    fn connector_pitch_is_one_eighth_inch() {
        assert_eq!(PX_PER_INCH, 72.0);
        assert_eq!(CONNECTOR_LINE_PITCH_PX, 9.0);
        assert!((px_to_in(CONNECTOR_LINE_PITCH_PX) - 0.125).abs() < 1e-9);
    }

    #[test]
    fn device_v2_width_is_one_point_seventy_five_inches() {
        assert!((px_to_in(DEVICE_V2_WIDTH_PX) - 1.75).abs() < 1e-9);
    }

    #[test]
    fn snap_placement_coord_snaps_to_full_grid() {
        assert_eq!(snap_placement_coord(0.0), 0.0);
        assert_eq!(snap_placement_coord(SNAP_GRID_PX), SNAP_GRID_PX);
        assert_eq!(snap_placement_coord(7.0), 6.0);
        assert_eq!(snap_placement_coord(8.0), 9.0);
        assert_eq!(
            snap_placement_coord(MIC_SPEAKER_FRAME_HEIGHT_PX / 2.0),
            MIC_SPEAKER_HANDLE_CENTER_Y_PX
        );
        assert_eq!(
            snap_placement_coord(VOLUME_CONTROL_FRAME_HEIGHT_PX / 2.0),
            VOLUME_CONTROL_HANDLE_CENTER_Y_PX
        );
    }

    #[test]
    fn snap_placement_half_grid_matches_v6() {
        assert_eq!(snap_placement_half_grid(0.0), 0.0);
        assert_eq!(snap_placement_half_grid(SNAP_GRID_PX), SNAP_GRID_PX);
        assert_eq!(snap_placement_half_grid(SNAP_GRID_PX / 2.0), SNAP_GRID_PX / 2.0);
        assert_eq!(snap_placement_half_grid(7.0), 7.5);
    }
}
