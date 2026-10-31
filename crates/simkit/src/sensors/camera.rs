//! Camera sensor for image capture
//!
//! Provides RGB, depth, segmentation, and other image types
//! for computer vision simulation.

use crate::common::{CameraInfo, ImageRequest, ImageResponse, ImageType, Pose};
use crate::physics::{EnvironmentState, KinematicsState};
use crate::sensors::{Sensor, SensorBase, SensorConfig, SensorType};

/// Camera sensor configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CameraConfig {
    /// Base sensor configuration
    pub base: SensorConfig,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Field of view in degrees (horizontal)
    pub fov_degrees: f32,
    /// Near clip plane
    pub near_clip: f32,
    /// Far clip plane
    pub far_clip: f32,
    /// Enable auto exposure
    pub auto_exposure: bool,
    /// Motion blur amount (0 = disabled)
    pub motion_blur_amount: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            base: SensorConfig {
                name: "camera".to_string(),
                sensor_type: SensorType::Camera,
                update_frequency: 30.0, // 30 FPS default
                enabled: true,
            },
            width: 640,
            height: 480,
            fov_degrees: 90.0,
            near_clip: 0.1,
            far_clip: 1000.0,
            auto_exposure: true,
            motion_blur_amount: 0.0,
        }
    }
}

impl CameraConfig {
    /// Create camera config with specified resolution
    pub fn with_resolution(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Create camera config with FOV
    pub fn with_fov(mut self, fov_degrees: f32) -> Self {
        self.fov_degrees = fov_degrees;
        self
    }
}

/// Camera sensor for image capture
#[derive(Debug)]
pub struct CameraSensor {
    base: SensorBase,
    config: CameraConfig,
    info: CameraInfo,
    last_request: Option<ImageRequest>,
    last_response: Option<ImageResponse>,
}

impl CameraSensor {
    /// Create new camera sensor
    pub fn new(config: CameraConfig) -> Self {
        let info = CameraInfo {
            pose: Pose::identity(),
            fov: config.fov_degrees,
            proj_mat: Self::compute_projection_matrix(&config),
        };

        Self {
            base: SensorBase::from_config(&config.base),
            config,
            info,
            last_request: None,
            last_response: None,
        }
    }

    /// Compute projection matrix from config
    fn compute_projection_matrix(config: &CameraConfig) -> [[f32; 4]; 4] {
        let aspect = config.width as f32 / config.height as f32;
        let fov_rad = config.fov_degrees.to_radians();
        let f = 1.0 / (fov_rad / 2.0).tan();

        let nf = config.near_clip - config.far_clip;

        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (config.far_clip + config.near_clip) / nf, -1.0],
            [0.0, 0.0, 2.0 * config.far_clip * config.near_clip / nf, 0.0],
        ]
    }

    /// Get camera configuration
    pub fn config(&self) -> &CameraConfig {
        &self.config
    }

    /// Get mutable config
    pub fn config_mut(&mut self) -> &mut CameraConfig {
        &mut self.config
    }

    /// Get camera info
    pub fn info(&self) -> &CameraInfo {
        &self.info
    }

    /// Capture image (synchronous stub - would integrate with renderer)
    pub fn capture_image(&mut self, request: ImageRequest) -> ImageResponse {
        let timestamp = self.base.last_timestamp();

        // Stub: generate placeholder image data
        let image_data = match request.image_type {
            ImageType::Scene => vec![0u8; (self.config.width * self.config.height * 3) as usize],
            ImageType::DepthPlanar | ImageType::DepthPerspective => {
                vec![0u8; (self.config.width * self.config.height * 4) as usize]
            }
            ImageType::Segmentation => vec![0u8; (self.config.width * self.config.height) as usize],
            ImageType::SurfaceNormals => {
                vec![0u8; (self.config.width * self.config.height * 3) as usize]
            }
            _ => vec![0u8; (self.config.width * self.config.height * 3) as usize],
        };

        let response = ImageResponse {
            image_data_uint8: if !request.pixels_as_float {
                image_data
            } else {
                vec![]
            },
            image_data_float: if request.pixels_as_float {
                vec![0.0; (self.config.width * self.config.height) as usize]
            } else {
                vec![]
            },
            camera_name: request.camera_name.clone(),
            camera_position: self.base.relative_pose().position,
            camera_orientation: self.base.relative_pose().orientation,
            time_stamp: timestamp,
            message: "Stub image capture".to_string(),
            pixels_as_float: request.pixels_as_float,
            compress: request.compress,
            width: self.config.width,
            height: self.config.height,
            image_type: request.image_type,
        };

        self.last_request = Some(request);
        self.last_response = Some(response.clone());

        response
    }

    /// Get last captured response
    pub fn last_response(&self) -> Option<&ImageResponse> {
        self.last_response.as_ref()
    }

    /// Set camera pose relative to vehicle
    pub fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
        self.info.pose = pose;
    }

    /// Update projection matrix when params change
    pub fn update_projection(&mut self) {
        self.info.proj_mat = Self::compute_projection_matrix(&self.config);
        self.info.fov = self.config.fov_degrees;
    }
}

impl Sensor for CameraSensor {
    fn sensor_type(&self) -> SensorType {
        self.base.sensor_type()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn initialize(&mut self, kinematics: &KinematicsState, _environment: &EnvironmentState) {
        self.base.initialize(kinematics, _environment);
    }

    fn update(&mut self, dt: f32) {
        self.base.update(dt);
        // Camera update would trigger render here
    }

    fn reset(&mut self) {
        self.base.reset();
        self.last_request = None;
        self.last_response = None;
    }

    fn enabled(&self) -> bool {
        self.base.enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.base.set_enabled(enabled);
    }

    fn relative_pose(&self) -> &Pose {
        self.base.relative_pose()
    }

    fn set_relative_pose(&mut self, pose: Pose) {
        self.base.set_relative_pose(pose);
        self.info.pose = pose;
    }
}

/// Camera capture request queue
#[derive(Debug, Default)]
pub struct CameraCaptureQueue {
    requests: Vec<ImageRequest>,
}

impl CameraCaptureQueue {
    /// Create empty queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Add request to queue
    pub fn add(&mut self, request: ImageRequest) {
        self.requests.push(request);
    }

    /// Get all requests and clear queue
    pub fn take_requests(&mut self) -> Vec<ImageRequest> {
        std::mem::take(&mut self.requests)
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Clear all requests
    pub fn clear(&mut self) {
        self.requests.clear();
    }
}
