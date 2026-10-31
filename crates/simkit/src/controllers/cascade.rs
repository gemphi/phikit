//! Cascade controller for multirotor flight control
//!
//! Implements a cascade control structure:
//! Position -> Velocity -> Attitude -> Rate -> Motors

use super::{ControllerInput, ControllerOutput, FlightCommand, FlightController};
use crate::control::{
    AttitudeSetpoint, ControllerGains, PidController, PositionConfig, PositionController,
    PositionSetpoint, RateConfig, RateController, RateSetpoint, VelocitySetpoint,
};

/// Flight mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum FlightMode {
    /// Manual/acro mode (direct rate control)
    Manual,
    /// Stabilized mode (angle control)
    Stabilized,
    /// Altitude hold mode
    AltitudeHold,
    /// Position hold mode
    #[default]
    PositionHold,
    /// Velocity control mode
    Velocity,
    /// Mission/waypoint mode
    Mission,
    /// Return to launch
    ReturnToLaunch,
    /// Land mode
    Land,
    /// Takeoff mode
    Takeoff,
    /// Emergency stop
    EmergencyStop,
}

/// Cascade controller configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CascadeConfig {
    /// Position controller config
    pub position: PositionConfig,
    /// Velocity controller gains
    pub velocity_xy_gains: ControllerGains,
    /// Velocity controller Z gains
    pub velocity_z_gains: ControllerGains,
    /// Attitude angle gains (for stabilized mode)
    pub attitude_gains: ControllerGains,
    /// Rate controller config
    pub rate: RateConfig,
    /// Maximum tilt angle (rad)
    pub max_tilt_angle: f32,
    /// Maximum ascent rate (m/s)
    pub max_ascent_rate: f32,
    /// Maximum descent rate (m/s)
    pub max_descent_rate: f32,
    /// Hover thrust (0-1)
    pub hover_thrust: f32,
    /// Motor count
    pub motor_count: usize,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            position: PositionConfig::default(),
            velocity_xy_gains: ControllerGains::new(2.0, 0.2, 0.0),
            velocity_z_gains: ControllerGains::new(2.0, 0.2, 0.0),
            attitude_gains: ControllerGains::new(6.0, 0.0, 0.0),
            rate: RateConfig::multirotor_default(),
            max_tilt_angle: 0.5, // ~28 degrees
            max_ascent_rate: 3.0,
            max_descent_rate: 2.0,
            hover_thrust: 0.5,
            motor_count: 4,
        }
    }
}

impl CascadeConfig {
    /// Create racing quad config
    pub fn racing() -> Self {
        Self {
            position: PositionConfig::default(),
            velocity_xy_gains: ControllerGains::new(4.0, 0.3, 0.1),
            velocity_z_gains: ControllerGains::new(4.0, 0.3, 0.1),
            attitude_gains: ControllerGains::new(8.0, 0.1, 0.1),
            rate: RateConfig::racing_default(),
            max_tilt_angle: 1.0, // ~57 degrees
            max_ascent_rate: 5.0,
            max_descent_rate: 3.0,
            hover_thrust: 0.45,
            motor_count: 4,
        }
    }
}

/// Velocity controller (generates attitude from velocity error)
#[derive(Debug, Clone, PartialEq)]
struct VelocityController {
    /// XY gains
    xy_gains: ControllerGains,
    /// Z gains
    z_gains: ControllerGains,
    /// Max tilt
    max_tilt: f32,
    /// Hover thrust
    hover_thrust: f32,
    /// XY PID
    xy_pid_x: PidController,
    xy_pid_y: PidController,
    /// Z PID
    z_pid: PidController,
}

impl VelocityController {
    fn new(
        xy_gains: ControllerGains,
        z_gains: ControllerGains,
        max_tilt: f32,
        hover_thrust: f32,
    ) -> Self {
        Self {
            xy_gains,
            z_gains,
            max_tilt,
            hover_thrust,
            xy_pid_x: PidController::from_gains(xy_gains.kp, xy_gains.ki, xy_gains.kd),
            xy_pid_y: PidController::from_gains(xy_gains.kp, xy_gains.ki, xy_gains.kd),
            z_pid: PidController::from_gains(z_gains.kp, z_gains.ki, z_gains.kd),
        }
    }

    fn reset(&mut self) {
        self.xy_pid_x.reset();
        self.xy_pid_y.reset();
        self.z_pid.reset();
    }

    fn update(
        &mut self,
        setpoint: &VelocitySetpoint,
        current: &ControllerInput,
        dt: f32,
    ) -> (AttitudeSetpoint, f32) {
        // Get body frame velocity
        let velocity_body = current.velocity_body();

        // XY velocity to tilt (pitch/roll) mapping
        let vx_cmd = self.xy_pid_x.update(setpoint.vx, velocity_body.x, dt);
        let vy_cmd = self.xy_pid_y.update(setpoint.vy, velocity_body.y, dt);

        // Clamp tilt angles
        let pitch = (-vx_cmd).clamp(-self.max_tilt, self.max_tilt);
        let roll = vy_cmd.clamp(-self.max_tilt, self.max_tilt);

        // Z velocity to thrust
        let thrust_cmd = self.z_pid.update(setpoint.vz, velocity_body.z, dt);
        let thrust = (self.hover_thrust + thrust_cmd * 0.1).clamp(0.0, 1.0);

        // Yaw rate is passed through
        let yaw = current.yaw() + setpoint.yaw_rate * dt;

        let attitude_sp = AttitudeSetpoint::new(roll, pitch, yaw, thrust);
        (attitude_sp, yaw)
    }
}

/// Attitude controller (generates rate from attitude error)
#[derive(Debug, Clone, PartialEq)]
struct AttitudeController {
    /// Angle gains (for stabilized mode)
    angle_gains: ControllerGains,
    /// Roll PID
    roll_pid: PidController,
    /// Pitch PID
    pitch_pid: PidController,
    /// Yaw PID
    yaw_pid: PidController,
}

impl AttitudeController {
    fn new(gains: ControllerGains) -> Self {
        Self {
            angle_gains: gains,
            roll_pid: PidController::from_gains(gains.kp, gains.ki, gains.kd),
            pitch_pid: PidController::from_gains(gains.kp, gains.ki, gains.kd),
            yaw_pid: PidController::from_gains(gains.kp, gains.ki, gains.kd),
        }
    }

    fn reset(&mut self) {
        self.roll_pid.reset();
        self.pitch_pid.reset();
        self.yaw_pid.reset();
    }

    fn update(
        &mut self,
        setpoint: &AttitudeSetpoint,
        current: &ControllerInput,
        dt: f32,
    ) -> RateSetpoint {
        // Current angles
        let current_roll = current.roll();
        let current_pitch = current.pitch();
        let current_yaw = current.yaw();

        // Compute rate commands from angle error
        let roll_rate = self.roll_pid.update(setpoint.roll, current_roll, dt);
        let pitch_rate = self.pitch_pid.update(setpoint.pitch, current_pitch, dt);

        // Yaw: handle wrap-around
        let yaw_error = normalize_angle(setpoint.yaw - current_yaw);
        let yaw_rate = self.yaw_pid.update(yaw_error, 0.0, dt);

        RateSetpoint::new(roll_rate, pitch_rate, yaw_rate, setpoint.thrust)
    }
}

/// Full cascade controller
#[derive(Debug, Clone, PartialEq)]
pub struct CascadeController {
    /// Configuration
    config: CascadeConfig,
    /// Current mode
    mode: FlightMode,
    /// Armed state
    armed: bool,
    /// Position controller
    position: PositionController,
    /// Velocity controller
    velocity: VelocityController,
    /// Attitude controller
    attitude: AttitudeController,
    /// Rate controller
    rate: RateController,
    /// Current altitude setpoint (for altitude hold)
    altitude_setpoint: f32,
}

impl Default for CascadeController {
    fn default() -> Self {
        Self::new(CascadeConfig::default())
    }
}

impl CascadeController {
    /// Create new cascade controller
    pub fn new(config: CascadeConfig) -> Self {
        let position = PositionController::new(config.position.clone());
        let velocity = VelocityController::new(
            config.velocity_xy_gains,
            config.velocity_z_gains,
            config.max_tilt_angle,
            config.hover_thrust,
        );
        let attitude = AttitudeController::new(config.attitude_gains);
        let rate = RateController::new(config.rate.clone());

        Self {
            config,
            mode: FlightMode::default(),
            armed: false,
            position,
            velocity,
            attitude,
            rate,
            altitude_setpoint: 0.0,
        }
    }

    /// Update controller
    pub fn update(
        &mut self,
        input: &ControllerInput,
        command: &FlightCommand,
        dt: f32,
    ) -> ControllerOutput {
        if !self.armed {
            return ControllerOutput::new(self.config.motor_count);
        }

        if self.mode == FlightMode::EmergencyStop {
            let mut output = ControllerOutput::new(self.config.motor_count);
            output.stop();
            return output;
        }

        // Rate setpoint computed based on mode
        let rate_sp = match self.mode {
            FlightMode::Manual => {
                // Direct rate control with thrust
                if let Some(rates) = command.rates {
                    RateSetpoint::new(rates.x, rates.y, rates.z, command.thrust)
                } else {
                    RateSetpoint::hover()
                }
            }
            FlightMode::Stabilized => {
                // Angle control
                let attitude_sp = if let Some(att) = command.attitude {
                    let (roll, pitch, yaw) = att.to_euler(glam::EulerRot::XYZ);
                    AttitudeSetpoint::new(roll, pitch, yaw, command.thrust)
                } else {
                    // Default: level
                    AttitudeSetpoint::new(0.0, 0.0, input.yaw(), command.thrust)
                };
                self.attitude.update(&attitude_sp, input, dt)
            }
            FlightMode::AltitudeHold => {
                // Maintain altitude, manual XY control
                let z_error = self.altitude_setpoint - input.position.z;
                let vz_cmd = (z_error * 0.5)
                    .clamp(-self.config.max_descent_rate, self.config.max_ascent_rate);

                let velocity_sp = VelocitySetpoint::new(
                    command.velocity.map(|v| v.x).unwrap_or(0.0),
                    command.velocity.map(|v| v.y).unwrap_or(0.0),
                    vz_cmd,
                    command.yaw_rate,
                );
                let (attitude_sp, _) = self.velocity.update(&velocity_sp, input, dt);
                self.attitude.update(&attitude_sp, input, dt)
            }
            FlightMode::PositionHold
            | FlightMode::Mission
            | FlightMode::ReturnToLaunch
            | FlightMode::Takeoff
            | FlightMode::Land => {
                // Full position control
                let pos_sp = if let Some(pos) = command.position {
                    PositionSetpoint::new(pos.x, pos.y, pos.z, input.yaw())
                } else {
                    PositionSetpoint::new(
                        input.position.x,
                        input.position.y,
                        self.altitude_setpoint,
                        input.yaw(),
                    )
                };

                let velocity_sp = self.position.update(&pos_sp, input.position, dt);
                let (attitude_sp, _) = self.velocity.update(&velocity_sp, input, dt);
                self.attitude.update(&attitude_sp, input, dt)
            }
            FlightMode::Velocity => {
                // Direct velocity control
                let velocity_sp = if let Some(vel) = command.velocity {
                    VelocitySetpoint::new(vel.x, vel.y, vel.z, command.yaw_rate)
                } else {
                    VelocitySetpoint::hover()
                };
                let (attitude_sp, _) = self.velocity.update(&velocity_sp, input, dt);
                self.attitude.update(&attitude_sp, input, dt)
            }
            FlightMode::EmergencyStop => {
                return ControllerOutput::new(self.config.motor_count);
            }
        };

        // Rate controller -> motors
        let mut output = if self.config.motor_count == 4 {
            let motor_outputs = self.rate.update_quad(&rate_sp, input.angular_velocity, dt);
            ControllerOutput {
                motors: motor_outputs.values,
                desired_attitude: None,
                desired_rates: Some(rate_sp.rates()),
            }
        } else {
            let _torque_cmd = self.rate.update(&rate_sp, input.angular_velocity, dt);
            ControllerOutput::new(self.config.motor_count)
        };

        output.clamp();
        output
    }

    /// Reset all controllers
    pub fn reset(&mut self) {
        self.position.reset();
        self.velocity.reset();
        self.attitude.reset();
        self.rate.reset();
    }

    /// Set altitude setpoint (for altitude hold mode)
    pub fn set_altitude_setpoint(&mut self, altitude: f32) {
        self.altitude_setpoint = altitude;
    }

    /// Get current altitude setpoint
    pub fn altitude_setpoint(&self) -> f32 {
        self.altitude_setpoint
    }
}

impl FlightController for CascadeController {
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

        // Initialize altitude setpoint when entering altitude hold
        if mode == FlightMode::AltitudeHold {
            // Will be set externally based on current altitude
        }
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
