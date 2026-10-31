//! The car's two systems: what the drivetrain is being asked for, and
//! where the car then is.
//!
//! Much simpler than the multirotor, because a car has no attitude loop to
//! close: it goes where its wheels point. The steering is the bicycle
//! model AirSim's `CarPawn` uses under its wheeled-vehicle component — a
//! steer angle at the front wheels, turned into a yaw rate by the
//! wheelbase and the speed, so a stationary car does not spin on the spot
//! however hard the wheel is turned.

use super::aspects::{Autopilot, CarLimits, Craft, Helm, Motion, Placement, Wanted};
use super::world::sim;
use crate::input::Gear;
use glam::Vec3;
use phiecs::prelude::{Stage, System, Word, World};

/// One of the car's two systems, told apart by its stage.
pub struct Drive {
    stage: Stage,
}

impl Drive {
    /// `Stage::PreCouple`: the drivetrain, from demand to [`Wanted`].
    pub fn command() -> Self {
        Self { stage: Stage::PreCouple }
    }

    /// `Stage::Couple`: the chassis, from [`Wanted`] to a new pose.
    pub fn integrate() -> Self {
        Self { stage: Stage::Couple }
    }
}

impl System for Drive {
    fn run(&mut self, world: &mut World, dt: f32) {
        let vehicles = sim(world).vehicles.clone();
        for word in vehicles {
            let car = world.read::<Craft>(word).and_then(|c| c.car_limits().cloned());
            let Some(limits) = car else { continue };
            match self.stage {
                Stage::PreCouple => command(world, word, &limits),
                _ => integrate(world, word, &limits, dt),
            }
        }
    }

    fn stage(&self) -> Stage {
        self.stage
    }

    fn name(&self) -> &str {
        match self.stage {
            Stage::PreCouple => "drive: command",
            _ => "drive: integrate",
        }
    }
}

/// The drivetrain. [`Wanted`] is a multirotor's word for it, so it is
/// read across: `collective` is the acceleration along the car's own x,
/// and `yaw_rate` is left for the integrator to work out from the speed.
fn command(world: &mut World, word: Word, limits: &CarLimits) {
    let Some(demand) = world.read::<Helm>(word).and_then(|h| h.control.car()) else { return };
    let along = world
        .read::<Placement>(word)
        .zip(world.read::<Motion>(word))
        .map(|(p, m)| m.twist.linear.dot(p.pose.orientation * Vec3::X))
        .unwrap_or(0.0);
    let Some(pilot) = world.read_mut::<Autopilot>(word) else { return };

    let drive = demand.gear.sense() * demand.throttle * limits.max_accel * torque_curve(demand.gear);
    // The handbrake locks the wheels, so it is the brakes at full and
    // then some.
    let braking = match demand.handbrake {
        true => limits.max_brake * 1.4,
        false => demand.brake * limits.max_brake,
    };
    // Brakes act against the way the car is going and do nothing at all
    // once it has stopped, and never take off more speed than there is,
    // so they cannot drive it backwards.
    let slowing = match along.abs() > STOPPED {
        true => -along.signum() * braking.min(along.abs() / DT_FLOOR),
        false => 0.0,
    };
    pilot.wanted = Wanted {
        collective: drive + slowing,
        roll: 0.0,
        pitch: 0.0,
        yaw_rate: demand.steer * limits.max_steer,
    };
}

/// How much of the engine a gear gives: a low gear pulls harder, a high
/// one less. A rough stand-in for a torque curve, enough that shifting
/// means something.
fn torque_curve(gear: Gear) -> f32 {
    match gear {
        Gear::Neutral => 0.0,
        Gear::Reverse => 0.6,
        Gear::Forward(n) => 1.0 / (0.6 + 0.25 * f32::from(n.max(1))),
    }
}

/// The chassis: accelerate along the car's own x, turn by the bicycle
/// model, roll along.
fn integrate(world: &mut World, word: Word, limits: &CarLimits, dt: f32) {
    match sim(world).paused {
        true => return,
        false => {}
    }
    let Some(wanted) = world.read::<Autopilot>(word).map(|a| a.wanted) else { return };
    let Some(mut place) = world.read::<Placement>(word).copied() else { return };
    let Some(mut motion) = world.read::<Motion>(word).copied() else { return };

    let heading = place.pose.orientation * Vec3::X;
    let along = motion.twist.linear.dot(heading);
    let rolling = -along * limits.rolling;
    let accel_along = wanted.collective + rolling;

    let speed = (along + accel_along * dt).clamp(
        -limits.config.vehicle.max_velocity * 0.4,
        limits.config.vehicle.max_velocity,
    );

    // The bicycle model: the steer angle and the wheelbase give a
    // curvature, and the speed along it gives the yaw rate. A car at a
    // standstill does not turn.
    let steer = wanted.yaw_rate.clamp(-limits.max_steer, limits.max_steer);
    let curvature = steer.tan() / limits.config.wheelbase.max(0.5);
    let yaw_rate = speed * curvature;

    place.euler.z += yaw_rate * dt;
    place.euler.x = 0.0;
    place.euler.y = 0.0;
    place.sync();

    let forward = place.pose.orientation * Vec3::X;
    motion.twist.linear = forward * speed;
    motion.twist.angular = Vec3::new(0.0, 0.0, yaw_rate);
    motion.accel.linear = forward * accel_along;
    place.pose.position += motion.twist.linear * dt;
    // The ground is flat and the car is on it.
    place.pose.position.z = 0.0;

    world.bind(word, place);
    world.bind(word, motion);
}

/// The shortest step the brakes are allowed to assume when working out how
/// much speed there is left to take off, so a long step cannot make them
/// reverse the car.
const DT_FLOOR: f32 = 0.05;

/// Below this speed, m/s, a car counts as stopped and its brakes do
/// nothing.
const STOPPED: f32 = 0.05;
