//! Rate controller for multirotor attitude rate control
//!
//! Controls angular velocity (roll rate, pitch rate, yaw rate).

use super::{ControllerGains, MotorOutputs, PidController3};
use glam::Vec3;

/// Rate controller configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RateConfig {
    /// Roll rate PID gains
    pub roll_gains: ControllerGains,
    /// Pitch rate PID gains  
    pub pitch_gains: ControllerGains,
    /// Yaw rate PID gains
    pub yaw_gains: ControllerGains,
    /// Maximum rate outputs (roll, pitch, yaw in rad/s command)
    pub output_limit: f32,
}

impl Default for RateConfig {
    fn default() -> Self {
        Self {
            roll_gains: ControllerGains::new(0.15, 0.0, 0.0),
            pitch_gains: ControllerGains::new(0.15, 0.0, 0.0),
            yaw_gains: ControllerGains::new(0.2, 0.0, 0.0),
            output_limit: 1.0,
        }
    }
}

impl RateConfig {
    /// Create default multirotor rate config
    pub fn multirotor_default() -> Self {
        Self {
            roll_gains: ControllerGains::new(0.15, 0.01, 0.005),
            pitch_gains: ControllerGains::new(0.15, 0.01, 0.005),
            yaw_gains: ControllerGains::new(0.2, 0.02, 0.0),
            output_limit: 1.0,
        }
    }

    /// Create racing quad config (more aggressive)
    pub fn racing_default() -> Self {
        Self {
            roll_gains: ControllerGains::new(0.25, 0.02, 0.01),
            pitch_gains: ControllerGains::new(0.25, 0.02, 0.01),
            yaw_gains: ControllerGains::new(0.3, 0.03, 0.0),
            output_limit: 1.0,
        }
    }

    /// Set roll gains
    pub fn with_roll_gains(mut self, kp: f32, ki: f32, kd: f32) -> Self {
        self.roll_gains = ControllerGains::new(kp, ki, kd);
        self
    }

    /// Set pitch gains
    pub fn with_pitch_gains(mut self, kp: f32, ki: f32, kd: f32) -> Self {
        self.pitch_gains = ControllerGains::new(kp, ki, kd);
        self
    }

    /// Set yaw gains
    pub fn with_yaw_gains(mut self, kp: f32, ki: f32, kd: f32) -> Self {
        self.yaw_gains = ControllerGains::new(kp, ki, kd);
        self
    }
}

/// Rate setpoint (desired angular rates)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct RateSetpoint {
    /// Desired roll rate (rad/s)
    pub roll_rate: f32,
    /// Desired pitch rate (rad/s)
    pub pitch_rate: f32,
    /// Desired yaw rate (rad/s)
    pub yaw_rate: f32,
    /// Desired thrust (0.0 to 1.0)
    pub thrust: f32,
}

impl RateSetpoint {
    /// Create new rate setpoint
    pub fn new(roll_rate: f32, pitch_rate: f32, yaw_rate: f32, thrust: f32) -> Self {
        Self {
            roll_rate,
            pitch_rate,
            yaw_rate,
            thrust: thrust.clamp(0.0, 1.0),
        }
    }

    /// Zero setpoint (hover with no rotation)
    pub fn hover() -> Self {
        Self {
            roll_rate: 0.0,
            pitch_rate: 0.0,
            yaw_rate: 0.0,
            thrust: 0.0,
        }
    }

    /// Get as Vec3 for rate components only
    pub fn rates(&self) -> Vec3 {
        Vec3::new(self.roll_rate, self.pitch_rate, self.yaw_rate)
    }
}

/// Rate controller for multirotor
#[derive(Debug, Clone, PartialEq)]
pub struct RateController {
    /// Configuration
    config: RateConfig,
    /// PID controllers for each axis
    pid: PidController3,
}

impl Default for RateController {
    fn default() -> Self {
        Self::new(RateConfig::default())
    }
}

impl RateController {
    /// Create new rate controller
    pub fn new(config: RateConfig) -> Self {
        let pid = PidController3::with_axis_gains(
            config.roll_gains,
            config.pitch_gains,
            config.yaw_gains,
        );

        Self { config, pid }
    }

    /// Reset controller
    pub fn reset(&mut self) {
        self.pid.reset();
    }

    /// Update controller with setpoint and measured rates
    pub fn update(
        &mut self,
        setpoint: &RateSetpoint,
        measured_rates: Vec3,
        dt: f32,
    ) -> AttitudeTorqueCommand {
        let rate_cmd = self.pid.update(setpoint.rates(), measured_rates, dt);

        AttitudeTorqueCommand {
            roll_torque: rate_cmd
                .x
                .clamp(-self.config.output_limit, self.config.output_limit),
            pitch_torque: rate_cmd
                .y
                .clamp(-self.config.output_limit, self.config.output_limit),
            yaw_torque: rate_cmd
                .z
                .clamp(-self.config.output_limit, self.config.output_limit),
            thrust: setpoint.thrust,
        }
    }

    /// Update and compute motor outputs for quadcopter
    pub fn update_quad(
        &mut self,
        setpoint: &RateSetpoint,
        measured_rates: Vec3,
        dt: f32,
    ) -> MotorOutputs {
        let cmd = self.update(setpoint, measured_rates, dt);
        cmd.to_quad_motors()
    }

    /// Get current gains
    pub fn config(&self) -> &RateConfig {
        &self.config
    }

    /// Set new configuration
    pub fn set_config(&mut self, config: RateConfig) {
        // Update PID gains first
        self.pid.x.set_gains(config.roll_gains);
        self.pid.y.set_gains(config.pitch_gains);
        self.pid.z.set_gains(config.yaw_gains);
        self.config = config;
    }

    /// Set roll gains
    pub fn set_roll_gains(&mut self, gains: ControllerGains) {
        self.config.roll_gains = gains;
        self.pid.x.set_gains(gains);
    }

    /// Set pitch gains
    pub fn set_pitch_gains(&mut self, gains: ControllerGains) {
        self.config.pitch_gains = gains;
        self.pid.y.set_gains(gains);
    }

    /// Set yaw gains
    pub fn set_yaw_gains(&mut self, gains: ControllerGains) {
        self.config.yaw_gains = gains;
        self.pid.z.set_gains(gains);
    }
}

/// Attitude torque command output from rate controller
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct AttitudeTorqueCommand {
    /// Roll torque command (-1 to 1)
    pub roll_torque: f32,
    /// Pitch torque command (-1 to 1)
    pub pitch_torque: f32,
    /// Yaw torque command (-1 to 1)
    pub yaw_torque: f32,
    /// Thrust command (0 to 1)
    pub thrust: f32,
}

impl AttitudeTorqueCommand {
    /// Create new torque command
    pub fn new(roll: f32, pitch: f32, yaw: f32, thrust: f32) -> Self {
        Self {
            roll_torque: roll.clamp(-1.0, 1.0),
            pitch_torque: pitch.clamp(-1.0, 1.0),
            yaw_torque: yaw.clamp(-1.0, 1.0),
            thrust: thrust.clamp(0.0, 1.0),
        }
    }

    /// Convert to quadcopter motor outputs (X configuration)
    ///
    /// Motor layout:
    ///   3 (FL)    1 (FR)
    ///      \    /
    ///       \  /
    ///        \/
    ///        /\
    ///       /  \
    ///   2 (RL)    4 (RR)
    ///
    /// FR: +roll, -pitch
    /// RL: -roll, +pitch  
    /// FL: +roll, +pitch
    /// RR: -roll, -pitch
    pub fn to_quad_motors(&self) -> MotorOutputs {
        let thrust = self.thrust;
        let roll = self.roll_torque;
        let pitch = self.pitch_torque;
        let yaw = self.yaw_torque;

        // Standard X-quad mixer
        // FR (1): thrust - pitch - roll + yaw
        // RL (2): thrust + pitch + roll + yaw
        // FL (3): thrust + pitch - roll - yaw
        // RR (4): thrust - pitch + roll - yaw
        let fr = thrust - pitch * 0.5 - roll * 0.5 + yaw * 0.5;
        let rl = thrust + pitch * 0.5 + roll * 0.5 + yaw * 0.5;
        let fl = thrust + pitch * 0.5 - roll * 0.5 - yaw * 0.5;
        let rr = thrust - pitch * 0.5 + roll * 0.5 - yaw * 0.5;

        MotorOutputs::quad(fr, rl, fl, rr)
    }

    /// Convert to quadcopter motor outputs (+ configuration)
    pub fn to_quad_plus_motors(&self) -> MotorOutputs {
        let thrust = self.thrust;
        let roll = self.roll_torque;
        let pitch = self.pitch_torque;
        let yaw = self.yaw_torque;

        // Plus (+) configuration
        // Front: thrust + pitch + yaw
        // Rear: thrust - pitch + yaw
        // Left: thrust - roll - yaw
        // Right: thrust + roll - yaw
        let front = thrust + pitch * 0.5 + yaw * 0.5;
        let rear = thrust - pitch * 0.5 + yaw * 0.5;
        let left = thrust - roll * 0.5 - yaw * 0.5;
        let right = thrust + roll * 0.5 - yaw * 0.5;

        MotorOutputs::quad(front, rear, left, right)
    }
}
