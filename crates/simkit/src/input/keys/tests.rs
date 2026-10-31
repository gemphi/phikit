//! What the keyboard promises: a ramped axis rather than a slammed
//! one, a sprung stick that centres, a lever that does not, and a
//! lost window that lets go of everything.

use super::*;

#[test]
fn a_held_key_ramps_its_axis_rather_than_slamming_it() {
    let mut keys = Keys::flight();
    keys.press(Key::Right);
    let first = keys.poll(0.1).axis(slot::AXIS_ROLL);
    assert!(first > 0.0 && first < 1.0, "a tenth of a second is not full stick: {first}");
    for _ in 0..40 {
        keys.poll(0.1);
    }
    assert_eq!(keys.poll(0.1).axis(slot::AXIS_ROLL), 1.0, "held long enough it reaches the stop");
}

#[test]
fn a_sprung_axis_centres_when_the_key_is_let_go() {
    let mut keys = Keys::flight();
    keys.press(Key::Right);
    for _ in 0..40 {
        keys.poll(0.1);
    }
    keys.release(Key::Right);
    for _ in 0..40 {
        keys.poll(0.1);
    }
    assert_eq!(keys.poll(0.1).axis(slot::AXIS_ROLL), 0.0);
}

#[test]
fn the_collective_stays_where_it_is_left() {
    let mut keys = Keys::flight();
    keys.press(Key::Char('w'));
    for _ in 0..5 {
        keys.poll(0.1);
    }
    let raised = keys.poll(0.1).axis(slot::AXIS_THROTTLE);
    keys.release(Key::Char('w'));
    for _ in 0..20 {
        keys.poll(0.1);
    }
    assert_eq!(keys.poll(0.1).axis(slot::AXIS_THROTTLE), raised, "a lever is not sprung");
}

#[test]
fn opposite_keys_cancel_and_losing_focus_lets_go_of_everything() {
    let mut keys = Keys::flight();
    keys.press(Key::Left);
    keys.press(Key::Right);
    for _ in 0..20 {
        keys.poll(0.1);
    }
    assert_eq!(keys.poll(0.1).axis(slot::AXIS_ROLL), 0.0, "both ways is neither way");

    keys.press(Key::Space);
    assert!(keys.poll(0.1).held(slot::BUTTON_ARM));
    keys.release_all();
    assert!(!keys.poll(0.1).held(slot::BUTTON_ARM));
    assert!(!keys.is_held(Key::Left));
}
