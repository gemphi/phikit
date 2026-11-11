// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
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

/// A material, as far as gate 1 shades it.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialSource {
    pub name: String,
    pub base_color: [f32; 3],
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
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
}
