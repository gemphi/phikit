//! PID controller implementation
//!
//! Standard PID controller with anti-windup and output limiting.

use super::ControllerGains;

/// PID controller output
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct PidOutput {
    /// Proportional component
    pub p: f32,
    /// Integral component
    pub i: f32,
    /// Derivative component
    pub d: f32,
    /// Total output (p + i + d)
    pub total: f32,
}

impl PidOutput {
    /// Create new PID output
    pub fn new(p: f32, i: f32, d: f32) -> Self {
        let total = p + i + d;
        Self { p, i, d, total }
    }

    /// Zero output
    pub fn zero() -> Self {
        Self {
            p: 0.0,
            i: 0.0,
            d: 0.0,
            total: 0.0,
        }
    }
}

/// PID controller configuration
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PidConfig {
    /// Controller gains
    pub gains: ControllerGains,
    /// Output lower limit
    pub output_min: f32,
    /// Output upper limit
    pub output_max: f32,
    /// Derivative filter coefficient (0-1, higher = more filtering)
    pub derivative_filter: f32,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            gains: ControllerGains::default(),
            output_min: -1.0,
            output_max: 1.0,
            derivative_filter: 0.1,
        }
    }
}

impl PidConfig {
    /// Create new PID config with gains
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            gains: ControllerGains::new(kp, ki, kd),
            ..Default::default()
        }
    }

    /// Set output limits
    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.output_min = min;
        self.output_max = max;
        self
    }

    /// Set derivative filter coefficient
    pub fn with_filter(mut self, filter: f32) -> Self {
        self.derivative_filter = filter.clamp(0.0, 1.0);
        self
    }
}

/// Single-axis PID controller
#[derive(Debug, Clone, PartialEq)]
pub struct PidController {
    /// Configuration
    config: PidConfig,
    /// Current integral value
    integral: f32,
    /// Previous error (for derivative)
    prev_error: f32,
    /// Filtered derivative
    filtered_derivative: f32,
    /// Is first update
    is_first: bool,
    /// Last output (for reference)
    last_output: f32,
}

impl Default for PidController {
    fn default() -> Self {
        Self::new(PidConfig::default())
    }
}

impl PidController {
    /// Create new PID controller
    pub fn new(config: PidConfig) -> Self {
        Self {
            config,
            integral: 0.0,
            prev_error: 0.0,
            filtered_derivative: 0.0,
            is_first: true,
            last_output: 0.0,
        }
    }

    /// Create with simple gains
    pub fn from_gains(kp: f32, ki: f32, kd: f32) -> Self {
        Self::new(PidConfig::new(kp, ki, kd))
    }

    /// Reset controller state
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.filtered_derivative = 0.0;
        self.is_first = true;
        self.last_output = 0.0;
    }

    /// Update controller with new setpoint and measurement
    pub fn update(&mut self, setpoint: f32, measurement: f32, dt: f32) -> f32 {
        let error = setpoint - measurement;

        // Proportional term
        let p = self.config.gains.kp * error;

        // Integral term with anti-windup
        if dt > 0.0 {
            self.integral += error * dt;
        }

        // Anti-windup: clamp integral to limit
        let i_limit = self.config.gains.i_limit;
        self.integral = self.integral.clamp(-i_limit, i_limit);
        let i = self.config.gains.ki * self.integral;

        // Derivative term with filtering
        let mut d = 0.0;
        if dt > 0.0 && !self.is_first {
            let raw_derivative = (error - self.prev_error) / dt;
            // Low-pass filter on derivative
            let alpha = self.config.derivative_filter;
            self.filtered_derivative =
                alpha * raw_derivative + (1.0 - alpha) * self.filtered_derivative;
            d = self.config.gains.kd * self.filtered_derivative;
        }

        // Update state
        self.prev_error = error;
        self.is_first = false;

        // Compute output
        let output = p + i + d;
        let clamped_output = output.clamp(self.config.output_min, self.config.output_max);

        // Anti-windup: if output is saturated, don't accumulate more integral
        if output != clamped_output && error.signum() as f32 == output.signum() {
            // Decay integral when saturated to reduce windup
            self.integral *= 0.99;
        }

        self.last_output = clamped_output;
        clamped_output
    }

    /// Update and return full output breakdown
    pub fn update_detailed(&mut self, setpoint: f32, measurement: f32, dt: f32) -> PidOutput {
        let setpoint = setpoint;
        let measurement = measurement;
        let _ = self.update(setpoint, measurement, dt);

        let error = setpoint - measurement;
        let p = self.config.gains.kp * error;
        let i = self.config.gains.ki * self.integral;
        let d = if self.is_first {
            0.0
        } else {
            self.config.gains.kd * self.filtered_derivative
        };

        PidOutput::new(p, i, d)
    }

    /// Get current gains
    pub fn gains(&self) -> &ControllerGains {
        &self.config.gains
    }

    /// Set gains
    pub fn set_gains(&mut self, gains: ControllerGains) {
        self.config.gains = gains;
    }

    /// Set output limits
    pub fn set_limits(&mut self, min: f32, max: f32) {
        self.config.output_min = min;
        self.config.output_max = max;
    }

    /// Get last output
    pub fn last_output(&self) -> f32 {
        self.last_output
    }

    /// Get current integral value
    pub fn integral(&self) -> f32 {
        self.integral
    }

    /// Set integral value (for initialization)
    pub fn set_integral(&mut self, integral: f32) {
        let i_limit = self.config.gains.i_limit;
        self.integral = integral.clamp(-i_limit, i_limit);
    }
}

/// Three-axis PID controller (for roll, pitch, yaw)
#[derive(Debug, Clone, PartialEq)]
pub struct PidController3 {
    /// X-axis controller
    pub x: PidController,
    /// Y-axis controller
    pub y: PidController,
    /// Z-axis controller
    pub z: PidController,
}

impl PidController3 {
    /// Create new 3-axis controller with same config for all axes
    pub fn new(config: PidConfig) -> Self {
        Self {
            x: PidController::new(config),
            y: PidController::new(config),
            z: PidController::new(config),
        }
    }

    /// Create with different gains for each axis
    pub fn with_axis_gains(
        x_gains: ControllerGains,
        y_gains: ControllerGains,
        z_gains: ControllerGains,
    ) -> Self {
        let mut x_config = PidConfig::default();
        x_config.gains = x_gains;

        let mut y_config = PidConfig::default();
        y_config.gains = y_gains;

        let mut z_config = PidConfig::default();
        z_config.gains = z_gains;

        Self {
            x: PidController::new(x_config),
            y: PidController::new(y_config),
            z: PidController::new(z_config),
        }
    }

    /// Reset all controllers
    pub fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
        self.z.reset();
    }

    /// Update all controllers
    pub fn update(&mut self, setpoint: glam::Vec3, measurement: glam::Vec3, dt: f32) -> glam::Vec3 {
        let x = self.x.update(setpoint.x, measurement.x, dt);
        let y = self.y.update(setpoint.y, measurement.y, dt);
        let z = self.z.update(setpoint.z, measurement.z, dt);

        glam::Vec3::new(x, y, z)
    }

    /// Get all controllers
    pub fn controllers(&mut self) -> [&mut PidController; 3] {
        [&mut self.x, &mut self.y, &mut self.z]
    }
}
