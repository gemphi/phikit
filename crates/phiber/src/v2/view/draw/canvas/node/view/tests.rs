//! View tests — split out so `mod.rs` stays inside the 240-line read budget
//! (`level_structure`), the way `canvas/time/row/` is.

use super::*;

/// A canvas 1000 × 600 pixels.
const REGION: (f32, f32) = (1000.0, 600.0);

/// A view one view-unit per pixel wide, near enough: 1001 units across a
/// 1000px region is zoom 1 exactly, because of Blender's inclusive `mask`.
fn unit_view() -> BNodeView {
    BNodeView { xmin: 0.0, ymin: 0.0, xmax: 1001.0, ymax: 600.0 }
}

#[test]
fn zoom_is_region_pixels_over_view_units() {
    // `(mask_width + 1) / cur_width`. The `+ 1` is Blender's: `mask` is an
    // inclusive pixel rect, so it spans one more pixel than its extent.
    assert_eq!(unit_view().zoom(REGION.0), 1.0);
    let half = BNodeView { xmax: 2002.0, ..unit_view() };
    assert!((half.zoom(REGION.0) - 0.5).abs() < 1e-4, "{}", half.zoom(REGION.0));
}

#[test]
fn the_y_flip_happens_once_and_only_here() {
    // View units grow up, canvas pixels grow down. The file's coordinates are
    // never pre-flipped; this is the one place it turns over.
    let view = unit_view();
    // The view's TOP maps to pixel row 0.
    assert_eq!(view.to_region((0.0, view.ymax), REGION).1, 0.0);
    // Its bottom maps to the bottom of the region.
    assert_eq!(view.to_region((0.0, view.ymin), REGION).1, 600.0);
    // And x is not flipped.
    assert_eq!(view.to_region((0.0, 0.0), REGION).0, 0.0);
}

#[test]
fn a_point_survives_the_round_trip_through_pixels() {
    let view = BNodeView { xmin: -40.0, ymin: 12.0, xmax: 360.0, ymax: 252.0 };
    for at in [(0.0, 0.0), (100.0, 100.0), (-35.0, 250.0), (359.0, 13.0)] {
        let back = view.to_view(view.to_region(at, REGION), REGION);
        assert!(
            (back.0 - at.0).abs() < 1e-2 && (back.1 - at.1).abs() < 1e-2,
            "{at:?} came back as {back:?}",
        );
    }
}

#[test]
fn panning_moves_the_view_against_the_pointer() {
    // The content follows the pointer, so the window onto it goes the other
    // way. Dragging right shows what was to the LEFT.
    let view = unit_view();
    let right = view.pan((100.0, 0.0), REGION);
    assert!(right.xmin < view.xmin, "dragging right moves the view left");
    // And the view keeps its size — a pan is not a zoom.
    assert!((right.width() - view.width()).abs() < 1e-4);
    // Y is flipped, so dragging down raises the view.
    let down = view.pan((0.0, 100.0), REGION);
    assert!(down.ymin > view.ymin, "dragging down shows what was above");
}

#[test]
fn panning_is_in_view_units_so_it_tracks_the_pointer_at_any_zoom() {
    // The pixel delta is divided by the zoom, which is what makes the content
    // stay under the finger rather than sliding faster when zoomed out.
    let close = unit_view();
    let far = BNodeView { xmax: 2002.0, ..unit_view() };
    let moved_close = close.xmin - close.pan((100.0, 0.0), REGION).xmin;
    let moved_far = far.xmin - far.pan((100.0, 0.0), REGION).xmin;
    assert!(
        (moved_far - moved_close * 2.0).abs() < 1e-2,
        "at half the zoom the same drag covers twice the view units: {moved_close} vs {moved_far}",
    );
}

#[test]
fn zooming_out_exactly_undoes_zooming_in() {
    // The whole point of Blender's asymmetric negative branch: "ensuring
    // symmetry so the old zoom level is restored after zooming back the same
    // amount". The obvious `(1 + 2f)` would drift further out every round trip.
    let view = unit_view();
    let there = view.zoom_step(ZOOM_STEP, None, REGION);
    let back = there.zoom_step(-ZOOM_STEP, None, REGION);
    assert!((back.width() - view.width()).abs() < 1e-3, "{} vs {}", back.width(), view.width());
    assert!((back.height() - view.height()).abs() < 1e-3);
    // Ten notches each way, to show it does not accumulate.
    let mut wandered = view;
    for _ in 0..10 {
        wandered = wandered.zoom_step(ZOOM_STEP, None, REGION);
    }
    for _ in 0..10 {
        wandered = wandered.zoom_step(-ZOOM_STEP, None, REGION);
    }
    assert!(
        (wandered.width() - view.width()).abs() < 1e-2,
        "twenty notches drifted to {}",
        wandered.width(),
    );
}

#[test]
fn one_notch_in_shrinks_the_view_by_twice_the_factor() {
    // `cur.xmin += dx; cur.xmax -= dx` with `dx = width * fac`, so the width
    // loses `2 * fac` of itself.
    let view = unit_view();
    let closer = view.zoom_step(ZOOM_STEP, None, REGION);
    let expected = view.width() * (1.0 - 2.0 * ZOOM_STEP);
    assert!((closer.width() - expected).abs() < 1e-3, "{} vs {expected}", closer.width());
}

#[test]
fn zooming_to_the_cursor_keeps_what_is_under_it_under_it() {
    let view = unit_view();
    let focus = (250.0, 150.0);
    let was = view.to_view(focus, REGION);
    let closer = view.zoom_step(ZOOM_STEP, Some(focus), REGION);
    let now = closer.to_view(focus, REGION);
    assert!(
        (now.0 - was.0).abs() < 1e-2 && (now.1 - was.1).abs() < 1e-2,
        "the point under the cursor moved from {was:?} to {now:?}",
    );
}

#[test]
fn the_zoom_limits_refuse_the_step_rather_than_clamping_it() {
    // `V2D_LIMITZOOM`. Clamping would let a wheel at the limit nudge the
    // framing sideways without changing the zoom, which reads as a glitch.
    let region = REGION;
    // Wound all the way in, the next notch is refused.
    let mut tight = unit_view();
    for _ in 0..200 {
        tight = tight.zoom_step(ZOOM_STEP, None, region);
    }
    assert!(tight.zoom(region.0) <= MAX_ZOOM, "{}", tight.zoom(region.0));
    assert_eq!(tight.zoom_step(ZOOM_STEP, None, region), tight, "refused, not clamped");
    // And all the way out.
    let mut loose = unit_view();
    for _ in 0..200 {
        loose = loose.zoom_step(-ZOOM_STEP, None, region);
    }
    assert!(loose.zoom(region.0) >= MIN_ZOOM, "{}", loose.zoom(region.0));
    assert_eq!(loose.zoom_step(-ZOOM_STEP, None, region), loose);
}

#[test]
fn the_limits_are_blenders_own_numbers() {
    // 2.31 is not a round number in Blender either, and a tidied-up 2.0 would
    // be a different editor at the far end of the wheel.
    assert_eq!(MIN_ZOOM, 0.05);
    assert_eq!(MAX_ZOOM, 2.31);
    assert_eq!(ZOOM_STEP, 0.0375);
}

#[test]
fn a_new_views_window_is_not_centred_on_the_origin() {
    // `tot` runs -12.8 to 38.4 widget units, and the asymmetry is the point:
    // a new tree's nodes sit at the origin and grow right and up, so the view
    // opens with more room ahead of them than behind.
    let view = BNodeView::default();
    assert_eq!((view.xmin, view.xmax), (-12.8, 38.4));
    assert_eq!((view.ymin, view.ymax), (-12.8, 38.4));
    assert!(view.xmax.abs() > view.xmin.abs(), "more room ahead than behind");
}

#[test]
fn zooming_out_brings_more_of_the_graph_into_the_canvas() {
    // What the wiring is for, stated as the user would: enough notches out and
    // a node that was off the right edge is on screen.
    let view = BNodeView { xmin: 0.0, ymin: 0.0, xmax: 100.0, ymax: 100.0 };
    let off_screen = (130.0, 50.0);
    assert!(view.to_fraction(off_screen).0 > 1.0, "the node starts off the canvas");
    let mut out = view;
    for _ in 0..60 {
        out = out.zoom_step(-ZOOM_STEP, None, REGION);
    }
    assert!(out.to_fraction(off_screen).0 <= 1.0, "zoomed out it should be on screen");
}

#[test]
fn a_view_tighter_than_the_zoom_limit_can_still_be_zoomed_out_of() {
    // The bug the test above found. A 100-unit view on a 1000px canvas is zoom
    // 10, five times `MAX_ZOOM` — and the limit check refused any step whose
    // RESULT was out of range, which is every step from there. The wheel was
    // dead in exactly the case you most want it, and an opening framing of a
    // small graph lands there.
    let tight = BNodeView { xmin: 0.0, ymin: 0.0, xmax: 100.0, ymax: 100.0 };
    assert!(tight.zoom(REGION.0) > MAX_ZOOM, "{}", tight.zoom(REGION.0));
    let out = tight.zoom_step(-ZOOM_STEP, None, REGION);
    assert!(out.width() > tight.width(), "a step back toward the limits is allowed");
    // Further IN from out of range is still refused — the escape is one way.
    assert_eq!(tight.zoom_step(ZOOM_STEP, None, REGION), tight);
    // And it lands inside the range rather than overshooting out the far side.
    let mut wound = tight;
    for _ in 0..200 {
        wound = wound.zoom_step(-ZOOM_STEP, None, REGION);
    }
    assert!((MIN_ZOOM..=MAX_ZOOM).contains(&wound.zoom(REGION.0)), "{}", wound.zoom(REGION.0));
}
