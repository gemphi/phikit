//! Gesture tests — split out so `mod.rs` stays inside the 240-line read
//! budget (`level_structure`), the way `canvas/time/row/` is.
//!
//! What is testable without a window: the arithmetic the handlers do before
//! they touch GPUI. The press, move and release themselves need a `Window` and
//! an `App`, so they are covered by the integration test
//! `tests/a_link_is_dragged_between_sockets.rs`, which runs a real frame.

use super::*;

#[test]
fn the_hit_tests_slack_now_answers_to_the_zoom() {
    // `node_find_indicated_socket` divides twenty screen pixels by
    // `view2d_scale`, so the grab area stays the same size under the pointer
    // however far you have zoomed out. That scale was pinned at 1 while the
    // canvas could not zoom; it is `BNodeView::zoom` now, and this is the fact
    // that pinning hid: zoomed out, a socket's grab reach covers MORE view
    // units, because the same screen pixels are worth more of them.
    let wide = pick::max_distance(20.0, 0.5);
    let close = pick::max_distance(20.0, 2.0);
    assert!(wide > close, "zoomed out the slack grows: {wide} vs {close}");
    // And the clamp still holds it between a quarter of a unit and one and a
    // half, so neither extreme runs away.
    assert_eq!(pick::max_distance(20.0, 1e-6), pick::max_distance(20.0, 0.001));
    assert_eq!(pick::max_distance(20.0, 1e6), pick::max_distance(20.0, 1000.0));
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
