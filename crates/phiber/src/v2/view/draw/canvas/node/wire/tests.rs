//! Gesture tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.
//!
//! What is testable without a window: the arithmetic the handlers do before
//! they touch GPUI. The press, move and release themselves need a `Window` and
//! an `App`, so they are covered by the integration test
//! `tests/a_link_is_dragged_between_sockets.rs`, which runs a real frame.

use super::*;

#[test]
fn the_hit_test_scale_is_one_until_the_canvas_zooms() {
    // Named rather than spelled inline so the day the canvas pans and zooms,
    // `node_find_indicated_socket`'s `20.0 / view2d_scale` has one place to
    // read its scale from.
    assert_eq!(SCALE, 1.0);
    assert_eq!(pick::max_distance(20.0, SCALE), pick::max_distance(20.0, 1.0));
}

#[test]
fn an_anchor_on_an_output_looks_for_an_input_and_the_reverse() {
    // The direction rule, as the release will apply it.
    let from_output = BLinking {
        anchor: BLinkEnd::new("a", "Out"),
        from_output: true,
        at: (0.5, 0.5),
        moving: None,
    };
    assert_eq!(from_output.wants(), BSocketSide::Input);
    assert_eq!(BLinking { from_output: false, ..from_output }.wants(), BSocketSide::Output);
}
