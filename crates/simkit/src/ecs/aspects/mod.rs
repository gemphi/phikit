//! What a vehicle *is*, as aspects bound to its word.
//!
//! A vehicle is a [`Word`](phiecs::prelude::Word) — `"drone"`, `"taxi 7"` —
//! and everything about it is an [`Aspect`] on that word, one store per
//! type, exactly as a phiplay player's position and role are. A system
//! reads the aspects it needs and writes the ones it owns; nothing holds a
//! vehicle struct.
//!
//! | aspect | what it holds | who writes it |
//! |---|---|---|
//! | [`Placement`] | where it is | the integrator, at `Couple` |
//! | [`Motion`] | how it is moving | the integrator, at `Couple` |
//! | [`Helm`] | what it is being asked for | `intake`, at `Ingest` |
//! | [`Autopilot`] | the controller, its mode and what it wants | `command`, at `PreCouple` |
//! | [`Senses`] | what its sensors read | `sense`, at `PostCouple` |
//! | [`Craft`] | what kind of vehicle it is, and its limits | nobody; set when it is coined |

pub mod craft;
pub mod pilotage;
pub mod senses;

pub use craft::{CarLimits, Craft, RotorLimits};
pub use pilotage::{Autopilot, Wanted};
pub use senses::Senses;

use super::link::Origin;
use crate::common::{GeoPoint, Pose, TimePoint, Twist};
use crate::input::ControlState;
use crate::physics::Accelerations;
use glam::{Quat, Vec3};
use phiecs::prelude::Aspect;

/// Where a vehicle is.
///
/// [`euler`](Self::euler) is the authority on attitude and
/// `pose.orientation` is derived from it by [`sync`](Self::sync), rather
/// than the other way round: a controller works in roll, pitch and yaw,
/// and pulling those back out of a quaternion every step loses the yaw's
/// winding and gimbals at the poles.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Placement {
    /// Position, and the orientation derived from `euler`. z is up.
    pub pose: Pose,
    /// Roll, pitch and yaw in radians.
    pub euler: Vec3,
}

impl Placement {
    /// Level, at `at`.
    pub fn at(at: Vec3) -> Self {
        let mut p = Self { pose: Pose::new(at, Quat::IDENTITY), euler: Vec3::ZERO };
        p.sync();
        p
    }

    /// Level, at the origin.
    pub fn home() -> Self {
        Self::at(Vec3::ZERO)
    }

    /// Rebuilds `pose.orientation` from `euler`. Every write to `euler`
    /// is followed by one of these.
    pub fn sync(&mut self) {
        let Vec3 { x: roll, y: pitch, z: yaw } = self.euler;
        self.pose.orientation = Quat::from_euler(glam::EulerRot::ZYX, yaw, pitch, roll);
    }

    /// Height above the ground plane, in metres.
    pub fn altitude(&self) -> f32 {
        self.pose.position.z
    }

    /// Roll, in radians.
    pub fn roll(&self) -> f32 {
        self.euler.x
    }

    /// Pitch, in radians.
    pub fn pitch(&self) -> f32 {
        self.euler.y
    }

    /// Yaw, in radians.
    pub fn yaw(&self) -> f32 {
        self.euler.z
    }
}

impl Aspect for Placement {}

/// How a vehicle is moving.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Motion {
    /// Linear and angular velocity, in the world frame.
    pub twist: Twist,
    /// Linear and angular acceleration, as the last step produced them.
    pub accel: Accelerations,
}

impl Motion {
    /// Still.
    pub fn still() -> Self {
        Self::default()
    }

    /// Speed over the ground, m/s.
    pub fn ground_speed(&self) -> f32 {
        self.twist.linear.truncate().length()
    }

    /// Climb rate, m/s, positive up.
    pub fn climb(&self) -> f32 {
        self.twist.linear.z
    }
}

impl Aspect for Motion {}

/// What a vehicle is being asked for, and who asked.
///
/// This is the control state from [`input`](crate::input) bound to the
/// vehicle. It is the *only* way a command reaches a vehicle, whether a
/// stick or an AirSim client produced it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Helm {
    /// The demand.
    pub control: ControlState,
    /// Which door it came through.
    pub from: Origin,
    /// When it was taken in.
    pub at: TimePoint,
}

impl Helm {
    /// A helm holding `control`, from `from`.
    pub fn new(control: ControlState, from: Origin) -> Self {
        Self { control, from, at: 0 }
    }
}

impl Aspect for Helm {}

/// Where a vehicle's home is, for the GPS and a return home.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Home {
    /// The geographic point the world's origin sits at.
    pub geo: GeoPoint,
}

impl Aspect for Home {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placement_derives_its_orientation_from_its_euler_angles() {
        let mut p = Placement::at(Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(p.altitude(), 5.0);
        assert_eq!(p.pose.orientation, Quat::IDENTITY);

        p.euler.z = std::f32::consts::FRAC_PI_2;
        p.sync();
        let nose = p.pose.orientation * Vec3::X;
        assert!(nose.y > 0.99, "a quarter turn of yaw points the nose along +y: {nose:?}");
        assert_eq!(p.yaw(), std::f32::consts::FRAC_PI_2);
    }
}
