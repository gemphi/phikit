// SPDX-License-Identifier: Apache-2.0
//! What `intern/cycles/blender/` reads from Blender through RNA, as plain
//! data (`BlenderCamera` of `blender/camera.cpp`, the arguments of
//! `BlenderSync::sync_mesh`, `sync_object` and `sync_background`): the
//! one shape the sync takes, whoever read the file. `phiphx.rs` fills it
//! from phiphx's `SceneModel`; nothing else here knows where it came from.

/// `BlenderCamera::sensor_fit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFit {
    Auto,
    Horizontal,
    Vertical,
}

/// `BlenderCamera`, the members the perspective and orthographic cameras
/// use.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraSource {
    pub name: String,
    /// The camera object's world matrix, Blender's `float[4][4]`:
    /// `matrix[column][row]`, the translation in `matrix[3]`.
    pub matrix: [[f32; 4]; 4],
    pub orthographic: bool,
    /// `lens`, in millimetres.
    pub lens: f32,
    pub sensor_width: f32,
    pub sensor_height: f32,
    pub sensor_fit: SensorFit,
    /// `shift`, in fractions of the fitted side.
    pub shift: [f32; 2],
    pub ortho_scale: f32,
    pub clip_start: f32,
    pub clip_end: f32,
}

/// The surface a material's node graph makes, as far as gate 1 shades it
/// (`blender/shader.cpp` builds a `ShaderGraph` from the nodes).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NodeSurface {
    /// No graph, or one that is not a lone closure: `base_color` and
    /// `emission_*` stand in (the Principled BSDF's).
    #[default]
    Unknown,
    /// `ShaderNodeBsdfDiffuse` with its `Color` and `Roughness` unlinked.
    Diffuse { color: [f32; 3], roughness: f32 },
    /// `ShaderNodeEmission` with its `Color` and `Strength` unlinked.
    Emission { color: [f32; 3], strength: f32 },
}

/// A material, as far as gate 1 shades it.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialSource {
    pub name: String,
    pub base_color: [f32; 3],
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
    /// `Material.diffuse_color`: the colour of a material that has no node tree,
    /// which Cycles shades as a diffuse BSDF of it (`blender/shader.cpp`,
    /// the branch for a material that does not use nodes).
    pub diffuse_color: [f32; 3],
    /// What the graph's Material Output takes its surface from.
    pub node: NodeSurface,
    /// The idname of every node of the graph, for knowing what a scene uses.
    pub node_kind: Vec<String>,
}

/// What `BlenderSync::sync_mesh` reads of a mesh: triangulated, as
/// `blender/mesh.cpp` hands the faces on.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshSource {
    pub vertex: Vec<[f32; 3]>,
    pub triangle: Vec<[u32; 3]>,
    /// For each triangle, the material slot of its face.
    pub slot: Vec<u16>,
    /// Whether each triangle is shaded smooth.
    pub smooth: Vec<bool>,
    /// One UV for each vertex, or none: the active UV map.
    pub uv: Vec<[f32; 2]>,
    /// The material slots (mesh.materials): a Cycles geometry owns its
    /// shaders, so two objects with different materials are two meshes.
    pub material: Vec<MaterialSource>,
}

/// `BlenderSync::sync_object`: a mesh placed in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSource {
    pub name: String,
    /// Its place in `SceneSource::mesh`; objects that share a mesh share it.
    pub mesh: usize,
    /// World matrix, `matrix[column][row]`.
    pub matrix: [[f32; 4]; 4],
}

/// A whole scene as the sync reads it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneSource {
    /// The rendered size in pixels (`resolution * percentage`).
    pub width: i32,
    pub height: i32,
    pub camera: Option<CameraSource>,
    pub mesh: Vec<MeshSource>,
    pub object: Vec<ObjectSource>,
    /// The world's background colour and strength.
    pub world_color: [f32; 3],
    pub world_strength: f32,
    /// The environment image that feeds the world's colour, if one does.
    pub world_texture: Option<String>,
    /// How many lights the scene has.
    pub light_count: usize,
    /// `scene["cycles"]`: the numbers the file holds (a flag is 0 or 1) by
    /// name. A name that is not there has its default (`properties.py`).
    pub cycles: std::collections::BTreeMap<String, f64>,
    /// Colour management, `Scene.view_settings`.
    pub view_transform: String,
    pub look: String,
    pub exposure: f32,
    pub gamma: f32,
    /// Film > Transparent.
    pub film_transparent: bool,
}
