//! The one door into the world, and the one window out of it.
//!
//! A vehicle can be flown from a stick in the room or from an AirSim
//! client over msgpack-rpc, and the two must not be able to disagree
//! about what happened. So neither touches a vehicle's aspects. Both push
//! an [`Order`] onto a [`ControlLink`], [`intake`](super::intake) is the
//! only thing that drains it, and `Readout`
//! writes what the world then did back into the same link for whoever is
//! asking. Orders in, reports out, one object, both directions.
//!
//! Which of the two is *obeyed* at any moment is AirSim's rule, not a new
//! one: `enableApiControl(true)` hands the vehicle to the client and the
//! stick is ignored; with it off the stick flies and the client's commands
//! are refused (`VehicleApiBase::enableApiControl`,
//! `MultirotorApiBase::canArm`). [`intake`](super::intake) counts what it
//! refuses rather than dropping it quietly, so a pilot wondering why the
//! drone will not move can be told.

use super::report::Report;
use crate::common::TimePoint;
use crate::controllers::FlightMode;
use crate::input::ControlState;
use glam::Vec3;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Which door an order came through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// A device in the room, read through [`Pilot`](crate::input::Pilot).
    Device,
    /// A client over msgpack-rpc.
    Api,
}

/// What is being asked of a vehicle.
///
/// [`Helm`](Act::Helm) is the stick-level command both doors share. The
/// rest are the higher-level AirSim calls, which only the holder of the
/// vehicle may give.
#[derive(Debug, Clone, PartialEq)]
pub enum Act {
    /// Fly or drive it like this. The whole control state, every frame.
    Helm(ControlState),
    /// Arm or disarm (`armDisarm`).
    Arm(bool),
    /// Take the vehicle, or hand it back (`enableApiControl`).
    ApiControl(bool),
    /// Climb to `altitude` metres and hold (`takeoff`).
    Takeoff {
        /// Metres above where it sits now.
        altitude: f32,
    },
    /// Descend until the ground, then disarm (`land`).
    Land,
    /// Hold where it is (`hover`).
    Hover,
    /// Fly to a point (`moveToPosition`), in simkit's z-up world frame.
    MoveToPosition {
        /// Where to.
        to: Vec3,
        /// How fast to go there, m/s.
        speed: f32,
    },
    /// Fly at a velocity (`moveByVelocity`), in simkit's z-up world frame.
    MoveByVelocity {
        /// How fast, m/s.
        velocity: Vec3,
        /// And turning how fast, rad/s.
        yaw_rate: f32,
    },
    /// Put the flight controller in a mode outright.
    Mode(FlightMode),
    /// Put the vehicle back where it started (`reset`).
    Reset,
}

/// One thing asked of one vehicle.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    /// Which vehicle, by the spelling of its word.
    pub vehicle: String,
    /// What is asked.
    pub act: Act,
    /// Which door it came through.
    pub origin: Origin,
    /// When it was sent.
    pub at: TimePoint,
}

impl Order {
    /// An order from a device in the room.
    pub fn from_device(vehicle: impl Into<String>, act: Act) -> Self {
        Self { vehicle: vehicle.into(), act, origin: Origin::Device, at: 0 }
    }

    /// An order from a client over the wire.
    pub fn from_api(vehicle: impl Into<String>, act: Act) -> Self {
        Self { vehicle: vehicle.into(), act, origin: Origin::Api, at: 0 }
    }

    /// The same order, stamped.
    pub fn at(mut self, at: TimePoint) -> Self {
        self.at = at;
        self
    }
}

/// Orders waiting to be taken in, and the last report of each vehicle.
///
/// Cloning a link shares it, so the RPC server, a device loop and the
/// world can each hold one.
#[derive(Clone, Default)]
pub struct ControlLink {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    orders: Mutex<VecDeque<Order>>,
    reports: Mutex<HashMap<String, Report>>,
}

impl ControlLink {
    /// A link with nothing on it.
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts `order` on the link.
    pub fn send(&self, order: Order) {
        if let Ok(mut q) = self.inner.orders.lock() {
            q.push_back(order);
        }
    }

    /// Asks for `act` on behalf of a device in the room.
    pub fn device(&self, vehicle: &str, act: Act) {
        self.send(Order::from_device(vehicle, act));
    }

    /// Asks for `act` on behalf of a client over the wire.
    pub fn api(&self, vehicle: &str, act: Act) {
        self.send(Order::from_api(vehicle, act));
    }

    /// Hands over everything waiting, in the order it arrived, leaving
    /// the link empty. Only [`intake`](super::intake) calls this.
    pub fn drain(&self) -> Vec<Order> {
        match self.inner.orders.lock() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// How many orders are waiting.
    pub fn waiting(&self) -> usize {
        self.inner.orders.lock().map(|q| q.len()).unwrap_or(0)
    }

    /// Records what a vehicle is doing, for whoever is asking.
    pub fn publish(&self, vehicle: &str, report: Report) {
        if let Ok(mut r) = self.inner.reports.lock() {
            r.insert(vehicle.to_string(), report);
        }
    }

    /// The last report of `vehicle`.
    pub fn report(&self, vehicle: &str) -> Option<Report> {
        self.inner.reports.lock().ok()?.get(vehicle).cloned()
    }

    /// The vehicles that have reported, in no particular order.
    pub fn reported(&self) -> Vec<String> {
        self.inner.reports.lock().map(|r| r.keys().cloned().collect()).unwrap_or_default()
    }
}

impl std::fmt::Debug for ControlLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControlLink").field("waiting", &self.waiting()).finish_non_exhaustive()
    }
}

/// AirSim's wire is NED — north, east, *down* — and simkit's world has z
/// up, which is the frame `glam` and phiray place a pose in. Every vector
/// crossing the wire turns here, and nowhere else, so there is one place
/// to be wrong about it. The turn is its own inverse.
pub fn flip_ned(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.y, -v.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::RotorDemand;

    #[test]
    fn a_link_hands_over_its_orders_in_the_order_they_arrived() {
        let link = ControlLink::new();
        link.device("drone", Act::Helm(ControlState::Rotor(RotorDemand::idle())));
        link.api("drone", Act::Arm(true));
        assert_eq!(link.waiting(), 2);

        let drained = link.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].origin, Origin::Device);
        assert_eq!(drained[1].act, Act::Arm(true));
        assert_eq!(link.waiting(), 0, "and the link is empty afterwards");
    }

    #[test]
    fn a_cloned_link_is_the_same_link() {
        let link = ControlLink::new();
        let held_elsewhere = link.clone();
        held_elsewhere.api("car", Act::Reset);
        assert_eq!(link.waiting(), 1, "what the server pushed, the world drains");
    }

    #[test]
    fn a_report_is_readable_by_whoever_holds_the_link() {
        let link = ControlLink::new();
        assert!(link.report("drone").is_none());
        link.publish("drone", Report::default());
        assert!(link.report("drone").is_some());
        assert_eq!(link.reported(), vec!["drone".to_string()]);
    }

    #[test]
    fn the_ned_flip_undoes_itself() {
        let up = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(flip_ned(flip_ned(up)), up);
        assert_eq!(flip_ned(up).z, -3.0, "a climb in the world is negative down the wire");
    }
}
