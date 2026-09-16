//! Export configuration types for Blender native scene export.

use super::types::BlenderSceneFormat;

/// Point cloud export configuration
#[derive(Debug, Clone, Copy)]
pub struct PointCloudConfig {
    /// Points per unit area
    pub density: f32,
    /// Minimum points per object
    pub min_points: usize,
    /// Maximum points per object
    pub max_points: usize,
    /// Use world space coordinates
    pub use_world_space: bool,
    /// Random seed (0 for random)
    pub seed: u64,
    /// Include normals
    pub include_normals: bool,
    /// Include colors
    pub include_colors: bool,
}

impl Default for PointCloudConfig {
    fn default() -> Self {
        Self {
            density: 100.0,
            min_points: 100,
            max_points: 1_000_000,
            use_world_space: true,
            seed: 0,
            include_normals: true,
            include_colors: true,
        }
    }
}

/// Scene export configuration
#[derive(Debug, Clone)]
pub struct SceneExportConfig {
    /// Export format
    pub format: BlenderSceneFormat,
    /// Point cloud generation config (for point cloud exports)
    pub point_cloud: PointCloudConfig,
    /// Include mesh geometry
    pub include_meshes: bool,
    /// Include lights
    pub include_lights: bool,
    /// Include camera
    pub include_camera: bool,
    /// Include materials
    pub include_materials: bool,
    /// Include instances/duplis
    pub include_instances: bool,
    /// Apply modifiers before export
    pub apply_modifiers: bool,
    /// Selected objects only (vs scene-wide)
    pub selected_only: bool,
    /// Specific collections to export (empty = all)
    pub collections: Vec<String>,
}

impl Default for SceneExportConfig {
    fn default() -> Self {
        Self {
            format: BlenderSceneFormat::PhxScene,
            point_cloud: PointCloudConfig::default(),
            include_meshes: true,
            include_lights: true,
            include_camera: true,
            include_materials: true,
            include_instances: true,
            apply_modifiers: true,
            selected_only: false,
            collections: Vec::new(),
        }
    }
}
