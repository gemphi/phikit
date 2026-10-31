//! The control state: what the pilot or driver is asking for, in the units
//! the vehicle is commanded in.
//!
//! This is the whole point of the input layer. Nothing downstream of here
//! ever sees a button code or an axis number — it sees roll, pitch, a
//! yaw rate and a throttle, or a steer, a throttle, a brake and a gear,
//! which is how a real vehicle is commanded and how AirSim's own APIs put
//! it: `RCData` and `MultirotorApiBase::moveByRC` for the drone,
//! `CarApiBase::CarControls` for the car.

pub mod car;
pub mod rotor;

pub use car::{CarDemand, Gear};
pub use rotor::{Manoeuvre, RotorDemand, RotorMode};

/// What a vehicle is being asked for, whatever asked.
///
/// A device produces one of these through [`Pilot`](super::Pilot); an
/// AirSim client's `moveByRC` or `setCarControls` produces the same thing;
/// and the phiecs bridge binds it to the vehicle's word as its
/// `Helm` aspect. There is no third form.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlState {
    /// A multirotor's.
    Rotor(RotorDemand),
    /// A car's.
    Car(CarDemand),
}

impl ControlState {
    /// Which kind of vehicle this commands.
    pub fn kind(&self) -> VehicleKind {
        match self {
            ControlState::Rotor(_) => VehicleKind::Rotor,
            ControlState::Car(_) => VehicleKind::Car,
        }
    }

    /// The multirotor demand, if that is what this is.
    pub fn rotor(&self) -> Option<RotorDemand> {
        match self {
            ControlState::Rotor(d) => Some(*d),
            ControlState::Car(_) => None,
        }
    }

    /// The car demand, if that is what this is.
    pub fn car(&self) -> Option<CarDemand> {
        match self {
            ControlState::Car(d) => Some(*d),
            ControlState::Rotor(_) => None,
        }
    }
}

/// Which kind of vehicle a device is driving, so one binding table can
/// serve both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VehicleKind {
    /// A multirotor.
    #[default]
    Rotor,
    /// A car.
    Car,
}
