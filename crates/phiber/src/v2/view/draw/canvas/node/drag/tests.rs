//! Drag tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.

use super::*;

#[test]
fn a_displacement_round_trips() {
    for offset in [(0.0, 0.0), (0.25, -0.125), (-0.5, 0.5)] {
        assert_eq!(decode(&encode(offset)), Some(offset), "{offset:?}");
    }
    assert_eq!(decode("banana"), None, "nonsense is no displacement, not a panic");
    assert_eq!(decode("0.5"), None, "a pair needs both halves");
}

#[test]
fn a_node_moves_by_the_delta_it_was_dragged() {
    // `td.iloc` + offset: the card keeps the relationship to the pointer it
    // had at the press.
    let offset = moved((0.0, 0.0), (40.0, -20.0));
    assert_eq!(offset, (40.0, -20.0));
    // A second drag starts from where the first left it.
    assert_eq!(moved(offset, (40.0, 0.0)).0, 80.0);
}

#[test]
fn a_pointer_delta_becomes_view_units_with_y_turned_over() {
    // The one place in the drag that knows which way Y runs. The canvas grows
    // down and `bNode.location` grows up, so a card dragged DOWN the screen
    // lowers its node's `y`.
    let view = BNodeView { xmin: 0.0, ymin: 0.0, xmax: 400.0, ymax: 300.0 };
    let delta = in_view_units((0.25, 0.5), &view);
    assert_eq!(delta.0, 100.0, "a quarter of a 400-unit view is 100 units");
    assert_eq!(delta.1, -150.0, "dragging down lowers y");
    // And at half the zoom the same travel covers twice the units, which is
    // what keeps the card under the cursor however far you are zoomed out.
    let far = BNodeView { xmax: 800.0, ..view };
    assert_eq!(in_view_units((0.25, 0.0), &far).0, 200.0);
}

#[test]
fn a_node_can_be_dragged_off_the_canvas_now_that_the_view_pans() {
    // The clamp that used to live here existed because a node past the edge of
    // a canvas with no pan was gone for good. Blender allows it for exactly the
    // reason phiber now can: you can pan back to it.
    let base = (100.0, 100.0);
    let far = placed(base, moved((0.0, 0.0), (9000.0, 9000.0)));
    assert_eq!(far, (9100.0, 9100.0), "nothing clamps it back into a band");
    let behind = placed(base, moved((0.0, 0.0), (-9000.0, -9000.0)));
    assert_eq!(behind, (-8900.0, -8900.0), "negative coordinates are ordinary node space");
}

#[test]
fn a_nonsense_displacement_leaves_the_node_where_the_file_put_it() {
    // A key edited by hand into something unrepresentable must not send a card
    // to infinity, where no pan would ever find it again.
    let base = (100.0, 50.0);
    assert_eq!(placed(base, (f32::INFINITY, 0.0)), base);
    assert_eq!(placed(base, (0.0, f32::NAN)), base);
}

#[test]
fn an_undragged_node_sits_exactly_where_the_file_put_it() {
    let base = (300.0, 400.0);
    assert_eq!(
        placed(base, offset_of(&BState::new(), BHostType::Shader, "Principled BSDF")),
        base,
        "no entry in state is no displacement"
    );
}

#[test]
fn a_drags_grab_origin_is_taken_on_the_first_move_and_not_retaken() {
    let mut drag = BNodeDrag::new("Principled BSDF", (0.0, 0.0));
    assert_eq!(drag.grabbed, None, "the press cannot know the canvas");
    assert_eq!(*drag.grabbed.get_or_insert((0.4, 0.4)), (0.4, 0.4));
    assert_eq!(*drag.grabbed.get_or_insert((0.9, 0.9)), (0.4, 0.4), "measured from one origin");
}
