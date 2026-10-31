//! What kind of vehicle a word is, and the limits it flies or drives
//! to — the controller's units, rather than the rotor counts and arm
//! lengths the airframe configs carry.

use crate::input::VehicleKind;
use crate::vehicles::car::CarConfig;
use crate::vehicles::multirotor::MultirotorConfig;
use phiecs::prelude::Aspect;

/// What kind of vehicle this is, and the limits it flies or drives to.
#[derive(Debug, Clone, PartialEq)]
pub enum Craft {
    /// A multirotor.
    Rotor(Box<RotorLimits>),
    /// A car.
    Car(Box<CarLimits>),
}

impl Craft {
    /// A stock multirotor.
    pub fn rotor() -> Self {
        Craft::Rotor(Box::new(RotorLimits::default()))
    }

    /// A stock car.
    pub fn car() -> Self {
        Craft::Car(Box::new(CarLimits::default()))
    }

    /// Which kind, for picking the demand a device should produce.
    pub fn kind(&self) -> VehicleKind {
        match self {
            Craft::Rotor(_) => VehicleKind::Rotor,
            Craft::Car(_) => VehicleKind::Car,
        }
    }

    /// The multirotor's limits, if that is what this is.
    pub fn rotor_limits(&self) -> Option<&RotorLimits> {
        match self {
            Craft::Rotor(l) => Some(l),
            Craft::Car(_) => None,
        }
    }

    /// The car's limits, if that is what this is.
    pub fn car_limits(&self) -> Option<&CarLimits> {
        match self {
            Craft::Car(l) => Some(l),
            Craft::Rotor(_) => None,
        }
    }
}

impl Aspect for Craft {}

/// What a multirotor can be asked to do, in the units the controller
/// works in rather than the rotor counts and arm lengths
/// [`MultirotorConfig`] carries.
#[derive(Debug, Clone, PartialEq)]
pub struct RotorLimits {
    /// The airframe it came from.
    pub config: MultirotorConfig,
    /// Most collective acceleration the rotors can make, m/s². Above
    /// gravity, or it cannot leave the ground.
    pub max_collective: f32,
    /// Most tilt the angle modes will command, radians.
    pub max_tilt: f32,
    /// Most body rate the acro mode will command, rad/s.
    pub max_rate: f32,
    /// Most yaw rate, rad/s.
    pub max_yaw_rate: f32,
    /// Most climb rate the hold modes will command, m/s.
    pub max_climb: f32,
    /// Most descent rate, m/s.
    pub max_descent: f32,
    /// Most horizontal speed the position mode will command, m/s.
    pub max_speed: f32,
    /// How fast the attitude loop closes on the attitude it is asked
    /// for, as a time constant in seconds.
    pub attitude_lag: f32,
    /// Drag per unit speed, 1/s.
    pub drag: f32,
}

impl Default for RotorLimits {
    fn default() -> Self {
        let config = MultirotorConfig::default();
        Self {
            max_tilt: config.max_tilt_angle.to_radians(),
            max_yaw_rate: config.max_yaw_rate.to_radians(),
            config,
            max_collective: 22.0,
            max_rate: 6.0,
            max_climb: 5.0,
            max_descent: 3.0,
            max_speed: 12.0,
            attitude_lag: 0.12,
            drag: 0.25,
        }
    }
}

impl RotorLimits {
    /// Limits for `config`'s airframe.
    pub fn for_config(config: MultirotorConfig) -> Self {
        Self {
            max_tilt: config.max_tilt_angle.to_radians(),
            max_yaw_rate: config.max_yaw_rate.to_radians(),
            max_collective: (config.max_thrust_per_rotor * config.rotor_count as f32
                / config.vehicle.mass)
                .max(12.0),
            drag: config.drag_coefficient.max(0.05),
            config,
            ..Self::default()
        }
    }
}

/// What a car can be asked to do.
#[derive(Debug, Clone, PartialEq)]
pub struct CarLimits {
    /// The car it came from.
    pub config: CarConfig,
    /// Most steering angle at the wheels, radians.
    pub max_steer: f32,
    /// Most acceleration the engine can make, m/s².
    pub max_accel: f32,
    /// Most deceleration the brakes can make, m/s².
    pub max_brake: f32,
    /// Rolling resistance per unit speed, 1/s.
    pub rolling: f32,
    /// How many forward gears.
    pub gears: u8,
}

impl Default for CarLimits {
    fn default() -> Self {
        let config = CarConfig::default();
        Self {
            max_steer: config.max_steering_angle.to_radians(),
            max_accel: config.max_force / config.vehicle.mass,
            max_brake: 8.0,
            rolling: 0.08,
            gears: 6,
            config,
        }
    }
}

impl CarLimits {
    /// Limits for `config`'s car.
    pub fn for_config(config: CarConfig) -> Self {
        Self {
            max_steer: config.max_steering_angle.to_radians(),
            max_accel: config.max_force / config.vehicle.mass,
            config,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_craft_knows_which_demand_it_wants() {
        assert_eq!(Craft::rotor().kind(), VehicleKind::Rotor);
        assert_eq!(Craft::car().kind(), VehicleKind::Car);
        assert!(Craft::rotor().car_limits().is_none());
        assert!(Craft::car().car_limits().is_some());
    }

    #[test]
    fn a_rotors_limits_can_lift_it() {
        let limits = RotorLimits::for_config(MultirotorConfig::default());
        assert!(
            limits.max_collective > 9.81,
            "an airframe that cannot hover is not an airframe"
        );
        assert!(limits.max_tilt > 0.0 && limits.max_tilt < std::f32::consts::FRAC_PI_2);
    }
}
