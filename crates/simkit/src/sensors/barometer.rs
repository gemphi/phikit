//! Barometer sensor for altitude and pressure measurement

use super::{Sensor, SensorBase, SensorType};
use crate::common::{Pose, TimePoint};
use crate::physics::{EnvironmentState, KinematicsState};

/// Barometer sensor output data
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BarometerOutput {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Altitude above sea level (m)
    pub altitude: f32,
    /// Pressure (Pa)
    pub pressure: f32,
    /// Temperature (Celsius)
    pub temperature: f32,
}

/// Barometer sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarometerConfig {
    /// Pressure noise standard deviation (Pa)
    pub pressure_noise_std: f32,
    /// Altitude noise standard deviation (m)
    pub altitude_noise_std: f32,
    /// Update rate (Hz)
    pub update_rate: f32,
    /// Sea level pressure reference (Pa)
    pub sea_level_pressure: f32,
}

impl Default for BarometerConfig {
    fn default() -> Self {
        Self {
            pressure_noise_std: 10.0,
            altitude_noise_std: 1.0,
            update_rate: 50.0,
            sea_level_pressure: 101_325.0, // Standard atmosphere at sea level
        }
    }
}

/// Barometer sensor
#[derive(Debug)]
pub struct BarometerSensor {
    base: SensorBase,
    config: BarometerConfig,
    output: BarometerOutput,
    time_since_update: f32,
}

impl BarometerSensor {
    /// Create new barometer sensor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Barometer),
            config: BarometerConfig::default(),
            output: BarometerOutput::default(),
            time_since_update: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: BarometerConfig) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Barometer),
            config,
            output: BarometerOutput::default(),
            time_since_update: 0.0,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &BarometerOutput {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &BarometerConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: BarometerConfig) {
        self.config = config;
    }

    /// Calculate altitude from pressure using barometric formula
    fn pressure_to_altitude(&self, pressure: f32) -> f32 {
        // Barometric formula: h = 44330 * (1 - (p/p0)^(1/5.255))
        let p0 = self.config.sea_level_pressure;
        44330.0 * (1.0 - (pressure / p0).powf(1.0 / 5.255))
    }
}

impl Sensor for BarometerSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.base.initialize(kinematics, environment);

        // Set initial output from environment
        self.output.pressure = environment.pressure;
        self.output.temperature = environment.temperature;
        self.output.altitude = self.pressure_to_altitude(environment.pressure);
        self.output.time_stamp = kinematics.timestamp;
    }

    fn update(&mut self, dt: f32) {
        self.time_since_update += dt;

        let update_interval = 1.0 / self.config.update_rate;
        if self.time_since_update < update_interval {
            return;
        }

        self.time_since_update = 0.0;

        if let Some((_, environment)) = self.base.ground_truth() {
            // Update from environment state
            self.output.pressure = environment.pressure;
            self.output.temperature = environment.temperature;
            self.output.altitude = self.pressure_to_altitude(environment.pressure);
            self.output.time_stamp = environment.timestamp;
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = BarometerOutput::default();
        self.time_since_update = 0.0;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
