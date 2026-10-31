//! Magnetometer sensor for heading/magnetic field measurement

use super::{Sensor, SensorBase, SensorType};
use crate::common::{Pose, TimePoint};
use crate::physics::{EnvironmentState, KinematicsState};
use glam::Vec3;

/// Magnetometer sensor output data
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct MagnetometerOutput {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Magnetic field in body frame (Gauss)
    pub magnetic_field_body: Vec3,
    /// Magnetic field covariance (9 elements, 3x3 matrix)
    pub magnetic_field_covariance: Vec<f64>,
}

/// Magnetometer sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnetometerConfig {
    /// Noise standard deviation per axis (Gauss)
    pub noise_std: f32,
    /// Earth's magnetic field reference (Gauss)
    pub reference_field: Vec3,
    /// Update rate (Hz)
    pub update_rate: f32,
    /// Magnetic declination offset (degrees)
    pub declination: f32,
}

impl Default for MagnetometerConfig {
    fn default() -> Self {
        // Earth's magnetic field is roughly 0.5 Gauss
        Self {
            noise_std: 0.005,
            reference_field: Vec3::new(0.25, 0.0, 0.43), // Simplified field vector
            update_rate: 100.0,
            declination: 0.0,
        }
    }
}

/// Magnetometer sensor
#[derive(Debug)]
pub struct MagnetometerSensor {
    base: SensorBase,
    config: MagnetometerConfig,
    output: MagnetometerOutput,
    time_since_update: f32,
}

impl MagnetometerSensor {
    /// Create new magnetometer sensor
    pub fn new(name: impl Into<String>) -> Self {
        let mut output = MagnetometerOutput::default();
        output.magnetic_field_covariance = vec![0.0; 9];

        Self {
            base: SensorBase::new(name, SensorType::Magnetometer),
            config: MagnetometerConfig::default(),
            output,
            time_since_update: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: MagnetometerConfig) -> Self {
        let mut output = MagnetometerOutput::default();
        output.magnetic_field_covariance = vec![0.0; 9];

        Self {
            base: SensorBase::new(name, SensorType::Magnetometer),
            config,
            output,
            time_since_update: 0.0,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &MagnetometerOutput {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &MagnetometerConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: MagnetometerConfig) {
        self.config = config;
    }

    /// Transform magnetic field to body frame
    fn world_to_body(&self, world_field: Vec3, orientation: glam::Quat) -> Vec3 {
        orientation.inverse() * world_field
    }
}

impl Sensor for MagnetometerSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, _environment: &EnvironmentState) {
        self.base.initialize(kinematics, _environment);

        // Set initial output
        let body_field =
            self.world_to_body(self.config.reference_field, kinematics.pose.orientation);
        self.output.magnetic_field_body = body_field;
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
            // Transform reference field to body frame
            let body_field =
                self.world_to_body(self.config.reference_field, kinematics.pose.orientation);

            self.output.magnetic_field_body = body_field;
            self.output.time_stamp = kinematics.timestamp;
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = MagnetometerOutput {
            magnetic_field_covariance: vec![0.0; 9],
            ..Default::default()
        };
        self.time_since_update = 0.0;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
