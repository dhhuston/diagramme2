use diagramme_wires::{
    drag_horizontal_inner_segment, inner_corners_from_chain, inner_segment_midpoints, FlowXY,
};

#[test]
fn inner_corners_from_chain_excludes_endpoints() {
    let chain = vec![
        FlowXY { x: 0.0, y: 0.0 },
        FlowXY { x: 100.0, y: 0.0 },
        FlowXY { x: 100.0, y: 50.0 },
        FlowXY { x: 200.0, y: 50.0 },
    ];
    assert_eq!(inner_corners_from_chain(&chain).len(), 2);
}

#[test]
fn inner_segment_midpoints_classifies_horizontal_and_vertical() {
    let chain = vec![
        FlowXY { x: 0.0, y: 0.0 },
        FlowXY { x: 100.0, y: 0.0 },
        FlowXY { x: 100.0, y: 50.0 },
    ];
    let grips = inner_segment_midpoints(&chain);
    assert_eq!(grips.len(), 2);
    assert_eq!(grips[0].orientation.as_str(), "h");
    assert_eq!(grips[1].orientation.as_str(), "v");
}

#[test]
fn drag_horizontal_inner_segment_moves_interior_row() {
    let chain = vec![
        FlowXY { x: 0.0, y: 0.0 },
        FlowXY { x: 0.0, y: 50.0 },
        FlowXY { x: 100.0, y: 50.0 },
        FlowXY { x: 100.0, y: 100.0 },
    ];
    let next = drag_horizontal_inner_segment(&chain, 1, 9.0);
    assert!(next[2].y > chain[2].y);
}
