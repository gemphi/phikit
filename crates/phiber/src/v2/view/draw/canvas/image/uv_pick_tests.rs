//! `uv_pick.rs`'s own geometry and decisions, without a window: the nearest
//! corner within the radius, what a press does to the selection, and what a
//! drag does to the offsets. The gesture's handlers are three lines over
//! these, so this is where the behaviour is.

use super::uv_pick::BUvGesture;
use crate::phiphx::scene::SceneUvMap;
use crate::v2::part::image::uv::BUvShown;
use crate::v2::scene::uv_edit::BUvEdit;
use gpui::{Bounds, point, px, size};
use std::cell::Cell;
use std::rc::Rc;

/// One quad over the whole square: corners at the four corners of 0..1.
fn quad() -> BUvShown {
    BUvShown {
        edit: true,
        maps: vec![std::sync::Arc::new(SceneUvMap {
            starts: vec![0, 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        })],
    }
}

/// A gesture over a 100 × 100 square at the origin, with `edit` in scope.
fn over(edit: BUvEdit) -> (BUvGesture, Bounds<gpui::Pixels>) {
    let square = Bounds::new(point(px(0.0), px(0.0)), size(px(100.0), px(100.0)));
    let shown = Rc::new(quad());
    let gesture = BUvGesture {
        edits: true,
        shown,
        edit,
        square: Rc::new(Cell::new(Some(square))),
        drag: None,
        handler: Rc::new(|_, _, _| {}),
    };
    (gesture, square)
}

#[test]
fn the_nearest_corner_inside_the_radius_is_the_one_picked_and_outside_it_none_is() {
    let (g, square) = over(BUvEdit::default());
    // `v` runs up, so UV (0, 0) is the square's *bottom* left — y = 100.
    assert_eq!(g.nearest(square, point(px(2.0), px(99.0))), Some((0, 0)), "near UV (0,0)");
    assert_eq!(g.nearest(square, point(px(99.0), px(2.0))), Some((0, 2)), "near UV (1,1)");
    // The middle of the square is 50 px from every corner: nothing within
    // the radius, so a press there selects nothing rather than the least
    // far thing.
    assert_eq!(g.nearest(square, point(px(50.0), px(50.0))), None, "the middle is no corner");
    // Between two corners, the nearer one wins.
    assert_eq!(g.nearest(square, point(px(4.0), px(94.0))), Some((0, 0)));
}

#[test]
fn a_moved_corner_is_picked_where_it_was_moved_to_not_where_the_file_put_it() {
    // Corner 0 moved to the middle of the square.
    let edit = BUvEdit { selected: vec![(0, 0)], moved: vec![((0, 0), [0.5, 0.5])], pinned: Vec::new() };
    let (g, square) = over(edit);
    assert_eq!(g.nearest(square, point(px(50.0), px(50.0))), Some((0, 0)), "it is where it was moved to");
    assert_eq!(g.nearest(square, point(px(2.0), px(99.0))), None, "and no longer where the file put it");
}

#[test]
fn a_press_picks_and_a_shift_press_extends_while_a_drag_moves_every_selected_corner() {
    // What the handlers do, as the store's own operations — the gesture is
    // `nearest` then `picked`, and `nearest` then `nudged`.
    let (g, square) = over(BUvEdit::default());
    let corner = g.nearest(square, point(px(2.0), px(99.0))).expect("a corner");
    assert_eq!(g.edit.picked(corner, false), [(0, 0)]);

    let edit = BUvEdit { selected: vec![(0, 0), (0, 1)], moved: Vec::new(), pinned: Vec::new() };
    let (g, _) = over(edit);
    // A drag of 25 px right and 50 px up over a 100 px square is +0.25 u
    // and +0.5 v — `v` runs the other way to `y`, which is the sign the
    // square's own mapping decides.
    let moved = g.edit.nudged([0.25, 0.5]);
    assert_eq!(moved.len(), 2, "both selected corners move; the unselected two do not");
    assert_eq!(moved[0], ((0, 0), [0.25, 0.5]));
    assert!(!moved.iter().any(|(c, _)| *c == (0, 2)), "corner 2 was not selected");
}
