//! Shrinkwrap (`eModifierType_Shrinkwrap` = 25) — Blender `MOD_shrinkwrap.cc`
//! + `blenkernel/intern/shrinkwrap.cc` (`shrinkwrapModifier_deform`), ported.
//!
//! Each vertex is taken into the target's object space (`SpaceTransform`
//! `local2target = target⁻¹ · ob`), moved to the target by the wrap method,
//! snapped by the wrap mode with the offset (`keepDist`, applied in the
//! target's space as Blender applies it), brought back, and blended in by
//! its vertex-group weight.
//!
//! Wrap methods: Nearest Surface Point, Nearest Vertex and Project (normal
//! or axis, positive/negative, culling, the auxiliary target, the limit).
//! Target Normal Project (Newton's method on smooth normals) is not ported:
//! the reader returns nothing for it and the modifier is skipped.
//!
//! The target is its **evaluated** mesh, its own modifier stack applied, as
//! `shrinkwrapModifier_deform` takes it (`shrinkwrap_target.rs`).

#[path = "shrinkwrap_math.rs"]
mod math;
#[path = "shrinkwrap_project.rs"]
mod project;
#[path = "shrinkwrap_snap.rs"]
mod snap;
#[path = "shrinkwrap_tree.rs"]
mod tree;

use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::{Mat3, Mat4, Vec3};
use math::{interp, normalize};
use std::sync::Arc;
use tree::{Nearest, Target};

/// `eShrinkwrapType`.
const NEAREST_SURFACE: u32 = 0;
const PROJECT: u32 = 1;
const NEAREST_VERTEX: u32 = 2;
const TARGET_PROJECT: u32 = 3;
/// `MOD_SHRINKWRAP_INVERT_VGROUP`.
const INVERT_VGROUP: u32 = 1 << 6;
/// `OB_MESH`.
const OB_MESH: u32 = 1;
/// The chunk `threading::parallel_for` hands each task; the nearest-point
/// heuristic starts fresh at each.
const GRAIN: usize = 512;

/// `SpaceTransform`: between the modified object's space and a target's.
#[derive(Clone, Copy)]
pub(super) struct Space {
    local2target: Mat4,
    target2local: Mat4,
}

impl Space {
    /// `BLI_SPACE_TRANSFORM_SETUP(&data, ob, target)`.
    fn new(local: Mat4, target: Mat4) -> Self {
        let local2target = target.inverse() * local;
        Self { local2target, target2local: local2target.inverse() }
    }
    /// `BLI_space_transform_apply`.
    pub(super) fn apply(&self, co: Vec3) -> Vec3 {
        self.local2target.transform_point3(co)
    }
    /// `BLI_space_transform_invert`.
    pub(super) fn invert(&self, co: Vec3) -> Vec3 {
        self.target2local.transform_point3(co)
    }
    /// `BLI_space_transform_apply_normal`.
    pub(super) fn apply_normal(&self, no: Vec3) -> Vec3 {
        normalize(Mat3::from_mat4(self.local2target) * no)
    }
    /// `BLI_space_transform_invert_normal`.
    pub(super) fn invert_normal(&self, no: Vec3) -> Vec3 {
        normalize(Mat3::from_mat4(self.target2local) * no)
    }
}

pub(crate) struct BShrinkwrap {
    target: u64,
    aux_target: u64,
    tree: Option<Arc<Target>>,
    aux_tree: Option<Arc<Target>>,
    keep_dist: f32,
    shrink_type: u32,
    opts: u32,
    mode: u32,
    proj_limit: f32,
    proj_axis: u32,
    /// Index of the modifier's vertex group in the object's groups, if named.
    group: Option<usize>,
    invert: bool,
}

/// The target's evaluated mesh as a tree (`shrinkwrap_target.rs`).
#[path = "shrinkwrap_target.rs"]
mod target;
/// A target's evaluated mesh, for any modifier that follows one.
pub(super) use target::evaluated_mesh_of;
use target::read_target;

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    const S: &str = "ShrinkwrapModifierData";
    let shrink_type = f.int_field(md, S, "shrinkType", dna);
    if shrink_type == TARGET_PROJECT {
        return None;
    }
    let target = f.ptr(md, f.off(S, "*target", dna));
    let aux_target = f.ptr(md, f.off(S, "*auxTarget", dna));
    let tree = read_target(f, target, ob, dna, shrink_type == NEAREST_VERTEX);
    // Only Project reads the auxiliary target.
    let aux_tree = match shrink_type == PROJECT && aux_target != 0 {
        true => read_target(f, aux_target, ob, dna, false),
        false => None,
    };
    let name = f.off(S, "vgroup_name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = (!name.is_empty())
        .then(|| f.mesh_block_of(ob, dna).and_then(|me| f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name)))
        .flatten();
    let opts = f.int_field(md, S, "shrinkOpts", dna);
    Some(Box::new(BShrinkwrap {
        target: tree.as_ref().map_or(0, |_| target),
        aux_target: aux_tree.as_ref().map_or(0, |_| aux_target),
        tree,
        aux_tree,
        keep_dist: f.float_field(md, S, "keepDist", dna, 0.0),
        shrink_type,
        opts,
        mode: f.int_field(md, S, "shrinkMode", dna),
        proj_limit: f.float_field(md, S, "projLimit", dna, 0.0),
        proj_axis: f.int_field(md, S, "projAxis", dna),
        group,
        invert: opts & INVERT_VGROUP != 0,
    }))
}

impl BShrinkwrap {
    /// `BKE_defvert_array_find_weight_safe`: no group named → 1; a group
    /// but no weights on the mesh → 0 (1 inverted).
    fn weight(&self, mesh: &BlendEvalMesh, i: usize) -> f32 {
        let Some(gi) = self.group else { return 1.0 };
        if mesh.dverts.is_empty() {
            return match self.invert { true => 1.0, false => 0.0 };
        }
        let w = mesh.dverts.get(i).and_then(|d| d.iter().find(|(g, _)| *g as usize == gi)).map_or(0.0, |(_, w)| *w);
        match self.invert { true => 1.0 - w, false => w }
    }

    /// `shrinkwrap_calc_nearest_surface_point` / `_vertex` and their `_cb_ex`.
    ///
    /// The search starts bounded by the previous vertex's hit (Blender's
    /// "local proximity heuristic"), which prunes without changing the answer.
    fn nearest(&self, mesh: &mut BlendEvalMesh, tree: &Target, space: &Space) {
        let mut nearest = Nearest::none();
        for i in 0..mesh.vertices.len() {
            if i % GRAIN == 0 {
                nearest = Nearest::none();
            }
            let mut weight = self.weight(mesh, i);
            if weight == 0.0 {
                continue;
            }
            let co = Vec3::from(mesh.vertices[i]);
            let tmp_co = space.apply(co);
            nearest.dist_sq = match nearest.index {
                Some(_) => tmp_co.distance_squared(nearest.co),
                None => f32::MAX,
            };
            tree.nearest(tmp_co, &mut nearest);
            let Some(index) = nearest.index else { continue };
            let goal = match self.shrink_type {
                NEAREST_VERTEX => {
                    // Scale the weight so the blend stops `keepDist` short.
                    if nearest.dist_sq > f32::EPSILON {
                        let dist = nearest.dist_sq.sqrt();
                        weight *= (dist - self.keep_dist) / dist;
                    }
                    nearest.co
                }
                _ => snap::snap_point_to_surface(tree, None, self.mode, index, nearest.co, nearest.no, self.keep_dist, tmp_co),
            };
            mesh.vertices[i] = interp(co, space.invert(goal), weight).into();
        }
    }

    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "offset" => self.keep_dist = value.as_f32(),
            "project_limit" => self.proj_limit = value.as_f32(),
            _ => return false,
        }
        true
    }
}

impl BModifier for BShrinkwrap {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self {
            target: self.target,
            aux_target: self.aux_target,
            tree: self.tree.clone(),
            aux_tree: self.aux_tree.clone(),
            keep_dist: self.keep_dist,
            shrink_type: self.shrink_type,
            opts: self.opts,
            mode: self.mode,
            proj_limit: self.proj_limit,
            proj_axis: self.proj_axis,
            group: self.group,
            invert: self.invert,
        })
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    /// `MOD_shrinkwrap.cc`'s `is_disabled`: no target, or a target that is
    /// not a mesh (here: one whose mesh did not read into a tree, which is
    /// also where `BKE_shrinkwrap_init_tree` would fail and do nothing).
    fn disabled(&self) -> bool {
        self.tree.is_none()
    }

    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let (Some(tree), Some(target_world)) = (&self.tree, ctx.worlds.get(&self.target)) else { return };
        let space = Space::new(ctx.world, *target_world);
        match self.shrink_type {
            NEAREST_SURFACE | NEAREST_VERTEX => self.nearest(mesh, tree, &space),
            PROJECT => {
                let aux = match (&self.aux_tree, ctx.worlds.get(&self.aux_target)) {
                    (Some(t), Some(w)) => Some((&**t, Space::new(ctx.world, *w))),
                    _ => None,
                };
                // An auxiliary target named but not found stops Blender's projection outright.
                if self.aux_target != 0 && aux.is_none() {
                    return;
                }
                self.project(mesh, tree, &space, aux);
            }
            _ => {}
        }
    }
}
