//! What the bridge is for, asserted: a command from either door reaching
//! the vehicle by the one path, and only the door that holds it winning.

use super::*;
use crate::input::{ControlState, Manoeuvre, RotorDemand, RotorMode};
use crate::controllers::FlightMode;
use glam::Vec3;

mod craft;

fn drone_world() -> (phiecs::prelude::World, ControlLink) {
    let world = build(Plan::new().with_drone("drone"));
    let link = link_of(&world);
    (world, link)
}

fn stick(throttle: f32, mode: RotorMode) -> ControlState {
    ControlState::Rotor(RotorDemand { armed: true, throttle, mode, ..RotorDemand::idle() })
}

#[test]
fn a_vehicle_is_a_word_with_aspects_on_it() {
    let (world, _) = drone_world();
    let drone = vehicle(&world, "drone").expect("the drone is a word in the world");
    assert_eq!(world.spelling(drone), "drone");
    assert!(world.read::<Placement>(drone).is_some());
    assert!(world.read::<Motion>(drone).is_some());
    assert!(world.read::<Helm>(drone).is_some());
    assert!(world.read::<Autopilot>(drone).is_some());
    assert!(world.read::<Senses>(drone).is_some());
    assert!(world.read::<Craft>(drone).is_some());
    assert_eq!(names(&world), vec!["drone".to_string()]);
    assert!(vehicle(&world, "nothing like that").is_none());
}

#[test]
fn an_order_becomes_a_helm_aspect_on_the_next_step() {
    let (mut world, link) = drone_world();
    let drone = vehicle(&world, "drone").expect("a drone");
    link.device("drone", Act::Helm(stick(0.9, RotorMode::AltitudeHold)));

    assert!(!world.read::<Autopilot>(drone).expect("a controller").armed, "not yet taken in");
    world.step(1.0 / 60.0);
    assert_eq!(link.waiting(), 0, "the link is drained by the step");

    let helm = world.read::<Helm>(drone).expect("a helm");
    assert_eq!(helm.from, Origin::Device);
    assert_eq!(helm.control.rotor().expect("a rotor demand").throttle, 0.9);
    let pilot = world.read::<Autopilot>(drone).expect("a controller");
    assert!(pilot.armed, "the arm switch in the demand armed it");
    assert_eq!(pilot.mode, FlightMode::AltitudeHold, "and its mode switch set the mode");
}

#[test]
fn only_the_door_holding_the_vehicle_is_obeyed() {
    let (mut world, link) = drone_world();
    let drone = vehicle(&world, "drone").expect("a drone");

    // With nobody having taken it, the stick flies it and the client
    // cannot.
    link.api("drone", Act::Helm(stick(1.0, RotorMode::Angle)));
    world.step(0.1);
    assert!(!world.read::<Autopilot>(drone).expect("a controller").armed);
    assert_eq!(world.read::<Autopilot>(drone).expect("a controller").refused, 1);

    // The client takes it, and now it is the stick that is refused.
    link.api("drone", Act::ApiControl(true));
    world.step(0.1);
    link.device("drone", Act::Helm(stick(1.0, RotorMode::Angle)));
    world.step(0.1);
    let pilot = world.read::<Autopilot>(drone).expect("a controller");
    assert!(!pilot.armed, "the stick cannot arm what the client holds");
    assert_eq!(pilot.refused, 2);

    // And the client's own command now lands.
    link.api("drone", Act::Helm(stick(1.0, RotorMode::Angle)));
    world.step(0.1);
    assert!(world.read::<Autopilot>(drone).expect("a controller").armed);
}

#[test]
fn taking_control_is_never_itself_refused() {
    let (mut world, link) = drone_world();
    let drone = vehicle(&world, "drone").expect("a drone");
    link.api("drone", Act::ApiControl(true));
    world.step(0.1);
    assert!(world.read::<Autopilot>(drone).expect("a controller").api_control);
    assert_eq!(
        world.read::<Autopilot>(drone).expect("a controller").refused,
        0,
        "a client may take the vehicle without already holding it"
    );
    link.api("drone", Act::ApiControl(false));
    world.step(0.1);
    assert!(!world.read::<Autopilot>(drone).expect("a controller").api_control);
}

#[test]
fn a_report_comes_back_out_of_the_same_link_the_orders_went_in() {
    let (mut world, link) = drone_world();
    assert!(link.report("drone").is_none(), "nothing has been stepped yet");
    world.step(1.0 / 60.0);
    let report = link.report("drone").expect("a report after a step");
    assert_eq!(report.frame, 0);
    assert!(report.landed);
    assert_eq!(report.altitude(), 0.0);
    assert_eq!(link.reported(), vec!["drone".to_string()]);
}

#[test]
fn the_stages_run_in_order_within_one_step() {
    // A command put on the link and a single step: taken in at Ingest,
    // flown at PreCouple and Couple, sensed at PostCouple, reported at
    // Readout. If the order were wrong the report would be a frame stale.
    let (mut world, link) = drone_world();
    link.device("drone", Act::Helm(stick(1.0, RotorMode::AltitudeHold)));
    world.step(0.1);
    let report = link.report("drone").expect("a report");
    assert!(report.armed, "the intake ran before the report");
    assert!(report.climb() > 0.0, "and the physics ran between them");
    assert!(
        report.senses.imu.linear_acceleration.z > 0.0,
        "and the sensors read the frame the physics had just produced"
    );
}

#[test]
fn a_paused_world_still_takes_orders_but_does_not_move() {
    let (mut world, link) = drone_world();
    link.device("drone", Act::Helm(stick(1.0, RotorMode::AltitudeHold)));
    sim_mut(&mut world).paused = true;
    for _ in 0..50 {
        world.step(0.05);
    }
    let report = link.report("drone").expect("a report");
    assert!(report.armed, "the order was taken in");
    assert_eq!(report.altitude(), 0.0, "but nothing moved");
}

#[test]
fn a_takeoff_switch_flies_itself_and_hands_the_sticks_back() {
    let (mut world, link) = drone_world();
    let drone = vehicle(&world, "drone").expect("a drone");
    link.device(
        "drone",
        Act::Helm(ControlState::Rotor(RotorDemand {
            armed: true,
            throttle: 0.5,
            mode: RotorMode::AltitudeHold,
            manoeuvre: Some(Manoeuvre::Takeoff),
            ..RotorDemand::idle()
        })),
    );
    world.step(0.05);
    assert_eq!(world.read::<Autopilot>(drone).expect("a controller").mode, FlightMode::Takeoff);

    // Centred sticks from here on: the manoeuvre must not be cancelled
    // by the very next frame of stick.
    for _ in 0..400 {
        link.device("drone", Act::Helm(stick(0.5, RotorMode::AltitudeHold)));
        world.step(0.02);
    }
    let report = link.report("drone").expect("a report");
    assert!(report.altitude() > 2.0, "it climbed to its takeoff altitude: {}", report.altitude());
    assert_eq!(
        report.mode,
        FlightMode::AltitudeHold,
        "and then handed the sticks back: the stick's own mode switch is in force again"
    );
    let drone_again = vehicle(&world, "drone").expect("a drone");
    assert!(
        !world.read::<Autopilot>(drone_again).expect("a controller").to_target,
        "and it is flying to the sticks, not to the takeoff point"
    );
}

#[test]
fn a_reset_puts_a_drone_back_where_it_was_armed() {
    let (mut world, link) = drone_world();
    for _ in 0..200 {
        link.device("drone", Act::Helm(stick(1.0, RotorMode::AltitudeHold)));
        world.step(0.02);
    }
    assert!(link.report("drone").expect("a report").altitude() > 0.5);

    link.device("drone", Act::Reset);
    world.step(0.02);
    let report = link.report("drone").expect("a report");
    assert_eq!(report.altitude(), 0.0);
    assert!(!report.armed);
    assert_eq!(report.climb(), 0.0);
}

#[test]
fn a_client_can_fly_a_drone_to_a_point_over_the_wire() {
    let (mut world, link) = drone_world();
    link.api("drone", Act::ApiControl(true));
    link.api("drone", Act::Arm(true));
    link.api("drone", Act::MoveToPosition { to: Vec3::new(8.0, 0.0, 6.0), speed: 5.0 });
    for _ in 0..900 {
        world.step(0.02);
    }
    let report = link.report("drone").expect("a report");
    let at = report.pose().position;
    assert!((at.z - 6.0).abs() < 1.2, "it reached the height asked for: {at:?}");
    assert!((at.x - 8.0).abs() < 2.0, "and the point asked for: {at:?}");
}
