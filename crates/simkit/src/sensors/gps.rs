//! GPS (Global Positioning System) sensor

use super::{Sensor, SensorBase, SensorType};
use crate::common::{GeoPoint, Pose, TimePoint};
use crate::physics::{EnvironmentState, KinematicsState};

/// GPS sensor output data
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct GpsOutput {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Geographic position
    pub geo_point: GeoPoint,
    /// Velocity in NED frame (North, East, Down) in m/s
    pub velocity: glam::Vec3,
    /// Horizontal accuracy in meters
    pub horizontal_accuracy: f32,
    /// Vertical accuracy in meters
    pub vertical_accuracy: f32,
    /// Velocity accuracy in m/s
    pub velocity_accuracy: f32,
    /// Whether fix is available
    pub has_fix: bool,
}

/// GPS sensor configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsConfig {
    /// Horizontal noise standard deviation (m)
    pub horizontal_noise_std: f32,
    /// Vertical noise standard deviation (m)
    pub vertical_noise_std: f32,
    /// Velocity noise standard deviation (m/s)
    pub velocity_noise_std: f32,
    /// Update rate (Hz)
    pub update_rate: f32,
    /// Number of satellites required for fix
    pub min_satellites: u32,
    /// Whether to simulate loss of fix
    pub simulate_dropouts: bool,
}

impl Default for GpsConfig {
    fn default() -> Self {
        Self {
            horizontal_noise_std: 1.0,
            vertical_noise_std: 2.0,
            velocity_noise_std: 0.1,
            update_rate: 10.0,
            min_satellites: 8,
            simulate_dropouts: false,
        }
    }
}

/// GPS sensor
#[derive(Debug)]
pub struct GpsSensor {
    base: SensorBase,
    config: GpsConfig,
    output: GpsOutput,
    time_since_update: f32,
    home_geo_point: Option<GeoPoint>,
}

impl GpsSensor {
    /// Create new GPS sensor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Gps),
            config: GpsConfig::default(),
            output: GpsOutput::default(),
            time_since_update: 0.0,
            home_geo_point: None,
        }
    }

    /// Create with custom config
    pub fn with_config(name: impl Into<String>, config: GpsConfig) -> Self {
        Self {
            base: SensorBase::new(name, SensorType::Gps),
            config,
            output: GpsOutput::default(),
            time_since_update: 0.0,
            home_geo_point: None,
        }
    }

    /// Get latest output
    pub fn output(&self) -> &GpsOutput {
        &self.output
    }

    /// Get configuration
    pub fn config(&self) -> &GpsConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: GpsConfig) {
        self.config = config;
    }

    /// Get home geo point
    pub fn home_geo_point(&self) -> Option<&GeoPoint> {
        self.home_geo_point.as_ref()
    }

    /// Convert local NED position to geo point (simplified)
    fn ned_to_geo(&self, local_position: glam::Vec3) -> Option<GeoPoint> {
        let home = self.home_geo_point?;

        // Simplified conversion (not accounting for Earth curvature)
        // 1 degree latitude ≈ 111 km
        // 1 degree longitude ≈ 111 km * cos(latitude)
        let lat_rad = home.latitude.to_radians();
        let cos_lat = lat_rad.cos();

        let delta_north = local_position.x;
        let delta_east = local_position.y;

        let delta_lat = delta_north as f64 / 111_000.0;
        let delta_lon = delta_east as f64 / (111_000.0 * cos_lat);

        Some(GeoPoint {
            latitude: home.latitude + delta_lat,
            longitude: home.longitude + delta_lon,
            altitude: home.altitude - local_position.z,
        })
    }
}

impl Sensor for GpsSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, environment: &EnvironmentState) {
        self.base.initialize(kinematics, environment);
        self.home_geo_point = Some(environment.home_geo_point);

        // Set initial output
        if let Some(geo_point) = self.ned_to_geo(kinematics.pose.position) {
            self.output.geo_point = geo_point;
            self.output.has_fix = true;
        }
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
            // Update position
            if let Some(geo_point) = self.ned_to_geo(kinematics.pose.position) {
                self.output.geo_point = geo_point;
                self.output.has_fix = true;
            }

            // Update velocity (in NED frame: North=x, East=y, Down=-z)
            let velocity = kinematics.twist.linear;
            self.output.velocity = glam::Vec3::new(velocity.x, velocity.y, -velocity.z);

            self.output.horizontal_accuracy = self.config.horizontal_noise_std;
            self.output.vertical_accuracy = self.config.vertical_noise_std;
            self.output.velocity_accuracy = self.config.velocity_noise_std;
            self.output.time_stamp = kinematics.timestamp;
        }
    }

    fn reset(&mut self) {
        self.base.reset();
        self.output = GpsOutput::default();
        self.time_since_update = 0.0;
        self.home_geo_point = None;
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
    }
}
