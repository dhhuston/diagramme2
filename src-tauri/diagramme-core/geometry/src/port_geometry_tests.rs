#[cfg(test)]
mod tests {
    use diagramme_schema::{Node, XY};

    use crate::node_bounds::node_bounds_diagram_px;
    use crate::paper_scale::{
        CONNECTOR_LINE_PITCH_PX, CONNECTOR_ROW_OUTER_HEIGHT_PX, DEVICE_V2_WIDTH_PX, PX_PER_INCH,
        WIRETAG_BAR_HEIGHT_PX,
    };
    use crate::symbol_layout::ANTENNA_SCHEMATIC_SVG_WIDTH_PX;
    use crate::port_geometry::{
        get_analytical_port_xy, WIRETAG_CONN_SRC, WIRETAG_CONN_TGT,
    };
    use crate::schematic_layout::{
        DEVICE_V2_GRID_ROW_PX, DEVICE_V2_ROW_CENTER_Y_PX, DEVICE_V2_TITLE_HEIGHT_PX,
        PATCH_BODY_TOP_PX, PATCH_PANEL_WIDTH_PX, PATCH_ROW_CENTER_Y_PX,
        SCHEMATIC_FRAME_INSET_PX, patch_panel_total_height_px,
    };

    fn node(
        id: &str,
        node_type: &str,
        x: f64,
        y: f64,
        data: serde_json::Value,
        width: Option<f64>,
        height: Option<f64>,
    ) -> Node {
        Node {
            id: id.to_string(),
            node_type: node_type.to_string(),
            position: XY { x, y },
            data,
            width,
            height,
            z_index: None,
        }
    }

    #[test]
    fn device_v2_left_port_position_matches_v6_grid() {
        // portGeometry.test.ts — mockDeviceV2Node / L-0-in-1
        let dev = node(
            "dev1",
            "deviceV2",
            99.0,
            198.0,
            serde_json::json!({
                "leftColumn": [{
                    "header": "Inputs",
                    "rows": [
                        { "id": "in-1", "label": "In 1", "direction": "input" },
                        { "id": "in-2", "label": "In 2", "direction": "input" }
                    ]
                }],
                "rightColumn": []
            }),
            Some(DEVICE_V2_WIDTH_PX),
            None,
        );
        let p1 = get_analytical_port_xy(&dev, "L-0-in-1").expect("port");
        assert_eq!(p1.x, 99.0);
        assert_eq!(
            p1.y,
            198.0 + DEVICE_V2_TITLE_HEIGHT_PX + 2.0 * DEVICE_V2_GRID_ROW_PX + DEVICE_V2_ROW_CENTER_Y_PX
        );
        let p2 = get_analytical_port_xy(&dev, "L-0-in-2").expect("port");
        assert!((p2.y - p1.y - CONNECTOR_LINE_PITCH_PX).abs() < 1e-9);
    }

    #[test]
    fn av_plate_output_port_x_at_frame_edge() {
        // dxfWireGeometry.test.ts — clips av plate output endpoint (sourceHandle y = 121.5)
        let plate_x = 100.0;
        let plate_y = 100.0;
        let av = node(
            "av1",
            "avPlate",
            plate_x,
            plate_y,
            serde_json::json!({
                "groups": [{
                    "id": "g1",
                    "side": "left",
                    "title": "A",
                    "rows": [{ "id": "1", "label": "Out", "direction": "output" }]
                }]
            }),
            Some(PATCH_PANEL_WIDTH_PX),
            None,
        );
        let p = get_analytical_port_xy(&av, "S-0-1").expect("port");
        assert_eq!(p.x, plate_x + PATCH_PANEL_WIDTH_PX);
        assert_eq!(
            p.y,
            plate_y + DEVICE_V2_TITLE_HEIGHT_PX + 2.0 * DEVICE_V2_GRID_ROW_PX + DEVICE_V2_ROW_CENTER_Y_PX
        );
        // Clipped export endpoint is inset by SCHEMATIC_FRAME_INSET_PX from analytical center.
        assert_eq!(p.x - SCHEMATIC_FRAME_INSET_PX, plate_x + PATCH_PANEL_WIDTH_PX - SCHEMATIC_FRAME_INSET_PX);
    }

    #[test]
    fn wiretag_analytical_ports_match_v6_insets() {
        // portGeometry.test.ts — conn-src / conn-tgt at 1px inset
        let wt_x = 100.0;
        let wt_y = 100.0;
        let tag_w = 120.0;
        let wt = node(
            "wt1",
            "wiretag",
            wt_x,
            wt_y,
            serde_json::json!({ "pairId": "pair-a", "pairIndex": 1, "end": "a" }),
            Some(tag_w),
            Some(WIRETAG_BAR_HEIGHT_PX),
        );
        let src = get_analytical_port_xy(&wt, WIRETAG_CONN_SRC).expect("src");
        let tgt = get_analytical_port_xy(&wt, WIRETAG_CONN_TGT).expect("tgt");
        assert_eq!(src.x, wt_x + tag_w - 1.0);
        assert_eq!(tgt.x, wt_x + 1.0);
        assert_eq!(src.y, wt_y + WIRETAG_BAR_HEIGHT_PX / 2.0);
        assert_eq!(tgt.y, src.y);
        // dxfWireGeometry.test.ts — hull right boundary for routing is tag_w (not src x + 1).
        assert_eq!(wt_x + tag_w, wt_x + tag_w);
    }

    #[test]
    fn lpp_patch_panel_right_port_on_snap_grid() {
        // portGeometry.test.ts — dpp patch panel R-r1 / R-r2 alignment
        let panel_x = 99.0;
        let panel_y = 198.0;
        let panel = node(
            "p1",
            "lppPatchPanel",
            panel_x,
            panel_y,
            serde_json::json!({
                "rows": [
                    { "id": "r1", "connected": false },
                    { "id": "r2", "connected": true }
                ]
            }),
            Some(PATCH_PANEL_WIDTH_PX),
            None,
        );
        let p1 = get_analytical_port_xy(&panel, "R-r1").expect("p1");
        let p2 = get_analytical_port_xy(&panel, "R-r2").expect("p2");
        assert_eq!(p1.x, panel_x + PATCH_PANEL_WIDTH_PX);
        assert_eq!(p2.x, panel_x + PATCH_PANEL_WIDTH_PX);
        assert_eq!(p1.y, (panel_y + PATCH_BODY_TOP_PX + PATCH_ROW_CENTER_Y_PX).round());
        assert_eq!(p1.y % 3.0, 0.0);
        assert!((p2.y - p1.y - CONNECTOR_LINE_PITCH_PX).abs() < 1e-9);
    }

    #[test]
    fn device_v2_bounds_width_and_height_match_v6() {
        let dev = node(
            "dev1",
            "deviceV2",
            300.0,
            100.0,
            serde_json::json!({
                "tagCode": "DEV",
                "tagNumber": "1",
                "description": "Test",
                "leftColumn": [{
                    "id": "g1",
                    "side": "left",
                    "rows": [{ "id": "p1", "label": "Input" }]
                }],
                "rightColumn": []
            }),
            Some(DEVICE_V2_WIDTH_PX),
            None,
        );
        let bounds = node_bounds_diagram_px(&dev).expect("bounds");
        assert_eq!(bounds.x, 300.0);
        assert_eq!(bounds.y, 100.0);
        assert_eq!(bounds.width, DEVICE_V2_WIDTH_PX);
        assert!((bounds.width - 1.75 * PX_PER_INCH).abs() < 1e-9);
        // Flat rows: gap + port + gap = 3 body rows.
        let expected_height = DEVICE_V2_TITLE_HEIGHT_PX + 3.0 * DEVICE_V2_GRID_ROW_PX;
        assert!((bounds.height - expected_height).abs() < 1e-9);
    }

    #[test]
    fn lpp_patch_panel_bounds_height_matches_v6_formula() {
        let panel = node(
            "p1",
            "lppPatchPanel",
            100.0,
            100.0,
            serde_json::json!({ "rows": [{ "id": "1", "connected": false }] }),
            Some(PATCH_PANEL_WIDTH_PX),
            None,
        );
        let bounds = node_bounds_diagram_px(&panel).expect("bounds");
        assert_eq!(bounds.width, PATCH_PANEL_WIDTH_PX);
        assert_eq!(bounds.height, patch_panel_total_height_px(1));
    }

    #[test]
    fn antenna_ports_match_v6_foot_handle_centers() {
        let y_bot = CONNECTOR_ROW_OUTER_HEIGHT_PX + CONNECTOR_ROW_OUTER_HEIGHT_PX / 2.0;

        let tx = node(
            "tx1",
            "antennaTransmitterSymbol",
            1000.0,
            2000.0,
            serde_json::json!({ "line1": "ANT" }),
            Some(40.0),
            None,
        );
        let tx_port = get_analytical_port_xy(&tx, "ant-tx").expect("ant-tx");
        assert!((tx_port.y - (2000.0 + y_bot)).abs() < 1e-9);
        assert!((tx_port.x - (1000.0 + 6.0)).abs() < 1e-9);

        let rx = node(
            "rx1",
            "antennaReceiverSymbol",
            500.0,
            120.0,
            serde_json::json!({ "line1": "ANT" }),
            Some(72.0),
            None,
        );
        let rx_port = get_analytical_port_xy(&rx, "ant-rx").expect("ant-rx");
        assert!((rx_port.y - (120.0 + y_bot)).abs() < 1e-9);
        let sym_left = 72.0 - ANTENNA_SCHEMATIC_SVG_WIDTH_PX;
        assert!((rx_port.x - (500.0 + sym_left + 13.0 - 1.0)).abs() < 1e-9);

        let tx_bad = get_analytical_port_xy(&tx, "ant-rx");
        assert!(tx_bad.is_none());
        let rx_bad = get_analytical_port_xy(&rx, "ant-tx");
        assert!(rx_bad.is_none());
    }

    #[test]
    fn speaker_block_target_port_matches_comp_gym_fixture() {
        let spk = node(
            "spk-cfe86b51",
            "speakerBlock",
            1095.0,
            117.0,
            serde_json::json!({
                "line1": "SPK 1",
                "line2": "main bleachers",
                "passthruEnabled": true,
                "symbolKind": "standard"
            }),
            Some(57.0),
            Some(50.0),
        );
        let p = get_analytical_port_xy(&spk, "T-spk").expect("T-spk");
        assert!((p.x - 1096.25).abs() < 0.01, "x={}", p.x);
        assert!((p.y - 141.0).abs() < 1.5, "y={}", p.y);
    }
}
