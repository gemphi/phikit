//! Kit-agnostic runtime scene data + the single blend→scene traversal.
//!
//! Pure Rust — no gpui, no OS deps — mirroring the
//! `apple::services::outliner` pattern. Consumers reach these types via
//! `crate::phiphx::services::scene::…`.
//!
//! ## File split (200-line file cap)
//!
//! - `mod.rs` (this file) — core types (`SceneObject`, `SceneMesh`,
//!   `SceneNormals`, `SceneModel`) + child module declarations + re-exports.
//! - `types_ext.rs` — extended scene surfaces (timeline / world / render /
//!   camera / light / action) read by non-mesh spaces.
//! - `error.rs` — `SceneLoadError` + its message clamp.
//! - `import.rs` — blend → scene conversion helpers (fan-triangulation,
//!   per-face normal synthesis, camera / light partitioning).
//! - `load.rs` — `load_blend_scene` orchestration + native-error category
//!   classification.
//! - `tests_traversal.rs`, `tests_load.rs` — split test suites.
//!
//! Requirements: 6.1, 6.3, 7.2

#![allow(dead_code)]

/// One renderable object in a runtime scene. Pure data.
///
/// `location` and `scale` mirror the Blender object's world-space transform
/// (Blender's `object.location` and `object.scale`). The renderer applies them
/// after `fit_vertices` normalises the mesh into the shared viewport-fit
/// space, so a loaded scene's objects preserve their relative placement /
/// size just as Blender draws them. Defaults are identity: no translation and
/// unit scale — use [`SceneObject::identity`] for fixtures.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneObject {
    pub name: String,
    pub mesh: SceneMesh,
    pub location: [f32; 3],
    pub scale: [f32; 3],
    /// Material slots for this object. Index 0 is the primary material.
    /// Empty when the source carried no material — the renderer then falls
    /// back to its neutral loaded colour.
    pub materials: Vec<SceneMaterial>,
    /// One material-slot index per triangle (`len == mesh.indices.len() / 3`
    /// when populated). Each entry indexes into `materials`. Empty when the
    /// object is single-material or carried no per-face assignment; the
    /// renderer then uses slot 0 for every triangle.
    pub material_indices: Vec<u16>,
    /// Per-vertex UV coordinates (`len == mesh.vertices.len()` when
    /// populated). Empty when the source carried no UV layer. Kept for
    /// texture sampling; flat-colour rendering ignores it.
    pub uvs: Vec<[f32; 2]>,
}

impl SceneObject {
    /// Construct a `SceneObject` with identity transform — zero translation,
    /// unit scale, and no materials / UVs. Convenience for fixtures and tests
    /// that do not care about per-object placement or appearance.
    pub fn identity(name: impl Into<String>, mesh: SceneMesh) -> Self {
        Self {
            name: name.into(),
            mesh,
            location: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            materials: Vec::new(),
            material_indices: Vec::new(),
            uvs: Vec::new(),
        }
    }

    /// The base colour the renderer should use for triangle `tri` (index into
    /// the triangulated `indices` buffer / 3). Resolves the per-face material
    /// slot when present, else slot 0, else `None` (caller uses its neutral
    /// loaded colour).
    pub fn triangle_base_color(&self, tri: usize) -> Option<[f32; 4]> {
        if self.materials.is_empty() {
            return None;
        }
        let slot = self
            .material_indices
            .get(tri)
            .copied()
            .map(usize::from)
            .unwrap_or(0);
        self.materials
            .get(slot)
            .or_else(|| self.materials.first())
            .map(|m| m.base_color)
    }
}

/// A single material slot on a [`SceneObject`], mirrored from Blender's
/// Principled BSDF: an RGBA base colour plus the two PBR scalars and an
/// optional base-colour texture path. Kept kit-agnostic (pure data) so both
/// the viewport and the Properties material panel can read it.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMaterial {
    pub name: String,
    /// Linear RGBA base / diffuse colour.
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    /// Base-colour image texture path (Blender `//`-relative or absolute), if
    /// the material's shader graph links an Image Texture node. `None` when
    /// the material is a flat colour.
    pub base_color_texture: Option<String>,
}

impl SceneMaterial {
    /// A neutral opaque-grey material used when a source object declares no
    /// material at all but a slot is required.
    pub fn neutral() -> Self {
        Self {
            name: String::from("Material"),
            base_color: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            base_color_texture: None,
        }
    }
}

/// Geometry for one object: interleaved-free parallel buffers.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMesh {
    /// Object-local positions.
    pub vertices: Vec<[f32; 3]>,
    /// Triangle list, `len % 3 == 0`.
    pub indices: Vec<u32>,
    /// Per-vertex OR per-face normals.
    pub normals: SceneNormals,
}

/// Normal layout for a [`SceneMesh`].
#[derive(Clone, Debug, PartialEq)]
pub enum SceneNormals {
    /// One normal per vertex; `len == vertices.len()`.
    PerVertex(Vec<[f32; 3]>),
    /// One normal per triangle; `len == indices.len() / 3`.
    PerFace(Vec<[f32; 3]>),
}

/// The full runtime scene read by both Viewport and Outliner.
///
/// Everything after `objects` is optional / defaulted: existing consumers see
/// no behaviour change unless they explicitly consult the new fields. New
/// spaces (Animation, Rendering, Shading, Compositing) read these to populate
/// their read-only surfaces without needing to touch DNA themselves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneModel {
    pub objects: Vec<SceneObject>,
    /// The `.blend` path this scene came from.
    pub source_path: Option<String>,
    pub timeline: SceneTimeline,
    pub world: SceneWorld,
    pub render: SceneRender,
    pub cameras: Vec<SceneCamera>,
    pub lights: Vec<SceneLight>,
    pub actions: Vec<SceneAction>,
}

// ── Child modules (declared after the core types so they can `use super::*`)

mod error;
mod import;
pub mod load;
mod types_ext;

#[cfg(test)]
mod tests_load;
#[cfg(test)]
mod tests_traversal;
#[cfg(test)]
mod tests_types;

pub use error::{MESSAGE_MAX, SceneLoadError};
pub use import::scene_from_blend;
pub use load::load_blend_scene;
pub use types_ext::{
    SceneAction, SceneCamera, SceneLight, SceneRender, SceneTimeline, SceneWorld,
};
