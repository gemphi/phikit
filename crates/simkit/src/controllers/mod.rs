//! High-level flight controllers for rust-sim-kit
//!
//! Implements flight control systems similar to AirSim's firmware controllers:
//! - SimpleFlight-style cascade controller
//! - Position -> Velocity -> Attitude -> Rate cascade
//! - Path following controllers

use glam::{Quat, Vec3};

pub mod cascade;
pub mod simple_flight;

pub use cascade::{CascadeConfig, CascadeController, FlightMode};
pub use simple_flight::{SimpleFlightConfig, SimpleFlightController};

/// Controller input containing all available data
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ControllerInput {
    /// Current position (m)
    pub position: Vec3,
    /// Current velocity (m/s)
    pub velocity: Vec3,
    /// Current orientation as quaternion
    pub orientation: Quat,
    /// Current angular velocity (rad/s)
    pub angular_velocity: Vec3,
    /// Current acceleration (m/s^2)
    pub acceleration: Vec3,
}

impl ControllerInput {
    /// Create new controller input
    pub fn new(position: Vec3, velocity: Vec3, orientation: Quat, angular_velocity: Vec3) -> Self {
        Self {
            position,
            velocity,
            orientation,
            angular_velocity,
            acceleration: Vec3::ZERO,
        }
    }

    /// With acceleration
    pub fn with_acceleration(mut self, acceleration: Vec3) -> Self {
        self.acceleration = acceleration;
        self
    }

    /// Get roll angle (rad)
    pub fn roll(&self) -> f32 {
        let (roll, _, _) = self.orientation.to_euler(glam::EulerRot::XYZ);
        roll
    }

    /// Get pitch angle (rad)
    pub fn pitch(&self) -> f32 {
        let (_, pitch, _) = self.orientation.to_euler(glam::EulerRot::XYZ);
        pitch
    }

    /// Get yaw angle (rad)
    pub fn yaw(&self) -> f32 {
        let (_, _, yaw) = self.orientation.to_euler(glam::EulerRot::XYZ);
        yaw
    }

    /// Get body frame velocity
    pub fn velocity_body(&self) -> Vec3 {
        self.orientation.inverse() * self.velocity
    }
}

/// Flight controller output (motor commands)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ControllerOutput {
    /// Motor outputs (0.0 to 1.0 for each motor)
    pub motors: Vec<f32>,
    /// Desired attitude (for logging/debug)
    pub desired_attitude: Option<Quat>,
    /// Desired rates (for logging/debug)
    pub desired_rates: Option<Vec3>,
}

impl ControllerOutput {
    /// Create new output for given motor count
    pub fn new(motor_count: usize) -> Self {
        Self {
            motors: vec![0.0; motor_count],
            desired_attitude: None,
            desired_rates: None,
        }
    }

    /// Create for quadcopter
    pub fn quad(fr: f32, rl: f32, fl: f32, rr: f32) -> Self {
        Self {
            motors: vec![
                fr.clamp(0.0, 1.0),
                rl.clamp(0.0, 1.0),
                fl.clamp(0.0, 1.0),
                rr.clamp(0.0, 1.0),
            ],
            desired_attitude: None,
            desired_rates: None,
        }
    }

    /// Get number of motors
    pub fn motor_count(&self) -> usize {
        self.motors.len()
    }

    /// Clamp all motor values to [0, 1]
    pub fn clamp(&mut self) {
        for m in &mut self.motors {
            *m = m.clamp(0.0, 1.0);
        }
    }

    /// Set all motors to idle (minimum throttle)
    pub fn idle(&mut self, idle_value: f32) {
        for m in &mut self.motors {
            *m = idle_value.clamp(0.0, 1.0);
        }
    }

    /// Set all motors to stop
    pub fn stop(&mut self) {
        self.idle(0.0);
    }
}

/// Flight command from user/API
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FlightCommand {
    /// Desired position (if in position mode)
    pub position: Option<Vec3>,
    /// Desired velocity (if in velocity mode)
    pub velocity: Option<Vec3>,
    /// Desired attitude (if in attitude mode)
    pub attitude: Option<Quat>,
    /// Desired rates (if in rate/acro mode)
    pub rates: Option<Vec3>,
    /// Thrust (0.0 to 1.0, for manual modes)
    pub thrust: f32,
    /// Yaw rate override (rad/s)
    pub yaw_rate: f32,
}

impl FlightCommand {
    /// Hover command (maintain position)
    pub fn hover() -> Self {
        Self {
            position: Some(Vec3::ZERO),
            velocity: None,
            attitude: None,
            rates: None,
            thrust: 0.0,
            yaw_rate: 0.0,
        }
    }

    /// Position command
    pub fn position(x: f32, y: f32, z: f32, yaw: f32) -> Self {
        Self {
            position: Some(Vec3::new(x, y, z)),
            velocity: None,
            attitude: Some(Quat::from_rotation_z(yaw)),
            rates: None,
            thrust: 0.0,
            yaw_rate: 0.0,
        }
    }

    /// Velocity command
    pub fn velocity(vx: f32, vy: f32, vz: f32, yaw_rate: f32) -> Self {
        Self {
            position: None,
            velocity: Some(Vec3::new(vx, vy, vz)),
            attitude: None,
            rates: None,
            thrust: 0.0,
            yaw_rate,
        }
    }

    /// Attitude command
    pub fn attitude(roll: f32, pitch: f32, yaw: f32, thrust: f32) -> Self {
        Self {
            position: None,
            velocity: None,
            attitude: Some(Quat::from_euler(glam::EulerRot::XYZ, roll, pitch, yaw)),
            rates: None,
            thrust: thrust.clamp(0.0, 1.0),
            yaw_rate: 0.0,
        }
    }

    /// Rate/acro command
    pub fn rate(roll_rate: f32, pitch_rate: f32, yaw_rate: f32, thrust: f32) -> Self {
        Self {
            position: None,
            velocity: None,
            attitude: None,
            rates: Some(Vec3::new(roll_rate, pitch_rate, yaw_rate)),
            thrust: thrust.clamp(0.0, 1.0),
            yaw_rate: 0.0,
        }
    }
}

/// Flight controller trait
pub trait FlightController {
    /// Update controller with new input and command
    fn update(
        &mut self,
        input: &ControllerInput,
        command: &FlightCommand,
        dt: f32,
    ) -> ControllerOutput;

    /// Reset controller state
    fn reset(&mut self);

    /// Get current flight mode
    fn mode(&self) -> FlightMode;

    /// Set flight mode
    fn set_mode(&mut self, mode: FlightMode);

    /// Check if controller is armed
    fn is_armed(&self) -> bool;

    /// Arm/disarm the controller
    fn arm(&mut self, armed: bool);
}

/// Trajectory point for path following
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TrajectoryPoint {
    /// Position
    pub position: Vec3,
    /// Velocity
    pub velocity: Vec3,
    /// Acceleration
    pub acceleration: Vec3,
    /// Yaw angle
    pub yaw: f32,
    /// Time (s)
    pub time: f32,
}

impl TrajectoryPoint {
    /// Create new trajectory point
    pub fn new(position: Vec3, velocity: Vec3, acceleration: Vec3, yaw: f32, time: f32) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            yaw,
            time,
        }
    }

    /// Position only (for simple waypoints)
    pub fn position_only(position: Vec3, yaw: f32) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            yaw,
            time: 0.0,
        }
    }
}

/// Trajectory for path following
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Trajectory {
    /// Waypoints
    pub points: Vec<TrajectoryPoint>,
}

impl Trajectory {
    /// Create new trajectory
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add point
    pub fn add_point(&mut self, point: TrajectoryPoint) {
        self.points.push(point);
    }

    /// Get point at index
    pub fn get(&self, index: usize) -> Option<&TrajectoryPoint> {
        self.points.get(index)
    }

    /// Interpolate at given time
    pub fn interpolate(&self, time: f32) -> Option<TrajectoryPoint> {
        if self.points.is_empty() {
            return None;
        }

        // Find surrounding points
        let idx = self.points.iter().position(|p| p.time >= time);

        match idx {
            None => self.points.last().copied(),
            Some(0) => Some(self.points[0]),
            Some(i) => {
                let p1 = &self.points[i - 1];
                let p2 = &self.points[i];

                let t = (time - p1.time) / (p2.time - p1.time);
                let t = t.clamp(0.0, 1.0);

                Some(TrajectoryPoint {
                    position: p1.position.lerp(p2.position, t),
                    velocity: p1.velocity.lerp(p2.velocity, t),
                    acceleration: p1.acceleration.lerp(p2.acceleration, t),
                    yaw: p1.yaw + t * (p2.yaw - p1.yaw),
                    time,
                })
            }
        }
    }

    /// Total duration
    pub fn duration(&self) -> f32 {
        self.points.last().map(|p| p.time).unwrap_or(0.0)
    }

    /// Number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}
