//! RPC API client for connecting to simulation servers
//!
//! Provides AirSim-compatible APIs for:
//! - Connection management
//! - Vehicle control
//! - Sensor data retrieval
//! - Scene manipulation
//! - Camera/image capture

use crate::common::{
    CollisionInfo, GeoPoint,
};
use crate::physics::KinematicsState;

pub mod client;

pub use client::RpcClient;

/// Connection states for the RPC client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Initial state
    Initial,
    /// Connected to server
    Connected,
    /// Disconnected
    Disconnected,
    /// Reset in progress
    Reset,
    /// Unknown state
    Unknown,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Initial
    }
}

/// Simulation weather parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherParameter {
    /// Rain intensity (0-1)
    Rain = 0,
    /// Snow intensity (0-1)
    Snow = 1,
    /// Fog density (0-1)
    Fog = 2,
    /// Cloud density (0-1)
    Cloud = 3,
    /// Wind intensity (0-1)
    Wind = 4,
}

/// Drivetrain type for multirotor control (Forward-only vs Max-degree-of-freedom)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum DrivetrainType {
    /// Only forward/backward motion
    ForwardOnly = 0,
    /// Full 6-DOF control (default)
    #[default]
    MaxDegreeOfFreedom = 1,
}

/// Yaw mode for multirotor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum YawMode {
    /// Maintain current yaw angle (default)
    #[default]
    None = 0,
    /// Look at target while moving
    LookAtTarget = 1,
}

/// Landed state for multirotors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum LandedState {
    /// Vehicle is on the ground
    #[default]
    Landed = 0,
    /// Vehicle is flying
    Flying = 1,
}

/// Rotor parameters
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct RotorParameters {
    /// Thrust produced by rotor (N)
    pub thrust: f32,
    /// Torque produced by rotor (N*m)
    pub torque: f32,
    /// Rotation speed (rad/s)
    pub speed: f32,
}

/// Rotor states for all rotors on a vehicle
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct RotorStates {
    /// States for each rotor
    pub rotors: Vec<RotorParameters>,
    /// Timestamp
    pub timestamp: u64,
}

/// Multirotor state information
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct MultirotorState {
    /// Collision information
    pub collision: CollisionInfo,
    /// Estimated kinematics
    pub kinematics_estimated: KinematicsState,
    /// Current GPS location
    pub gps_location: GeoPoint,
    /// Timestamp when state was captured
    pub timestamp: u64,
    /// Current landed state
    pub landed_state: LandedState,
    /// RC data (if available)
    pub rc_data: crate::common::RcData,
    /// Whether vehicle is ready for commands
    pub ready: bool,
    /// Status message
    pub ready_message: String,
    /// Whether vehicle can be armed
    pub can_arm: bool,
}

/// Car control inputs (steering, throttle, brake)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct CarControls {
    /// Throttle (0 to 1)
    pub throttle: f32,
    /// Steering angle (-1 to 1, left to right)
    pub steering: f32,
    /// Brake (0 to 1)
    pub brake: f32,
    /// Hand brake applied
    pub handbrake: bool,
    /// Is manual gear
    pub is_manual_gear: bool,
    /// Gear number (0=neutral, -1=reverse, 1-5=forward gears)
    pub gear: i8,
    /// Apply gear change immediately
    pub gear_immediate: bool,
}

impl CarControls {
    /// Create default car controls
    pub fn new() -> Self {
        Self {
            gear_immediate: true,
            ..Default::default()
        }
    }

    /// Set steering value
    pub fn with_steering(mut self, steering: f32) -> Self {
        self.steering = steering.clamp(-1.0, 1.0);
        self
    }

    /// Set throttle value
    pub fn with_throttle(mut self, throttle: f32) -> Self {
        self.throttle = throttle.clamp(0.0, 1.0);
        self
    }

    /// Set brake value
    pub fn with_brake(mut self, brake: f32) -> Self {
        self.brake = brake.clamp(0.0, 1.0);
        self
    }

    /// Set handbrake
    pub fn with_handbrake(mut self, handbrake: bool) -> Self {
        self.handbrake = handbrake;
        self
    }

    /// Set gear
    pub fn with_gear(mut self, gear: i8) -> Self {
        self.gear = gear;
        self.is_manual_gear = true;
        self
    }
}

/// Car state information
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct CarState {
    /// Current speed (m/s)
    pub speed: f32,
    /// Current gear
    pub gear: i8,
    /// RPM of engine
    pub rpm: f32,
    /// Maximum RPM
    pub max_rpm: f32,
    /// Is handbrake applied
    pub handbrake: bool,
    /// Estimated kinematics
    pub kinematics_estimated: KinematicsState,
    /// Timestamp when state was captured
    pub timestamp: u64,
}
