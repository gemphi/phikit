//! Sensor Settings
//!
//! Configuration for all sensor types.

use serde::{Deserialize, Serialize};

/// Sensor settings enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "sensor_type")]
pub enum SensorSettings {
    /// Camera sensor
    #[serde(rename = "Camera")]
    Camera(CameraSettings),
    /// Lidar sensor
    #[serde(rename = "Lidar")]
    Lidar(LidarSettings),
    /// IMU sensor
    #[serde(rename = "Imu")]
    Imu(ImuSettings),
    /// GPS sensor
    #[serde(rename = "Gps")]
    Gps(GpsSettings),
    /// Barometer sensor
    #[serde(rename = "Barometer")]
    Barometer(BarometerSettings),
    /// Magnetometer sensor
    #[serde(rename = "Magnetometer")]
    Magnetometer(MagnetometerSettings),
    /// Distance sensor
    #[serde(rename = "Distance")]
    Distance(DistanceSettings),
}

/// Camera sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Image type
    #[serde(default)]
    pub image_type: u8,
    /// Width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
    /// Field of view (degrees)
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    /// Auto exposure enabled
    #[serde(default = "default_true")]
    pub auto_exposure: bool,
    /// Motion blur amount
    #[serde(default)]
    pub motion_blur_amount: f32,
    /// Gamma correction
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    /// Target gamma
    #[serde(default = "default_target_gamma")]
    pub target_gamma: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
    /// Draw debug frustum
    #[serde(default)]
    pub draw_debug_frustum: bool,
}

fn default_width() -> u32 {
    256
}

fn default_height() -> u32 {
    144
}

fn default_fov() -> f32 {
    90.0
}

fn default_true() -> bool {
    true
}

fn default_gamma() -> f32 {
    2.2
}

fn default_target_gamma() -> f32 {
    2.2
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            image_type: 0,
            width: default_width(),
            height: default_height(),
            fov_degrees: default_fov(),
            auto_exposure: true,
            motion_blur_amount: 0.0,
            gamma: default_gamma(),
            target_gamma: default_target_gamma(),
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
            draw_debug_frustum: false,
        }
    }
}

/// Lidar sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Channels (number of lasers)
    #[serde(default = "default_channels")]
    pub channels: u32,
    /// Range (meters)
    #[serde(default = "default_range")]
    pub range: f32,
    /// Points per second
    #[serde(default = "default_points_per_sec")]
    pub points_per_second: u32,
    /// Rotation frequency (Hz)
    #[serde(default = "default_rot_freq")]
    pub rotation_frequency: f32,
    /// Horizontal field of view (degrees)
    #[serde(default = "default_hfov")]
    pub horizontal_fov: f32,
    /// Vertical field of view (degrees)
    #[serde(default)]
    pub vertical_fov: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
    /// Data frame
    #[serde(default)]
    pub data_frame: String,
    /// Draw debug points
    #[serde(default)]
    pub draw_debug_points: bool,
    /// Ignore marked surfaces
    #[serde(default)]
    pub ignore_marked_surfaces: bool,
}

fn default_channels() -> u32 {
    16
}

fn default_range() -> f32 {
    100.0
}

fn default_points_per_sec() -> u32 {
    100000
}

fn default_rot_freq() -> f32 {
    10.0
}

fn default_hfov() -> f32 {
    360.0
}

impl Default for LidarSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            channels: default_channels(),
            range: default_range(),
            points_per_second: default_points_per_sec(),
            rotation_frequency: default_rot_freq(),
            horizontal_fov: default_hfov(),
            vertical_fov: -20.0,
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
            data_frame: String::new(),
            draw_debug_points: false,
            ignore_marked_surfaces: false,
        }
    }
}

/// IMU sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Update rate (Hz)
    #[serde(default = "default_imu_rate")]
    pub update_rate: f32,
    /// Linear noise standard deviation
    #[serde(default)]
    pub linear_noise_std: f32,
    /// Angular noise standard deviation
    #[serde(default)]
    pub angular_noise_std: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
}

fn default_imu_rate() -> f32 {
    1000.0
}

impl Default for ImuSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            update_rate: default_imu_rate(),
            linear_noise_std: 0.0,
            angular_noise_std: 0.0,
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}

/// GPS sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Update rate (Hz)
    #[serde(default = "default_gps_rate")]
    pub update_rate: f32,
    /// EPH (horizontal position error)
    #[serde(default)]
    pub eph: f32,
    /// EPV (vertical position error)
    #[serde(default)]
    pub epv: f32,
    /// Velocity error
    #[serde(default)]
    pub velocity_error: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
}

fn default_gps_rate() -> f32 {
    10.0
}

impl Default for GpsSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            update_rate: default_gps_rate(),
            eph: 0.0,
            epv: 0.0,
            velocity_error: 0.0,
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}

/// Barometer sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarometerSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Update rate (Hz)
    #[serde(default = "default_baro_rate")]
    pub update_rate: f32,
    /// Pressure noise standard deviation
    #[serde(default)]
    pub pressure_noise_std: f32,
    /// Temperature noise standard deviation
    #[serde(default)]
    pub temperature_noise_std: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
}

fn default_baro_rate() -> f32 {
    50.0
}

impl Default for BarometerSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            update_rate: default_baro_rate(),
            pressure_noise_std: 0.0,
            temperature_noise_std: 0.0,
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}

/// Magnetometer sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetometerSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Update rate (Hz)
    #[serde(default = "default_mag_rate")]
    pub update_rate: f32,
    /// Noise standard deviation
    #[serde(default)]
    pub noise_std: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
}

fn default_mag_rate() -> f32 {
    50.0
}

impl Default for MagnetometerSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            update_rate: default_mag_rate(),
            noise_std: 0.0,
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}

/// Distance sensor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceSettings {
    /// Sensor name
    #[serde(default)]
    pub sensor_name: String,
    /// Update rate (Hz)
    #[serde(default = "default_dist_rate")]
    pub update_rate: f32,
    /// Minimum distance (meters)
    #[serde(default)]
    pub min_distance: f32,
    /// Maximum distance (meters)
    #[serde(default = "default_max_dist")]
    pub max_distance: f32,
    /// Field of view (degrees)
    #[serde(default = "default_dist_fov")]
    pub fov: f32,
    /// Position relative to vehicle
    #[serde(default)]
    pub position: [f32; 3],
    /// Orientation (pitch, roll, yaw) in degrees
    #[serde(default)]
    pub orientation: [f32; 3],
}

fn default_dist_rate() -> f32 {
    50.0
}

fn default_max_dist() -> f32 {
    40.0
}

fn default_dist_fov() -> f32 {
    30.0
}

impl Default for DistanceSettings {
    fn default() -> Self {
        Self {
            sensor_name: String::new(),
            update_rate: default_dist_rate(),
            min_distance: 0.2,
            max_distance: default_max_dist(),
            fov: default_dist_fov(),
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
        }
    }
}
