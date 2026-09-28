//! Default scene (Cube, Sphere, Gem) + the Scene_Source that both the
//! Viewport and the Outliner iterate. Pure data + pure math — no gpui, no
//! theme widgets — so it can be unit-tested in isolation.
//!
//! View orientation, zoom, pitch, pan, and every scene-related state key
//! live in `keys.rs`. Projection + viewport-fit math lives in `project.rs`.

use crate::phiphx::scene::{SceneMaterial, SceneMesh, SceneModel};
use crate::v2::state::BState;

// ── Child modules ───────────────────────────────────────────────────────

pub mod skin;
pub use skin::RaySkin;

pub mod source;
pub use source::{
    clear_loaded_scene, clear_scene_objects, scene_actions, scene_cameras, scene_lights,
    object_builds, scene_objects, scene_render, scene_source, scene_timeline, scene_world, set_loaded_scene,
    set_loaded_shared, with_loaded,
};

pub mod key;
#[cfg(test)]
mod keys_tests;
pub use key::{
    LoadStage, ZOOM_MAX, ZOOM_MIN, error_key, is_blend_path, is_perspective, load_cancel_key, load_progress_key,
    load_stage_key, loaded_path_key, loading_key, pan_x_key, pan_y_key, perspective_key,
    pitch_key, pitch_step, pivot_key, lens_key, selected_key, view_load_progress, view_load_stage, view_pan,
    view_pitch, view_yaw, view_zoom, yaw_key, zoom_key, zoom_step,
};

pub mod project;
pub mod render_job;
pub mod render_status;
pub use render_job::{BRenderPlan, active_camera, camera_of, plan as render_plan, render_size};
pub mod output_set;
pub use output_set::{output_set, pass_set, pass_state};
pub mod render_save;
pub use render_save::{BSaveKind, kind_of as save_kind, save as save_render, save_to as save_render_to, written_pass};
pub mod render_set;
pub use render_set::{BRenderSet, render_set};
pub mod action;
pub mod camera;
pub mod camera_view;
pub mod color;
pub mod keyframe;
pub use keyframe::BShownKeys;
pub mod edit;
pub mod export;
pub mod extra;
pub mod import;
pub mod marker;
pub mod grab;
pub mod motion;
pub mod save;
pub mod template;
pub mod boxsel;
pub mod circlesel;
pub mod lasso;
pub mod pick;
pub mod select;
pub mod view_op;
pub mod shading;
pub mod viewpoint;
pub mod view_slot;
pub mod links;
pub mod transform;
pub mod xform;
pub use shading::{BShading, shading_key, view_shading};
pub use project::{
    FIT_EPSILON, MESH_SCALE, fit_vertex, fit_vertices, mesh_bounds, project, project3,
    project3_full, project_zoom,
};

#[cfg(test)]
mod tests_project;
#[cfg(test)]
mod tests_scene;
#[cfg(test)]
mod tests_state;

// ── Default scene: shapes, static objects, and the runtime scene source ──

/// A primitive shape an object can take. Drives both its viewport silhouette
/// and its outliner icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShape {
    Cube,
    Sphere,
    Gem,
}

impl BShape {
    pub fn icon(self) -> &'static str {
        match self {
            BShape::Cube => "cube",
            BShape::Sphere => "sphere",
            BShape::Gem => "diamond",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            BShape::Cube => "Cube",
            BShape::Sphere => "Sphere",
            BShape::Gem => "Gem",
        }
    }
    pub fn mesh_type(self) -> crate::phiphx::spatial::PhiQuiMeshType {
        use crate::phiphx::spatial::PhiQuiMeshType;
        match self {
            BShape::Cube => PhiQuiMeshType::Cube,
            BShape::Sphere => PhiQuiMeshType::Sphere,
            BShape::Gem => PhiQuiMeshType::Icosahedron,
        }
    }
}

/// One scene object. Position is in an abstract world space centred on the
/// origin; the viewport projects `(x, y)` to screen using the current yaw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BObject {
    pub name: &'static str,
    pub shape: BShape,
    /// World position on the ground plane, roughly in the range [-1.0, 1.0].
    pub x: f32,
    pub y: f32,
}

/// The default scene — three objects arranged in a triangle on the ground
/// plane. Shown in the Outliner and drawn in the viewport.
pub static OBJECTS: &[BObject] = &[
    BObject { name: "Cube",   shape: BShape::Cube,   x: -0.45, y: -0.25 },
    BObject { name: "Sphere", shape: BShape::Sphere, x:  0.45, y: -0.10 },
    BObject { name: "Gem",    shape: BShape::Gem,    x:  0.00, y:  0.45 },
];

/// A renderable object read from the [`current_scene`] Scene_Source, iterated
/// by both the Viewport and the Outliner. Generalises the flat [`BObject`]
/// so the same value can carry either a Default_Scene primitive or a loaded
/// mesh (with the Blender object's world-space transform).
#[derive(Clone, Debug, PartialEq)]
pub enum BObject3D {
    /// A Default_Scene primitive.
    Primitive {
        name: String,
        shape: BShape,
        x: f32,
        y: f32,
    },
    /// A loaded object carrying its imported geometry, its material slots and
    /// per-vertex UVs, plus the Blender object's world-space transform.
    Loaded {
        name: String,
        mesh: std::sync::Arc<SceneMesh>,
        location: [f32; 3],
        scale: [f32; 3],
        /// World matrix (Blender layout, translation in `[3]`); the viewport
        /// transforms raw vertices by this so rotation and parenting show.
        matrix_world: [[f32; 4]; 4],
        /// Material slots (base colour + PBR + optional texture path). Empty
        /// when the source object declared no material.
        materials: Vec<SceneMaterial>,
        /// Per-triangle material-slot index (`len == indices.len() / 3` when
        /// populated), else empty for single-material objects.
        material_indices: std::sync::Arc<Vec<u16>>,
        /// Per-vertex UVs (`len == vertices.len()` when populated), else empty.
        uvs: std::sync::Arc<Vec<[f32; 2]>>,
    },
}


/// Aggregate counts summarising the current Scene_Source: object count,
/// total vertex count and total triangle count. Used by the viewport
/// overlay to display live scene statistics (Blender's info-line stats).
pub fn scene_metrics(objects: &[BObject3D]) -> (usize, usize, usize) {
    let mut verts = 0usize;
    let mut faces = 0usize;
    for object in objects {
        match object {
            BObject3D::Primitive { shape, .. } => {
                let (v, i, _n) =
                    crate::phiphx::spatial::SpatialObject3D::buffers(shape.mesh_type());
                verts += v.len();
                faces += i.len() / 3;
            }
            BObject3D::Loaded { mesh, .. } => {
                verts += mesh.vertices.len();
                faces += mesh.indices.len() / 3;
            }
        }
    }
    (objects.len(), verts, faces)
}

/// The single Scene_Source read by BOTH the Viewport and the Outliner.
///
/// Returns the Default_Scene (one [`BObject3D::Primitive`] per entry in the
/// static [`OBJECTS`] table) when no blend model is loaded, otherwise one
/// [`BObject3D::Loaded`] per object in `loaded`, in the same order.
pub fn current_scene(s: &BState, loaded: Option<&SceneModel>) -> Vec<BObject3D> {
    let objects = match loaded {
        None => OBJECTS
            .iter()
            .map(|o| BObject3D::Primitive {
                name: o.name.to_string(),
                shape: o.shape,
                x: o.x,
                y: o.y,
            })
            .collect(),
        Some(model) => {
            // Where every object stands at the shown frame, while the scene moves.
            let posed = motion::posed(model, motion::shown_frame(s));
            model
                .objects
                .iter()
                .enumerate()
                .map(|(i, o)| BObject3D::Loaded {
                    name: o.name.clone(),
                    mesh: o.mesh.clone(),
                    location: o.location,
                    scale: o.scale,
                    matrix_world: posed.as_ref().and_then(|p| p.object(i)).unwrap_or(o.matrix_world),
                    materials: o.materials.clone(),
                    material_indices: o.material_indices.clone(),
                    uvs: o.uvs.clone(),
                })
                .collect()
        }
    };
    // What was added, deleted and duplicated since the file was read, then
    // where the Transform panel put them.
    let mut objects = edit::BSceneEdit::now(s).apply(objects);
    if let Some(model) = loaded {
        transform::BTransforms::now(s).apply(s, model, &mut objects);
    }
    objects
}
