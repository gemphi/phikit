//! What the world says a vehicle did, for whoever is asking.
//!
//! The way out of the world, matching [`Act`](super::link::Act) as the way
//! in. An AirSim client's `simGetVehiclePose`,
//! `simGetGroundTruthKinematics`, `getMultirotorState` and the sensor
//! calls are all answered from one of these, read off a
//! [`ControlLink`](super::ControlLink) — so a client reads the same
//! stepped world a pilot is flying, never a parallel copy of it.

use super::aspects::{Autopilot, Motion, Placement, Senses};
use crate::common::{Pose, TimePoint};
use crate::controllers::FlightMode;
use crate::physics::KinematicsState;
use glam::Vec3;

/// A vehicle as of the last step.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Which frame this is from.
    pub frame: usize,
    /// Simulation time, seconds.
    pub time: f64,
    /// Pose, velocity and acceleration together, as
    /// `simGetGroundTruthKinematics` answers.
    pub kinematics: KinematicsState,
    /// Roll, pitch and yaw, radians.
    pub euler: Vec3,
    /// The mode in force.
    pub mode: FlightMode,
    /// Whether it is armed.
    pub armed: bool,
    /// Whether a client holds it.
    pub api_control: bool,
    /// How many orders have been refused for want of control.
    pub refused: u32,
    /// What its sensors read.
    pub senses: Senses,
    /// Whether it is sitting on the ground.
    pub landed: bool,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            frame: 0,
            time: 0.0,
            kinematics: KinematicsState::default(),
            euler: Vec3::ZERO,
            mode: FlightMode::Stabilized,
            armed: false,
            api_control: false,
            refused: 0,
            senses: Senses::default(),
            landed: true,
        }
    }
}

impl Report {
    /// The report a vehicle's aspects add up to.
    pub fn of(
        frame: usize,
        time: f64,
        at: TimePoint,
        place: &Placement,
        motion: &Motion,
        pilot: &Autopilot,
        senses: &Senses,
    ) -> Self {
        Self {
            frame,
            time,
            kinematics: KinematicsState {
                pose: place.pose,
                twist: motion.twist,
                accelerations: motion.accel,
                timestamp: at,
            },
            euler: place.euler,
            mode: pilot.mode,
            armed: pilot.armed,
            api_control: pilot.api_control,
            refused: pilot.refused,
            senses: *senses,
            landed: place.altitude() <= GROUND_TOUCH && motion.climb().abs() < 0.05,
        }
    }

    /// Where it is.
    pub fn pose(&self) -> Pose {
        self.kinematics.pose
    }

    /// Height above the ground plane, metres.
    pub fn altitude(&self) -> f32 {
        self.kinematics.pose.position.z
    }

    /// Climb rate, m/s, positive up.
    pub fn climb(&self) -> f32 {
        self.kinematics.twist.linear.z
    }

    /// Speed over the ground, m/s.
    pub fn ground_speed(&self) -> f32 {
        self.kinematics.twist.linear.truncate().length()
    }
}

/// How close to the ground plane counts as touching it, metres. A
/// simulated ground is exactly flat, so this only has to swallow the
/// integrator's last step.
pub const GROUND_TOUCH: f32 = 0.05;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_on_the_ground_says_so_and_one_in_the_air_does_not() {
        let pilot = Autopilot::new();
        let senses = Senses::default();
        let down = Report::of(1, 0.1, 0, &Placement::home(), &Motion::still(), &pilot, &senses);
        assert!(down.landed);
        assert_eq!(down.altitude(), 0.0);

        let up = Report::of(
            2,
            0.2,
            0,
            &Placement::at(Vec3::new(0.0, 0.0, 10.0)),
            &Motion::still(),
            &pilot,
            &senses,
        );
        assert!(!up.landed);
        assert_eq!(up.altitude(), 10.0);
    }

    #[test]
    fn a_craft_settling_onto_the_ground_is_not_landed_until_it_stops() {
        let mut motion = Motion::still();
        motion.twist.linear.z = -2.0;
        let r = Report::of(1, 0.1, 0, &Placement::home(), &motion, &Autopilot::new(), &Senses::default());
        assert!(!r.landed, "still coming down at 2 m/s is not landed");
        assert_eq!(r.climb(), -2.0);
    }
}
