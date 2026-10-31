//! Vehicle Settings
//!
//! Configuration for multirotor and car vehicles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SensorSettings;

/// Vehicle settings trait
pub trait VehicleSettingsTrait {
    /// Get vehicle type name
    fn vehicle_type(&self) -> &str;
    /// Get default vehicle name
    fn default_vehicle_name(&self) -> &str;
    /// Get auto-create flag
    fn auto_create(&self) -> bool;
}

/// Multirotor vehicle settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultirotorSettings {
    /// Vehicle name
    #[serde(default = "default_multirotor_name")]
    pub vehicle_name: String,
    /// Vehicle type
    #[serde(default = "default_multirotor_type")]
    pub vehicle_type: String,
    /// Default vehicle
    #[serde(default)]
    pub default_vehicle: bool,
    /// Auto-create vehicle
    #[serde(default = "default_true")]
    pub auto_create: bool,
    /// Enable collision passthrough
    #[serde(default)]
    pub enable_collision_passthrough: bool,
    /// Enable collision
    #[serde(default = "default_true")]
    pub enable_collision: bool,
    /// Enable rays
    #[serde(default)]
    pub enable_rays: bool,
    /// Ray length
    #[serde(default = "default_ray_length")]
    pub ray_length: f32,
    /// Ray draw distance
    #[serde(default)]
    pub ray_draw_distance: f32,
    /// Ray draw thickness
    #[serde(default)]
    pub ray_draw_thickness: f32,
    /// Yaw mode
    #[serde(default)]
    pub yaw_mode: YawMode,
    /// Sensors
    #[serde(default)]
    pub sensors: HashMap<String, SensorSettings>,
    /// Cameras
    #[serde(default)]
    pub cameras: HashMap<String, super::CameraSettings>,
}

fn default_multirotor_name() -> String {
    "drone_1".to_string()
}

fn default_multirotor_type() -> String {
    "simpleflight".to_string()
}

fn default_ray_length() -> f32 {
    7.0
}

fn default_true() -> bool {
    true
}

impl Default for MultirotorSettings {
    fn default() -> Self {
        Self {
            vehicle_name: default_multirotor_name(),
            vehicle_type: default_multirotor_type(),
            default_vehicle: true,
            auto_create: true,
            enable_collision_passthrough: false,
            enable_collision: true,
            enable_rays: false,
            ray_length: default_ray_length(),
            ray_draw_distance: 0.0,
            ray_draw_thickness: 0.0,
            yaw_mode: YawMode::default(),
            sensors: HashMap::new(),
            cameras: HashMap::new(),
        }
    }
}

/// Yaw mode for multirotor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum YawMode {
    /// Rate control
    Rate,
    /// Angle control
    #[default]
    Angle,
}

/// Car vehicle settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarSettings {
    /// Vehicle name
    #[serde(default = "default_car_name")]
    pub vehicle_name: String,
    /// Vehicle type
    #[serde(default = "default_car_type")]
    pub vehicle_type: String,
    /// Default vehicle
    #[serde(default)]
    pub default_vehicle: bool,
    /// Auto-create vehicle
    #[serde(default = "default_true")]
    pub auto_create: bool,
    /// Enable collision
    #[serde(default = "default_true")]
    pub enable_collision: bool,
    /// Speed units (mph or kph)
    #[serde(default)]
    pub speed_units: SpeedUnits,
    /// Sensors
    #[serde(default)]
    pub sensors: HashMap<String, SensorSettings>,
    /// Cameras
    #[serde(default)]
    pub cameras: HashMap<String, super::CameraSettings>,
}

fn default_car_name() -> String {
    "car_1".to_string()
}

fn default_car_type() -> String {
    "physxcar".to_string()
}

impl Default for CarSettings {
    fn default() -> Self {
        Self {
            vehicle_name: default_car_name(),
            vehicle_type: default_car_type(),
            default_vehicle: false,
            auto_create: true,
            enable_collision: true,
            speed_units: SpeedUnits::default(),
            sensors: HashMap::new(),
            cameras: HashMap::new(),
        }
    }
}

/// Speed units for car
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpeedUnits {
    /// Miles per hour
    #[default]
    Mph,
    /// Kilometers per hour
    Kph,
}

/// Vehicle settings wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleSettings {
    /// Multirotor settings
    Multirotor(MultirotorSettings),
    /// Car settings
    Car(CarSettings),
}

impl VehicleSettings {
    /// Get vehicle name
    pub fn vehicle_name(&self) -> &str {
        match self {
            Self::Multirotor(s) => &s.vehicle_name,
            Self::Car(s) => &s.vehicle_name,
        }
    }

    /// Get vehicle type
    pub fn vehicle_type(&self) -> &str {
        match self {
            Self::Multirotor(s) => &s.vehicle_type,
            Self::Car(s) => &s.vehicle_type,
        }
    }

    /// Check if auto-create is enabled
    pub fn auto_create(&self) -> bool {
        match self {
            Self::Multirotor(s) => s.auto_create,
            Self::Car(s) => s.auto_create,
        }
    }

    /// Get sensors
    pub fn sensors(&self) -> &HashMap<String, SensorSettings> {
        match self {
            Self::Multirotor(s) => &s.sensors,
            Self::Car(s) => &s.sensors,
        }
    }

    /// Get cameras
    pub fn cameras(&self) -> &HashMap<String, super::CameraSettings> {
        match self {
            Self::Multirotor(s) => &s.cameras,
            Self::Car(s) => &s.cameras,
        }
    }
}

/// Vehicle physics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePhysicsSettings {
    /// Mass (kg)
    #[serde(default = "default_mass")]
    pub mass: f32,
    /// Inertia tensor
    #[serde(default)]
    pub inertia: [f32; 3],
    /// Center of mass offset
    #[serde(default)]
    pub center_of_mass: [f32; 3],
    /// Drag coefficient
    #[serde(default)]
    pub drag_coefficient: f32,
}

fn default_mass() -> f32 {
    1.0
}

impl Default for VehiclePhysicsSettings {
    fn default() -> Self {
        Self {
            mass: default_mass(),
            inertia: [1.0, 1.0, 1.0],
            center_of_mass: [0.0, 0.0, 0.0],
            drag_coefficient: 0.0,
        }
    }
}

/// Remote control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcSettings {
    /// Remote control ID
    #[serde(default)]
    pub remote_control_id: u32,
    /// Allow API always
    #[serde(default)]
    pub allow_api_always: bool,
}

impl Default for RcSettings {
    fn default() -> Self {
        Self {
            remote_control_id: 0,
            allow_api_always: true,
        }
    }
}
