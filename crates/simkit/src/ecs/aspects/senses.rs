//! What a vehicle's sensors read, as an aspect on its word.

use crate::sensors::{BarometerOutput, GpsOutput, ImuOutput};
use phiecs::prelude::Aspect;

/// What a vehicle's sensors read, as of the last step.
///
/// Ground truth with the sensor models' own output types, so an AirSim
/// client's `getImuData` and the rest have something real to answer with.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Senses {
    /// Rates and specific force in the body frame.
    pub imu: ImuOutput,
    /// Where it thinks it is.
    pub gps: GpsOutput,
    /// Pressure altitude.
    pub baro: BarometerOutput,
    /// Height above the ground directly below, as a downward-facing
    /// distance sensor reads it.
    pub agl: f32,
}

impl Aspect for Senses {}
