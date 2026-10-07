//! A rig at a frame — task 17 (d): an armature's pose as Blender 5.0
//! evaluates it, bone by bone (`BKE_pose_where_is_bone`), for rigs whose
//! deforming bones are moved by constraints and drivers, not keys.
//!
//! 1. The action's curves over each channel and custom property
//!    ([`read`], [`BRigState`]).
//! 2. The rig object's drivers over constraint influences, mutes, Armature
//!    constraint weights and channels ([`driver`]).
//! 3. Each bone in dependency order ([`solve`]): its parent's pose through
//!    the bone's inheritance (`BKE_bone_parent_transform_calc_from_matrices`,
//!    [`space`]), its channel, then its constraints ([`constraint`]) in the
//!    spaces they ask for; an IK chain as one unit, solved by Blender's
//!    legacy solver ported step for step ([`ik`]).
//!
//! The same constraints serve objects (`constraint/object.rs`): the scene
//! around an owner ([`view`]) gives other objects' worlds and local spaces
//! and other rigs' posed bones, and `anim/follow.rs` solves every object
//! after what it follows.
//!
//! Follow Path reads a curve's path (`blend/curve`).
//!
//! What Blender evaluates that this does not yet: Spline IK, and the
//! Shrinkwrap and motion-tracking constraints (they need a mesh or a
//! clip); an Action constraint playing the
//! object's action, or on an object; a vertex group as a target; B-Bone
//! shaped targets; driver variables other than a single property of the
//! rig; and NLA strips on the rig. A bone behind one of them keeps its
//! channel's pose. `tests.rs` holds every bone of THYMUS's Rigify rig to
//! Blender's pose at six frames, `constraint/tests.rs` a bone under each
//! other constraint kind, `anim/follow_tests.rs` objects under them.

mod constraint;
pub mod display;
pub mod driver;
mod ik;
pub(crate) mod math;
mod quat;
pub(crate) mod read;
pub mod solve;
mod source;
mod space;
mod view;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod display_tests;

pub use constraint::{BConstraint, BConstraintKind, BObjectWork, BTarget};
pub use display::{BBoneDraw, BBoneLook, BBoneShape, BBoneShown, BRigLook};
pub use driver::{BDriven, BDriverSource, BDriverVar, BRigDriver};
pub use solve::BRigPose;
pub use source::{BlendRigSource, BlendRigs};
pub use space::BRigSpace;
pub use view::{BEmptyScene, BObjectSpace, BSceneView};

use crate::blend::idprop::BIdGroup;
use crate::blend::object::transform::ObjectTransformRaw;
use glam::{Mat3, Mat4, Vec3};
use std::collections::HashMap;

/// `BONE_CONNECTED`: the head sits on the parent's tail; the channel's
/// location is ignored.
pub const BONE_CONNECTED: u32 = 1 << 4;
/// `BONE_HINGE`: no parent rotation or scale.
pub const BONE_HINGE: u32 = 1 << 9;
/// `BONE_NO_DEFORM`.
pub const BONE_NO_DEFORM: u32 = 1 << 12;
/// `BONE_NO_LOCAL_LOCATION`: the location in the parent's orientation.
pub const BONE_NO_LOCAL_LOCATION: u32 = 1 << 22;

/// One bone of a rig: its rest (`Bone`) and its channel (`bPoseChannel`).
#[derive(Clone, Debug)]
pub struct BRigBone {
    pub name: String,
    /// The pose channel's block address (what B-Bone handles name).
    pub address: u64,
    pub parent: Option<usize>,
    /// `Bone.arm_mat`: the rest matrix, armature space.
    pub arm: Mat4,
    /// `Bone.bone_mat`: the rest rotation relative to the parent.
    pub bone_mat: Mat3,
    /// `Bone.head`: the head in the parent's space.
    pub head: Vec3,
    pub length: f32,
    /// `Bone.flag` (`BONE_CONNECTED`, `BONE_HINGE`, …).
    pub flag: u32,
    /// `Bone.inherit_scale_mode` (`BONE_INHERIT_SCALE_*`).
    pub inherit_scale: u8,
    /// `Bone.segments`: B-Bone segments (1: a plain bone).
    pub segments: i16,
    /// The channel's loc / rot / scale as saved.
    pub raw: ObjectTransformRaw,
    /// `bPoseChannel.ikflag`, `ikstretch`, `stiffness`, `limitmin` / `max`.
    pub ikflag: u32,
    pub ikstretch: f32,
    pub stiffness: Vec3,
    pub limit_min: Vec3,
    pub limit_max: Vec3,
    pub constraints: Vec<BConstraint>,
    /// The channel's custom properties (`bPoseChannel.prop`).
    pub props: BIdGroup,
}

/// An armature object's bones, in Blender's channel order (a parent before
/// its children), and its drivers.
#[derive(Clone, Debug, Default)]
pub struct BRig {
    /// The armature object's block address: a constraint targeting it
    /// targets this rig.
    pub object: u64,
    pub bones: Vec<BRigBone>,
    pub by_name: HashMap<String, usize>,
    pub drivers: Vec<BRigDriver>,
}

impl BRig {
    pub fn bone(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }
}

/// What the animation and the drivers leave on a rig at a frame, before the
/// pose is solved: each channel, custom property and constraint setting.
#[derive(Clone, Debug)]
pub struct BRigState {
    pub raw: Vec<ObjectTransformRaw>,
    /// Animated custom properties, by bone and name (the rest read from
    /// the bone's `props`).
    pub props: HashMap<(usize, String), f32>,
    /// Per bone, per constraint: influence, muted, Armature target weights.
    pub influence: Vec<Vec<f32>>,
    pub muted: Vec<Vec<bool>>,
    pub weights: Vec<Vec<Vec<f32>>>,
}
