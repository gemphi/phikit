//! Distance sensor (rangefinder)

use super::{Sensor, SensorBase, SensorType};
use crate::common::{DistanceSensorData, Pose};
use crate::physics::{EnvironmentState, KinematicsState};

/// Distance sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceConfig {
    /// Minimum measurable distance (m)
    pub min_distance: f32,
    /// Maximum measurable distance (m)
    pub max_distance: f32,
    /// Measurement noise standard deviation (m)
    pub noise_std: f32,
    /// Update rate (Hz)
    pub update_rate: f32,
    /// Field of view angle (degrees)
    pub fov: f32,
    /// Whether to include ground in measurement
    pub measure_ground: bool,
}

impl Default for DistanceConfig {
    fn default() -> Self {
        Self {
            min_distance: 0.1,
            max_distance: 10.0,
            noise_std: 0.01,
            update_rate: 50.0,
            fov: 5.0,
            measure_ground: true,
        }
    }
}

/// Distance sensor (ultrasonic, infrared, laser rangefinder, etc.)
#[derive(Debug)]
pub struct DistanceSensor {
    base: SensorBase,
    config: DistanceConfig,
    output: DistanceSensorData,
    time_since_update: f32,
}

impl DistanceSensor {
    /// Create new distance sensor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Distance),
            config: DistanceConfig::default(),
            output: DistanceSensorData::default(),
            time_since_update: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: DistanceConfig) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Distance),
            config,
            output: DistanceSensorData::default(),
            time_since_update: 0.0,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &DistanceSensorData {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &DistanceConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: DistanceConfig) {
        self.config = config;
    }

    /// Simulate distance measurement (simplified - would integrate with collision/raycast system)
    fn simulate_measurement(&self, _kinematics: &KinematicsState) -> f32 {
        // Placeholder - would use raycast against scene geometry
        // Returns a simulated distance within valid range
        let range = self.config.max_distance - self.config.min_distance;
        self.config.min_distance + range * 0.5 // Return middle of range as default
    }
}

impl Sensor for DistanceSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.base.initialize(kinematics, environment);

        // Set initial output
        let distance = self.simulate_measurement(kinematics);
        self.output.distance = distance as f64;
        self.output.min_distance = self.config.min_distance as f64;
        self.output.max_distance = self.config.max_distance as f64;
        self.output.relative_pose = *self.base.relative_pose();
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
            let distance = self.simulate_measurement(kinematics);
            self.output.distance = distance as f64;
            self.output.time_stamp = kinematics.timestamp;
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = DistanceSensorData::default();
        self.time_since_update = 0.0;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
