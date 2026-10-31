//! Lidar (Light Detection and Ranging) sensor

use super::{Sensor, SensorBase, SensorType};
use crate::common::{LidarData, Pose};
use crate::physics::{EnvironmentState, KinematicsState};

/// Lidar sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LidarConfig {
    /// Number of horizontal samples
    pub num_horizontal_samples: u32,
    /// Number of vertical samples
    pub num_vertical_samples: u32,
    /// Horizontal field of view (degrees)
    pub horizontal_fov: f32,
    /// Vertical field of view (degrees)
    pub vertical_fov: f32,
    /// Maximum range (m)
    pub max_range: f32,
    /// Minimum range (m)
    pub min_range: f32,
    /// Update rate (Hz)
    pub update_rate: f32,
    /// Whether to generate segmentation labels
    pub enable_segmentation: bool,
}

impl Default for LidarConfig {
    fn default() -> Self {
        Self {
            num_horizontal_samples: 1024,
            num_vertical_samples: 64,
            horizontal_fov: 360.0,
            vertical_fov: 30.0,
            max_range: 100.0,
            min_range: 0.5,
            update_rate: 10.0,
            enable_segmentation: false,
        }
    }
}

/// Lidar sensor
#[derive(Debug)]
pub struct LidarSensor {
    base: SensorBase,
    config: LidarConfig,
    output: LidarData,
    time_since_update: f32,
}

impl LidarSensor {
    /// Create new Lidar sensor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Lidar),
            config: LidarConfig::default(),
            output: LidarData::default(),
            time_since_update: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: LidarConfig) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Lidar),
            config,
            output: LidarData::default(),
            time_since_update: 0.0,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &LidarData {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &LidarConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: LidarConfig) {
        self.config = config;
    }

    /// Simulate lidar point cloud (simplified - would integrate with scene)
    fn simulate_point_cloud(&self, _kinematics: &KinematicsState) -> Vec<f64> {
        // Placeholder - would cast rays into scene and return hit points
        // Returns empty point cloud for now
        Vec::new()
    }
}

impl Sensor for LidarSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.base.initialize(kinematics, environment);

        // Set initial output
        self.output.point_cloud = self.simulate_point_cloud(kinematics);
        self.output.pose = kinematics.pose;
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
            self.output.point_cloud = self.simulate_point_cloud(kinematics);
            self.output.pose = kinematics.pose;
            self.output.time_stamp = kinematics.timestamp;

            if self.config.enable_segmentation {
                // Would populate segmentation IDs for each point
                self.output.segmentation.clear();
            }
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = LidarData::default();
        self.time_since_update = 0.0;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
