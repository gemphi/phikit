//! Sensor implementations for rust-sim-kit
//!
//! Provides sensor models that can be used in simulation:
//! - IMU (Inertial Measurement Unit)
//! - GPS (Global Positioning System)
//! - Lidar (Light Detection and Ranging)
//! - Barometer (Pressure/Altitude)
//! - Magnetometer (Magnetic field)
//! - Distance sensor

use crate::common::Pose;
use crate::physics::{EnvironmentState, KinematicsState};

/// Common sensor configuration (used by all sensors)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SensorConfig {
    /// Sensor name
    pub name: String,
    /// Sensor type
    pub sensor_type: SensorType,
    /// Update frequency in Hz
    pub update_frequency: f32,
    /// Whether sensor is enabled
    pub enabled: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            name: "sensor".to_string(),
            sensor_type: SensorType::Imu,
            update_frequency: 10.0,
            enabled: true,
        }
    }
}

pub mod barometer;
pub mod camera;
pub mod distance;
pub mod gps;
pub mod imu;
pub mod lidar;
pub mod magnetometer;

pub use barometer::{BarometerOutput, BarometerSensor};
pub use camera::{CameraCaptureQueue, CameraConfig, CameraSensor};
pub use distance::DistanceSensor;
pub use gps::{GpsOutput, GpsSensor};
pub use imu::{ImuOutput, ImuSensor};
pub use lidar::LidarSensor;
pub use magnetometer::{MagnetometerOutput, MagnetometerSensor};

/// Sensor type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SensorType {
    /// Barometer for altitude/pressure
    Barometer = 1,
    /// Inertial Measurement Unit
    Imu = 2,
    /// GPS receiver
    Gps = 3,
    /// Magnetometer for heading
    Magnetometer = 4,
    /// Distance sensor
    Distance = 5,
    /// Lidar sensor
    Lidar = 6,
    /// Camera for image capture
    Camera = 7,
}

/// Ground truth data provided to sensors
#[derive(Debug, Clone, Copy)]
pub struct GroundTruth<'a> {
    /// Kinematics state reference
    pub kinematics: &'a KinematicsState,
    /// Environment state reference
    pub environment: &'a EnvironmentState,
}

/// Base trait for all sensors
pub trait Sensor: Send + Sync {
    /// Get sensor type
    fn sensor_type(&self) -> SensorType;

    /// Get sensor name
    fn name(&self) -> &str;

    /// Initialize sensor with ground truth data
    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState);

    /// Update sensor (called each simulation step)
    fn update(&mut self, dt: f32);

    /// Reset sensor to initial state
    fn reset(&mut self);

    /// Check if sensor is enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Set enabled state
    fn set_enabled(&mut self, _enabled: bool) {}

    /// Get relative pose of sensor on vehicle
    fn relative_pose(&self) -> &Pose;

    /// Set relative pose
    fn set_relative_pose(&mut self, pose: Pose);
}

/// Base sensor implementation
#[derive(Debug)]
pub struct SensorBase {
    /// Sensor name
    name: String,
    /// Sensor type
    sensor_type: SensorType,
    /// Relative pose on vehicle
    relative_pose: Pose,
    /// Current ground truth
    ground_truth: Option<(KinematicsState, EnvironmentState)>,
    /// Whether sensor is enabled
    enabled: bool,
    /// Last timestamp
    last_timestamp: u64,
}

impl SensorBase {
    /// Create new sensor base
    pub fn new(name: impl Into<String>, sensor_type: SensorType) -> Self {
        Self {
            name: name.into(),
            sensor_type,
            relative_pose: Pose::identity(),
            ground_truth: None,
            enabled: true,
            last_timestamp: 0,
        }
    }

    /// Create from config
    pub fn from_config(config: &SensorConfig) -> Self {
        Self {
            name: config.name.clone(),
            sensor_type: config.sensor_type,
            relative_pose: Pose::identity(),
            ground_truth: None,
            enabled: config.enabled,
            last_timestamp: 0,
        }
    }

    /// Check if enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get last timestamp
    pub fn last_timestamp(&self) -> u64 {
        self.last_timestamp
    }

    /// Get ground truth
    pub fn ground_truth(&self) -> Option<&(KinematicsState, EnvironmentState)> {
        self.ground_truth.as_ref()
    }

    /// Set ground truth
    pub fn set_ground_truth(
        &mut self,
        kinematics: &KinematicsState,
        environment: &EnvironmentState,
    ) {
        self.ground_truth = Some((*kinematics, *environment));
    }
}

impl Sensor for SensorBase {
    fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.set_ground_truth(kinematics, environment);
    }

    fn update(&mut self, _dt: f32) {
        // Base implementation does nothing
    }

    fn reset(&mut self) {
        self.ground_truth = None;
    }

    fn relative_pose(&self) -> &Pose {
        &self.relative_pose
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.relative_pose = pose;
    }
}

/// Collection of sensors on a vehicle
pub struct SensorCollection {
    sensors: Vec<Box<dyn Sensor + Send + Sync>>,
}

impl Default for SensorCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorCollection {
    /// Create empty sensor collection
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
        }
    }

    /// Add a sensor
    pub fn add(&mut self, sensor: Box<dyn Sensor + Send + Sync>) {
        self.sensors.push(sensor);
    }

    /// Get sensor by name
    pub fn get(&self, name: &str) -> Option<&(dyn Sensor + Send + Sync)> {
        self.sensors
            .iter()
            .find(|s| s.name() == name)
            .map(|s| s.as_ref())
    }

    /// Get all sensors
    pub fn all(&self) -> &[Box<dyn Sensor + Send + Sync>] {
        &self.sensors
    }

    /// Get all sensors as mutable slice (use this for mutable access)
    pub fn all_as_mut_slice(&mut self) -> &mut [Box<dyn Sensor + Send + Sync>] {
        &mut self.sensors
    }

    /// Update all sensors
    pub fn update(&mut self, dt: f32) {
        for sensor in &mut self.sensors {
            sensor.update(dt);
        }
    }

    /// Reset all sensors
    pub fn reset(&mut self) {
        for sensor in &mut self.sensors {
            sensor.reset();
        }
    }

    /// Initialize all sensors
    pub fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        for sensor in &mut self.sensors {
            sensor.initialize(kinematics, environment);
        }
    }
}
