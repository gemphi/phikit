//! The cascade itself: a demand and a state in, a [`Wanted`] out.
//!
//! One arm per [`FlightMode`], because how far down the cascade the
//! sticks reach is what a mode *is*. AirSim's `simple_flight` says
//! the same thing with its per-channel `GoalMode`.

use super::super::aspects::{Autopilot, Helm, Motion, Placement, RotorLimits, Wanted};
use super::super::world::GRAVITY;
use crate::controllers::FlightMode;
use crate::input::RotorDemand;
use glam::{Vec2, Vec3};
use phiecs::prelude::{Word, World};

/// The controller: what this step is being asked of the rotors.
pub(super) fn command(world: &mut World, word: Word, limits: &RotorLimits, dt: f32) {
    let Some(demand) = world.read::<Helm>(word).and_then(|h| h.control.rotor()) else { return };
    let Some(place) = world.read::<Placement>(word).copied() else { return };
    let Some(motion) = world.read::<Motion>(word).copied() else { return };
    let Some(pilot) = world.read_mut::<Autopilot>(word) else { return };

    // A disarmed craft asks for nothing: the rotors stop and gravity has
    // the rest of the argument.
    match pilot.armed {
        true => {}
        false => {
            pilot.wanted = Wanted::default();
            return;
        }
    }

    let yaw_rate = demand.yaw * limits.max_yaw_rate;
    let climbing = motion.climb();
    let over_ground = motion.twist.linear.truncate();

    let wanted = match pilot.mode {
        // Acro: the sticks are rates, so the attitude the integrator is
        // asked to hold is this step's worth of them away from where it
        // already is. The throttle is collective thrust outright.
        FlightMode::Manual => Wanted {
            collective: demand.throttle * limits.max_collective,
            roll: place.roll() + demand.roll * limits.max_rate * dt,
            pitch: place.pitch() + demand.pitch * limits.max_rate * dt,
            yaw_rate,
        },
        // Levelled: the sticks are angles.
        FlightMode::Stabilized => Wanted {
            collective: demand.throttle * limits.max_collective,
            roll: demand.roll * limits.max_tilt,
            pitch: demand.pitch * limits.max_tilt,
            yaw_rate,
        },
        // The throttle is a climb rate about its centre, so a centred
        // throttle holds height whatever the craft weighs.
        FlightMode::AltitudeHold => Wanted {
            collective: vertical(pilot, climb_demand(demand, limits), climbing, limits, dt),
            roll: demand.roll * limits.max_tilt,
            pitch: demand.pitch * limits.max_tilt,
            yaw_rate,
        },
        // Position hold means one of two things and the flag says
        // which: fly to the point a client commanded, or take the sticks
        // as velocities over the ground so that centring them holds
        // position. Both arrive as this mode, so both are read here.
        FlightMode::PositionHold => match pilot.to_target {
            true => to_point(pilot, place, over_ground, climbing, limits, dt, yaw_rate),
            false => {
                let want = stick_velocity(demand, place.yaw(), limits);
                let tilt = level(pilot, want, over_ground, limits, dt);
                Wanted {
                    collective: vertical(pilot, climb_demand(demand, limits), climbing, limits, dt),
                    roll: tilt.x,
                    pitch: tilt.y,
                    yaw_rate,
                }
            }
        },
        // A client's `moveByVelocity`: the target *is* the velocity.
        FlightMode::Velocity => {
            let want = pilot.target;
            let tilt = level(pilot, want.truncate(), over_ground, limits, dt);
            let held = pilot.wanted.yaw_rate;
            Wanted {
                collective: vertical(pilot, want.z, climbing, limits, dt),
                roll: tilt.x,
                pitch: tilt.y,
                yaw_rate: held,
            }
        }
        // Climbing to the takeoff altitude, then holding there.
        FlightMode::Takeoff => {
            let want = (pilot.target.z - place.altitude()).clamp(-limits.max_descent, limits.max_climb);
            let tilt = level(pilot, Vec2::ZERO, over_ground, limits, dt);
            Wanted {
                collective: vertical(pilot, want, climbing, limits, dt),
                roll: tilt.x,
                pitch: tilt.y,
                yaw_rate: 0.0,
            }
        }
        // Down at a measured rate, slowing near the ground.
        FlightMode::Land => {
            let want = -(place.altitude() * 0.6).clamp(0.3, limits.max_descent);
            let tilt = level(pilot, Vec2::ZERO, over_ground, limits, dt);
            Wanted {
                collective: vertical(pilot, want, climbing, limits, dt),
                roll: tilt.x,
                pitch: tilt.y,
                yaw_rate: 0.0,
            }
        }
        // The rotors stopped: an emergency stop is a fall, not a hover.
        FlightMode::EmergencyStop => Wanted::default(),
        // A return home and a mission are both a point to get to, which
        // is the same loop a commanded hold runs.
        _ => to_point(pilot, place, over_ground, climbing, limits, dt, yaw_rate),
    };
    pilot.wanted = wanted;
}

/// The loop that flies to [`Autopilot::target`]: the distance left over
/// becomes a velocity to want, which [`level`] turns into a lean and
/// [`vertical`] into a collective. Capped below the airframe's top speed,
/// so arriving is a deceleration rather than an overshoot.
fn to_point(
    pilot: &mut Autopilot,
    place: Placement,
    over_ground: Vec2,
    climbing: f32,
    limits: &RotorLimits,
    dt: f32,
    yaw_rate: f32,
) -> Wanted {
    let to = pilot.target - place.pose.position;
    let want = to.truncate().clamp_length_max(limits.max_speed * 0.6);
    let tilt = level(pilot, want, over_ground, limits, dt);
    Wanted {
        collective: vertical(
            pilot,
            to.z.clamp(-limits.max_descent, limits.max_climb),
            climbing,
            limits,
            dt,
        ),
        roll: tilt.x,
        pitch: tilt.y,
        yaw_rate,
    }
}

/// The climb rate a throttle asks for in the hold modes: centre holds,
/// above centre climbs, below descends. The asymmetry is real — a
/// multirotor can climb harder than it dare descend.
fn climb_demand(demand: RotorDemand, limits: &RotorLimits) -> f32 {
    let off_centre = demand.throttle * 2.0 - 1.0;
    match off_centre >= 0.0 {
        true => off_centre * limits.max_climb,
        false => off_centre * limits.max_descent,
    }
}

/// Where the sticks point over the ground, turned from the craft's own
/// heading into the world's, because a pilot pushes the stick forward and
/// means "that way", not "+x".
fn stick_velocity(demand: RotorDemand, yaw: f32, limits: &RotorLimits) -> Vec2 {
    let forward = demand.pitch * limits.max_speed;
    let right = demand.roll * limits.max_speed;
    let (sin, cos) = yaw.sin_cos();
    Vec2::new(forward * cos + right * sin, forward * sin - right * cos)
}

/// The vertical loop: a climb rate in, a collective out. Gravity is fed
/// forward rather than integrated up to, so the loop only has to correct
/// the difference and a hover does not need a wound-up integrator.
fn vertical(
    pilot: &mut Autopilot,
    want: f32,
    climbing: f32,
    limits: &RotorLimits,
    dt: f32,
) -> f32 {
    let want = want.clamp(-limits.max_descent, limits.max_climb);
    let error = want - climbing;
    pilot.climb_wind = (pilot.climb_wind + error * dt).clamp(-6.0, 6.0);
    let correction = error * CLIMB_P + pilot.climb_wind * CLIMB_I;
    (GRAVITY + correction).clamp(0.0, limits.max_collective)
}

/// The horizontal loop: a velocity over the ground in, the roll and pitch
/// to lean at out. A multirotor goes where it points, so wanting to move
/// is wanting to tilt.
fn level(
    pilot: &mut Autopilot,
    want: Vec2,
    over_ground: Vec2,
    limits: &RotorLimits,
    dt: f32,
) -> Vec2 {
    let want = want.clamp_length_max(limits.max_speed);
    let error = want - over_ground;
    let wind = pilot.drift_wind.truncate() + error * dt;
    let wind = wind.clamp_length_max(4.0);
    pilot.drift_wind = Vec3::new(wind.x, wind.y, 0.0);
    let lean = (error * DRIFT_P + wind * DRIFT_I).clamp_length_max(limits.max_tilt);
    // A lean along +x is a nose-down pitch; a lean along +y is a roll to
    // the left, so the signs are not the same.
    Vec2::new(-lean.y, lean.x)
}

/// The vertical loop's proportional gain, 1/s: how hard a metre a second
/// of climb-rate error pulls on the collective.
const CLIMB_P: f32 = 4.0;
/// Its integral gain, 1/s².
const CLIMB_I: f32 = 1.2;
/// The horizontal loop's proportional gain, rad per m/s.
const DRIFT_P: f32 = 0.09;
/// Its integral gain.
const DRIFT_I: f32 = 0.02;
