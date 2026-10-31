//! Car (ground vehicle) implementation

use super::{Vehicle, VehicleBase, VehicleConfig};
use crate::common::{GeoPoint, Pose, Twist};
use crate::physics::{Environment, KinematicsState, Wrench};
use glam::Vec3;

/// Car drive mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveMode {
    /// Manual driving
    Manual,
    /// Autonomous/API control
    Autonomous,
}

/// Car configuration
#[derive(Debug, Clone, PartialEq)]
pub struct CarConfig {
    /// Base vehicle config
    pub vehicle: VehicleConfig,
    /// Maximum steering angle (degrees)
    pub max_steering_angle: f32,
    /// Maximum throttle/brake force (N)
    pub max_force: f32,
    /// Wheel radius (m)
    pub wheel_radius: f32,
    /// Track width (m)
    pub track_width: f32,
    /// Wheelbase (m)
    pub wheelbase: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            vehicle: VehicleConfig {
                name: String::from("car"),
                mass: 1500.0,
                max_velocity: 50.0,
                ..Default::default()
            },
            max_steering_angle: 30.0,
            max_force: 5000.0,
            wheel_radius: 0.35,
            track_width: 1.6,
            wheelbase: 2.8,
        }
    }
}

/// Car vehicle
#[derive(Debug)]
pub struct CarVehicle {
    base: VehicleBase,
    config: CarConfig,
    /// Current steering (-1 to 1, full left to full right)
    steering: f32,
    /// Current throttle (0 to 1)
    throttle: f32,
    /// Current brake (0 to 1)
    brake: f32,
    /// Drive mode
    drive_mode: DriveMode,
    /// Target speed for autonomous mode
    target_speed: f32,
}

impl CarVehicle {
    /// Create new car
    pub fn new(config: CarConfig) -> Self {
        let base = VehicleBase::new(config.vehicle.clone());

        Self {
            base,
            config,
            steering: 0.0,
            throttle: 0.0,
            brake: 0.0,
            drive_mode: DriveMode::Manual,
            target_speed: 0.0,
        }
    }

    /// Get car config
    pub fn car_config(&self) -> &CarConfig {
        &self.config
    }

    /// Set steering (-1 to 1)
    pub fn set_steering(&mut self, steering: f32) {
        self.steering = steering.clamp(-1.0, 1.0);
    }

    /// Get current steering
    pub fn steering(&self) -> f32 {
        self.steering
    }

    /// Set throttle (0 to 1)
    pub fn set_throttle(&mut self, throttle: f32) {
        self.throttle = throttle.clamp(0.0, 1.0);
    }

    /// Get current throttle
    pub fn throttle(&self) -> f32 {
        self.throttle
    }

    /// Set brake (0 to 1)
    pub fn set_brake(&mut self, brake: f32) {
        self.brake = brake.clamp(0.0, 1.0);
    }

    /// Get current brake
    pub fn brake(&self) -> f32 {
        self.brake
    }

    /// Get drive mode
    pub fn drive_mode(&self) -> DriveMode {
        self.drive_mode
    }

    /// Set drive mode
    pub fn set_drive_mode(&mut self, mode: DriveMode) {
        self.drive_mode = mode;
    }

    /// Set target speed for autonomous mode (m/s)
    pub fn set_target_speed(&mut self, speed: f32) {
        self.target_speed = speed.clamp(0.0, self.config.vehicle.max_velocity);
    }

    /// Get target speed
    pub fn target_speed(&self) -> f32 {
        self.target_speed
    }

    /// Move car forward by distance at given speed
    pub fn move_forward(&mut self, distance: f32, speed: f32) {
        let current = self.base.kinematics().pose.position;
        let orientation = self.base.kinematics().pose.orientation;
        let forward = orientation * Vec3::X;
        let target = current + forward * distance;
        self.base.kinematics_mut().pose.position = target;
        let _ = speed; // Speed would be used with proper physics timing
    }

    /// Calculate control wrench from inputs
    fn calculate_control_wrench(&self, _dt: f32) -> Wrench {
        // Calculate force from throttle/brake
        let force_mag =
            self.throttle * self.config.max_force - self.brake * self.config.max_force * 2.0;

        // Get forward direction in body frame
        let forward = Vec3::X;
        let force = forward * force_mag;

        // Simple ackermann steering model for yaw rate
        if self.steering.abs() > 0.01 {
            let speed = self.base.kinematics().twist.linear.length();
            let steering_angle = self.steering * self.config.max_steering_angle.to_radians();
            let curvature = steering_angle.tan() / self.config.wheelbase;
            let yaw_rate = speed * curvature;
            let torque = Vec3::new(0.0, 0.0, yaw_rate * self.config.vehicle.mass * 0.1);

            Wrench { force, torque }
        } else {
            Wrench {
                force,
                torque: Vec3::ZERO,
            }
        }
    }

    /// Apply simplified physics
    fn apply_physics(&mut self, dt: f32) {
        let wrench = self.calculate_control_wrench(dt);
        let mass = self.config.vehicle.mass;

        // Linear acceleration
        let accel = wrench.force / mass;
        self.base.kinematics_mut().twist.linear += accel * dt;

        // Apply rolling resistance
        let resistance = -self.base.kinematics().twist.linear * 0.05;
        self.base.kinematics_mut().twist.linear += resistance * dt;

        // Clamp to ground (z = 0)
        let pos = &mut self.base.kinematics_mut().pose.position;
        pos.z = 0.0;

        // Update position
        let velocity = self.base.kinematics().twist.linear;
        self.base.kinematics_mut().pose.position += velocity * dt;

        // Update orientation from yaw rate
        let yaw_rate = wrench.torque.z / (mass * 0.1); // Simplified
        if yaw_rate.abs() > 0.001 {
            let delta_yaw = yaw_rate * dt;
            let rotation = glam::Quat::from_rotation_z(delta_yaw);
            self.base.kinematics_mut().pose.orientation =
                rotation * self.base.kinematics().pose.orientation;
        }

        // Update timestamp
        let ts = &mut self.base.kinematics_mut().timestamp;
        *ts += (dt * 1e9) as u64;
    }
}

impl Vehicle for CarVehicle {
    fn name(&self) -> &str {
        &self.base.config().name
    }

    fn pose(&self) -> &Pose {
        &self.base.kinematics().pose
    }

    fn set_pose(&mut self, pose: Pose) {
        self.base.kinematics_mut().pose = pose;
    }

    fn kinematics(&self) -> &KinematicsState {
        self.base.kinematics()
    }

    fn sensors(&self) -> &super::SensorCollection {
        self.base.sensors()
    }

    fn sensors_mut(&mut self) -> &mut super::SensorCollection {
        self.base.sensors_mut()
    }

    fn update(&mut self, dt: f32, environment: &Environment) {
        self.apply_physics(dt);
        self.base.update_sensors(dt, environment);
    }

    fn reset(&mut self) {
        self.base.kinematics_mut().pose = self.base.config().initial_pose;
        self.base.kinematics_mut().twist = Twist::zero();
        self.base.kinematics_mut().accelerations = crate::physics::Accelerations::zero();
        self.base.set_armed(false);
        self.base.sensors_mut().reset();
        self.steering = 0.0;
        self.throttle = 0.0;
        self.brake = 0.0;
        self.target_speed = 0.0;
    }

    fn arm_disarm(&mut self, arm: bool) {
        self.base.set_armed(arm);
    }

    fn is_armed(&self) -> bool {
        self.base.is_armed()
    }

    fn enable_api_control(&mut self, enabled: bool) {
        self.base.set_api_control_enabled(enabled);
    }

    fn is_api_control_enabled(&self) -> bool {
        self.base.is_api_control_enabled()
    }

    fn home_geo_point(&self) -> &GeoPoint {
        &self.base.config().home_geo_point
    }

    fn set_home_geo_point(&mut self, home: GeoPoint) {
        self.base.config_mut().home_geo_point = home;
    }
}
