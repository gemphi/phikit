//! PHX Scene data types and format constants.

use serde::{Deserialize, Serialize};

/// PHX Scene file magic number
pub const PHX_SCENE_MAGIC: &[u8; 10] = b"PHX_SCENE\x01";

/// PHX Scene format version
pub const PHX_SCENE_VERSION: u32 = 1;

/// Complete PHX scene data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxScene {
    /// Scene name
    pub name: String,
    /// Frame number (for animation)
    pub frame: i32,
    /// Render resolution [width, height]
    pub resolution: [i32; 2],
    /// Format identifier
    pub format: String,
    /// Camera data
    pub camera: Option<PhxCamera>,
    /// Light sources
    pub lights: Vec<PhxLight>,
    /// Mesh objects
    pub meshes: Vec<PhxMesh>,
    /// Object instances
    pub instances: Vec<PhxInstance>,
    /// Materials
    pub materials: Vec<PhxMaterial>,
    /// Collections/hierarchy
    pub collections: Vec<PhxCollection>,
}

/// Camera data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxCamera {
    pub name: String,
    pub camera_type: String,
    pub location: [f32; 3],
    pub rotation: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    pub fov: f32,
    pub clip_start: f32,
    pub clip_end: f32,
}

/// Light source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxLight {
    pub name: String,
    pub light_type: String,
    pub color: [f32; 3],
    pub energy: f32,
    pub location: [f32; 3],
    pub direction: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot_angle: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot_blend: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_size: Option<[f32; 2]>,
}

/// Mesh object data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxMesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<[u32; 3]>,
    pub materials: Vec<String>,
    pub matrix_world: [[f32; 4]; 4],
    pub visible: bool,
}

/// Instance data for object duplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxInstance {
    pub name: String,
    pub instance_type: String,
    pub prototype: String,
    pub matrix_world: [[f32; 4]; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<[[f32; 4]; 4]>>,
}

/// PBR material data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub emission_strength: f32,
    pub alpha: f32,
    pub use_nodes: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness_texture: Option<String>,
}

/// Collection/hierarchy data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhxCollection {
    pub name: String,
    pub objects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    pub instance_offset: [f32; 3],
    #[serde(default)]
    pub is_instance: bool,
}
