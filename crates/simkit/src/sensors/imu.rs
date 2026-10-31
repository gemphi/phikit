//! Inertial Measurement Unit (IMU) sensor

use super::{Sensor, SensorBase, SensorType};
use crate::common::{Pose, TimePoint};
use crate::physics::{EnvironmentState, KinematicsState};
use glam::Vec3;

/// IMU sensor output data
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ImuOutput {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Angular velocity in body frame (rad/s)
    pub angular_velocity: Vec3,
    /// Linear acceleration in body frame (m/s^2)
    pub linear_acceleration: Vec3,
    /// Orientation quaternion
    pub orientation: glam::Quat,
}

/// IMU sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuConfig {
    /// Gyroscope noise standard deviation (rad/s)
    pub gyro_noise_std: f32,
    /// Accelerometer noise standard deviation (m/s^2)
    pub accel_noise_std: f32,
    /// Gyroscope bias
    pub gyro_bias: Vec3,
    /// Accelerometer bias
    pub accel_bias: Vec3,
    /// Update rate (Hz)
    pub update_rate: f32,
}

impl Default for ImuConfig {
    fn default() -> Self {
        Self {
            gyro_noise_std: 0.001,
            accel_noise_std: 0.01,
            gyro_bias: Vec3::ZERO,
            accel_bias: Vec3::ZERO,
            update_rate: 1000.0,
        }
    }
}

/// Inertial Measurement Unit sensor
#[derive(Debug)]
pub struct ImuSensor {
    base: SensorBase,
    config: ImuConfig,
    output: ImuOutput,
    time_since_update: f32,
}

impl ImuSensor {
    /// Create new IMU sensor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Imu),
            config: ImuConfig::default(),
            output: ImuOutput::default(),
            time_since_update: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: ImuConfig) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Imu),
            config,
            output: ImuOutput::default(),
            time_since_update: 0.0,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &ImuOutput {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &ImuConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ImuConfig) {
        self.config = config;
    }
}

impl Sensor for ImuSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.base.initialize(kinematics, environment);

        // Set initial output from ground truth
        self.output.orientation = kinematics.pose.orientation;
        self.output.angular_velocity = kinematics.twist.angular;

        // Calculate body-frame acceleration
        let orientation = kinematics.pose.orientation;
        let world_accel = kinematics.accelerations.linear;
        let gravity = Vec3::new(0.0, 0.0, -9.81);
        let accel_with_gravity = world_accel - gravity;
        self.output.linear_acceleration = orientation.inverse() * accel_with_gravity;

        self.output.time_stamp = kinematics.timestamp;
    }

    fn update(&mut self, dt: f32) {
        self.time_since_update += dt;

        let update_interval = 1.0 / self.config.update_rate;
        if self.time_since_update < update_interval {
            return;
        }

        self.time_since_update = 0.0;

        if let Some((kinematics, _)) = self.base.ground_truth() {
            // Update orientation
            self.output.orientation = kinematics.pose.orientation;

            // Update angular velocity
            self.output.angular_velocity = kinematics.twist.angular + self.config.gyro_bias;

            // Update linear acceleration (convert from world to body frame, add gravity)
            let orientation = kinematics.pose.orientation;
            let world_accel = kinematics.accelerations.linear;
            let gravity = Vec3::new(0.0, 0.0, -9.81);
            let accel_with_gravity = world_accel - gravity;
            self.output.linear_acceleration =
                orientation.inverse() * accel_with_gravity + self.config.accel_bias;

            self.output.time_stamp = kinematics.timestamp;
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = ImuOutput::default();
        self.time_since_update = 0.0;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
