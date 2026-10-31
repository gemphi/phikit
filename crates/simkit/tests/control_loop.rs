//! A drone flown through phiecs from a scripted stick, end to end.
//!
//! Every piece of the control layer is in this loop and none of it is
//! stubbed: a [`Scripted`] device hands out axis and button frames, the
//! gamepad [`Bindings`] and their [`AxisShape`]s shape them, a [`Pilot`]
//! turns them into a [`ControlState`], a [`Driver`] puts that on the
//! [`ControlLink`], and `World::step` runs the five stages that take it in,
//! fly it, sense it and report it. What is asserted is what a pilot would
//! see: it climbs when the throttle is up, it holds when the throttle is
//! centred, and it comes down under control when the landing switch is
//! pressed rather than falling.

use simkit::ecs::{build, link_of, ControlLink, Driver, Plan, Report, GROUND_TOUCH};
use simkit::input::{slot, Pilot, Reading, RotorMode, Scripted};

/// Fifty steps a second, which is a usual control period for a small
/// multirotor.
const DT: f32 = 0.02;

/// A frame with nothing held, on the gamepad layout. A centred collective
/// stick reads as half throttle, which in the hold modes means "stay
/// where you are".
fn frame() -> Reading {
    Reading::resting(slot::AXES)
}

/// A frame with the arm switch on.
fn armed() -> Reading {
    frame().with_held(slot::BUTTON_ARM)
}

/// What the drone was doing at one step.
#[derive(Debug, Clone, Copy)]
struct Moment {
    altitude: f32,
    climb: f32,
    armed: bool,
}

/// Flies `script` and hands back a moment per step.
fn fly(script: Vec<Reading>) -> (Vec<Moment>, ControlLink, Report) {
    let mut world = build(Plan::new().with_drone("drone"));
    let link = link_of(&world);
    let steps = script.len();
    let mut driver =
        Driver::new(&world, "drone", Scripted::new("stick", script), Pilot::gamepad());

    let mut trace = Vec::with_capacity(steps);
    for _ in 0..steps {
        driver.poll(DT);
        world.step(DT);
        let report = link.report("drone").expect("a report after every step");
        trace.push(Moment {
            altitude: report.altitude(),
            climb: report.climb(),
            armed: report.armed,
        });
    }
    let last = link.report("drone").expect("a last report");
    (trace, link, last)
}

#[test]
fn a_scripted_stick_climbs_holds_and_lands() {
    // The flight, in phases, as a pilot would fly it.
    let mut script = Vec::new();

    // Arm, and step the mode switch once: Angle to AltitudeHold, so the
    // throttle becomes a climb rate about its centre.
    script.push(armed().with_held(slot::BUTTON_MODE));
    let arming = script.len();

    // Three seconds of full throttle: a climb.
    for _ in 0..150 {
        script.push(armed().with_axis(slot::AXIS_THROTTLE, 1.0));
    }
    let climbed = script.len();

    // Three seconds of centred throttle: a hold.
    for _ in 0..150 {
        script.push(armed());
    }
    let held = script.len();

    // The landing switch, then centred sticks all the way down.
    script.push(armed().with_held(slot::BUTTON_LAND));
    for _ in 0..300 {
        script.push(armed());
    }
    let landed = script.len();

    // And the arm switch dropped, as a pilot does once it is down.
    for _ in 0..50 {
        script.push(frame());
    }

    let (trace, _link, last) = fly(script);

    // It climbed.
    let top = trace[arming..climbed].iter().map(|m| m.altitude).fold(0.0f32, f32::max);
    assert!(top > 5.0, "three seconds of full throttle should climb well clear: {top} m");
    assert!(
        trace[arming..climbed].iter().all(|m| m.armed),
        "and the arm switch held it armed throughout"
    );

    // It held. The band is wide enough for the overshoot coming off a
    // five-metre-a-second climb and no wider.
    let hold = &trace[held - 60..held];
    let lowest = hold.iter().map(|m| m.altitude).fold(f32::MAX, f32::min);
    let highest = hold.iter().map(|m| m.altitude).fold(f32::MIN, f32::max);
    assert!(
        highest - lowest < 0.75,
        "a centred throttle should hold height, not drift: {lowest} m to {highest} m"
    );
    assert!(
        hold.iter().all(|m| m.altitude > 4.0),
        "and it should hold where it got to, not sink back: {lowest} m"
    );

    // It came down under control, not in free fall. A fall from five
    // metres passes 9 m/s; the airframe's descent limit is 3.
    let descent = &trace[held..landed];
    let fastest = descent.iter().map(|m| m.climb).fold(0.0f32, f32::min);
    assert!(
        fastest > -4.0,
        "the landing should be flown, not dropped: {fastest} m/s at its fastest"
    );
    assert!(
        descent.iter().any(|m| m.climb < -0.5),
        "but it should actually come down"
    );

    // And it ended on the ground, still and disarmed.
    assert!(last.altitude() <= GROUND_TOUCH, "it is on the ground: {} m", last.altitude());
    assert!(last.landed, "and the world says so");
    assert!(!last.armed, "and the arm switch being dropped disarmed it");
    assert_eq!(last.climb(), 0.0, "and it is not still moving");
}

#[test]
fn the_sensors_follow_the_flight_they_are_sensing() {
    let mut script = vec![armed().with_held(slot::BUTTON_MODE)];
    for _ in 0..200 {
        script.push(armed().with_axis(slot::AXIS_THROTTLE, 1.0));
    }
    let (_, _link, last) = fly(script);

    assert!(last.altitude() > 3.0, "it climbed");
    assert!(
        last.senses.baro.altitude > last.senses.gps.geo_point.altitude - 1.0,
        "the barometer and the GPS agree about roughly where it is"
    );
    assert!(
        last.senses.baro.pressure < 101_325.0,
        "the pressure fell as it climbed: {} Pa",
        last.senses.baro.pressure
    );
    assert!(last.senses.gps.has_fix, "and the GPS has a fix");
    assert!(
        last.senses.gps.velocity.z < 0.0,
        "which reports the climb as a negative descent, as a GPS does: {}",
        last.senses.gps.velocity.z
    );
    assert!(
        (last.senses.agl - last.altitude()).abs() < 1e-3,
        "and the downward sensor reads the height it is at"
    );
}

#[test]
fn a_dead_zone_a_pilot_widens_takes_a_drifting_stick_out_of_the_flight() {
    // A worn gimbal that rests a tenth off centre. Through the stock
    // dead zone it drifts; widened past the fault it does not. Same
    // flight, same world, one number different — which is the whole
    // point of the binding table being a value.
    let drifting = |dead_zone: f32| {
        let mut world = build(Plan::new().with_drone("drone"));
        let link = link_of(&world);
        let mut pilot = Pilot::gamepad();
        pilot.set_mode(RotorMode::PositionHold);
        for intent in [simkit::input::Intent::Roll, simkit::input::Intent::Pitch] {
            pilot.bindings_mut().shape_mut(intent).expect("a stick").dead_zone = dead_zone;
        }
        let script: Vec<Reading> = (0..400)
            .map(|_| armed().with_axis(slot::AXIS_ROLL, 0.1).with_axis(slot::AXIS_PITCH, 0.1))
            .collect();
        let mut driver =
            Driver::new(&world, "drone", Scripted::new("worn stick", script), pilot);
        for _ in 0..400 {
            driver.poll(DT);
            world.step(DT);
        }
        let at = link.report("drone").expect("a report").pose().position;
        at.truncate().length()
    };

    let stock = drifting(0.05);
    let widened = drifting(0.15);
    assert!(stock > 0.5, "a stick resting off centre drifts the drone: {stock} m");
    assert!(widened < 0.05, "and a dead zone past the fault stops it: {widened} m");
}
