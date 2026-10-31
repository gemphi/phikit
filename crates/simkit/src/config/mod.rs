//! Configuration Module
//!
//! Handles simulation settings and configuration loading.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod sensor_settings;
pub mod vehicle_settings;

pub use sensor_settings::{
    CameraSettings, GpsSettings, ImuSettings, LidarSettings, SensorSettings,
};
pub use vehicle_settings::{CarSettings, MultirotorSettings, VehicleSettings};

/// Main simulation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSettings {
    /// Simulation settings
    #[serde(default)]
    pub sim_mode: SimMode,
    /// Physics engine settings
    #[serde(default)]
    pub physics_engine: PhysicsEngineSettings,
    /// Vehicle settings
    #[serde(default)]
    pub vehicles: HashMap<String, SimulationVehicleConfig>,
    /// Camera defaults
    #[serde(default)]
    pub camera_defaults: CameraDefaults,
    /// Segmentation settings
    #[serde(default)]
    pub segmentation_settings: SegmentationSettings,
    /// Sub-windows for camera views
    #[serde(default)]
    pub subwindows: Vec<SubWindow>,
    /// Recording settings
    #[serde(default)]
    pub recording: RecordingSettings,
    /// Clock speed
    #[serde(default = "default_clock_speed")]
    pub clock_speed: f32,
    /// Scene settings
    #[serde(default)]
    pub scene: SceneSettings,
    /// View mode
    #[serde(default)]
    pub view_mode: ViewMode,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            sim_mode: SimMode::default(),
            physics_engine: PhysicsEngineSettings::default(),
            vehicles: HashMap::new(),
            camera_defaults: CameraDefaults::default(),
            segmentation_settings: SegmentationSettings::default(),
            subwindows: Vec::new(),
            recording: RecordingSettings::default(),
            clock_speed: default_clock_speed(),
            scene: SceneSettings::default(),
            view_mode: ViewMode::default(),
        }
    }
}

fn default_clock_speed() -> f32 {
    1.0
}

impl SimSettings {
    /// Load settings from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::SimKitError::Configuration(format!("Failed to read settings file: {}", e))
        })?;

        let settings: SimSettings = serde_json::from_str(&content).map_err(|e| {
            crate::SimKitError::Configuration(format!("Failed to parse settings: {}", e))
        })?;

        Ok(settings)
    }

    /// Save settings to JSON file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::SimKitError::Configuration(format!("Failed to serialize settings: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            crate::SimKitError::Configuration(format!("Failed to write settings file: {}", e))
        })?;

        Ok(())
    }

    /// Add a vehicle configuration
    pub fn add_vehicle(&mut self, name: &str, config: SimulationVehicleConfig) {
        self.vehicles.insert(name.to_string(), config);
    }

    /// Get vehicle configuration
    pub fn get_vehicle(&self, name: &str) -> Option<&SimulationVehicleConfig> {
        self.vehicles.get(name)
    }
}

/// Simulation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimMode {
    /// Multirotor simulation
    #[default]
    Multirotor,
    /// Car simulation
    Car,
    /// Computer vision mode
    ComputerVision,
}

/// Physics engine settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsEngineSettings {
    /// Physics engine name
    #[serde(default = "default_physics_engine")]
    pub engine_name: String,
    /// Physics update rate (Hz)
    #[serde(default = "default_physics_fps")]
    pub physics_fps: f32,
    /// Enable collision detection
    #[serde(default = "default_true")]
    pub enable_collision: bool,
    /// Enable substepping
    #[serde(default = "default_true")]
    pub enable_substepping: bool,
    /// Max substep delta time
    #[serde(default = "default_max_substep")]
    pub max_substep_delta_time: f32,
    /// Max substeps
    #[serde(default = "default_max_substeps")]
    pub max_substeps: u32,
}

fn default_physics_engine() -> String {
    "FastPhysicsEngine".to_string()
}

fn default_physics_fps() -> f32 {
    120.0
}

fn default_max_substep() -> f32 {
    0.01
}

fn default_max_substeps() -> u32 {
    10
}

fn default_true() -> bool {
    true
}

impl Default for PhysicsEngineSettings {
    fn default() -> Self {
        Self {
            engine_name: default_physics_engine(),
            physics_fps: default_physics_fps(),
            enable_collision: true,
            enable_substepping: true,
            max_substep_delta_time: default_max_substep(),
            max_substeps: default_max_substeps(),
        }
    }
}

/// Vehicle configuration wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SimulationVehicleConfig {
    /// Multirotor configuration
    #[serde(rename = "simpleflight")]
    Multirotor(MultirotorSettings),
    /// Car configuration
    #[serde(rename = "physxcar")]
    Car(CarSettings),
    /// Computer vision mode
    #[serde(rename = "computervision")]
    ComputerVision,
}

/// Camera default settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraDefaults {
    /// Capture settings
    #[serde(default)]
    pub capture_settings: CaptureSettings,
    /// Gimbal settings
    #[serde(default)]
    pub gimbal: GimbalSettings,
}

impl Default for CameraDefaults {
    fn default() -> Self {
        Self {
            capture_settings: CaptureSettings::default(),
            gimbal: GimbalSettings::default(),
        }
    }
}

/// Capture settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    /// Image width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Image height
    #[serde(default = "default_height")]
    pub height: u32,
    /// Field of view (degrees)
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
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

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            fov_degrees: default_fov(),
        }
    }
}

/// Gimbal settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GimbalSettings {
    /// Stabilization enabled
    #[serde(default)]
    pub stabilization: bool,
    /// Pitch angle
    #[serde(default)]
    pub pitch: f32,
    /// Roll angle
    #[serde(default)]
    pub roll: f32,
    /// Yaw angle
    #[serde(default)]
    pub yaw: f32,
}

/// Segmentation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationSettings {
    /// Initialize with existing preset values
    #[serde(default)]
    pub init_with_existing: bool,
    /// Override existing values
    #[serde(default)]
    pub override_existing: bool,
}

impl Default for SegmentationSettings {
    fn default() -> Self {
        Self {
            init_with_existing: true,
            override_existing: false,
        }
    }
}

/// Sub-window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubWindow {
    /// Window ID
    #[serde(default)]
    pub window_id: u32,
    /// Camera name
    #[serde(default)]
    pub camera_name: String,
    /// Image type
    #[serde(default)]
    pub image_type: u8,
    /// Vehicle name
    #[serde(default)]
    pub vehicle_name: String,
    /// Visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// X position
    #[serde(default)]
    pub x: u32,
    /// Y position
    #[serde(default)]
    pub y: u32,
    /// Width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Height
    #[serde(default = "default_height")]
    pub height: u32,
}

/// Recording settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    /// Record on collision
    #[serde(default)]
    pub record_on_collision: bool,
    /// Record interval (seconds)
    #[serde(default = "default_record_interval")]
    pub record_interval: f32,
    /// Folder path for recordings
    #[serde(default)]
    pub folder: String,
    /// Enabled
    #[serde(default)]
    pub enabled: bool,
    /// Compression format
    #[serde(default)]
    pub compression: String,
}

fn default_record_interval() -> f32 {
    0.05
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            record_on_collision: false,
            record_interval: default_record_interval(),
            folder: String::new(),
            enabled: false,
            compression: String::new(),
        }
    }
}

/// Scene settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSettings {
    /// Default vehicle name
    #[serde(default)]
    pub default_vehicle: String,
    /// Player start location
    #[serde(default)]
    pub player_start_location: [f32; 3],
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            default_vehicle: String::new(),
            player_start_location: [0.0, 0.0, 0.0],
        }
    }
}

/// View mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    /// FPV (First Person View)
    #[default]
    Fpv,
    /// Ground observer
    GroundObserver,
    /// Fly with me
    FlyWithMe,
    /// Manual
    Manual,
    /// Spring arm chase
    SpringArmChase,
    /// Back up camera
    BackupCamera,
    /// Front camera
    FrontCamera,
    /// GPS camera
    GpsCamera,
    /// No display
    NoDisplay,
}

/// Settings loader utility
pub struct SettingsLoader {
    search_paths: Vec<PathBuf>,
}

impl SettingsLoader {
    /// Create new loader with default paths
    pub fn new() -> Self {
        let mut paths = vec![
            PathBuf::from("./settings.json"),
            PathBuf::from("./config/settings.json"),
        ];

        // Add user's home directory
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".airsim/settings.json"));
        }

        Self {
            search_paths: paths,
        }
    }

    /// Add a search path
    pub fn add_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Find and load settings
    pub fn load(&self) -> crate::Result<SimSettings> {
        // Try each path in order
        for path in &self.search_paths {
            if path.exists() {
                log::info!("Loading settings from: {:?}", path);
                return SimSettings::from_file(path);
            }
        }

        // Return default settings if no file found
        log::info!("No settings file found, using defaults");
        Ok(SimSettings::default())
    }
}

impl Default for SettingsLoader {
    fn default() -> Self {
        Self::new()
    }
}
