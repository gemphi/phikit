//! Vehicle types for rust-sim-kit
//!
//! Supports:
//! - Multirotor (drones)
//! - Cars (ground vehicles)

use crate::common::{GeoPoint, Pose, RcData};
use crate::physics::{Environment, KinematicsState};
use crate::sensors::SensorCollection;

pub mod car;
pub mod multirotor;

pub use car::CarVehicle;
pub use multirotor::MultirotorVehicle;

/// Common vehicle trait for multirotors and cars
pub trait Vehicle: Send + Sync {
    /// Update vehicle physics
    fn update(&mut self, dt: f32, environment: &Environment);

    /// Get vehicle name
    fn name(&self) -> &str;

    /// Get current pose
    fn pose(&self) -> &Pose;

    /// Set vehicle pose (for teleportation)
    fn set_pose(&mut self, pose: Pose);

    /// Get current kinematics
    fn kinematics(&self) -> &KinematicsState;

    /// Reset vehicle to initial state
    fn reset(&mut self);

    /// Get sensor collection
    fn sensors(&self) -> &SensorCollection;

    /// Get mutable sensor collection
    fn sensors_mut(&mut self) -> &mut SensorCollection;

    /// Arm/disarm the vehicle
    fn arm_disarm(&mut self, arm: bool);

    /// Check if vehicle is armed
    fn is_armed(&self) -> bool;

    /// Enable/disable API control
    fn enable_api_control(&mut self, enabled: bool);

    /// Check if API control is enabled
    fn is_api_control_enabled(&self) -> bool;

    /// Get home geo point
    fn home_geo_point(&self) -> &GeoPoint;

    /// Set home geo point
    fn set_home_geo_point(&mut self, home: GeoPoint);
}

/// Common vehicle configuration
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleConfig {
    /// Vehicle name
    pub name: String,
    /// Initial pose
    pub initial_pose: Pose,
    /// Home geo point
    pub home_geo_point: GeoPoint,
    /// Mass (kg)
    pub mass: f32,
    /// Maximum velocity (m/s)
    pub max_velocity: f32,
    /// Enable API control by default
    pub api_control_enabled: bool,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            name: String::from("vehicle"),
            initial_pose: Pose::identity(),
            home_geo_point: GeoPoint::new(0.0, 0.0, 0.0),
            mass: 1.0,
            max_velocity: 10.0,
            api_control_enabled: false,
        }
    }
}

/// Common vehicle implementation
pub struct VehicleBase {
    config: VehicleConfig,
    kinematics: KinematicsState,
    sensors: SensorCollection,
    armed: bool,
    api_control_enabled: bool,
    rc_data: RcData,
}

impl std::fmt::Debug for VehicleBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VehicleBase")
            .field("config", &self.config)
            .field("kinematics", &self.kinematics)
            .field("armed", &self.armed)
            .field("api_control_enabled", &self.api_control_enabled)
            .field("rc_data", &self.rc_data)
            .finish_non_exhaustive()
    }
}

impl VehicleBase {
    /// Create new vehicle base
    pub fn new(config: VehicleConfig) -> Self {
        let kinematics = KinematicsState {
            pose: config.initial_pose,
            ..Default::default()
        };

        Self {
            config,
            kinematics,
            sensors: SensorCollection::new(),
            armed: false,
            api_control_enabled: false,
            rc_data: RcData::default(),
        }
    }

    /// Get config
    pub fn config(&self) -> &VehicleConfig {
        &self.config
    }

    /// Get mutable config
    pub fn config_mut(&mut self) -> &mut VehicleConfig {
        &mut self.config
    }

    /// Get kinematics
    pub fn kinematics(&self) -> &KinematicsState {
        &self.kinematics
    }

    /// Get mutable kinematics
    pub fn kinematics_mut(&mut self) -> &mut KinematicsState {
        &mut self.kinematics
    }

    /// Get sensors
    pub fn sensors(&self) -> &SensorCollection {
        &self.sensors
    }

    /// Get mutable sensors
    pub fn sensors_mut(&mut self) -> &mut SensorCollection {
        &mut self.sensors
    }

    /// Get RC data
    pub fn rc_data(&self) -> &RcData {
        &self.rc_data
    }

    /// Set RC data
    pub fn set_rc_data(&mut self, rc_data: RcData) {
        self.rc_data = rc_data;
    }

    /// Check if armed
    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// Set armed state
    pub fn set_armed(&mut self, armed: bool) {
        self.armed = armed;
    }

    /// Check if API control enabled
    pub fn is_api_control_enabled(&self) -> bool {
        self.api_control_enabled
    }

    /// Set API control enabled
    pub fn set_api_control_enabled(&mut self, enabled: bool) {
        self.api_control_enabled = enabled;
    }

    /// Update sensors
    pub fn update_sensors(&mut self, dt: f32, environment: &Environment) {
        self.sensors.update(dt);

        // Re-initialize sensors with current kinematics
        self.sensors
            .initialize(&self.kinematics, environment.state());
    }

    /// Get camera sensor by name
    pub fn get_camera(&self, name: &str) -> Option<&crate::sensors::CameraSensor> {
        self.sensors.all().iter().find_map(|s| {
            if s.name() == name && s.sensor_type() == crate::sensors::SensorType::Camera {
                // SAFETY: We just checked the type, this is safe
                unsafe {
                    Some(
                        &*(s.as_ref() as *const dyn crate::sensors::Sensor
                            as *const crate::sensors::CameraSensor),
                    )
                }
            } else {
                None
            }
        })
    }

    /// Get mutable camera sensor by name
    pub fn get_camera_mut(&mut self, name: &str) -> Option<&mut crate::sensors::CameraSensor> {
        self.sensors.all_as_mut_slice().iter_mut().find_map(|s| {
            if s.name() == name && s.sensor_type() == crate::sensors::SensorType::Camera {
                // SAFETY: We just checked the type, this is safe
                unsafe {
                    Some(
                        &mut *(s.as_mut() as *mut dyn crate::sensors::Sensor
                            as *mut crate::sensors::CameraSensor),
                    )
                }
            } else {
                None
            }
        })
    }

    /// Add a camera to this vehicle
    pub fn add_camera(&mut self, camera: crate::sensors::CameraSensor) {
        self.sensors.add(Box::new(camera));
    }

    /// Get all camera names
    pub fn list_cameras(&self) -> Vec<String> {
        self.sensors
            .all()
            .iter()
            .filter(|s| s.sensor_type() == crate::sensors::SensorType::Camera)
            .map(|s| s.name().to_string())
            .collect()
    }
}
