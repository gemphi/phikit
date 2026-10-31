//! What a table promises: an unbound intent asks for nothing, a
//! remap keeps its shaping, and each preset reads its own device.

use super::*;
use crate::input::device::slot;

#[test]
fn an_unbound_intent_asks_for_nothing_rather_than_failing() {
    let mut table = Bindings::gamepad();
    table.unbind(Intent::Yaw);
    let reading = Reading::resting(slot::AXES).with_axis(slot::AXIS_YAW, 1.0);
    assert_eq!(table.value(Intent::Yaw, &reading), 0.0);
    assert!(!table.is_held(Intent::Yaw, &reading));
}

#[test]
fn a_pair_of_buttons_reads_as_one_axis() {
    let mut table = Bindings::empty(DeviceKind::Keyboard);
    table.binds.push(Bind::new(
        Source::Pair { minus: 10, plus: 11 },
        Intent::Roll,
        AxisShape::stick().with_dead_zone(0.0).with_expo(0.0),
    ));
    let none = Reading::resting(1);
    assert_eq!(table.value(Intent::Roll, &none), 0.0);
    assert_eq!(table.value(Intent::Roll, &none.clone().with_held(11)), 1.0);
    assert_eq!(table.value(Intent::Roll, &none.clone().with_held(10)), -1.0);
    let both = none.with_held(10).with_held(11);
    assert_eq!(table.value(Intent::Roll, &both), 0.0, "both ways is neither way");
}

#[test]
fn a_remap_keeps_the_shaping_it_was_given() {
    let mut table = Bindings::gamepad();
    table.shape_mut(Intent::Roll).expect("roll is bound").trim = 0.2;
    table.remap(Intent::Roll, Source::Axis(7));
    assert_eq!(table.source_of(Intent::Roll), Some(Source::Axis(7)));
    assert_eq!(table.bind(Intent::Roll).expect("still bound").shape.trim, 0.2);
}

#[test]
fn a_gamepads_axes_keep_their_sense_and_its_collective_reads_as_a_lever() {
    let table = Bindings::gamepad();
    let pushed = Reading::resting(slot::AXES).with_axis(slot::AXIS_PITCH, 1.0);
    assert!(
        table.value(Intent::Pitch, &pushed) > 0.0,
        "a reading already comes in the pilot's sense: positive is forward"
    );

    let centred = Reading::resting(slot::AXES);
    assert_eq!(
        table.value(Intent::Throttle, &centred),
        0.5,
        "a centred collective stick is half throttle, which holds height"
    );
    let up = Reading::resting(slot::AXES).with_axis(slot::AXIS_THROTTLE, 1.0);
    assert_eq!(table.value(Intent::Throttle, &up), 1.0);
}

#[test]
fn a_flight_sticks_collective_lever_is_reversed_from_a_gamepads() {
    let table = Bindings::flight_stick();
    let at_stop = Reading::resting(slot::AXES);
    assert_eq!(
        table.value(Intent::Throttle, &at_stop),
        1.0,
        "a reversed lever reads open at its own zero"
    );
    let open = Reading::resting(slot::AXES).with_axis(slot::AXIS_THROTTLE, 1.0);
    assert_eq!(table.value(Intent::Throttle, &open), 0.0);
}

#[test]
fn an_axis_past_half_its_travel_counts_as_a_switch_held() {
    let mut table = Bindings::gamepad();
    table.remap(Intent::Arm, Source::Axis(slot::AXIS_BRAKE));
    let half = Reading::resting(slot::AXES).with_axis(slot::AXIS_BRAKE, 0.4);
    let pulled = Reading::resting(slot::AXES).with_axis(slot::AXIS_BRAKE, 0.9);
    assert!(!table.is_held(Intent::Arm, &half));
    assert!(table.is_held(Intent::Arm, &pulled));
}
