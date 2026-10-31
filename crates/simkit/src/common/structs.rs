//! Additional common structs (placeholder for extensions)

use crate::common::Pose;

/// Image type enumeration (matches AirSim)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
pub enum ImageType {
    /// RGB scene image
    Scene = 0,
    /// Depth in planar projection
    DepthPlanar = 1,
    /// Depth in perspective projection
    DepthPerspective = 2,
    /// Depth visualization
    DepthVis = 3,
    /// Disparity normalized
    DisparityNormalized = 4,
    /// Segmentation image
    Segmentation = 5,
    /// Surface normals
    SurfaceNormals = 6,
    /// Infrared image
    Infrared = 7,
    /// Optical flow
    OpticalFlow = 8,
    /// Optical flow visualization
    OpticalFlowVis = 9,
}

impl Default for ImageType {
    fn default() -> Self {
        ImageType::Scene
    }
}

/// Image capture request
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageRequest {
    /// Camera name
    pub camera_name: String,
    /// Image type to capture
    pub image_type: ImageType,
    /// Return pixels as float (for depth)
    pub pixels_as_float: bool,
    /// Compress the image (PNG/JPEG)
    pub compress: bool,
}

impl ImageRequest {
    /// Create new image request
    pub fn new(camera_name: impl Into<String>, image_type: ImageType) -> Self {
        Self {
            camera_name: camera_name.into(),
            image_type,
            pixels_as_float: false,
            compress: true,
        }
    }

    /// Create depth image request
    pub fn depth(camera_name: impl Into<String>) -> Self {
        Self {
            camera_name: camera_name.into(),
            image_type: ImageType::DepthPlanar,
            pixels_as_float: true,
            compress: false,
        }
    }

    /// Create segmentation request
    pub fn segmentation(camera_name: impl Into<String>) -> Self {
        Self {
            camera_name: camera_name.into(),
            image_type: ImageType::Segmentation,
            pixels_as_float: false,
            compress: false,
        }
    }
}

impl Default for ImageRequest {
    fn default() -> Self {
        Self {
            camera_name: String::new(),
            image_type: ImageType::Scene,
            pixels_as_float: false,
            compress: true,
        }
    }
}

/// Image capture response
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageResponse {
    /// Image data as bytes (compressed or raw)
    pub image_data_uint8: Vec<u8>,
    /// Image data as floats (for depth)
    pub image_data_float: Vec<f32>,
    /// Camera name
    pub camera_name: String,
    /// Camera position
    pub camera_position: glam::Vec3,
    /// Camera orientation
    pub camera_orientation: glam::Quat,
    /// Capture timestamp
    pub time_stamp: u64,
    /// Status message
    pub message: String,
    /// Whether pixels are float format
    pub pixels_as_float: bool,
    /// Whether image is compressed
    pub compress: bool,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Image type captured
    pub image_type: ImageType,
}

impl Default for ImageResponse {
    fn default() -> Self {
        Self {
            image_data_uint8: Vec::new(),
            image_data_float: Vec::new(),
            camera_name: String::new(),
            camera_position: glam::Vec3::ZERO,
            camera_orientation: glam::Quat::IDENTITY,
            time_stamp: 0,
            message: String::new(),
            pixels_as_float: false,
            compress: false,
            width: 0,
            height: 0,
            image_type: ImageType::Scene,
        }
    }
}

/// Camera information
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CameraInfo {
    /// Camera pose relative to vehicle
    pub pose: Pose,
    /// Field of view in degrees
    pub fov: f32,
    /// Projection matrix (4x4)
    pub proj_mat: [[f32; 4]; 4],
}

impl Default for CameraInfo {
    fn default() -> Self {
        Self {
            pose: Pose::default(),
            fov: 90.0,
            proj_mat: [[0.0; 4]; 4],
        }
    }
}

// This file is reserved for additional common structs that don't fit
// in the main common module.
