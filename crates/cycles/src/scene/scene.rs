// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/scene.h` (`Scene`): what a render is
//! asked to draw, the lists the managers keep. So far the camera, the
//! shaders, the meshes and the objects; the lights, the film, the
//! integrator and the managers' state join as they are needed.
use super::{Background, Camera, Film, Integrator, LookupTableSet, Mesh, Object, SceneParam, Shader};

/// A surface in the one shape the kernels will have before the node graph
/// is compiled (gate 2): `scene/shader.cpp` keeps a `ShaderGraph`, and
/// `blender/shader.cpp` builds it from the material's nodes. Until the
/// graph is ported a material is a diffuse BSDF or an emission, which is
/// all gate 1 shades.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Surface {
    /// A `DiffuseBsdf` of this colour.
    Diffuse { color: [f32; 3] },
    /// An `EmissionNode` of this colour and strength.
    Emission { color: [f32; 3], strength: f32 },
    /// A `BackgroundNode`, the world's: colour and strength.
    Background { color: [f32; 3], strength: f32 },
}

impl Default for Surface {
    /// `Shader::default_surface`'s stand-in: Blender's grey diffuse.
    fn default() -> Self {
        Surface::Diffuse { color: [0.8; 3] }
    }
}

/// The world's shader as a colour, as `Surface` stands in for the surface
/// shaders: the `BackgroundNode` of `blender/shader.cpp`'s world graph,
/// colour times strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    pub color: [f32; 3],
    pub strength: f32,
}

impl Default for World {
    fn default() -> Self {
        Self { color: [0.05; 3], strength: 1.0 }
    }
}

impl World {
    /// What a ray that hits nothing sees.
    pub fn radiance(&self) -> [f32; 3] {
        self.color.map(|channel| channel * self.strength)
    }
}

/// `Scene`.
#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    /// `shaders`: the places the meshes' `used_shader` and `shader`
    /// indices name; `shader[i].id == i`.
    pub shader: Vec<Shader>,
    /// What each of `shader` is, one for one.
    pub surface: Vec<Surface>,
    /// `geometry`: so far the meshes. An object names one by its place.
    pub mesh: Vec<Mesh>,
    pub object: Vec<Object>,
    pub world: World,
    /// The film, the integrator and the scene's parameters, which the managers read.
    pub background: Background,
    pub film: Film,
    pub integrator: Integrator,
    pub param: SceneParam,
    /// The lookup tables the managers add to.
    pub lookup: LookupTableSet,
}

impl Scene {
    pub fn new() -> Scene {
        Scene { camera: Camera::new(), shader: Vec::new(), surface: Vec::new(), mesh: Vec::new(), object: Vec::new(), world: World::default(), background: Background::default(), film: Film::new(), integrator: Integrator::new(), param: SceneParam::default(), lookup: LookupTableSet::new() }
    }
    /// The triangles of every object, an instanced mesh once for each
    /// object that uses it: what the renderer draws.
    pub fn triangle_count(&self) -> usize {
        self.object.iter().filter_map(|object| self.mesh.get(object.geometry)).map(Mesh::triangle_count).sum()
    }
    /// `Scene::create_shader`'s bookkeeping: add a shader of this surface
    /// and return its place.
    pub fn add_shader(&mut self, surface: Surface) -> usize {
        let id = self.shader.len();
        self.shader.push(Shader { id: id as u32, has_volume: false, emission: matches!(surface, Surface::Emission { .. }), ..Shader::default() });
        self.surface.push(surface);
        id
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
