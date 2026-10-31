//! Position controller for multirotor position hold and navigation
//!
//! Controls position (X, Y, Z) and generates velocity setpoints.

use super::{ControllerGains, PidController3, VelocitySetpoint};
use glam::Vec3;

/// Position controller configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PositionConfig {
    /// X (forward/back) PID gains
    pub x_gains: ControllerGains,
    /// Y (left/right) PID gains
    pub y_gains: ControllerGains,
    /// Z (up/down) PID gains
    pub z_gains: ControllerGains,
    /// Maximum velocity output (m/s)
    pub max_velocity: f32,
    /// Maximum horizontal velocity (m/s)
    pub max_horizontal_vel: f32,
    /// Maximum vertical velocity (m/s)  
    pub max_vertical_vel: f32,
    /// Hover thrust (0-1, typically 0.5)
    pub hover_thrust: f32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            x_gains: ControllerGains::new(1.0, 0.1, 0.5),
            y_gains: ControllerGains::new(1.0, 0.1, 0.5),
            z_gains: ControllerGains::new(2.0, 0.2, 1.0),
            max_velocity: 10.0,
            max_horizontal_vel: 10.0,
            max_vertical_vel: 3.0,
            hover_thrust: 0.5,
        }
    }
}

impl PositionConfig {
    /// Create default outdoor config (more conservative)
    pub fn outdoor_default() -> Self {
        Self {
            x_gains: ControllerGains::new(0.5, 0.05, 0.3),
            y_gains: ControllerGains::new(0.5, 0.05, 0.3),
            z_gains: ControllerGains::new(1.0, 0.1, 0.5),
            max_velocity: 5.0,
            max_horizontal_vel: 5.0,
            max_vertical_vel: 2.0,
            hover_thrust: 0.5,
        }
    }

    /// Create indoor config (more aggressive)
    pub fn indoor_default() -> Self {
        Self {
            x_gains: ControllerGains::new(2.0, 0.2, 1.0),
            y_gains: ControllerGains::new(2.0, 0.2, 1.0),
            z_gains: ControllerGains::new(3.0, 0.3, 1.5),
            max_velocity: 3.0,
            max_horizontal_vel: 3.0,
            max_vertical_vel: 1.5,
            hover_thrust: 0.5,
        }
    }

    /// Set XY gains
    pub fn with_xy_gains(mut self, kp: f32, ki: f32, kd: f32) -> Self {
        self.x_gains = ControllerGains::new(kp, ki, kd);
        self.y_gains = ControllerGains::new(kp, ki, kd);
        self
    }

    /// Set Z gains
    pub fn with_z_gains(mut self, kp: f32, ki: f32, kd: f32) -> Self {
        self.z_gains = ControllerGains::new(kp, ki, kd);
        self
    }

    /// Set velocity limits
    pub fn with_velocity_limits(mut self, horizontal: f32, vertical: f32) -> Self {
        self.max_horizontal_vel = horizontal;
        self.max_vertical_vel = vertical;
        self.max_velocity = horizontal.max(vertical);
        self
    }

    /// Set hover thrust
    pub fn with_hover_thrust(mut self, thrust: f32) -> Self {
        self.hover_thrust = thrust.clamp(0.0, 1.0);
        self
    }
}

/// Position setpoint
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct PositionSetpoint {
    /// Desired X position (m, forward positive)
    pub x: f32,
    /// Desired Y position (m, right positive)  
    pub y: f32,
    /// Desired Z position (m, down positive - NED frame)
    pub z: f32,
    /// Desired yaw angle (rad)
    pub yaw: f32,
    /// Desired yaw rate (rad/s, overrides yaw if non-zero)
    pub yaw_rate: f32,
}

impl PositionSetpoint {
    /// Create new position setpoint
    pub fn new(x: f32, y: f32, z: f32, yaw: f32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            yaw_rate: 0.0,
        }
    }

    /// Create with yaw rate instead of fixed yaw
    pub fn with_yaw_rate(mut self, yaw_rate: f32) -> Self {
        self.yaw_rate = yaw_rate;
        self
    }

    /// Get position as Vec3
    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Zero setpoint (origin, yaw=0)
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            yaw_rate: 0.0,
        }
    }
}

/// Position controller
#[derive(Debug, Clone, PartialEq)]
pub struct PositionController {
    /// Configuration
    config: PositionConfig,
    /// PID controllers
    pid: PidController3,
}

impl Default for PositionController {
    fn default() -> Self {
        Self::new(PositionConfig::default())
    }
}

impl PositionController {
    /// Create new position controller
    pub fn new(config: PositionConfig) -> Self {
        let pid = PidController3::with_axis_gains(config.x_gains, config.y_gains, config.z_gains);

        Self { config, pid }
    }

    /// Reset controller
    pub fn reset(&mut self) {
        self.pid.reset();
    }

    /// Update controller and return velocity setpoint
    pub fn update(
        &mut self,
        setpoint: &PositionSetpoint,
        current_position: Vec3,
        dt: f32,
    ) -> VelocitySetpoint {
        let velocity_cmd = self.pid.update(setpoint.position(), current_position, dt);

        // Limit horizontal velocity
        let mut horizontal = Vec3::new(velocity_cmd.x, velocity_cmd.y, 0.0);
        if horizontal.length() > self.config.max_horizontal_vel {
            horizontal = horizontal.normalize() * self.config.max_horizontal_vel;
        }

        // Limit vertical velocity
        let vz = velocity_cmd
            .z
            .clamp(-self.config.max_vertical_vel, self.config.max_vertical_vel);

        VelocitySetpoint {
            vx: horizontal.x,
            vy: horizontal.y,
            vz,
            yaw_rate: setpoint.yaw_rate,
        }
    }

    /// Update and return velocity setpoint with yaw tracking
    pub fn update_with_yaw(
        &mut self,
        setpoint: &PositionSetpoint,
        current_position: Vec3,
        current_yaw: f32,
        dt: f32,
    ) -> (VelocitySetpoint, f32) {
        let mut vel_sp = self.update(setpoint, current_position, dt);

        // Compute yaw error and desired yaw rate
        let yaw_error = normalize_angle(setpoint.yaw - current_yaw);
        let yaw_rate_cmd = yaw_error * 2.0; // Simple P controller for yaw

        // If no explicit yaw rate set, use the tracking controller
        if setpoint.yaw_rate == 0.0 {
            vel_sp.yaw_rate = yaw_rate_cmd.clamp(-1.0, 1.0);
        }

        (vel_sp, yaw_error)
    }

    /// Check if reached setpoint
    pub fn is_reached(
        &self,
        setpoint: &PositionSetpoint,
        current_position: Vec3,
        tolerance: f32,
    ) -> bool {
        let error = (setpoint.position() - current_position).length();
        error < tolerance
    }

    /// Get current position error
    pub fn position_error(&self, setpoint: &PositionSetpoint, current_position: Vec3) -> Vec3 {
        setpoint.position() - current_position
    }

    /// Get configuration
    pub fn config(&self) -> &PositionConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: PositionConfig) {
        self.pid.x.set_gains(config.x_gains);
        self.pid.y.set_gains(config.y_gains);
        self.pid.z.set_gains(config.z_gains);
        self.config = config;
    }
}

/// Normalize angle to [-pi, pi]
fn normalize_angle(angle: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let mut angle = angle;
    while angle > pi {
        angle -= 2.0 * pi;
    }
    while angle < -pi {
        angle += 2.0 * pi;
    }
    angle
}

/// Path following controller for waypoint navigation
#[derive(Debug, Clone, PartialEq)]
pub struct PathController {
    /// Position controller
    position: PositionController,
    /// Waypoints
    waypoints: Vec<Vec3>,
    /// Current waypoint index
    current_index: usize,
    /// Reached tolerance
    tolerance: f32,
    /// Is path complete
    complete: bool,
}

impl PathController {
    /// Create new path controller
    pub fn new(position_config: PositionConfig) -> Self {
        Self {
            position: PositionController::new(position_config),
            waypoints: Vec::new(),
            current_index: 0,
            tolerance: 0.5,
            complete: false,
        }
    }

    /// Set path waypoints
    pub fn set_path(&mut self, waypoints: Vec<Vec3>) {
        self.waypoints = waypoints;
        self.current_index = 0;
        self.complete = false;
    }

    /// Set tolerance
    pub fn set_tolerance(&mut self, tolerance: f32) {
        self.tolerance = tolerance;
    }

    /// Update path following
    pub fn update(
        &mut self,
        current_position: Vec3,
        current_yaw: f32,
        dt: f32,
    ) -> Option<(VelocitySetpoint, f32)> {
        if self.complete || self.waypoints.is_empty() {
            return None;
        }

        let target = self.waypoints[self.current_index];
        let setpoint = PositionSetpoint::new(target.x, target.y, target.z, 0.0);

        // Check if reached current waypoint
        if self
            .position
            .is_reached(&setpoint, current_position, self.tolerance)
        {
            self.current_index += 1;
            if self.current_index >= self.waypoints.len() {
                self.complete = true;
                return None;
            }
        }

        // Update to next target
        let next_target = self.waypoints[self.current_index];
        let next_setpoint = PositionSetpoint::new(next_target.x, next_target.y, next_target.z, 0.0);

        Some(
            self.position
                .update_with_yaw(&next_setpoint, current_position, current_yaw, dt),
        )
    }

    /// Check if path is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get current waypoint index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get total waypoints
    pub fn total_waypoints(&self) -> usize {
        self.waypoints.len()
    }

    /// Reset path
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.complete = false;
        self.position.reset();
    }
}
