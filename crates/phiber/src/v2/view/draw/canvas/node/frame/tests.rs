//! Frame-layout tests — the bounds maths of `calc_node_frame_dimensions`,
//! split out so `frame.rs` stays inside the 240-line read budget.

use super::*;

fn size() -> BCardSize {
    BCardSize { header: 30.0, unit: 10.0, width: 100.0 }
}

fn frame(address: u64, label: &str) -> BlendNode {
    BlendNode { idname: "NodeFrame".into(), label: label.into(), address, ..Default::default() }
}

fn child(address: u64, parent: u64, x: f32, y: f32) -> BlendNode {
    BlendNode { name: format!("n{address}"), x, y, width: 80.0, address, parent: Some(parent), ..Default::default() }
}

#[test]
fn a_frame_bounds_its_child_padded_by_the_margins() {
    // `calc_node_frame_dimensions` pads each child's rect by the side margin on
    // x and the bottom, and the top margin on top, then unions them. With no
    // label the top margin equals the sides.
    let c = child(1, 99, 0.0, 0.0);
    let f = frame(99, "");
    let nodes = vec![f.clone(), c.clone()];
    let cb = node_bounds(&c, size().height_of(&c), size().width);
    let m = MARGIN * 10.0;
    assert_eq!(
        bounds_of(&f, &nodes, size(), &[], 0),
        Some((cb.0 - m, cb.1 - m, cb.2 + m, cb.3 + m)),
        "unlabelled: the top margin is the side margin",
    );
}

#[test]
fn a_label_makes_the_top_margin_taller_than_the_sides() {
    // `max(margin, 1.5 * label + 0.2 * margin)` — the label needs room, and a
    // reader that used the side margin on top would clip it.
    let (side, top) = margins(true);
    assert!(top > side, "a labelled frame's top margin {top} is taller than its side {side}");
    assert_eq!(margins(false), (MARGIN, MARGIN), "no label, no extra room");

    let c = child(1, 99, 0.0, 0.0);
    let f = frame(99, "Group");
    let nodes = vec![f.clone(), c.clone()];
    let cb = node_bounds(&c, size().height_of(&c), size().width);
    let got = bounds_of(&f, &nodes, size(), &[], 0).unwrap();
    // Y up: the top (`ymax`) carries the taller top margin, the other three the side.
    assert_eq!(got.3, cb.3 + top * 10.0, "top edge uses the top margin");
    assert_eq!(got.1, cb.1 - side * 10.0, "bottom edge uses the side margin");
}

#[test]
fn a_childless_frame_keeps_its_stored_rectangle() {
    // Nothing to bound, so `node_to_updated_rect`: the location is the top-left,
    // Y up, running down and right by the stored size.
    let mut f = frame(99, "");
    f.x = 5.0;
    f.y = 50.0;
    f.width = 40.0;
    f.height = 20.0;
    assert_eq!(bounds_of(&f, &[f.clone()], size(), &[], 0), Some((5.0, 30.0, 45.0, 50.0)));
    // A frame with neither children nor a stored size is nothing to draw.
    assert_eq!(bounds_of(&frame(99, ""), &[], size(), &[], 0), None);
}

#[test]
fn the_box_follows_a_child_that_has_been_dragged() {
    // A frame is sized off where its children are DRAWN, not where the file put
    // them, so a child's displacement shifts the box. This is what makes a
    // frame travel with the node dragged inside it — and what makes dragging the
    // frame itself (which moves its children) move the box at all.
    let c = child(1, 99, 0.0, 0.0);
    let f = frame(99, "");
    let nodes = vec![f.clone(), c.clone()];
    let still = bounds_of(&f, &nodes, size(), &[], 0).unwrap();
    let offsets = vec![("n1".to_string(), (12.0, -7.0))];
    let moved = bounds_of(&f, &nodes, size(), &offsets, 0).unwrap();
    assert_eq!(moved.0, still.0 + 12.0, "left edge follows the child's x");
    assert_eq!(moved.1, still.1 - 7.0, "bottom edge follows the child's y");
    assert_eq!((moved.2 - moved.0, moved.3 - moved.1), (still.2 - still.0, still.3 - still.1), "shape unchanged");
}

#[test]
fn a_frame_inside_a_frame_is_bounded_recursively() {
    // A child frame's own box is what the outer frame bounds — the padding is
    // applied twice, once at each level.
    let inner_child = child(1, 50, 0.0, 0.0);
    // the inner frame (address 50) is itself parented into the outer one (99)
    let mut inner = frame(50, "");
    inner.parent = Some(99);
    let outer = frame(99, "");
    let nodes = vec![outer.clone(), inner.clone(), inner_child.clone()];
    let inner_bounds = bounds_of(&inner, &nodes, size(), &[], 0).unwrap();
    let outer_bounds = bounds_of(&outer, &nodes, size(), &[], 0).unwrap();
    let m = MARGIN * 10.0;
    // The outer bounds are the inner bounds padded once more by the side margin.
    assert_eq!(outer_bounds.0, inner_bounds.0 - m);
    assert_eq!(outer_bounds.2, inner_bounds.2 + m);
}

#[test]
fn a_frame_parented_into_itself_does_not_recurse_forever() {
    // A malformed file could parent a frame into itself. The depth guard turns
    // that into "nothing to bound" rather than a stack overflow.
    let mut f = frame(99, "");
    f.parent = Some(99);
    assert_eq!(bounds_of(&f, &[f.clone()], size(), &[], 0), None, "no stored size, no children it can resolve");
}
