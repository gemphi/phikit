//! One order applied: what each [`Act`] does to a vehicle's aspects.
//!
//! Called only by [`Intake`](super::Intake), and only once an order
//! has passed the test of who holds the vehicle.

use super::super::aspects::{Autopilot, Craft, Helm, Motion, Placement};
use super::super::link::{Act, Order, Origin};
use crate::controllers::FlightMode;
use crate::input::{CarDemand, ControlState, Manoeuvre, RotorDemand, VehicleKind};
use glam::Vec3;
use phiecs::prelude::{Word, World};

pub(super) fn apply(world: &mut World, word: Word, order: Order) {
    let at = order.at;
    match order.act {
        Act::Helm(control) => {
            let mut helm = Helm::new(control, order.origin);
            helm.at = at;
            world.bind(word, helm);
            // A stick command takes the setpoint back off the target,
            // which is what `moveByRC` arriving mid-flight means.
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.to_target = false;
            }
            follow(world, word, control);
        }
        Act::Arm(armed) => arm(world, word, armed),
        Act::ApiControl(on) => {
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.api_control = on;
                pilot.unwind();
            }
        }
        Act::Takeoff { altitude } => {
            let from = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
            arm(world, word, true);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::Takeoff;
                pilot.target = from + Vec3::new(0.0, 0.0, altitude.abs().max(0.5));
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        Act::Land => {
            let from = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::Land;
                pilot.target = Vec3::new(from.x, from.y, 0.0);
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        Act::Hover => hold_here(world, word, FlightMode::PositionHold),
        Act::MoveToPosition { to, speed } => {
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::PositionHold;
                pilot.target = to;
                pilot.to_target = true;
                pilot.unwind();
            }
            let _ = speed;
        }
        Act::MoveByVelocity { velocity, yaw_rate } => {
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::Velocity;
                pilot.target = velocity;
                pilot.wanted.yaw_rate = yaw_rate;
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        Act::Mode(mode) => {
            match mode {
                FlightMode::PositionHold => hold_here(world, word, mode),
                _ => {}
            }
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = mode;
                pilot.unwind();
            }
        }
        Act::Reset => reset(world, word),
    }
}

/// A [`Helm`] carries a mode switch and an arm switch as well as four
/// sticks, so taking one in changes the controller too. This is where the
/// local and the remote path converge: an AirSim `moveByRC` and a gamepad
/// frame both land here, by the same route.
fn follow(world: &mut World, word: Word, control: ControlState) {
    let ControlState::Rotor(demand) = control else { return };
    let manoeuvre = demand.manoeuvre;
    arm(world, word, demand.armed);
    match manoeuvre {
        Some(Manoeuvre::Takeoff) => {
            let from = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::Takeoff;
                pilot.target = from + Vec3::new(0.0, 0.0, takeoff_altitude(demand));
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        Some(Manoeuvre::Land) => {
            let from = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::Land;
                pilot.target = Vec3::new(from.x, from.y, 0.0);
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        Some(Manoeuvre::ReturnHome) => {
            let home = world.read::<Autopilot>(word).map(|a| a.home).unwrap_or(Vec3::ZERO);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::ReturnToLaunch;
                pilot.target = home;
                pilot.to_target = true;
                pilot.unwind();
            }
        }
        // No manoeuvre asked for: the stick's own mode switch is in
        // charge, unless a manoeuvre is still flying itself out.
        None => stick_mode(world, word, demand),
    }
}

/// The takeoff a switch asks for, which is the mode's own default rather
/// than something the stick can set: a radio has no field for metres.
fn takeoff_altitude(_demand: RotorDemand) -> f32 {
    3.0
}

/// Puts the controller in the mode the stick's switch is on — but leaves
/// a manoeuvre in the air alone, so a takeoff is not cancelled by the
/// very next frame of stick.
fn stick_mode(world: &mut World, word: Word, demand: RotorDemand) {
    let flying_itself = matches!(
        world.read::<Autopilot>(word).map(|a| a.mode),
        Some(FlightMode::Takeoff) | Some(FlightMode::Land) | Some(FlightMode::ReturnToLaunch)
    );
    match flying_itself {
        true => {}
        false => {
            let want = FlightMode::from(demand.mode);
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                match pilot.mode == want {
                    true => {}
                    false => {
                        pilot.mode = want;
                        pilot.unwind();
                    }
                }
            }
        }
    }
}

/// Holds the vehicle where it is now, in `mode`.
fn hold_here(world: &mut World, word: Word, mode: FlightMode) {
    let here = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
    if let Some(pilot) = world.read_mut::<Autopilot>(word) {
        pilot.mode = mode;
        pilot.target = here;
        pilot.to_target = true;
        pilot.unwind();
    }
}

/// Arms or disarms, noting where it was armed so it can come home.
fn arm(world: &mut World, word: Word, armed: bool) {
    let here = world.read::<Placement>(word).map(|p| p.pose.position).unwrap_or(Vec3::ZERO);
    if let Some(pilot) = world.read_mut::<Autopilot>(word) {
        match (pilot.armed, armed) {
            (false, true) => {
                pilot.home = here;
                pilot.target = here;
                pilot.unwind();
            }
            (true, false) => pilot.unwind(),
            _ => {}
        }
        pilot.armed = armed;
    }
}

/// Puts the vehicle back where it started, still and disarmed.
fn reset(world: &mut World, word: Word) {
    let start = world.read::<Autopilot>(word).map(|a| a.home).unwrap_or(Vec3::ZERO);
    world.bind(word, Placement::at(start));
    world.bind(word, Motion::still());
    let idle = match world.read::<Craft>(word).map(Craft::kind) {
        Some(VehicleKind::Car) => ControlState::Car(CarDemand::idle()),
        _ => ControlState::Rotor(RotorDemand::idle()),
    };
    world.bind(word, Helm::new(idle, Origin::Api));
    if let Some(pilot) = world.read_mut::<Autopilot>(word) {
        pilot.armed = false;
        pilot.mode = FlightMode::Stabilized;
        pilot.target = start;
        pilot.to_target = false;
        pilot.refused = 0;
        pilot.unwind();
    }
}

