//! The drawn scroller stays inside the gutter it rides in.
//!
//! Blender swells its scroller under the pointer (`UI_view2d_scrollers_draw`
//! draws a wider bar while the mouse is over the region). That works there
//! because its scroller owns a strip of its own; here the thumb rides the
//! `pad_md` the content is already held off the edge by, and swelling it to
//! `V2D_SCROLL_WIDTH` put 1.5px of thumb over the content on one side and the
//! same over the area's border on the other — an edge moving under the
//! pointer, which is the one thing a scrollbar must not do.
//!
//! So the thumb has one width, narrower than the gutter, and the pointer
//! changes its colour instead. These pin the geometry that makes that true;
//! `view/draw/scroller.rs` is the only caller.

use phiber::v2::theme::style::metric::DEFAULT_SPACING;
use phiber::v2::view::scroll;

#[test]
fn the_thumb_is_two_pixels_under_the_inset_bar() {
    // The bar is Blender's; the thumb is that less its inset either side,
    // and two pixels thinner again.
    assert_eq!(scroll::SCROLL_WIDTH, 11.0);
    assert_eq!(scroll::THUMB_INSET, 2.0);
    assert_eq!(scroll::THUMB_WIDTH, scroll::SCROLL_WIDTH - 2.0 * scroll::THUMB_INSET - 2.0);
    assert_eq!(scroll::THUMB_WIDTH, 5.0);
}

#[test]
fn the_thumb_clears_both_edges_of_its_gutter() {
    // The scroller is drawn about the centre line of the gutter, so it clears
    // each edge by half of what is left over. Nothing of it reaches the
    // content on one side or the area's border on the other.
    let gutter = DEFAULT_SPACING.pad_md;
    assert!(
        scroll::THUMB_WIDTH < gutter,
        "the thumb is {} in a {gutter} gutter — it would touch its edges",
        scroll::THUMB_WIDTH
    );
    let clearance = (gutter - scroll::THUMB_WIDTH) / 2.0;
    assert!(clearance >= 1.0, "only {clearance} clear either side");
}

#[test]
fn the_thumb_never_swells_under_the_pointer() {
    // One width, whatever the pointer is doing: the paint code reads
    // `THUMB_WIDTH` once and branches only on colour and alpha. A `near`
    // branch that picked a width is what this is here to stop coming back.
    let source = include_str!("../src/v2/view/draw/scroller.rs");
    let width_line = source
        .lines()
        .find(|line| line.trim_start().starts_with("let width ="))
        .expect("the scroller computes one width");
    assert!(
        !width_line.contains("if near"),
        "the thumb's width must not depend on the pointer: {width_line}"
    );
    assert!(
        width_line.contains("THUMB_WIDTH"),
        "the thumb is drawn at THUMB_WIDTH: {width_line}"
    );
    // The pointer still says something — it lights the thumb.
    assert!(source.contains("if near { lit } else { colour }"), "the pointer changes the thumb's colour");
}
