//! A rig kept to be posed at any frame, and what its pose deforms: each
//! deforming bone's matrix and B-Bone segments for the Armature modifier
//! (`modifier/armature.rs`), from a solved pose rather than the saved one.

use super::math::BMat4;
use super::{BONE_NO_DEFORM, BRig, BRigPose, BSceneView};
use crate::blend::anim::fcurve::BFCurve;
use crate::blend::file::types::*;
use crate::blend::modifier::armature::BDeformBone;
use crate::blend::modifier::bbone;
use glam::Mat4;
use std::collections::HashMap;

/// An armature object whose pose moves — or that an object follows a bone
/// of: its rig, its action's curves, and its bones' B-Bone shapes.
#[derive(Clone, Debug)]
pub struct BlendRigSource {
    /// Index into the scene's objects.
    pub object: usize,
    /// The armature object's block address (what an Armature modifier names).
    pub address: u64,
    pub rig: BRig,
    pub curves: Vec<BFCurve>,
    pub(crate) shapes: bbone::Pose,
    /// Whether the pose moves with the frame: keys, drivers, or constraints
    /// reading other objects. A rig only followed or drawn keeps its pose,
    /// and its meshes are not posed again.
    pub moves: bool,
    /// How the viewport draws it (`display.rs`).
    pub look: Option<std::sync::Arc<super::BRigLook>>,
}

impl BlendFile {
    /// Every armature object as a rig to pose: it moves with an action,
    /// drivers or constraints on other objects; otherwise it keeps its
    /// saved pose, which the viewport draws and objects may follow.
    pub(crate) fn rig_sources(
        &self,
        objects: &[BlendSceneObject],
        dna: &DnaType,
        actions: &HashMap<u64, crate::blend::anim::read::BAction>,
    ) -> Vec<BlendRigSource> {
        objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.object_type == "ARMATURE")
            .filter_map(|(i, o)| {
                let ob = self.block_at_code(o.block_address, "OB")?;
                let rig = self.rig(ob)?;
                let curves = self.binding(ob, dna).and_then(|b| actions.get(&b.action).map(|a| a.curves(b.slot).to_vec())).unwrap_or_default();
                let reads = rig.bones.iter().flat_map(|b| &b.constraints).any(|c| c.reads().next().is_some());
                let moves = !curves.is_empty() || !rig.drivers.is_empty() || reads;
                let look = self.rig_look(ob, &rig, dna).map(std::sync::Arc::new);
                Some(BlendRigSource { object: i, address: o.block_address, rig, curves, shapes: self.pose_shapes(ob, dna), moves, look })
            })
            .collect()
    }
}

impl BlendRigSource {
    /// The pose at `frame`, the rig object standing at `world`, `scene` the
    /// objects and rigs around it.
    pub fn pose_at(&self, frame: f32, world: Mat4, scene: &dyn BSceneView) -> BRigPose {
        self.rig.pose_at(&self.curves, frame, world, scene)
    }

    /// Each deforming bone's deform in `pose`, by name — what the Armature
    /// modifier reads (`pose_deform_mats` for a saved pose).
    pub(crate) fn deform_bones(&self, pose: &BRigPose) -> HashMap<String, BDeformBone> {
        let by_address: HashMap<u64, usize> = self.rig.bones.iter().enumerate().map(|(i, b)| (b.address, i)).collect();
        let tail = |i: usize| pose.pose[i].loc() + pose.pose[i].y_axis.truncate() * self.rig.bones[i].length;
        let posed = self.shapes.posed(&|addr| by_address.get(&addr).map(|&i| (pose.pose[i], pose.pose[i].loc(), tail(i))));
        self.rig
            .bones
            .iter()
            .enumerate()
            .filter(|(_, b)| b.flag & BONE_NO_DEFORM == 0)
            .map(|(i, b)| (b.name.clone(), BDeformBone { chan: pose.chan[i], bbone: posed.bbone(b.address) }))
            .collect()
    }
}

/// The rigs that move (or that objects follow) and the meshes they deform,
/// as a read scene carries them. Shared, not copied: a clone is two `Arc` lists, and two are equal
/// when they hold the same sources.
#[derive(Clone, Debug, Default)]
pub struct BlendRigs {
    pub rigs: Vec<std::sync::Arc<BlendRigSource>>,
    pub skins: Vec<std::sync::Arc<crate::blend::modifier::BlendSkinSource>>,
    /// Non-rigged meshes with modifier drivers, retained for per-frame
    /// re-evaluation (`modifier/driven.rs`).
    pub driven: Vec<std::sync::Arc<crate::blend::modifier::driven::BlendDrivenSource>>,
}

impl PartialEq for BlendRigs {
    fn eq(&self, other: &Self) -> bool {
        self.rigs.len() == other.rigs.len()
            && self.rigs.iter().zip(&other.rigs).all(|(x, y)| std::sync::Arc::ptr_eq(x, y))
            && self.skins.len() == other.skins.len()
            && self.skins.iter().zip(&other.skins).all(|(x, y)| std::sync::Arc::ptr_eq(x, y))
            && self.driven.len() == other.driven.len()
            && self.driven.iter().zip(&other.driven).all(|(x, y)| std::sync::Arc::ptr_eq(x, y))
    }
}

impl BlendFile {
    /// Every rig, and every mesh a moving one deforms.
    pub(crate) fn rigs(
        &self,
        objects: &[BlendSceneObject],
        dna: &DnaType,
        actions: &HashMap<u64, crate::blend::anim::read::BAction>,
    ) -> BlendRigs {
        let rigs = self.rig_sources(objects, dna, actions);
        let driven = self.driven_sources(objects, dna).into_iter().map(std::sync::Arc::new).collect();
        if rigs.is_empty() {
            return BlendRigs { rigs: Vec::new(), skins: Vec::new(), driven };
        }
        let skins = self.skin_sources(objects, dna).into_iter().filter(|s| rigs.iter().any(|r| r.moves && r.address == s.armature)).map(std::sync::Arc::new).collect();
        BlendRigs { rigs: rigs.into_iter().map(std::sync::Arc::new).collect(), skins, driven }
    }
}
