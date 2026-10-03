//! Kit-agnostic runtime scene data + the single blend→scene traversal.
//!
//! Pure Rust — no gpui, no OS deps — mirroring the
//! `apple::services::outliner` pattern. Consumers reach these types via
//! `crate::scene::…`.
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
//! - `tests_traversal.rs` — traversal tests (opening lives in `crate::phx`).
//!
//! Requirements: 6.1, 6.3, 7.2

#![allow(dead_code)]

/// Identity in Blender's `float[4][4]` layout.
pub const IDENTITY_M4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

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
    pub mesh: std::sync::Arc<SceneMesh>,
    pub location: [f32; 3],
    pub scale: [f32; 3],
    /// Full world matrix (Blender `float[4][4]` layout: `m[col][row]`,
    /// translation in `m[3]`), rebuilt from loc/rot/scale and the parent
    /// chain like `BKE_object_where_is_calc`. The viewport transforms raw
    /// mesh vertices by this — `location`/`scale` are kept for the
    /// Properties panel readouts.
    pub matrix_world: [[f32; 4]; 4],
    /// Material slots for this object. Index 0 is the primary material.
    /// Empty when the source carried no material — the renderer then falls
    /// back to its neutral loaded colour.
    pub materials: Vec<SceneMaterial>,
    /// One material-slot index per triangle (`len == mesh.indices.len() / 3`
    /// when populated). Each entry indexes into `materials`. Empty when the
    /// object is single-material or carried no per-face assignment; the
    /// renderer then uses slot 0 for every triangle.
    pub material_indices: std::sync::Arc<Vec<u16>>,
    /// Per-vertex UV coordinates (`len == mesh.vertices.len()` when
    /// populated). Empty when the source carried no UV layer. Kept for
    /// texture sampling; flat-colour rendering ignores it.
    pub uvs: std::sync::Arc<Vec<[f32; 2]>>,
    /// `Object.col`, scene-linear RGBA: Solid's Color ▸ Object (white by default).
    pub color: [f32; 4],
}

impl SceneObject {
    /// Construct a `SceneObject` with identity transform — zero translation,
    /// unit scale, and no materials / UVs. Convenience for fixtures and tests
    /// that do not care about per-object placement or appearance.
    pub fn identity(name: impl Into<String>, mesh: SceneMesh) -> Self {
        Self {
            name: name.into(),
            mesh: mesh.into(),
            location: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            matrix_world: IDENTITY_M4,
            materials: Vec::new(),
            material_indices: Default::default(),
            uvs: Default::default(),
            color: [1.0; 4],
        }
    }

    /// The base colour the renderer should use for triangle `tri` (index into
    /// the triangulated `indices` buffer / 3). Resolves the per-face material
    /// slot when present, else slot 0, else `None` (caller uses its neutral
    /// loaded colour).
    pub fn triangle_base_color(&self, tri: usize) -> Option<[f32; 4]> {
        self.triangle_slot(tri).map(|m| m.base_color)
    }

    /// Solid-mode colour for triangle `tri` (`Material.r/g/b`, what Blender
    /// draws in Solid shading with the default MATERIAL colour type).
    pub fn triangle_viewport_color(&self, tri: usize) -> Option<[f32; 4]> {
        self.triangle_slot(tri).map(|m| m.viewport_color)
    }

    fn triangle_slot(&self, tri: usize) -> Option<&SceneMaterial> {
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
    /// Shading normal per triangle corner (`len == indices.len()`) for
    /// renderers that interpolate: split/custom normals, smooth faces; a zero
    /// entry means flat. Empty when every face is flat.
    pub corners: SceneCorners,
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
    /// Every animated object's NLA: its action line and tracks.
    pub nla: Vec<SceneNla>,
    /// Every driven object's drivers.
    pub drivers: Vec<SceneDrivers>,
    /// Collection tree (Scene Collection first) with view-layer flags.
    pub collections: Vec<SceneCollection>,
    /// Active / selected / hidden objects of the opened view layer.
    pub selection: SceneSelection,
    /// Read report: counts, scene name, file version, duration.
    pub report: SceneReadReport,
    /// Empties (drawn by the overlay; listed in the outliner by name).
    pub empties: Vec<SceneEmpty>,
    /// Blender object type per entry of `objects` ("MESH", "EMPTY",
    /// "CURVE", …). Empty for sources that do not carry types.
    pub object_types: Vec<String>,
    /// Workspace + 3D view saved in the file ("Load UI").
    pub ui: SceneUi,
    /// Every object's transform over time and who moves with it.
    pub motion: SceneMotion,
    /// The timeline markers, in file order.
    pub markers: Vec<crate::blend::BlendMarker>,
    /// The Playback popover's settings, as the file has them.
    pub playback: crate::blend::BlendPlayback,
    /// `(child, parent)` by object name, for every object whose `Object.parent`
    /// is in the scene — the Outliner's Object Children.
    pub parents: Vec<(String, String)>,
    /// Each entry of `objects`' own matrix, before its parent's
    /// (`matrix_local`: `T · R · S` from loc / rot / size) — what a
    /// transform edit replaces, the parent's part kept.
    pub locals: Vec<[[f32; 4]; 4]>,
    /// Each camera's, light's and empty's own matrix by name, as `locals`
    /// holds the meshes'.
    pub extra_locals: Vec<(String, [[f32; 4]; 4])>,
    /// Each entry of `objects`' active UV map, face by face (`uv.rs`) —
    /// what the UV editor draws. Empty for sources without faces.
    pub uv_maps: Vec<std::sync::Arc<SceneUvMap>>,
    /// The 3D cursor, as the file left it.
    pub cursor: crate::blend::BlendCursor,
}

impl SceneModel {
    /// The object `name` is parented to, if any.
    pub fn parent_of(&self, name: &str) -> Option<&str> {
        self.parents.iter().find(|(child, _)| child == name).map(|(_, parent)| parent.as_str())
    }
}

// ── Child modules (declared after the core types so they can `use super::*`)

mod collection;
mod corners;
pub mod motion;
pub use motion::{SceneMotion, SceneMotionFrame};
pub mod rig;
pub use rig::{SceneRigs, SceneSkin};
pub mod relation;
pub use relation::SceneRelation;
pub mod uv;
pub use uv::SceneUvMap;
mod error;
mod extras;
mod geometry;
mod import;
mod import_extras;
mod import_phx;
mod material;
mod object;
pub(crate) mod polyfill;
mod report;
mod texture;
mod types_ext;
mod ui;

#[cfg(test)]
mod tests_traversal;
#[cfg(test)]
mod tests_types;

pub use collection::{SceneCollection, SceneSelection};
pub use corners::SceneCorners;
pub use error::{MESSAGE_MAX, SceneLoadError};
pub use material::{SceneBsdf, SceneMap, SceneMaps, SceneMaterial};
pub use report::SceneReadReport;
pub use texture::{SceneEnvironment, SceneTexture, TextureCache};
pub use ui::{SceneSolidFx, SceneSolidShade, SceneUi, SceneView, SceneXray, VIEW_WIDTH_PER_DIST};
pub use types_ext::{
    SceneAction, SceneCamera, SceneDrivers, SceneEmpty, SceneLight, SceneNla, SceneRender, SceneShadow, SceneSsao, SceneTimeline, SceneWorld,
};
