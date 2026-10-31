//! AirSim's msgpack-rpc calls, as orders and answers.
//!
//! This is the remote half of "one does not bypass the other". A client's
//! `armDisarm`, `takeoff`, `moveByVelocity`, `moveByRC` or `setCarControls`
//! becomes an [`Act`] here and goes onto the [`ControlLink`](super::ControlLink) — the same
//! link a gamepad pushes onto, drained by the same
//! [`Intake`](super::Intake). The RPC server's handlers do no flying of
//! their own; they translate, send, and answer from the last [`Report`].
//!
//! The method names, their parameter order and the shape of the replies
//! are AirSim's, so a client written against `airsim.MultirotorClient`
//! talks to this unchanged. Vectors arrive and leave in NED, which is
//! AirSim's frame; [`flip_ned`] is where that becomes simkit's z-up world
//! and the only place it does.
//!
//! What is *not* here is anything whose meaning would have to be guessed.
//! `moveByVelocityZ` carries a height where [`Act::MoveByVelocity`] wants
//! a rate, so it is left uncovered rather than covered wrongly, and a
//! client calling it is told the method is unknown.

use super::link::{flip_ned, Act};
use super::report::Report;
use crate::common::RcData;
use crate::input::{CarDemand, ControlState, RotorDemand};
use glam::Vec3;

/// The `vehicle_name` an AirSim call carries, which is `""` when the
/// client did not say and means "the only one".
pub const ANY_VEHICLE: &str = "";

/// The method names this translation covers, for a server to route on and
/// a test to hold it to.
pub const METHODS: &[&str] = &[
    "enableApiControl",
    "armDisarm",
    "takeoff",
    "land",
    "hover",
    "moveToPosition",
    "moveByVelocity",
    "moveByRC",
    "setCarControls",
    "reset",
];

/// One AirSim call read off the wire: which vehicle, and what it asks.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The `vehicle_name` parameter, or [`ANY_VEHICLE`].
    pub vehicle: String,
    /// What it asks for.
    pub act: Act,
}

/// A call's parameters, as much of them as matter, pulled out of the
/// msgpack array the client sent.
///
/// A trait rather than `rmpv::Value` directly so the translation can be
/// tested without building msgpack, and so a server that has already
/// deserialised into its own types can reuse it.
pub trait Params {
    /// The parameter at `index` as a float.
    fn number(&self, index: usize) -> Option<f64>;
    /// The parameter at `index` as a flag.
    fn flag(&self, index: usize) -> Option<bool>;
    /// The parameter at `index` as a string.
    fn text(&self, index: usize) -> Option<String>;
    /// How many parameters there are.
    fn count(&self) -> usize;

    /// The parameter at `index` as a float, or `fallback`.
    fn number_or(&self, index: usize, fallback: f32) -> f32 {
        self.number(index).map(|n| n as f32).unwrap_or(fallback)
    }

    /// The `vehicle_name` parameter, which AirSim puts last.
    fn vehicle_at(&self, index: usize) -> String {
        self.text(index).unwrap_or_else(|| ANY_VEHICLE.to_string())
    }
}

/// `method` and `params` as a [`Call`], or `None` if the method is not one
/// that commands a vehicle — a getter, a `ping`, an image request.
///
/// The parameter positions are AirSim's own, from `airsim/client.py`:
/// `armDisarm(arm, vehicle_name)`,
/// `takeoff(timeout_sec, vehicle_name)`,
/// `moveToPosition(x, y, z, velocity, …, vehicle_name)`,
/// `moveByVelocity(vx, vy, vz, duration, …, vehicle_name)`.
pub fn read<P: Params>(method: &str, params: &P) -> Option<Call> {
    let call = match method {
        "enableApiControl" => Call {
            vehicle: params.vehicle_at(1),
            act: Act::ApiControl(params.flag(0).unwrap_or(true)),
        },
        "armDisarm" => {
            Call { vehicle: params.vehicle_at(1), act: Act::Arm(params.flag(0).unwrap_or(true)) }
        }
        // AirSim's `takeoff` takes a timeout, not an altitude; the
        // altitude is the airframe's own default, as it is on a radio.
        "takeoff" => Call { vehicle: params.vehicle_at(1), act: Act::Takeoff { altitude: 3.0 } },
        "land" => Call { vehicle: params.vehicle_at(1), act: Act::Land },
        "hover" => Call { vehicle: params.vehicle_at(0), act: Act::Hover },
        "moveToPosition" => Call {
            vehicle: params.vehicle_at(params.count().saturating_sub(1)),
            act: Act::MoveToPosition {
                to: flip_ned(Vec3::new(
                    params.number_or(0, 0.0),
                    params.number_or(1, 0.0),
                    params.number_or(2, 0.0),
                )),
                speed: params.number_or(3, 5.0).abs(),
            },
        },
        "moveByVelocity" => Call {
            vehicle: params.vehicle_at(params.count().saturating_sub(1)),
            act: Act::MoveByVelocity {
                velocity: flip_ned(Vec3::new(
                    params.number_or(0, 0.0),
                    params.number_or(1, 0.0),
                    params.number_or(2, 0.0),
                )),
                yaw_rate: 0.0,
            },
        },
        "moveByRC" => Call {
            vehicle: params.vehicle_at(params.count().saturating_sub(1)),
            act: Act::Helm(ControlState::Rotor(rc_from(params))),
        },
        // `setCarControls(controls, vehicle_name)`, with the controls
        // flattened by the server into the leading parameters.
        "setCarControls" => Call {
            vehicle: params.vehicle_at(params.count().saturating_sub(1)),
            act: Act::Helm(ControlState::Car(CarDemand::from_car_controls(
                params.number_or(0, 0.0),
                params.number_or(1, 0.0),
                params.number_or(2, 0.0),
                params.flag(3).unwrap_or(false),
                params.flag(4).unwrap_or(false),
                params.number(5).unwrap_or(0.0) as i32,
            ))),
        },
        "reset" => Call { vehicle: params.vehicle_at(0), act: Act::Reset },
        _ => return None,
    };
    Some(call)
}

/// The four channels and two switches of a `moveByRC`, read as the demand
/// they are.
fn rc_from<P: Params>(params: &P) -> RotorDemand {
    let rc = RcData {
        roll: params.number_or(0, 0.0),
        pitch: params.number_or(1, 0.0),
        throttle: params.number_or(2, -1.0),
        yaw: params.number_or(3, 0.0),
        switches: params.number(4).unwrap_or(0.0).max(0.0) as u16,
        is_initialized: true,
        is_valid: true,
        ..RcData::default()
    };
    RotorDemand::from_rc(&rc)
}

/// Puts a client's call on the link as an API-origin order,
/// resolving [`ANY_VEHICLE`] against `only` — the vehicle a client that
/// named none must have meant. Hands back `false` if there is no such
/// vehicle to resolve to, which is what a client gets told.
pub fn send(link: &super::ControlLink, call: Call, only: &str) -> bool {
    let vehicle = match call.vehicle.as_str() {
        ANY_VEHICLE => only,
        named => named,
    };
    match vehicle.is_empty() {
        true => false,
        false => {
            link.api(vehicle, call.act);
            true
        }
    }
}

/// A report as the seven floats AirSim's `simGetVehiclePose` answers with:
/// a position in NED and an orientation as `w, x, y, z`.
pub fn pose_reply(report: &Report) -> [f64; 7] {
    let at = flip_ned(report.pose().position);
    let q = report.pose().orientation;
    [
        f64::from(at.x),
        f64::from(at.y),
        f64::from(at.z),
        f64::from(q.w),
        f64::from(q.x),
        f64::from(q.y),
        f64::from(q.z),
    ]
}

/// A report as the three vectors `simGetGroundTruthKinematics` answers
/// with: position, linear velocity and linear acceleration, all in NED.
pub fn kinematics_reply(report: &Report) -> [[f64; 3]; 3] {
    let k = report.kinematics;
    let ned = |v: Vec3| {
        let v = flip_ned(v);
        [f64::from(v.x), f64::from(v.y), f64::from(v.z)]
    };
    [ned(k.pose.position), ned(k.twist.linear), ned(k.accelerations.linear)]
}

#[cfg(test)]
mod tests;
