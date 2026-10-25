//! The pose at a frame: each channel's basis from its curves, then
//! `BKE_pose_where_is` for the plain case — full inheritance of the parent's
//! rotation and scale, local location — which is
//! `pose_mat = parent_pose · parent_arm⁻¹ · arm · basis`. Bones that inherit
//! differently (`inherit_scale_mode`, `BONE_NO_LOCAL_LOCATION`) and every
//! constraint and driver are stages (b) and (c) of task 17; a rig like
//! Rigify does not move visibly until those land.

use super::read::BAction;
use crate::blend::file::types::*;
use glam::Mat4;
use std::collections::HashMap;

/// One channel of an armature's pose, read once.
pub struct BChannel {
    pub name: String,
    pub parent: Option<usize>,
    /// Rest matrix in armature space (`Bone.arm_mat`).
    pub arm: Mat4,
    pub raw: crate::blend::object::transform::ObjectTransformRaw,
}

impl BlendFile {
    /// The armature object's channels, parents first.
    pub(crate) fn pose_channels(&self, arm_ob: &BlendBlock, dna: &DnaType) -> Vec<BChannel> {
        let mut out = Vec::new();
        let (Some(pose_o), Some(chan_o)) = (self.off("Object", "*pose", dna), self.off("bPose", "chanbase", dna)) else { return out };
        let (name_o, bone_o, parent_o, arm_o) = (
            self.off("bPoseChannel", "name[64]", dna),
            self.off("bPoseChannel", "*bone", dna),
            self.off("bPoseChannel", "*parent", dna),
            self.off("Bone", "arm_mat[4][4]", dna),
        );
        let (Some(name_o), Some(arm_o)) = (name_o, arm_o) else { return out };
        let _ = bone_o;
        let rest = self.bones_by_name(arm_ob, dna);
        self.scoped(arm_ob, || {
            let Some(pose) = self.block_at(self.ptr(arm_ob, Some(pose_o))) else { return };
            let chans = self.list(self.ptr(pose, Some(chan_o)), self.off("bPoseChannel", "*next", dna));
            let index: HashMap<u64, usize> = chans.iter().enumerate().map(|(i, c)| (c.old_address, i)).collect();
            for pc in &chans {
                let arm = self
                    .pchan_bone(pc, dna, &rest)
                    .and_then(|b| b.data.get(arm_o..arm_o + 64))
                    .map(|s| Mat4::from_cols_array(&std::array::from_fn(|i| self.read_f32(&s[i * 4..i * 4 + 4]))))
                    .unwrap_or(Mat4::IDENTITY);
                out.push(BChannel {
                    name: crate::blend::modifier::armature::name64(&pc.data, name_o),
                    parent: index.get(&self.ptr(pc, parent_o)).copied(),
                    arm,
                    raw: self.pchan_raw(pc, dna),
                });
            }
        });
        out
    }
}

/// `matrix_basis` per bone at `frame`: its own channels, curves applied.
pub fn bases_at(channels: &[BChannel], action: Option<&BAction>, slot: i32, frame: f32) -> Vec<Mat4> {
    let curves = action.map_or(&[][..], |a| a.curves(slot));
    channels
        .iter()
        .map(|c| super::channels::basis_at(c.raw, curves.iter().filter(|f| f.bone() == Some(c.name.as_str())), frame))
        .collect()
}

/// `pose_mat` per bone at `frame`, parents first. Channels are listed in
/// Blender's order, which already puts a parent before its children.
pub fn poses_at(channels: &[BChannel], action: Option<&BAction>, slot: i32, frame: f32) -> Vec<Mat4> {
    let bases = bases_at(channels, action, slot, frame);
    let mut out: Vec<Mat4> = Vec::with_capacity(channels.len());
    for (i, c) in channels.iter().enumerate() {
        let local = match c.parent.filter(|&p| p < i) {
            Some(p) => out[p] * channels[p].arm.inverse() * c.arm,
            None => c.arm,
        };
        out.push(local * bases[i]);
    }
    out
}
