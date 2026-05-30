use diagramme_wires::{translate_inner_corners, FlowXY};

#[test]
fn translate_inner_corners_same_motion_shifts_all() {
    let corners = vec![FlowXY { x: 100.0, y: 200.0 }];
    let out = translate_inner_corners(
        &corners,
        FlowXY { x: 6.0, y: 0.0 },
        FlowXY { x: 6.0, y: 0.0 },
        None,
        None,
    );
    assert_eq!(out, vec![FlowXY { x: 105.0, y: 201.0 }]);
}

#[test]
fn translate_inner_corners_interpolates_asymmetric_stub_motion() {
    let corners = vec![FlowXY { x: 150.0, y: 200.0 }];
    let prev_s1 = FlowXY { x: 100.0, y: 200.0 };
    let prev_t1 = FlowXY { x: 200.0, y: 200.0 };
    let out = translate_inner_corners(
        &corners,
        FlowXY { x: 10.0, y: 0.0 },
        FlowXY { x: 0.0, y: 0.0 },
        Some(prev_s1),
        Some(prev_t1),
    );
    // Corner at midpoint (t=0.5) gets half the source delta.
    assert_eq!(out, vec![FlowXY { x: 156.0, y: 201.0 }]);
}
