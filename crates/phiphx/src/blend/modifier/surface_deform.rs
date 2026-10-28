//! Surface Deform (`eModifierType_SurfaceDeform` = 53) — the deform half
//! of Blender `MOD_surfacedeform.cc`: `surfacedeformModifier_do` and
//! `deformVert`, applying the **saved** bind (`SDefVert` / `SDefBind`,
//! `surface_deform_read.rs`) over the target's evaluated mesh.
//!
//! Binding itself (`surfacedeformBind`) is not ported: a modifier the file
//! saved unbound stays where it is, as Blender's would until the user
//! binds it. Each bound vertex is the sum of its binds' points — a weighted
//! corner triangle, an n-gon's weights, or two corners and the centroid —
//! each pushed along its polygon's normal (`normal_dist`) and scaled by
//! its `influence`; the vertex moves to that point by `strength` times its
//! vertex-group weight. The target's coordinates are taken through the
//! bind matrix `smd->mat` (the target's world into the object's at bind
//! time), never through either object's current world, so moving the
//! target object moves nothing — only its mesh does.

use super::driver::BModValue;
use super::surface_deform_read::{BSdefVert, count, read_binds};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// `MOD_SDEF_BIND`, `MOD_SDEF_INVERT_VGROUP`.
const BIND: u32 = 1 << 0;
const INVERT_VGROUP: u32 = 1 << 1;
/// `SurfaceDeformModifierBindMode`.
pub(super) const MODE_CORNER_TRIS: u32 = 0;
pub(super) const MODE_NGONS: u32 = 1;
pub(super) const MODE_CENTROID: u32 = 2;

#[derive(Clone)]
pub(crate) struct BSurfaceDeform {
    /// The bind, one entry per bound vertex (`smd->verts`), empty when the
    /// file holds none (`smd->verts == nullptr`).
    pub binds: Arc<Vec<BSdefVert>>,
    /// The target's evaluated vertices through `smd->mat`
    /// (`BKE_mesh_wrapper_vert_coords_copy_with_mat4`); `None` without a
    /// mesh target.
    pub target: Option<Arc<Vec<[f32; 3]>>>,
    /// The target's evaluated face count, for the legacy count check.
    pub target_faces: u32,
    /// `mesh_verts_num`, `target_verts_num`, `target_polys_num` at bind.
    pub bound_verts: u32,
    pub bound_target_verts: u32,
    pub bound_target_faces: u32,
    pub strength: f32,
    pub flags: u32,
    pub group: Option<usize>,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let st = "SurfaceDeformModifierData";
    let flags = f.int_field(md, st, "flags", dna);
    let target_addr = f.ptr(md, f.off(st, "*target", dna));
    let name = f.off(st, "defgrp_name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = (!name.is_empty())
        .then(|| f.mesh_block_of(ob, dna).and_then(|me| f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name)))
        .flatten();
    let mat = Mat4::from_cols_array_2d(&f.off(st, "mat[4][4]", dna).map_or(crate::blend::object::transform::IDENTITY4, |o| {
        std::array::from_fn(|c| std::array::from_fn(|r| f.f32_at(&md.data, o + (c * 4 + r) * 4)))
    }));
    let evaluated = super::shrinkwrap::evaluated_mesh_of(f, target_addr, ob, dna);
    let target_faces = evaluated.as_ref().map_or(0, |m| m.faces.len() as u32);
    let target = evaluated.map(|m| Arc::new(m.vertices.iter().map(|v| mat.transform_point3(Vec3::from(*v)).to_array()).collect()));
    // The DNA names these `num_mesh_verts`, `target_verts_num` and
    // `numpoly` in the files that carry a bind (`surface_deform_read.rs`).
    Some(Box::new(BSurfaceDeform {
        binds: Arc::new(read_binds(f, md, dna)),
        target,
        target_faces,
        bound_verts: count(f, md, st, "mesh_verts_num", "num_mesh_verts", dna),
        bound_target_verts: count(f, md, st, "target_verts_num", "target_verts_num", dna),
        bound_target_faces: count(f, md, st, "target_polys_num", "numpoly", dna),
        strength: f.float_field(md, st, "strength", dna, 1.0),
        flags,
        group,
    }))
}

/// `normal_poly_v3`: Newell's method over the polygon, normalised.
fn normal_poly(cos: &[Vec3]) -> Vec3 {
    let mut n = Vec3::ZERO;
    for (k, cur) in cos.iter().enumerate() {
        let prev = cos[(k + cos.len() - 1) % cos.len()];
        // `add_newell_cross_v3_v3v3`.
        n.x += (prev.y - cur.y) * (prev.z + cur.z);
        n.y += (prev.z - cur.z) * (prev.x + cur.x);
        n.z += (prev.x - cur.x) * (prev.y + cur.y);
    }
    n.normalize_or_zero()
}

impl BSurfaceDeform {
    /// `is_disabled`: no mesh target, unless the bind was kept after the
    /// modifier was unbound.
    fn is_disabled(&self) -> bool {
        self.target.is_none() && !(!self.binds.is_empty() && self.flags & BIND == 0)
    }

    /// The sanity checks of `surfacedeformModifier_do` before any vertex
    /// moves: `Some(target)` to go on, `None` to leave the mesh.
    fn ready(&self, verts: usize) -> Option<&[[f32; 3]]> {
        if self.flags & BIND == 0 || self.binds.is_empty() {
            return None;
        }
        let target = self.target.as_deref()?;
        if self.bound_verts as usize != verts {
            return None;
        }
        if self.bound_target_faces != self.target_faces && self.bound_target_verts == 0 {
            return None;
        }
        // A target that grew is taken as the old vertices first; one that
        // shrank cannot be used.
        if self.bound_target_verts != 0 && self.bound_target_verts as usize > target.len() {
            return None;
        }
        (self.strength != 0.0).then_some(target.as_slice())
    }
}

impl BModifier for BSurfaceDeform {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        match prop {
            "strength" => self.strength = value.as_f32(),
            _ => return false,
        }
        true
    }

    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    fn disabled(&self) -> bool {
        self.is_disabled()
    }

    /// `deformVert` over every bound vertex.
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let Some(target) = self.ready(mesh.vertices.len()) else { return };
        let at = |i: u32| target.get(i as usize).map_or(Vec3::ZERO, |v| Vec3::from(*v));
        // `MOD_get_vgroup`: a group the mesh has no weights for leaves
        // every weight at 1.
        let weighted = self.group.filter(|_| mesh.dverts.len() == mesh.vertices.len());
        let invert = self.flags & INVERT_VGROUP != 0;
        let mut cos: Vec<Vec3> = Vec::new();
        for bv in self.binds.iter() {
            let idx = bv.vertex_idx as usize;
            let Some(v) = mesh.vertices.get(idx).copied() else { continue };
            let weight = match weighted {
                None => 1.0,
                Some(g) => {
                    let w = mesh.dverts[idx].iter().find(|(gr, _)| *gr as usize == g).map_or(0.0, |(_, w)| *w);
                    match invert {
                        true => 1.0 - w,
                        false => w,
                    }
                }
            };
            if weight == 0.0 {
                continue;
            }
            let mut offset = Vec3::ZERO;
            for b in &bv.binds {
                cos.clear();
                cos.extend(b.verts.iter().map(|&i| at(i)));
                let norm = normal_poly(&cos);
                let w = |k: usize| b.weights.get(k).copied().unwrap_or(0.0);
                let mut temp = match b.mode {
                    MODE_NGONS => cos.iter().enumerate().map(|(k, c)| *c * w(k)).sum(),
                    MODE_CENTROID => {
                        let cent = cos.iter().copied().sum::<Vec3>() / (cos.len().max(1) as f32);
                        cos.first().copied().unwrap_or_default() * w(0) + cos.get(1).copied().unwrap_or_default() * w(1) + cent * w(2)
                    }
                    _ => (0..3).map(|k| cos.get(k).copied().unwrap_or_default() * w(k)).sum(),
                };
                temp += norm * b.normal_dist;
                offset += temp * b.influence;
            }
            offset -= Vec3::from(v);
            mesh.vertices[idx] = (Vec3::from(v) + offset * (self.strength * weight)).to_array();
        }
    }
}
