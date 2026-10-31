//! Multirotor (drone) vehicle implementation

use super::{Vehicle, VehicleBase, VehicleConfig};
use crate::common::{GeoPoint, Pose, Twist};
use crate::physics::{Environment, KinematicsState, Wrench};
use glam::Vec3;

/// Multirotor configuration
#[derive(Debug, Clone, PartialEq)]
pub struct MultirotorConfig {
    /// Base vehicle config
    pub vehicle: VehicleConfig,
    /// Number of rotors
    pub rotor_count: u32,
    /// Maximum thrust per rotor (N)
    pub max_thrust_per_rotor: f32,
    /// Arm length (m)
    pub arm_length: f32,
    /// Drag coefficient
    pub drag_coefficient: f32,
    /// Pitch/Roll max angle (degrees)
    pub max_tilt_angle: f32,
    /// Yaw rate max (deg/s)
    pub max_yaw_rate: f32,
}

impl Default for MultirotorConfig {
    fn default() -> Self {
        Self {
            vehicle: VehicleConfig {
                name: String::from("drone"),
                mass: 1.5,
                max_velocity: 20.0,
                ..Default::default()
            },
            rotor_count: 4,
            max_thrust_per_rotor: 20.0,
            arm_length: 0.25,
            drag_coefficient: 0.1,
            max_tilt_angle: 45.0,
            max_yaw_rate: 90.0,
        }
    }
}

/// Multirotor vehicle (drone)
#[derive(Debug)]
pub struct MultirotorVehicle {
    base: VehicleBase,
    config: MultirotorConfig,
    /// Target velocity commands (from API or RC)
    target_velocity: Vec3,
    /// Target yaw rate
    target_yaw_rate: f32,
    /// Current rotor thrusts
    rotor_thrusts: Vec<f32>,
}

impl MultirotorVehicle {
    /// Create new multirotor
    pub fn new(config: MultirotorConfig) -> Self {
        let rotor_count = config.rotor_count as usize;
        let base = VehicleBase::new(config.vehicle.clone());

        Self {
            base,
            config,
            target_velocity: Vec3::ZERO,
            target_yaw_rate: 0.0,
            rotor_thrusts: vec![0.0; rotor_count],
        }
    }

    /// Get multirotor config
    pub fn multirotor_config(&self) -> &MultirotorConfig {
        &self.config
    }

    /// Set target velocity (API command)
    pub fn set_target_velocity(&mut self, velocity: Vec3) {
        self.target_velocity = velocity.clamp_length_max(self.config.vehicle.max_velocity);
    }

    /// Get target velocity
    pub fn target_velocity(&self) -> Vec3 {
        self.target_velocity
    }

    /// Set target yaw rate (API command)
    pub fn set_target_yaw_rate(&mut self, yaw_rate: f32) {
        let max_yaw_rad = self.config.max_yaw_rate.to_radians();
        self.target_yaw_rate = yaw_rate.clamp(-max_yaw_rad, max_yaw_rad);
    }

    /// Get current rotor thrusts
    pub fn rotor_thrusts(&self) -> &[f32] {
        &self.rotor_thrusts
    }

    /// Move by velocity (API command)
    pub fn move_by_velocity(&mut self, vx: f32, vy: f32, vz: f32, duration: f32) {
        self.set_target_velocity(Vec3::new(vx, vy, vz));
        // Duration would be used in a real implementation with timing
        let _ = duration;
    }

    /// Move to position (API command)
    pub fn move_to_position(&mut self, x: f32, y: f32, z: f32, velocity: f32) {
        let target = Vec3::new(x, y, z);
        let current = self.base.kinematics.pose.position;
        let diff = target - current;
        let direction = diff.normalize_or_zero();
        let vel = velocity.min(self.config.vehicle.max_velocity);
        self.set_target_velocity(direction * vel);
    }

    /// Calculate control wrench from target velocity
    fn calculate_control_wrench(&self, _dt: f32) -> Wrench {
        // Simplified PID-like controller
        let current_vel = self.base.kinematics.twist.linear;
        let vel_error = self.target_velocity - current_vel;

        // Gravity compensation
        let gravity = Vec3::new(0.0, 0.0, -9.81 * self.config.vehicle.mass);

        // Simple P controller for velocity
        let thrust_force = gravity + vel_error * 2.0;

        Wrench {
            force: thrust_force,
            torque: Vec3::new(0.0, 0.0, self.target_yaw_rate * 0.1),
        }
    }

    /// Apply physics step
    fn apply_physics(&mut self, dt: f32) {
        let wrench = self.calculate_control_wrench(dt);
        let mass = self.config.vehicle.mass;

        // F = ma -> a = F/m
        let acceleration = wrench.force / mass;

        // Update velocity
        self.base.kinematics.twist.linear += acceleration * dt;

        // Apply drag
        let drag = -self.base.kinematics.twist.linear * self.config.drag_coefficient;
        self.base.kinematics.twist.linear += drag * dt;

        // Update position
        self.base.kinematics.pose.position += self.base.kinematics.twist.linear * dt;

        // Update angular velocity (simplified)
        self.base.kinematics.twist.angular = Vec3::new(
            0.0, // Roll rate
            0.0, // Pitch rate
            self.target_yaw_rate,
        );

        // Update orientation
        let angular_vel = self.base.kinematics.twist.angular;
        if angular_vel.length_squared() > 0.0001 {
            let angle = angular_vel.length() * dt;
            let axis = angular_vel.normalize();
            let rotation = glam::Quat::from_axis_angle(axis, angle);
            self.base.kinematics.pose.orientation =
                rotation * self.base.kinematics.pose.orientation;
        }

        // Update timestamp
        self.base.kinematics.timestamp += (dt * 1e9) as u64;
    }
}

impl Vehicle for MultirotorVehicle {
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
        // Apply physics
        self.apply_physics(dt);

        // Update sensors
        self.base.update_sensors(dt, environment);
    }

    fn reset(&mut self) {
        self.base.kinematics_mut().pose = self.base.config().initial_pose;
        self.base.kinematics_mut().twist = Twist::zero();
        self.base.kinematics_mut().accelerations = crate::physics::Accelerations::zero();
        self.base.set_armed(false);
        self.base.sensors_mut().reset();
        self.target_velocity = Vec3::ZERO;
        self.target_yaw_rate = 0.0;
        for thrust in &mut self.rotor_thrusts {
            *thrust = 0.0;
        }
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
