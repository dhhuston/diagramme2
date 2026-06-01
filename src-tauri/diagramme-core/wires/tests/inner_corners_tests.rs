use diagramme_wires::{reflow_inner_corners_for_stub_move, translate_inner_corners, FlowXY};

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

#[test]
fn reflow_inner_corners_stretches_source_leg_not_bus_x() {
    // S1 --horizontal--> c1 --vertical--> c2 --horizontal--> T1
    let corners = vec![
        FlowXY { x: 150.0, y: 100.0 },
        FlowXY { x: 150.0, y: 50.0 },
    ];
    let prev_s1 = FlowXY { x: 100.0, y: 100.0 };
    let prev_t1 = FlowXY { x: 200.0, y: 50.0 };
    let new_s1 = FlowXY { x: 100.0, y: 130.0 };
    let new_t1 = prev_t1;
    let out = reflow_inner_corners_for_stub_move(&corners, prev_s1, prev_t1, new_s1, new_t1);
    assert_eq!(out[0], FlowXY { x: 150.0, y: 129.0 });
    assert_eq!(out[1], FlowXY { x: 150.0, y: 50.0 });
}

#[test]
fn reflow_inner_corners_stretches_target_leg_not_bus_y() {
    // corner --horizontal--> T1
    let corners = vec![FlowXY { x: 100.0, y: 51.0 }];
    let prev_s1 = FlowXY { x: 100.0, y: 100.0 };
    let prev_t1 = FlowXY { x: 200.0, y: 51.0 };
    let new_t1 = FlowXY { x: 170.0, y: 51.0 };
    let out = reflow_inner_corners_for_stub_move(
        &corners,
        prev_s1,
        prev_t1,
        prev_s1,
        new_t1,
    );
    assert_eq!(out[0], FlowXY { x: 171.0, y: 51.0 });
}
