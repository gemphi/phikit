//! The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to
//! modifiers that change how an existing mesh looks.
//!
//! Declarative: [`KINDS`] maps a DNA `ModifierData.type` to a reader that
//! turns the saved settings into a [`BModifier`]. The stack runs in file
//! order on an [`BlendEvalMesh`]; unknown or disabled modifiers are skipped
//! (and listed in the read report by the caller).

pub mod driver;
pub(crate) mod driver_read;
pub(crate) mod armature;
pub(crate) mod deform;
pub(crate) mod driven;
pub(crate) mod prefix;
#[cfg(test)]
mod prefix_tests;
#[cfg(test)]
mod driven_tests;
pub(crate) mod bbone;
mod array;
mod displace;
mod geom;
mod mirror;
mod solidify;
pub mod skin;
mod subsurf;
mod topology;
mod triangulate;
mod weld;
mod merge_verts;
mod lattice;
mod crease;
mod corrective_smooth;
mod mask;
mod shrinkwrap;
mod curve;
mod multires;
mod simple_deform;
mod surface_deform;
mod surface_deform_read;
mod cloth;
mod cloth_cache;
mod cloth_disk;
mod nodes;
pub(crate) mod nodes_corners;
mod nodes_fans;
mod nodes_ops;
mod nodes_topo;
mod nodes_field;
mod nodes_inputs;
mod nodes_legacy;
#[cfg(test)]
mod nodes_tests;
pub(crate) mod report;
#[cfg(test)]
mod tests;

mod deferred;
mod job;
mod kind;
mod source;
mod stack;

pub(crate) use deferred::{BEvalJob, DeferredRaw, apply_eval_jobs};
pub(crate) use kind::KINDS;
pub use kind::supported_modifiers;
pub use source::BlendSkinSource;

use crate::device::blend::file::types::*;
use glam::Mat4;
use std::collections::HashMap;

/// `eModifierMode_Realtime` — shown in the viewport.
const MODE_REALTIME: u32 = 1;

/// The mesh a modifier stack works on (object space).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlendEvalMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: crate::device::blend::BFaces,
    /// Per-corner UVs in face-corner order (empty when the mesh has none).
    pub loop_uvs: Vec<[f32; 2]>,
    /// The colour attribute per corner, in the same order (empty when the
    /// mesh has none); every modifier carries it as it carries the UVs.
    pub loop_colors: Vec<[f32; 3]>,
    pub face_materials: Vec<u16>,
    pub face_smooth: Vec<bool>,
    /// Per-vertex `(group, weight)`; carried through topology changes so a
    /// later Armature still deforms (Blender interpolates `MDeformVert`).
    pub dverts: Vec<Vec<(u32, f32)>>,
    /// Creased edges `(a, b, crease)`, `a < b`, crease `0..1`
    /// (`crease_edge`), which the Subdivision Surface sharpens by.
    pub edge_creases: Vec<(u32, u32, f32)>,
    /// Vertex creases (`crease_vert`), one per vertex or empty.
    pub vert_creases: Vec<f32>,
    /// Edges not used by any face — wire-only geometry (hair guides,
    /// curve-to-mesh outlines, construction lines). Each pair is `(a, b)`
    /// with `a < b`. Carried through the modifier stack and subdivided by
    /// the Subdivision Surface so the vertex count stays in parity with
    /// Blender.
    pub loose_edges: Vec<(u32, u32)>,
    /// Edges marked sharp (`sharp_edge`), `(a, b)` with `a < b`: what the
    /// Geometry Nodes "Smooth by Angle" reads and writes, and what the
    /// corner normals split on (`nodes_corners.rs`). Carried by vertex
    /// pair, so a modifier that keeps its vertices keeps them and one that
    /// rebuilds them leaves pairs that name no edge, which split nothing.
    pub sharp_edges: Vec<(u32, u32)>,
}

/// What a modifier may look up while evaluating.
pub(crate) struct EvalCtx<'a> {
    /// The evaluated object's world matrix.
    pub world: Mat4,
    /// World matrices of all scene objects by saved address.
    pub worlds: &'a HashMap<u64, Mat4>,
}

pub(crate) trait BModifier: Send + Sync {
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx);
    /// Whether the stack must load vertex-group weights for this modifier.
    fn wants_weights(&self) -> bool {
        false
    }
    /// As a batched skin job instead of `apply` (only asked of the last modifier).
    /// Test hook: is this the lattice deform?
    #[cfg(test)]
    fn is_lattice(&self) -> bool {
        false
    }
    fn skin_job(&self, _mesh: &BlendEvalMesh, _ctx: &EvalCtx) -> Option<skin::SkinJob> {
        None
    }
    /// An Armature modifier's rig object and vertex-group names.
    fn armature_source(&self) -> Option<(u64, &[String])> {
        None
    }
    /// Take this frame's driven value for one of this modifier's settings,
    /// named as the driver's path names it (`driver.rs`'s `SETTINGS` table).
    ///
    /// Returns whether it landed. `false` is not an error: a setting the
    /// table models but this struct has no field for — `Subsurf.render_levels`
    /// on a viewport-only reader, `Solidify.material_offset` — is reported
    /// rather than silently dropped, which is how the gap stays visible.
    ///
    /// The value arrives already typed and clamped as Blender clamps it,
    /// integers truncated toward zero; an implementation applies its own
    /// reader's guards on top (a Subdivision's level cap) and nothing else.
    fn set_driven(&mut self, _prop: &str, _component: usize, _value: driver::BModValue) -> bool {
        false
    }
    /// Blender's `copy_data` (`ModifierTypeInfo.copy_data`, and the memcpy
    /// behind `BKE_modifier_copydata_generic`): one modifier, copied.
    ///
    /// This is how Blender gets a *per-frame* stack at all. The depsgraph
    /// evaluates a **copy** of the object, its drivers write into that copy's
    /// modifier data, and the copy is what runs — the file's own stack is
    /// never touched. So a frame that wants its own driven settings takes its
    /// own stack, which is why this is a clone and not interior mutability:
    /// two frames evaluated at once must not share a `levels` field.
    ///
    /// Blender's generic copy ends with *"Runtime fields are never to be
    /// preserved"*, so a clone starts with an empty cache rather than
    /// inheriting one (`BSubsurf`'s topology tables are exactly that).
    fn clone_box(&self) -> Box<dyn BModifier>;
    /// Blender's `is_disabled`: a modifier whose settings make it a no-op is
    /// dropped from the stack rather than applied.
    ///
    /// It is asked **after** the driven settings land, because Blender asks
    /// it of the *evaluated* modifier — `is_disabled(scene, md, …)` runs on
    /// the depsgraph's copy, whose fields the drivers have already written.
    /// Asking before would read the file's saved value and drop a modifier a
    /// driver was about to switch on.
    fn disabled(&self) -> bool {
        false
    }
}
