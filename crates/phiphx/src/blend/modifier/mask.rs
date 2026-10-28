//! Mask modifier (`MaskModifierData`, Blender `MOD_mask.cc`): keep only the
//! vertices a vertex group (or the groups of an armature's selected bones)
//! weights above `threshold`, and the edges and faces made of them.
//!
//! Translated: both modes — `MOD_MASK_MODE_VGROUP` and `MOD_MASK_MODE_ARM`
//! (the deform groups of the bones selected in the rig's saved pose) — the
//! invert, the threshold, and `use_smooth`, which instead of dropping a face
//! that straddles the threshold cuts it where the weight crosses it
//! (`mask_copy.rs`). Every attribute is compacted the way `copy_masked_*`
//! copies it.
//!
//! Not yet: the armature mode's selection for files older than the
//! per-channel `POSE_SELECTED` flag (they kept it on the `Bone`).

#[path = "mask_copy.rs"]
mod copy;

use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;

const S: &str = "MaskModifierData";
/// `MaskModifierMode`.
const MODE_VGROUP: u32 = 0;
const MODE_ARM: u32 = 1;
/// `MaskModifierFlag`.
const MASK_INV: u32 = 1 << 0;
const MASK_SMOOTH: u32 = 1 << 1;
/// `bPoseChannel.flag`'s `POSE_SELECTED`.
const POSE_SELECTED: u32 = 1 << 6;
/// `OB_ARMATURE`.
const OB_ARMATURE: u32 = 25;

#[derive(Clone)]
pub(crate) struct BMask {
    pub mode: u32,
    pub flag: u32,
    pub threshold: f32,
    /// `BKE_id_defgroup_name_index` of `vgroup`, if it names a group.
    pub group: Option<usize>,
    /// Armature mode's `selected_bone_uses_group`, one per vertex group;
    /// `None` when there is no armature or it has no pose.
    pub bones: Option<Vec<bool>>,
    /// `!mesh->vertex_group_names.is_empty()`.
    pub has_groups: bool,
    /// `is_disabled`: `ob_arm` is set but is not an armature.
    pub not_armature: bool,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let me = f.mesh_block_of(ob, dna);
    let groups = me.map(|me| f.vertex_group_names(ob, me, dna)).unwrap_or_default();
    let name = f.off(S, "vgroup[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let arm = f.ptr(md, f.off(S, "*ob_arm", dna));
    let arm_ob = f.block_at_code(arm, "OB");
    let is_armature = arm_ob.is_some_and(|a| f.int_field(a, "Object", "type", dna) == OB_ARMATURE);
    Some(Box::new(BMask {
        mode: f.int_field(md, S, "mode", dna),
        flag: f.int_field(md, S, "flag", dna) & 0xffff,
        threshold: f.float_field(md, S, "threshold", dna, 0.0),
        group: groups.iter().position(|g| *g == name),
        bones: arm_ob.filter(|_| is_armature).and_then(|a| selected_bone_groups(f, a, &groups, dna)),
        has_groups: !groups.is_empty(),
        not_armature: arm != 0 && !is_armature,
    }))
}

/// `compute_vertex_mask__armature_mode`'s first loop: for each vertex group,
/// whether a channel of that name exists in the rig's pose, has a bone, and
/// is selected. `None` for a rig without a pose (`armature_ob->pose`).
fn selected_bone_groups(f: &BlendFile, arm_ob: &BlendBlock, groups: &[String], dna: &DnaType) -> Option<Vec<bool>> {
    let name_o = f.off("bPoseChannel", "name[64]", dna)?;
    let bones = f.bones_by_name(arm_ob, dna);
    f.scoped(arm_ob, || {
        let pose = f.block_at(f.ptr(arm_ob, f.off("Object", "*pose", dna)))?;
        let chans = f.list(f.ptr(pose, f.off("bPose", "chanbase", dna)), f.off("bPoseChannel", "*next", dna));
        let selected: Vec<String> = chans
            .iter()
            .filter(|pc| f.int_field(pc, "bPoseChannel", "flag", dna) & POSE_SELECTED != 0)
            .filter(|pc| f.pchan_bone(pc, dna, &bones).is_some())
            .map(|pc| super::armature::name64(&pc.data, name_o))
            .collect();
        Some(groups.iter().map(|g| selected.contains(g)).collect())
    })
}

/// `BKE_defvert_find_weight`: the vertex's weight in `group`, 0 when absent.
fn weight(dv: &[(u32, f32)], group: usize) -> f32 {
    dv.iter().find(|(g, _)| *g as usize == group).map_or(0.0, |(_, w)| *w)
}

impl BMask {
    /// The two `compute_vertex_mask__*` functions, before the invert; `None`
    /// is `modify_mesh`'s "return the input mesh" (no rig, no such group).
    fn vertex_mask(&self, mesh: &BlendEvalMesh) -> Option<Vec<bool>> {
        match self.mode {
            MODE_ARM => {
                let bones = self.bones.as_ref()?;
                let uses = |g: u32, w: f32| bones.get(g as usize).is_some_and(|&b| b && w > self.threshold);
                Some(mesh.dverts.iter().map(|dv| dv.iter().any(|&(g, w)| uses(g, w))).collect())
            }
            _ => {
                let group = self.group?;
                Some(mesh.dverts.iter().map(|dv| weight(dv, group) > self.threshold).collect())
            }
        }
    }
}

impl BModifier for BMask {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }

    /// `required_data_mask`: `CD_MASK_MDEFORMVERT`.
    fn wants_weights(&self) -> bool {
        true
    }

    /// `is_disabled`: `mmd->ob_arm && mmd->ob_arm->type != OB_ARMATURE`.
    fn disabled(&self) -> bool {
        self.not_armature
    }

    /// `modify_mesh`.
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let invert = self.flag & MASK_INV != 0;
        // "Return empty or input mesh when there are no vertex groups."
        if mesh.dverts.is_empty() || mesh.dverts.len() != mesh.vertices.len() {
            if !invert {
                *mesh = BlendEvalMesh::default();
            }
            return;
        }
        // "Quick test to see if we can return early."
        if !matches!(self.mode, MODE_ARM | MODE_VGROUP) || mesh.vertices.is_empty() || !self.has_groups {
            return;
        }
        let Some(mut mask) = self.vertex_mask(mesh) else { return };
        if invert {
            mask.iter_mut().for_each(|m| *m = !*m);
        }
        let use_interpolation = self.mode == MODE_VGROUP && self.flag & MASK_SMOOTH != 0;
        let weights: Vec<f32> = match (use_interpolation, self.group) {
            (true, Some(g)) => mesh.dverts.iter().map(|dv| weight(dv, g)).collect(),
            _ => Vec::new(),
        };
        let cut = use_interpolation.then_some(copy::Cut { weights: &weights, threshold: self.threshold });
        *mesh = copy::masked(mesh, &mask, cut.as_ref());
    }
}
