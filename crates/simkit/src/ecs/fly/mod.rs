//! The multirotor's two systems: what the controller wants, and what the
//! airframe then does.
//!
//! [`Fly::command`] runs at `Stage::PreCouple` and turns a
//! [`Helm`] into a [`Wanted`](super::Wanted) — a collective
//! acceleration, an attitude and a yaw rate.
//! [`Fly::integrate`] runs at `Stage::Couple` and moves the craft.
//!
//! The cascade is AirSim's `simple_flight`: position, then velocity, then
//! attitude, then rate. How far down it the sticks reach is what a mode
//! *is* — in [`FlightMode::Manual`] they are rates, in
//! [`FlightMode::Stabilized`] angles, in [`FlightMode::AltitudeHold`] the
//! throttle becomes a climb rate, in [`FlightMode::PositionHold`] the
//! sticks become velocities and the loops hold what is left. simple_flight
//! stops at motor values and mixes them into a wrench; this stops one rung
//! earlier, at the wrench, because simkit's physics takes a wrench.

use super::aspects::{Autopilot, Craft, Helm, Motion, Placement, RotorLimits};
use super::report::GROUND_TOUCH;
use super::world::{sim, GRAVITY};
use crate::controllers::FlightMode;
use crate::input::{ControlState, RotorDemand, RotorMode};
use glam::Vec3;
use phiecs::prelude::{Stage, System, Word, World};

mod loops;

use loops::command;

/// One of the multirotor's two systems, told apart by its stage.
pub struct Fly {
    stage: Stage,
}

impl Fly {
    /// `Stage::PreCouple`: the controller, from demand to
    /// [`Wanted`](super::Wanted).
    pub fn command() -> Self {
        Self { stage: Stage::PreCouple }
    }

    /// `Stage::Couple`: the airframe, from [`Wanted`](super::Wanted) to a
    /// new pose.
    pub fn integrate() -> Self {
        Self { stage: Stage::Couple }
    }
}

impl System for Fly {
    fn run(&mut self, world: &mut World, dt: f32) {
        let vehicles = sim(world).vehicles.clone();
        for word in vehicles {
            let rotor = world.read::<Craft>(word).and_then(|c| c.rotor_limits().cloned());
            let Some(limits) = rotor else { continue };
            match self.stage {
                Stage::PreCouple => command(world, word, &limits, dt),
                _ => integrate(world, word, &limits, dt),
            }
        }
    }

    fn stage(&self) -> Stage {
        self.stage
    }

    fn name(&self) -> &str {
        match self.stage {
            Stage::PreCouple => "fly: command",
            _ => "fly: integrate",
        }
    }
}

/// The airframe: the wrench the rotors make, plus gravity and drag,
/// integrated.
fn integrate(world: &mut World, word: Word, limits: &RotorLimits, dt: f32) {
    match sim(world).paused {
        true => return,
        false => {}
    }
    let Some(pilot) = world.read::<Autopilot>(word).map(|a| (a.wanted, a.armed)) else { return };
    let (wanted, armed) = pilot;
    let Some(mut place) = world.read::<Placement>(word).copied() else { return };
    let Some(mut motion) = world.read::<Motion>(word).copied() else { return };

    // The attitude loop, as a first-order lag: the craft comes round to
    // the attitude it was asked for at a rate its airframe allows,
    // rather than snapping to it.
    let gain = (dt / limits.attitude_lag.max(dt)).min(1.0);
    let hold = Vec3::new(
        wanted.roll.clamp(-limits.max_tilt, limits.max_tilt),
        wanted.pitch.clamp(-limits.max_tilt, limits.max_tilt),
        place.yaw() + wanted.yaw_rate * dt,
    );
    let turned = Vec3::new(
        place.euler.x + (hold.x - place.euler.x) * gain,
        place.euler.y + (hold.y - place.euler.y) * gain,
        hold.z,
    );
    motion.twist.angular = (turned - place.euler) / dt.max(1e-6);
    place.euler = turned;
    place.sync();

    // Thrust along the craft's own z, which is why tilting moves it.
    let thrust = place.pose.orientation * Vec3::new(0.0, 0.0, wanted.collective);
    let drag = -motion.twist.linear * limits.drag;
    let accel = thrust + Vec3::new(0.0, 0.0, -GRAVITY) + drag + sim(world).environment.state().wind * 0.1;

    motion.accel.linear = accel;
    motion.twist.linear += accel * dt;
    place.pose.position += motion.twist.linear * dt;

    // The ground. A craft on it does not sink through it, and a
    // disarmed one sitting on it does not slide about either.
    match place.pose.position.z <= 0.0 {
        true => {
            place.pose.position.z = 0.0;
            motion.twist.linear.z = motion.twist.linear.z.max(0.0);
            let settled = !armed || wanted.collective < GRAVITY;
            match settled {
                true => {
                    motion.twist.linear = Vec3::ZERO;
                    place.euler.x = 0.0;
                    place.euler.y = 0.0;
                    place.sync();
                }
                false => {}
            }
        }
        false => {}
    }

    world.bind(word, place);
    world.bind(word, motion);

    // A landing that has reached the ground is a landing that is over.
    let touched = place.altitude() <= GROUND_TOUCH;
    let landing = world.read::<Autopilot>(word).map(|a| a.mode) == Some(FlightMode::Land);
    match (landing, touched) {
        (true, true) => {
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.armed = false;
                pilot.mode = FlightMode::Stabilized;
                pilot.unwind();
            }
            disarm_helm(world, word);
        }
        _ => {}
    }

    // A takeoff that has reached its altitude hands the sticks back.
    let climbed = world
        .read::<Autopilot>(word)
        .map(|a| a.mode == FlightMode::Takeoff && (a.target.z - place.altitude()).abs() < 0.3)
        .unwrap_or(false);
    match climbed {
        true => {
            if let Some(pilot) = world.read_mut::<Autopilot>(word) {
                pilot.mode = FlightMode::PositionHold;
                pilot.unwind();
            }
        }
        false => {}
    }
}

/// Puts the arm switch in the helm down too, so the next frame of stick
/// does not simply arm it again.
fn disarm_helm(world: &mut World, word: Word) {
    let Some(helm) = world.read::<Helm>(word).copied() else { return };
    let ControlState::Rotor(demand) = helm.control else { return };
    let down = RotorDemand {
        armed: false,
        manoeuvre: None,
        mode: match demand.mode {
            RotorMode::Rate => RotorMode::Angle,
            other => other,
        },
        ..demand
    };
    world.bind(word, Helm { control: ControlState::Rotor(down), ..helm });
}
