//! Armature deform (#16, first modifier) — Blender `armature_deform.cc`.
//!
//! For each mesh object with an enabled Armature modifier
//! (`eModifierType_Armature`, `eModifierMode_Realtime`), every vertex is
//! moved into armature space (`target_to_armature`), displaced by the
//! weighted sum of `chan_mat · co − co` over its vertex-group bones, divided
//! by the total weight (when > 0.0001), and moved back. `chan_mat` is
//! `pose_mat · arm_mat⁻¹`; Blender saves the evaluated `pose_mat` of every
//! pose channel (`pose_channel_flush_to_orig_if_needed`), so no animation or
//! constraint evaluation is needed to show the saved pose.
//! B-Bone segments are in `bbone.rs`: a bone with `segments > 1` deforms
//! through a curve rather than rigidly, and this rig's spine, thighs and shins
//! all use it.
//! Not yet: envelopes, dual quaternions, multi-modifier.

use crate::blend::file::types::*;
use super::bbone::BBone;
use super::skin::{SkinJob, SkinPair};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

const BONE_NO_DEFORM: u32 = 1 << 12;
const CONTRIB_MIN: f32 = 0.0001;

fn m4(b: &[u8], o: usize, f: &BlendFile) -> Option<Mat4> {
    let s = b.get(o..o + 64)?;
    Some(Mat4::from_cols_array(&std::array::from_fn(|i| f.read_f32(&s[i * 4..i * 4 + 4]))))
}

pub(crate) fn name64(b: &[u8], o: usize) -> String {
    let s = b.get(o..(o + 64).min(b.len())).unwrap_or(&[]);
    let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..n]).to_string()
}

impl BlendFile {
    /// `chan_mat` (pose · rest⁻¹) per deforming bone name of armature `arm`,
    /// with the bone's B-Bone segments when it has any.
    pub(crate) fn pose_deform_mats(&self, arm: &BlendBlock, dna: &DnaType) -> HashMap<String, BDeformBone> {
        let shapes = self.pose_shapes(arm, dna);
        let mut out = HashMap::new();
        let (Some(pose_o), Some(name_o), Some(bone_o), Some(pose_mat_o)) = (
            self.off("Object", "*pose", dna),
            self.off("bPoseChannel", "name[64]", dna),
            self.off("bPoseChannel", "*bone", dna),
            self.off("bPoseChannel", "pose_mat[4][4]", dna),
        ) else {
            return out;
        };
        let (Some(arm_mat_o), Some(chan_o)) = (self.off("Bone", "arm_mat[4][4]", dna), self.off("bPose", "chanbase", dna)) else {
            return out;
        };
        self.scoped(arm, || {
            let Some(pose) = self.block_at(self.ptr(arm, Some(pose_o))) else { return };
            for pc in self.list(self.ptr(pose, Some(chan_o)), self.off("bPoseChannel", "*next", dna)) {
                let bone = self.block_at(self.ptr(pc, Some(bone_o)));
                let rest = bone.and_then(|b| m4(&b.data, arm_mat_o, self));
                let posed = m4(&pc.data, pose_mat_o, self);
                let deforms = bone.is_some_and(|b| self.int_field(b, "Bone", "flag", dna) & BONE_NO_DEFORM == 0);
                if let (Some(rest), Some(posed), true) = (rest, posed, deforms) {
                    let bbone = shapes.bbone(pc.old_address);
                    out.insert(name64(&pc.data, name_o), BDeformBone { chan: posed * rest.inverse(), bbone });
                }
            }
        });
        out
    }

    /// Vertex-group names (`Mesh.vertex_group_names`, 3.0+, else `Object.defbase`).
    pub(crate) fn vertex_group_names(&self, ob: &BlendBlock, me: &BlendBlock, dna: &DnaType) -> Vec<String> {
        let next = self.off("bDeformGroup", "*next", dna);
        let name = self.off("bDeformGroup", "name[64]", dna).unwrap_or(16);
        let from = |b: &BlendBlock, st: &str, f: &str| {
            let first = self.ptr(b, self.off(st, f, dna));
            self.scoped(b, || self.list(first, next).iter().map(|g| name64(&g.data, name)).collect::<Vec<_>>())
        };
        let names = if self.off("Mesh", "vertex_group_names", dna).is_some() { from(me, "Mesh", "vertex_group_names") } else { Vec::new() };
        if names.is_empty() { from(ob, "Object", "defbase") } else { names }
    }
}

/// How one bone deforms: the rigid `pose · rest⁻¹` matrix, and its B-Bone
/// segments when the bone has more than one.
#[derive(Clone, Debug)]
pub(crate) struct BDeformBone {
    pub chan: Mat4,
    pub bbone: Option<BBone>,
}

impl BDeformBone {
    /// The matrix that moves a point at `co` (armature space).
    pub(crate) fn matrix(&self, co: Vec3) -> Mat4 {
        match &self.bbone {
            Some(b) => b.matrix(co),
            None => self.chan,
        }
    }
}

/// A read Armature modifier: pose matrices, vertex groups and weights.
///
/// `mats` is shared rather than owned: every mesh an armature deforms
/// carries one of these, and they all point at the same rig
/// (`deform.rs`).
struct BArmatureDeform {
    armature: u64,
    mats: std::sync::Arc<super::deform::BDeformMats>,
    groups: Vec<String>,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let armature = f.ptr(md, Some(f.off("ArmatureModifierData", "*object", dna)?));
    let arm_ob = f.block_at_code(armature, "OB")?;
    let me = f.mesh_block_of(ob, dna)?;
    Some(Box::new(BArmatureDeform {
        armature,
        mats: f.deform_mats(arm_ob, dna),
        groups: f.vertex_group_names(ob, me, dna),
    }))
}

impl BModifier for BArmatureDeform {
    fn wants_weights(&self) -> bool {
        true
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let Some(arm_world) = ctx.worlds.get(&self.armature) else { return };
        if mesh.dverts.len() != mesh.vertices.len() {
            return;
        }
        let to_arm = arm_world.inverse() * ctx.world;
        if let Some(d) = deform(&mesh.vertices, &mesh.dverts, &self.groups, &self.mats, to_arm) {
            mesh.vertices = d;
        }
    }

    fn armature_source(&self) -> Option<(u64, &[String])> {
        Some((self.armature, &self.groups))
    }

    fn skin_job(&self, mesh: &BlendEvalMesh, ctx: &EvalCtx) -> Option<SkinJob> {
        let arm_world = ctx.worlds.get(&self.armature)?;
        if mesh.dverts.len() != mesh.vertices.len() {
            return None;
        }
        let to_arm = arm_world.inverse() * ctx.world;
        let from_arm = to_arm.inverse();
        // Group index → the bone's slots. A rigid bone takes one slot; a
        // B-Bone takes one per segment boundary and each vertex names the two
        // it falls between (`bbone.rs`).
        let mut bones = Vec::new();
        let mut push = |m: Mat4| {
            bones.push((from_arm * m * to_arm).to_cols_array());
            bones.len() as u32 - 1
        };
        let slot: Vec<Option<(u32, Option<&BBone>)>> = self
            .groups
            .iter()
            .map(|g| {
                self.mats.get(g).map(|d| match &d.bbone {
                    Some(b) => {
                        let mut base = None;
                        for m in b.slots() {
                            let at = push(m);
                            base.get_or_insert(at);
                        }
                        (base.expect("a B-Bone has segments"), Some(b))
                    }
                    None => (push(d.chan), None),
                })
            })
            .collect();
        let mut offsets = Vec::with_capacity(mesh.vertices.len() + 1);
        let mut pairs = Vec::new();
        offsets.push(0);
        for (i, dv) in mesh.dverts.iter().enumerate() {
            let co = to_arm.transform_point3(Vec3::from(mesh.vertices[i]));
            pairs.extend(dv.iter().filter(|&&(_, w)| w != 0.0).filter_map(|&(g, w)| {
                let (base, bbone) = (*slot.get(g as usize)?)?;
                Some(match bbone {
                    Some(b) => {
                        let (index, blend) = b.segment(co);
                        SkinPair { bone: base + index as u32, weight: w, next: base + index as u32 + 1, blend }
                    }
                    None => SkinPair { bone: base, weight: w, next: base, blend: 0.0 },
                })
            }));
            offsets.push(pairs.len() as u32);
        }
        (!pairs.is_empty()).then(|| SkinJob { rest: mesh.vertices.clone(), offsets, pairs, bones, out: Vec::new() })
    }
}

/// Deform `verts` (object space) by the armature; `None` when nothing moved.
pub(crate) fn deform(
    verts: &[[f32; 3]],
    dverts: &[Vec<(u32, f32)>],
    groups: &[String],
    mats: &HashMap<String, BDeformBone>,
    target_to_arm: Mat4,
) -> Option<Vec<[f32; 3]>> {
    let by_group: Vec<Option<&BDeformBone>> = groups.iter().map(|g| mats.get(g)).collect();
    let arm_to_target = target_to_arm.inverse();
    let mut moved = false;
    let out = verts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let co = target_to_arm.transform_point3(Vec3::from(*v));
            let (mut delta, mut contrib) = (Vec3::ZERO, 0.0f32);
            for &(g, w) in dverts.get(i).map(Vec::as_slice).unwrap_or(&[]) {
                if let (Some(Some(m)), true) = (by_group.get(g as usize), w != 0.0) {
                    delta += w * (m.matrix(co).transform_point3(co) - co);
                    contrib += w;
                }
            }
            if contrib <= CONTRIB_MIN {
                return *v;
            }
            moved = true;
            arm_to_target.transform_point3(co + delta / contrib).to_array()
        })
        .collect();
    moved.then_some(out)
}
