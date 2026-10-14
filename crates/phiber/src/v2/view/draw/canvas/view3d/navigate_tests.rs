//! `BViewNavigate` against Blender's own Navigation keymap.

use super::*;
use gpui::{Modifiers, Point, ScrollDelta, TouchPhase, px};

fn pixels(x: f32, y: f32) -> ScrollDelta {
    ScrollDelta::Pixels(Point::new(px(x), px(y)))
}
fn lines(y: f32) -> ScrollDelta {
    ScrollDelta::Lines(Point::new(0.0, y))
}
fn drag(delta: ScrollDelta, modifiers: Modifiers) -> Option<BViewMove> {
    BViewNavigate::scroll(delta, modifiers, TouchPhase::Moved)
}

#[test]
fn a_pinch_zooms_because_trackpadzoom_maps_to_view3d_zoom() {
    // Spreading the fingers is a positive magnification and zooms in.
    assert_eq!(BViewNavigate::pinch(0.2), Some(BViewMove::Zoom { factor: 1.2 }));
    // Closing them zooms out.
    assert_eq!(BViewNavigate::pinch(-0.25), Some(BViewMove::Zoom { factor: 0.75 }));
    // A pinch of nothing is nothing.
    assert_eq!(BViewNavigate::pinch(0.0), None);
}

#[test]
fn a_wheel_notch_zooms_a_step_either_way() {
    assert_eq!(drag(lines(1.0), Modifiers::none()), Some(BViewMove::Zoom { factor: 1.1 }));
    let out = drag(lines(-1.0), Modifiers::none()).expect("a notch down");
    assert!(matches!(out, BViewMove::Zoom { factor } if (factor - 1.0 / 1.1).abs() < 1e-6));
}

#[test]
fn a_two_finger_drag_orbits_it_does_not_zoom() {
    // This is the rule the old handler had backwards: `TRACKPADPAN` is
    // bound to `view3d.rotate`, so a drag turns the view. Zoom is the
    // pinch, and only the pinch.
    let out = drag(pixels(0.0, 10.0), Modifiers::none()).expect("a drag");
    assert!(matches!(out, BViewMove::Orbit { .. }), "a drag orbits, got {out:?}");
    let out = drag(pixels(10.0, 0.0), Modifiers::none()).expect("a drag");
    assert!(matches!(out, BViewMove::Orbit { yaw, pitch } if yaw != 0.0 && pitch == 0.0));
}

#[test]
fn shift_with_a_drag_pans_as_the_default_keymap_has_it() {
    let out = drag(pixels(10.0, -4.0), Modifiers::shift()).expect("a shift-drag");
    assert!(matches!(out, BViewMove::Pan { .. }), "shift pans, got {out:?}");
    // And without shift the same drag orbits, so the modifier is what
    // chooses — not the direction or the distance.
    assert!(matches!(drag(pixels(10.0, -4.0), Modifiers::none()), Some(BViewMove::Orbit { .. })));
}

#[test]
fn ctrl_with_a_drag_still_orbits_in_the_three_d_view() {
    // The 3D view has no `TRACKPADPAN` + ctrl binding; that is a 2D editor
    // thing. Stated rather than left to a missing branch.
    assert!(!BViewNavigate::ctrl_drag_zooms());
    assert!(matches!(drag(pixels(8.0, 0.0), Modifiers::control()), Some(BViewMove::Orbit { .. })));
}

#[test]
fn the_end_of_a_gesture_moves_nothing() {
    for phase in [TouchPhase::Ended, TouchPhase::Cancelled] {
        assert_eq!(BViewNavigate::scroll(pixels(20.0, 20.0), Modifiers::none(), phase), None, "{phase:?}");
    }
}

#[test]
fn a_delta_below_the_floor_is_noise() {
    assert_eq!(drag(pixels(0.1, 0.2), Modifiers::none()), None);
    assert_eq!(drag(lines(0.0), Modifiers::none()), None);
    // But a small drag on one axis alone still counts.
    assert!(drag(pixels(0.0, 1.0), Modifiers::none()).is_some());
}

#[test]
fn dragging_right_turns_the_view_right_and_dragging_down_tips_it_down() {
    // The trackball direction, which is what makes a trackpad feel right
    // rather than inverted.
    let right = drag(pixels(10.0, 0.0), Modifiers::none()).expect("right");
    let down = drag(pixels(0.0, -10.0), Modifiers::none()).expect("down");
    assert!(matches!(right, BViewMove::Orbit { yaw, .. } if yaw < 0.0));
    assert!(matches!(down, BViewMove::Orbit { pitch, .. } if pitch > 0.0));
}
