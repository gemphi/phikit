//! Dispatch and parsing helpers for Blender native export.

use super::config::SceneExportConfig;
use super::types::{BlenderLightType, BlenderObjectType, BlenderSceneFormat};
use crate::phx::{PointCloud, PointCloudError, Result};

/// Dispatch scene export by format
pub fn generate_scene_export_script(_config: &SceneExportConfig) -> Result<String> {
    // This now returns empty — callers should use export_scene_to_phx/gltf/usda directly
    Ok(String::new())
}

/// Export point cloud using native Rust format writers
pub fn generate_point_cloud_script(
    _cloud: &PointCloud,
    _format: BlenderSceneFormat,
) -> Result<String> {
    Err(PointCloudError::ExportError(
        "Blender script generation is deprecated. Use native exporters directly.".to_string(),
    ))
}

/// Convert Blender object type string to enum
pub fn parse_object_type(type_str: &str) -> BlenderObjectType {
    match type_str {
        "MESH" => BlenderObjectType::Mesh,
        "LIGHT" => BlenderObjectType::Light,
        "CAMERA" => BlenderObjectType::Camera,
        "EMPTY" => BlenderObjectType::Empty,
        "CURVE" => BlenderObjectType::Curve,
        "SURFACE" => BlenderObjectType::Surface,
        "FONT" => BlenderObjectType::Text,
        "ARMATURE" => BlenderObjectType::Armature,
        "LATTICE" => BlenderObjectType::Lattice,
        _ => BlenderObjectType::Empty,
    }
}

/// Convert Blender light type string to enum
pub fn parse_light_type(type_str: &str) -> BlenderLightType {
    match type_str {
        "POINT" => BlenderLightType::Point,
        "SUN" => BlenderLightType::Sun,
        "SPOT" => BlenderLightType::Spot,
        "AREA" => BlenderLightType::Area,
        _ => BlenderLightType::Point,
    }
}
