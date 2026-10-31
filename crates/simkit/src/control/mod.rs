//! Control systems for rust-sim-kit
//!
//! Provides PID controllers, rate controllers, and position controllers
//! similar to AirSim's firmware implementations.

use glam::Vec3;

pub mod pid;
pub mod position;
pub mod rate;

pub use pid::{PidConfig, PidController, PidController3, PidOutput};
pub use position::{PositionConfig, PositionController, PositionSetpoint};
pub use rate::{RateConfig, RateController, RateSetpoint};

/// Control mode for vehicles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ControlMode {
    /// Manual RC control
    Manual,
    /// Position control (GPS/position hold)
    #[default]
    Position,
    /// Velocity control
    Velocity,
    /// Attitude/rate control
    Attitude,
    /// Acro/rate-only mode
    Acro,
}

/// Controller trait for generic control algorithms
pub trait Controller {
    /// Type of input (setpoint)
    type Setpoint;
    /// Type of measurement/feedback
    type Measurement;
    /// Type of output (control signal)
    type Output;

    /// Reset controller state
    fn reset(&mut self);

    /// Update controller with new setpoint and measurement
    fn update(
        &mut self,
        setpoint: &Self::Setpoint,
        measurement: &Self::Measurement,
        dt: f32,
    ) -> Self::Output;

    /// Check if controller is active
    fn is_active(&self) -> bool;
}

/// Attitude setpoint (roll, pitch, yaw)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct AttitudeSetpoint {
    /// Desired roll angle (rad)
    pub roll: f32,
    /// Desired pitch angle (rad)
    pub pitch: f32,
    /// Desired yaw angle (rad)
    pub yaw: f32,
    /// Desired thrust (0.0 to 1.0)
    pub thrust: f32,
}

impl AttitudeSetpoint {
    /// Create new attitude setpoint
    pub fn new(roll: f32, pitch: f32, yaw: f32, thrust: f32) -> Self {
        Self {
            roll,
            pitch,
            yaw,
            thrust: thrust.clamp(0.0, 1.0),
        }
    }

    /// Zero attitude setpoint (level, no thrust)
    pub fn zero() -> Self {
        Self {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            thrust: 0.0,
        }
    }

    /// Level setpoint (roll=0, pitch=0, with thrust)
    pub fn level(thrust: f32) -> Self {
        Self {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            thrust: thrust.clamp(0.0, 1.0),
        }
    }
}

/// Velocity setpoint
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct VelocitySetpoint {
    /// Desired velocity in body X (m/s)
    pub vx: f32,
    /// Desired velocity in body Y (m/s)
    pub vy: f32,
    /// Desired velocity in body Z (m/s)
    pub vz: f32,
    /// Desired yaw rate (rad/s)
    pub yaw_rate: f32,
}

impl VelocitySetpoint {
    /// Create new velocity setpoint
    pub fn new(vx: f32, vy: f32, vz: f32, yaw_rate: f32) -> Self {
        Self {
            vx,
            vy,
            vz,
            yaw_rate,
        }
    }

    /// Zero velocity setpoint (hover)
    pub fn hover() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw_rate: 0.0,
        }
    }

    /// Get as Vec3
    pub fn linear(&self) -> Vec3 {
        Vec3::new(self.vx, self.vy, self.vz)
    }
}

/// Motor outputs (normalized 0.0 to 1.0)
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct MotorOutputs {
    /// Output for each motor
    pub values: Vec<f32>,
}

impl MotorOutputs {
    /// Create new motor outputs for given number of motors
    pub fn new(count: usize) -> Self {
        Self {
            values: vec![0.0; count],
        }
    }

    /// Create for quadcopter (4 motors)
    pub fn quad(front_right: f32, rear_left: f32, front_left: f32, rear_right: f32) -> Self {
        Self {
            values: vec![
                front_right.clamp(0.0, 1.0),
                rear_left.clamp(0.0, 1.0),
                front_left.clamp(0.0, 1.0),
                rear_right.clamp(0.0, 1.0),
            ],
        }
    }

    /// Set all motors to same value
    pub fn uniform(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        for v in &mut self.values {
            *v = clamped;
        }
    }

    /// Get number of motors
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// Clamp all values to [0, 1]
    pub fn clamp(&mut self) {
        for v in &mut self.values {
            *v = v.clamp(0.0, 1.0);
        }
    }
}

/// Controller gains for PID
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ControllerGains {
    /// Proportional gain
    pub kp: f32,
    /// Integral gain
    pub ki: f32,
    /// Derivative gain
    pub kd: f32,
    /// Integral limit (anti-windup)
    pub i_limit: f32,
}

impl Default for ControllerGains {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.0,
            kd: 0.0,
            i_limit: 1.0,
        }
    }
}

impl ControllerGains {
    /// Create new gains
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            i_limit: 1.0,
        }
    }

    /// Create with integral limit
    pub fn with_i_limit(mut self, limit: f32) -> Self {
        self.i_limit = limit;
        self
    }
}
