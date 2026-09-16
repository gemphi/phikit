//! Core data types for Blender native scene export.
//!
//! Defines the format/object/light/camera enums and the scene data
//! structures (meshes, lights, camera, materials, instances, collections).

/// Blender scene export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlenderSceneFormat {
    /// PHX Scene format (extended PHX with scene metadata)
    PhxScene,
    /// glTF 2.0 with instancing and PHX extensions
    Gltf,
    /// Binary glTF
    Glb,
    /// USD ASCII with PointInstancer
    Usda,
    /// USD binary (via usdc conversion)
    Usdc,
}

/// Blender object type mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlenderObjectType {
    Mesh,
    Light,
    Camera,
    Empty,
    Curve,
    Surface,
    Text,
    Armature,
    Lattice,
    EmptyInstance, // Collection instance
}

/// Light type mapping from Blender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlenderLightType {
    Point,
    Sun,
    Spot,
    Area,
}

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    Perspective,
    Orthographic,
    Panoramic,
}

/// Instance data for object duplication
#[derive(Debug, Clone)]
pub struct InstanceData {
    /// Prototype object name being instanced
    pub prototype_name: String,
    /// World transform matrices for each instance
    pub transforms: Vec<[f32; 16]>,
    /// Instance scales (optional, for non-uniform scaling)
    pub scales: Option<Vec<[f32; 3]>>,
}

/// Complete Blender scene export data
#[derive(Debug, Clone)]
pub struct BlenderScene {
    /// Scene name
    pub name: String,
    /// Frame number (for animated exports)
    pub frame: i32,
    /// Render resolution
    pub resolution: [i32; 2],
    /// Camera data (if present)
    pub camera: Option<CameraData>,
    /// Light sources
    pub lights: Vec<LightData>,
    /// Mesh objects with geometry
    pub meshes: Vec<MeshObjectData>,
    /// Empty/collection instances
    pub instances: Vec<InstanceData>,
    /// Materials referenced by meshes
    pub materials: Vec<MaterialData>,
    /// Scene collections hierarchy
    pub collections: Vec<CollectionData>,
}

/// Camera export data
#[derive(Debug, Clone)]
pub struct CameraData {
    pub name: String,
    pub location: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub camera_type: CameraType,
    pub matrix_world: [[f32; 4]; 4],
}

/// Light source export data
#[derive(Debug, Clone)]
pub struct LightData {
    pub name: String,
    pub light_type: BlenderLightType,
    pub color: [f32; 3],
    pub energy: f32,
    pub location: [f32; 3],
    pub direction: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    /// For spot lights
    pub spot_angle: Option<f32>,
    pub spot_blend: Option<f32>,
    /// For area lights
    pub area_size: Option<[f32; 2]>,
}

/// Mesh object with geometry
#[derive(Debug, Clone)]
pub struct MeshObjectData {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<[u32; 3]>,
    pub material_names: Vec<String>,
    pub matrix_world: [[f32; 4]; 4],
    pub visible: bool,
}

/// PBR material data
#[derive(Debug, Clone)]
pub struct MaterialData {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub emission_strength: f32,
    pub alpha: f32,
    pub use_nodes: bool,
    /// Texture paths (if any)
    pub base_color_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub roughness_texture: Option<String>,
}

/// Collection hierarchy data
#[derive(Debug, Clone)]
pub struct CollectionData {
    pub name: String,
    pub object_names: Vec<String>,
    pub child_collection_names: Vec<String>,
    pub instance_offset: [f32; 3],
    pub is_instance: bool,
}
