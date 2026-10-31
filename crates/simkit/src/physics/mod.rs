//! Physics simulation types for rust-sim-kit

/// Physics Engines
pub mod engine;
mod physics_body;

mod physics_world;

pub use physics_body::{CollisionInfo, InertiaTensor, PhysicsBody, PhysicsBodyVertex};
pub use engine::{
    FastPhysicsEngine, PhysicsEngine,
};
pub use physics_world::{PhysicsParams, PhysicsWorld};

use crate::common::{GeoPoint, Pose, TimePoint, Twist};
use glam::{Quat, Vec3};

/// Kinematics state for a vehicle
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct KinematicsState {
    /// Current pose
    pub pose: Pose,
    /// Linear and angular velocity
    pub twist: Twist,
    /// Linear and angular acceleration
    pub accelerations: Accelerations,
    /// Timestamp
    pub timestamp: TimePoint,
}

impl KinematicsState {
    /// Create a new kinematics state
    pub fn new(pose: Pose, twist: Twist, accelerations: Accelerations) -> Self {
        Self {
            pose,
            twist,
            accelerations,
            timestamp: 0,
        }
    }

    /// Get position
    pub fn position(&self) -> Vec3 {
        self.pose.position
    }

    /// Get orientation
    pub fn orientation(&self) -> Quat {
        self.pose.orientation
    }

    /// Get linear velocity
    pub fn linear_velocity(&self) -> Vec3 {
        self.twist.linear
    }

    /// Get angular velocity
    pub fn angular_velocity(&self) -> Vec3 {
        self.twist.angular
    }
}

/// Acceleration components
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Accelerations {
    /// Linear acceleration (m/s^2)
    pub linear: Vec3,
    /// Angular acceleration (rad/s^2)
    pub angular: Vec3,
}

impl Accelerations {
    /// Zero accelerations
    pub fn zero() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }
}

/// Environment state (wind, temperature, pressure)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentState {
    /// Wind vector (m/s)
    pub wind: Vec3,
    /// Temperature (Celsius)
    pub temperature: f32,
    /// Barometric pressure (Pa)
    pub pressure: f32,
    /// Timestamp
    pub timestamp: TimePoint,
    /// Home geo point for GPS reference
    pub home_geo_point: GeoPoint,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            wind: Vec3::ZERO,
            temperature: 20.0,
            pressure: 101325.0,
            timestamp: 0,
            home_geo_point: GeoPoint::default(),
        }
    }
}

/// Environment simulation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Environment {
    /// Current state
    state: EnvironmentState,
    /// Home geo point for reference
    home_geo_point: GeoPoint,
}

impl Environment {
    /// Create new environment
    pub fn new(home_geo_point: GeoPoint) -> Self {
        Self {
            state: EnvironmentState::default(),
            home_geo_point,
        }
    }

    /// Get current state
    pub fn state(&self) -> &EnvironmentState {
        &self.state
    }

    /// Get home geo point
    pub fn home_geo_point(&self) -> &GeoPoint {
        &self.home_geo_point
    }

    /// Set wind
    pub fn set_wind(&mut self, wind: Vec3) {
        self.state.wind = wind;
    }

    /// Set temperature
    pub fn set_temperature(&mut self, temperature: f32) {
        self.state.temperature = temperature;
    }

    /// Set pressure
    pub fn set_pressure(&mut self, pressure: f32) {
        self.state.pressure = pressure;
    }
}

/// Wrench (force and torque) for physics simulation
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Wrench {
    /// Force vector (N)
    pub force: Vec3,
    /// Torque vector (N*m)
    pub torque: Vec3,
}

impl Wrench {
    /// Zero wrench
    pub fn zero() -> Self {
        Self {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        }
    }
}

impl std::ops::Add for Wrench {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            force: self.force + other.force,
            torque: self.torque + other.torque,
        }
    }
}

impl std::ops::AddAssign for Wrench {
    fn add_assign(&mut self, other: Self) {
        self.force += other.force;
        self.torque += other.torque;
    }
}
