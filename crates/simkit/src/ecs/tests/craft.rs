//! A car driven and two vehicles in one world: each keeps its own
//! controller, and a gearbox in neutral drives nothing.

use super::super::*;
use crate::input::{CarDemand, ControlState, Gear, RotorDemand, RotorMode};
use glam::Vec3;

fn stick(throttle: f32, mode: RotorMode) -> ControlState {
    ControlState::Rotor(RotorDemand { armed: true, throttle, mode, ..RotorDemand::idle() })
}

#[test]
fn a_car_drives_where_it_is_steered() {
    let mut world = build(Plan::new().with_car("taxi"));
    let link = link_of(&world);
    for _ in 0..300 {
        link.device(
            "taxi",
            Act::Helm(ControlState::Car(CarDemand {
                throttle: 1.0,
                steer: 0.5,
                gear: Gear::Forward(1),
                ..CarDemand::idle()
            })),
        );
        world.step(0.02);
    }
    let report = link.report("taxi").expect("a report");
    assert!(report.ground_speed() > 1.0, "it got going: {}", report.ground_speed());
    assert!(report.euler.z.abs() > 0.1, "and it turned: {}", report.euler.z);
    assert_eq!(report.altitude(), 0.0, "staying on the road");

    // Then the handbrake.
    for _ in 0..300 {
        link.device(
            "taxi",
            Act::Helm(ControlState::Car(CarDemand { handbrake: true, ..CarDemand::idle() })),
        );
        world.step(0.02);
    }
    assert!(
        link.report("taxi").expect("a report").ground_speed() < 0.2,
        "the handbrake stopped it: {}",
        link.report("taxi").expect("a report").ground_speed()
    );
}

#[test]
fn a_car_in_neutral_goes_nowhere_however_hard_the_throttle_is_held() {
    let mut world = build(Plan::new().with_car("taxi"));
    let link = link_of(&world);
    for _ in 0..200 {
        link.device(
            "taxi",
            Act::Helm(ControlState::Car(CarDemand {
                throttle: 1.0,
                gear: Gear::Neutral,
                ..CarDemand::idle()
            })),
        );
        world.step(0.02);
    }
    assert!(link.report("taxi").expect("a report").ground_speed() < 0.01);
}

#[test]
fn two_vehicles_in_one_world_keep_their_own_controllers() {
    let mut world = build(Plan::new().with_drone("one").with_vehicle(
        "two",
        Craft::rotor(),
        Vec3::new(10.0, 0.0, 0.0),
    ));
    let link = link_of(&world);
    for _ in 0..200 {
        link.device("one", Act::Helm(stick(1.0, RotorMode::AltitudeHold)));
        link.device("two", Act::Helm(ControlState::Rotor(RotorDemand::idle())));
        world.step(0.02);
    }
    let one = link.report("one").expect("a report");
    let two = link.report("two").expect("a report");
    assert!(one.altitude() > 0.5, "the one asked to climb climbed");
    assert_eq!(two.altitude(), 0.0, "and the one left alone did not");
    assert_eq!(two.pose().position.x, 10.0, "nor did it move");
}
