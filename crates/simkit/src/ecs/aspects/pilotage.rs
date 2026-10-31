//! The vehicle's own controller as an aspect: what mode it is in,
//! who holds it, and what it wants of the airframe this step.

use super::Origin;
use crate::controllers::FlightMode;
use glam::Vec3;
use phiecs::prelude::Aspect;

/// What the vehicle's own controller wants of it this step: a collective
/// acceleration along its z, the attitude to hold, and how fast to turn.
///
/// This is the cascade's last rung before the physics — AirSim's
/// `simple_flight` ends at motor values and a mixer; simkit ends at the
/// wrench those motors would make, which is the same thing one layer up.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Wanted {
    /// Collective acceleration along the craft's own z, m/s².
    pub collective: f32,
    /// Roll to hold, radians.
    pub roll: f32,
    /// Pitch to hold, radians.
    pub pitch: f32,
    /// Yaw rate, rad/s.
    pub yaw_rate: f32,
}

/// A vehicle's controller: which mode it is in, whether it is armed, who
/// holds it, and what it wants this step.
///
/// The PID loops live here rather than in a resource because each vehicle
/// has its own integrator state — two drones in one world must not share
/// a wound-up altitude term.
#[derive(Debug, Clone)]
pub struct Autopilot {
    /// The mode in force. A stick's [`RotorMode`](crate::input::RotorMode)
    /// maps onto this, and so does an AirSim `Mode` call, so the two meet
    /// in one field.
    pub mode: FlightMode,
    /// Whether the vehicle is armed.
    pub armed: bool,
    /// Whether a client over the wire holds the vehicle. With this on, a
    /// device's orders are refused; with it off, a client's are.
    pub api_control: bool,
    /// What the controller wants this step.
    pub wanted: Wanted,
    /// Where a position-holding or landing manoeuvre is aiming.
    pub target: Vec3,
    /// Whether the controller is flying to [`target`](Self::target)
    /// rather than to the sticks.
    ///
    /// Both reach the controller in the same mode — `moveToPosition`,
    /// `hover` and a pilot's position-hold switch are all
    /// [`FlightMode::PositionHold`] — and they mean different things: a
    /// commanded point, or sticks that are velocities. A commanded move
    /// sets this; the next [`Helm`](super::Helm) clears it, which is what an AirSim
    /// `moveByRC` arriving mid-flight is *for*.
    pub to_target: bool,
    /// Where the vehicle was armed, for a return home.
    pub home: Vec3,
    /// How many orders have been refused for want of control, so a pilot
    /// can be told why nothing is moving.
    pub refused: u32,
    /// Integral of the climb-rate error, for the vertical loop.
    pub climb_wind: f32,
    /// Integral of the horizontal velocity error, for the position loop.
    pub drift_wind: Vec3,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self {
            mode: FlightMode::Stabilized,
            armed: false,
            api_control: false,
            wanted: Wanted::default(),
            target: Vec3::ZERO,
            to_target: false,
            home: Vec3::ZERO,
            refused: 0,
            climb_wind: 0.0,
            drift_wind: Vec3::ZERO,
        }
    }
}

impl Autopilot {
    /// A fresh controller, disarmed, with the stick holding it.
    pub fn new() -> Self {
        Self::default()
    }

    /// A controller a client already holds, as `enableApiControl` in
    /// `settings.json` would leave it.
    pub fn under_api() -> Self {
        Self { api_control: true, ..Self::default() }
    }

    /// Whether an order from `origin` may command this vehicle.
    ///
    /// AirSim's rule: the client holds it or the stick does, never both.
    pub fn holds_control(&self, origin: Origin) -> bool {
        match (origin, self.api_control) {
            (Origin::Api, true) => true,
            (Origin::Device, false) => true,
            _ => false,
        }
    }

    /// Forgets the wound-up integrator terms, as a mode change should.
    pub fn unwind(&mut self) {
        self.climb_wind = 0.0;
        self.drift_wind = Vec3::ZERO;
    }
}

impl Aspect for Autopilot {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_is_held_by_exactly_one_side() {
        let stick = Autopilot::new();
        assert!(stick.holds_control(Origin::Device));
        assert!(
            !stick.holds_control(Origin::Api),
            "the client cannot command what it has not taken"
        );

        let client = Autopilot::under_api();
        assert!(client.holds_control(Origin::Api));
        assert!(
            !client.holds_control(Origin::Device),
            "and the stick is out while the client has it"
        );
    }
}
