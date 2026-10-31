//! SimpleFlight-style controller
//!
//! Simplified flight controller inspired by AirSim's SimpleFlight firmware.
//! A more basic cascade controller with fewer tuning parameters.

use super::{ControllerInput, ControllerOutput, FlightCommand, FlightController, FlightMode};
use crate::control::{AttitudeSetpoint, RateConfig, RateController, RateSetpoint};
use glam::Vec3;

/// SimpleFlight configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SimpleFlightConfig {
    /// Rate controller config
    pub rate: RateConfig,
    /// Angle P gain (for stabilized mode)
    pub angle_p_gain: f32,
    /// Velocity P gain
    pub velocity_p_gain: f32,
    /// Position P gain  
    pub position_p_gain: f32,
    /// Max angle (rad)
    pub max_angle: f32,
    /// Max rate (rad/s)
    pub max_rate: f32,
    /// Hover thrust (0-1)
    pub hover_thrust: f32,
    /// Max vertical velocity (m/s)
    pub max_vertical_velocity: f32,
    /// Max horizontal velocity (m/s)
    pub max_horizontal_velocity: f32,
}

impl Default for SimpleFlightConfig {
    fn default() -> Self {
        Self {
            rate: RateConfig::multirotor_default(),
            angle_p_gain: 4.0,
            velocity_p_gain: 2.0,
            position_p_gain: 1.0,
            max_angle: 0.5, // ~28 degrees
            max_rate: 2.0,
            hover_thrust: 0.5,
            max_vertical_velocity: 3.0,
            max_horizontal_velocity: 10.0,
        }
    }
}

/// SimpleFlight controller
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleFlightController {
    /// Configuration
    config: SimpleFlightConfig,
    /// Current mode
    mode: FlightMode,
    /// Armed state
    armed: bool,
    /// Rate controller
    rate: RateController,
    /// Integral terms for position control
    position_integral: Vec3,
    /// Integral terms for velocity control  
    velocity_integral: Vec3,
    /// Altitude setpoint
    altitude_sp: f32,
    /// Position setpoint
    position_sp: Vec3,
}

impl Default for SimpleFlightController {
    fn default() -> Self {
        Self::new(SimpleFlightConfig::default())
    }
}

impl SimpleFlightController {
    /// Create new SimpleFlight controller
    pub fn new(config: SimpleFlightConfig) -> Self {
        let rate = RateController::new(config.rate.clone());

        Self {
            config,
            mode: FlightMode::default(),
            armed: false,
            rate,
            position_integral: Vec3::ZERO,
            velocity_integral: Vec3::ZERO,
            altitude_sp: 0.0,
            position_sp: Vec3::ZERO,
        }
    }

    /// Update position control (P + integral)
    fn position_control(&mut self, position: Vec3, dt: f32) -> Vec3 {
        let error = self.position_sp - position;

        // Update integral
        self.position_integral += error * dt;
        self.position_integral = self
            .position_integral
            .clamp(Vec3::splat(-1.0), Vec3::splat(1.0));

        // P + I
        let p_term = error * self.config.position_p_gain;
        let i_term = self.position_integral * 0.1; // Small integral

        let velocity_cmd = p_term + i_term;

        // Clamp to max velocity
        let vel_xy = Vec3::new(velocity_cmd.x, velocity_cmd.y, 0.0);
        let vel_xy = if vel_xy.length() > self.config.max_horizontal_velocity {
            vel_xy.normalize() * self.config.max_horizontal_velocity
        } else {
            vel_xy
        };

        let vel_z = velocity_cmd.z.clamp(
            -self.config.max_vertical_velocity,
            self.config.max_vertical_velocity,
        );

        Vec3::new(vel_xy.x, vel_xy.y, vel_z)
    }

    /// Update velocity control (P + integral)
    fn velocity_control(&mut self, velocity: Vec3, velocity_cmd: Vec3, dt: f32) -> (f32, f32, f32) {
        let velocity_body = velocity; // Assume already in body frame
        let error = velocity_cmd - velocity_body;

        // Update integral
        self.velocity_integral += error * dt;
        self.velocity_integral = self
            .velocity_integral
            .clamp(Vec3::splat(-1.0), Vec3::splat(1.0));

        // P + I
        let p_term = error * self.config.velocity_p_gain;
        let i_term = self.velocity_integral * 0.05;

        let acceleration_cmd = p_term + i_term;

        // Convert acceleration to tilt angle
        let tilt_x =
            (-acceleration_cmd.x * 0.1).clamp(-self.config.max_angle, self.config.max_angle);
        let tilt_y =
            (acceleration_cmd.y * 0.1).clamp(-self.config.max_angle, self.config.max_angle);

        // Vertical acceleration to thrust
        let thrust_cmd = (acceleration_cmd.z * 0.05).clamp(-0.2, 0.2);
        let thrust = self.config.hover_thrust + thrust_cmd;

        (tilt_x, tilt_y, thrust.clamp(0.0, 1.0))
    }

    /// Update attitude control (P only)
    fn attitude_control(
        &self,
        roll: f32,
        pitch: f32,
        _yaw: f32,
        roll_sp: f32,
        pitch_sp: f32,
        yaw_rate_sp: f32,
    ) -> RateSetpoint {
        let roll_error = roll_sp - roll;
        let pitch_error = pitch_sp - pitch;

        // Simple P control to rate
        let roll_rate = (roll_error * self.config.angle_p_gain)
            .clamp(-self.config.max_rate, self.config.max_rate);
        let pitch_rate = (pitch_error * self.config.angle_p_gain)
            .clamp(-self.config.max_rate, self.config.max_rate);
        let yaw_rate = yaw_rate_sp.clamp(-self.config.max_rate, self.config.max_rate);

        // Thrust determined by velocity controller
        RateSetpoint::new(roll_rate, pitch_rate, yaw_rate, 0.5)
    }

    /// Update controller
    pub fn update(
        &mut self,
        input: &ControllerInput,
        command: &FlightCommand,
        dt: f32,
    ) -> ControllerOutput {
        if !self.armed {
            return ControllerOutput::quad(0.0, 0.0, 0.0, 0.0);
        }

        if self.mode == FlightMode::EmergencyStop {
            return ControllerOutput::quad(0.0, 0.0, 0.0, 0.0);
        }

        let rate_sp = match self.mode {
            FlightMode::Manual => {
                // Direct rate control
                if let Some(rates) = command.rates {
                    RateSetpoint::new(
                        rates.x.clamp(-self.config.max_rate, self.config.max_rate),
                        rates.y.clamp(-self.config.max_rate, self.config.max_rate),
                        rates.z.clamp(-self.config.max_rate, self.config.max_rate),
                        command.thrust.clamp(0.0, 1.0),
                    )
                } else {
                    RateSetpoint::new(0.0, 0.0, 0.0, command.thrust.clamp(0.0, 1.0))
                }
            }
            FlightMode::Stabilized => {
                // Angle control
                let attitude_sp = if let Some(att) = command.attitude {
                    let (roll, pitch, yaw) = att.to_euler(glam::EulerRot::XYZ);
                    AttitudeSetpoint::new(roll, pitch, yaw, command.thrust)
                } else {
                    let yaw = if command.yaw_rate != 0.0 {
                        input.yaw() + command.yaw_rate * dt
                    } else {
                        input.yaw()
                    };
                    AttitudeSetpoint::new(0.0, 0.0, yaw, command.thrust)
                };

                RateSetpoint::new(
                    ((attitude_sp.roll - input.roll()) * self.config.angle_p_gain)
                        .clamp(-self.config.max_rate, self.config.max_rate),
                    ((attitude_sp.pitch - input.pitch()) * self.config.angle_p_gain)
                        .clamp(-self.config.max_rate, self.config.max_rate),
                    0.0,
                    attitude_sp.thrust,
                )
            }
            FlightMode::AltitudeHold => {
                // Maintain altitude
                let z_error = self.altitude_sp - input.position.z;
                let vz_cmd = (z_error * 0.5).clamp(
                    -self.config.max_vertical_velocity,
                    self.config.max_vertical_velocity,
                );

                // Manual XY
                let vx_cmd = command.velocity.map(|v| v.x).unwrap_or(0.0);
                let vy_cmd = command.velocity.map(|v| v.y).unwrap_or(0.0);
                let velocity_cmd = Vec3::new(vx_cmd, vy_cmd, vz_cmd);

                let (pitch_sp, roll_sp, _thrust) =
                    self.velocity_control(input.velocity, velocity_cmd, dt);
                let yaw_rate = command.yaw_rate;

                self.attitude_control(
                    input.roll(),
                    input.pitch(),
                    input.yaw(),
                    roll_sp,
                    pitch_sp,
                    yaw_rate,
                )
            }
            FlightMode::PositionHold | FlightMode::Mission => {
                // Full position control
                if let Some(pos) = command.position {
                    self.position_sp = pos;
                }

                let velocity_cmd = self.position_control(input.position, dt);
                let (pitch_sp, roll_sp, thrust) =
                    self.velocity_control(input.velocity, velocity_cmd, dt);
                let yaw_rate = command.yaw_rate;

                let mut rate_sp = self.attitude_control(
                    input.roll(),
                    input.pitch(),
                    input.yaw(),
                    roll_sp,
                    pitch_sp,
                    yaw_rate,
                );
                rate_sp.thrust = thrust;
                rate_sp
            }
            FlightMode::Velocity => {
                // Direct velocity control
                let velocity_cmd = if let Some(vel) = command.velocity {
                    vel
                } else {
                    Vec3::ZERO
                };

                let (pitch_sp, roll_sp, thrust) =
                    self.velocity_control(input.velocity, velocity_cmd, dt);
                let yaw_rate = command.yaw_rate;

                let mut rate_sp = self.attitude_control(
                    input.roll(),
                    input.pitch(),
                    input.yaw(),
                    roll_sp,
                    pitch_sp,
                    yaw_rate,
                );
                rate_sp.thrust = thrust;
                rate_sp
            }
            _ => {
                // Default to hover
                RateSetpoint::new(0.0, 0.0, 0.0, 0.0)
            }
        };

        // Rate -> motors
        let motors = self.rate.update_quad(&rate_sp, input.angular_velocity, dt);

        let mut output = ControllerOutput {
            motors: motors.values,
            desired_attitude: None,
            desired_rates: Some(rate_sp.rates()),
        };
        output.clamp();
        output
    }

    /// Reset controller
    pub fn reset(&mut self) {
        self.position_integral = Vec3::ZERO;
        self.velocity_integral = Vec3::ZERO;
        self.rate.reset();
    }

    /// Set altitude setpoint
    pub fn set_altitude_setpoint(&mut self, altitude: f32) {
        self.altitude_sp = altitude;
    }

    /// Get altitude setpoint
    pub fn altitude_setpoint(&self) -> f32 {
        self.altitude_sp
    }

    /// Set position setpoint
    pub fn set_position_setpoint(&mut self, position: Vec3) {
        self.position_sp = position;
    }

    /// Get position setpoint
    pub fn position_setpoint(&self) -> Vec3 {
        self.position_sp
    }
}

impl FlightController for SimpleFlightController {
    fn update(
        &mut self,
        input: &ControllerInput,
        command: &FlightCommand,
        dt: f32,
    ) -> ControllerOutput {
        self.update(input, command, dt)
    }

    fn reset(&mut self) {
        self.reset();
    }

    fn mode(&self) -> FlightMode {
        self.mode
    }

    fn set_mode(&mut self, mode: FlightMode) {
        if self.mode != mode {
            self.reset();
        }
        self.mode = mode;
    }

    fn is_armed(&self) -> bool {
        self.armed
    }

    fn arm(&mut self, armed: bool) {
        if armed && !self.armed {
            self.reset();
        }
        self.armed = armed;
    }
}
