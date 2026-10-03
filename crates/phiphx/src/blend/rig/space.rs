//! Where a bone's pose comes from and the spaces a constraint works in:
//! `BKE_bone_parent_transform_calc_from_matrices` (how much of its parent a
//! bone takes, by `inherit_scale_mode`, hinge and local location) and
//! `BKE_constraint_mat_convertspace` (world, pose, local, local with
//! parent, owner local, custom).

use super::math::BMat4;
use super::solve::BPoseWork;
use super::view::BObjectSpace;
use super::{BONE_HINGE, BONE_NO_LOCAL_LOCATION, BRig, BRigBone};
use glam::{Mat3, Mat4, Vec3};

/// `CONSTRAINT_SPACE_*`.
pub const SPACE_WORLD: u8 = 0;
pub const SPACE_LOCAL: u8 = 1;
pub const SPACE_POSE: u8 = 2;
pub const SPACE_PARLOCAL: u8 = 3;
pub const SPACE_CUSTOM: u8 = 5;
pub const SPACE_OWNLOCAL: u8 = 6;

/// `BONE_INHERIT_SCALE_*`.
const SCALE_FULL: u8 = 0;
const SCALE_FIX_SHEAR: u8 = 1;
const SCALE_AVERAGE: u8 = 2;
const SCALE_NONE: u8 = 3;
const SCALE_NONE_LEGACY: u8 = 4;
const SCALE_ALIGNED: u8 = 5;

/// `BoneParentTransform`: what carries a bone's own matrix into pose space.
#[derive(Clone, Copy, Debug)]
pub struct BParentTransform {
    pub rotscale: Mat4,
    pub loc: Mat4,
    pub post_scale: Vec3,
}

impl BParentTransform {
    /// `BKE_bone_offset_matrix_get`: the bone's rest relative to its
    /// parent's tail.
    fn offset(bone: &BRigBone, parent: &BRigBone) -> Mat4 {
        let mut m = Mat4::from_mat3(bone.bone_mat);
        m.set_loc(bone.head + Vec3::new(0.0, parent.length, 0.0));
        m
    }

    /// `BKE_bone_parent_transform_calc_from_pchan`, with the parent's pose
    /// as it stands in `pose`.
    pub fn of(bones: &[BRigBone], b: usize, pose: &[Mat4]) -> Self {
        let bone = &bones[b];
        match bone.parent {
            Some(p) => Self::from_matrices(bone.flag, bone.inherit_scale, Self::offset(bone, &bones[p]), Some((bones[p].arm, pose[p]))),
            None => Self::from_matrices(bone.flag, bone.inherit_scale, bone.arm, None),
        }
    }

    /// `BKE_bone_parent_transform_calc_from_matrices`.
    pub fn from_matrices(flag: u32, mode: u8, offs: Mat4, parent: Option<(Mat4, Mat4)>) -> Self {
        let mut post_scale = Vec3::ONE;
        let Some((parent_arm, parent_pose)) = parent else {
            let loc = if flag & BONE_NO_LOCAL_LOCATION != 0 { Mat4::from_translation(offs.loc()) } else { offs };
            return Self { rotscale: offs, loc, post_scale };
        };
        let use_rotation = flag & BONE_HINGE == 0;
        let full = use_rotation && mode == SCALE_FULL;
        let rotscale = if full {
            parent_pose * offs
        } else {
            let mut t = if use_rotation {
                let mut t = parent_pose;
                match mode {
                    SCALE_NONE | SCALE_AVERAGE => t.orthogonalize_stable(1, true),
                    SCALE_ALIGNED => {
                        t.orthogonalize_stable(1, false);
                        post_scale = t.normalize_ex();
                    }
                    SCALE_NONE_LEGACY => _ = t.normalize_ex(),
                    _ => {}
                }
                t
            } else {
                let mut t = parent_arm;
                match mode {
                    SCALE_FULL => t.rescale(parent_pose.size()),
                    SCALE_FIX_SHEAR => t.rescale(parent_pose.size_fix_shear()),
                    SCALE_ALIGNED => post_scale = parent_pose.size_fix_shear(),
                    _ => {}
                }
                t
            };
            if mode == SCALE_AVERAGE {
                let s = parent_pose.volume().abs().cbrt();
                t = t.with_m3(Mat3::from_mat4(t) * s);
            }
            let mut r = t * offs;
            if mode == SCALE_FIX_SHEAR {
                r.orthogonalize_stable(1, false);
            }
            r
        };
        let loc = if flag & BONE_NO_LOCAL_LOCATION != 0 {
            let mut m = Mat4::from_mat3(Mat3::from_mat4(parent_pose));
            m.set_loc(parent_pose.transform_point3(offs.loc()));
            m
        } else if !full {
            parent_pose * offs
        } else {
            rotscale
        };
        Self { rotscale, loc, post_scale }
    }

    /// `BKE_bone_parent_transform_apply`.
    pub fn apply(&self, m: Mat4) -> Mat4 {
        let mut out = self.rotscale * m;
        out.set_loc(self.loc.transform_point3(m.loc()));
        out.rescale(self.post_scale);
        out
    }

    /// `BKE_bone_parent_transform_invert` (`invert_v3_safe` for the scale).
    pub fn inverse(&self) -> Self {
        let post_scale = self.post_scale.to_array().map(|v| if v != 0.0 { 1.0 / v } else { 0.0 });
        Self { rotscale: self.rotscale.inverse(), loc: self.loc.inverse(), post_scale: Vec3::from_array(post_scale) }
    }
}

/// The owner a space conversion serves: its bone's rest matrix (for
/// owner-local space) and the custom space's world matrix.
#[derive(Clone, Copy, Debug)]
pub struct BSpaceOwner {
    /// `Bone.arm_mat` of the owning bone; `None` for an object.
    pub arm: Option<Mat4>,
    pub custom: Mat4,
}

/// A rig's spaces as constraints convert between them — its bones, their
/// poses so far (armature space) and its object — for the rig being solved
/// or another one a constraint reads a bone of.
#[derive(Clone, Copy)]
pub struct BRigSpace<'a> {
    pub rig: &'a BRig,
    pub pose: &'a [Mat4],
    /// The rig object's world matrix and its inverse.
    pub ob: Mat4,
    pub ob_inv: Mat4,
    /// The rig object's local space, when known ([`BObjectSpace`]).
    pub basis: Option<Mat4>,
}

impl BPoseWork<'_> {
    /// This rig's spaces, as the pose stands so far.
    pub fn space(&self) -> BRigSpace<'_> {
        BRigSpace { rig: self.rig, pose: &self.pose, ob: self.ob, ob_inv: self.ob_inv, basis: self.others.basis(self.rig.object) }
    }

    /// `BKE_constraint_mat_convertspace` for a bone of this rig, or for the
    /// rig object itself when `pchan` is `None`.
    pub fn convert(&self, pchan: Option<usize>, owner: &BSpaceOwner, mat: Mat4, from: u8, to: u8) -> Mat4 {
        self.space().convert(pchan, owner, mat, from, to)
    }
}

impl BRigSpace<'_> {
    /// `BKE_armature_mat_bone_to_pose`.
    pub fn bone_to_pose(&self, b: usize, m: Mat4) -> Mat4 {
        BParentTransform::of(&self.rig.bones, b, self.pose).apply(m)
    }

    /// `BKE_armature_mat_pose_to_bone`.
    pub fn pose_to_bone(&self, b: usize, m: Mat4) -> Mat4 {
        BParentTransform::of(&self.rig.bones, b, self.pose).inverse().apply(m)
    }

    /// `constraint_target_to_mat4` for bone `b`: its pose in world space,
    /// `headtail` of the way to its tail, in `space`.
    pub fn bone_target(&self, b: usize, owner: &BSpaceOwner, space: u8, headtail: f32) -> Mat4 {
        let pose = self.pose[b];
        let mut m = pose;
        if headtail >= 0.000_001 {
            let tail = pose.loc() + pose.y_axis.truncate() * self.rig.bones[b].length;
            m.set_loc(pose.loc().lerp(tail, headtail));
        }
        self.convert(Some(b), owner, self.ob * m, SPACE_WORLD, space)
    }

    /// Bone `b`'s deform in world space (`ob · chan_mat · ob⁻¹`, the
    /// channel's `pose · rest⁻¹`), what the Armature constraint blends.
    pub fn deform(&self, b: usize) -> Mat4 {
        self.ob * self.pose[b] * self.rig.bones[b].arm.inverse() * self.ob_inv
    }

    /// `BKE_constraint_mat_convertspace` for bone `pchan`, or for the rig
    /// object when `None`.
    pub fn convert(&self, pchan: Option<usize>, owner: &BSpaceOwner, mat: Mat4, from: u8, to: u8) -> Mat4 {
        if from == to {
            return mat;
        }
        let Some(b) = pchan else { return BObjectSpace { basis: self.basis, custom: owner.custom }.convert(mat, from, to) };
        let arm = self.rig.bones[b].arm;
        match from {
            SPACE_WORLD if to == SPACE_CUSTOM => owner.custom.inverse() * mat,
            SPACE_WORLD => {
                let m = self.ob_inv * mat;
                if matches!(to, SPACE_LOCAL | SPACE_PARLOCAL | SPACE_OWNLOCAL) { self.convert(pchan, owner, m, SPACE_POSE, to) } else { m }
            }
            SPACE_POSE => match to {
                SPACE_LOCAL => self.pose_to_bone(b, mat),
                SPACE_OWNLOCAL => {
                    let m = self.pose_to_bone(b, mat);
                    self.convert(pchan, owner, m, SPACE_LOCAL, to)
                }
                SPACE_PARLOCAL => arm.inverse() * mat,
                _ => {
                    let m = self.ob * mat;
                    if to != SPACE_WORLD { self.convert(pchan, owner, m, SPACE_WORLD, to) } else { m }
                }
            },
            SPACE_LOCAL => {
                if to == SPACE_OWNLOCAL {
                    let d = self.own_local_diff(b, owner);
                    d * mat * d.inverse()
                } else {
                    let m = self.bone_to_pose(b, mat);
                    if matches!(to, SPACE_WORLD | SPACE_PARLOCAL | SPACE_CUSTOM) { self.convert(pchan, owner, m, SPACE_POSE, to) } else { m }
                }
            }
            SPACE_OWNLOCAL => {
                let d = self.own_local_diff(b, owner);
                let m = d.inverse() * mat * d;
                if to != SPACE_LOCAL { self.convert(pchan, owner, m, SPACE_LOCAL, to) } else { m }
            }
            SPACE_PARLOCAL => {
                let m = arm * mat;
                if matches!(to, SPACE_WORLD | SPACE_LOCAL | SPACE_OWNLOCAL | SPACE_CUSTOM) { self.convert(pchan, owner, m, SPACE_POSE, to) } else { m }
            }
            SPACE_CUSTOM => {
                let m = owner.custom * mat;
                if to != SPACE_WORLD { self.convert(pchan, owner, m, SPACE_WORLD, to) } else { m }
            }
            _ => mat,
        }
    }

    /// The owner-local difference: `bone.arm_mat` in the owner bone's rest,
    /// its translation dropped.
    fn own_local_diff(&self, b: usize, owner: &BSpaceOwner) -> Mat4 {
        let mut d = self.rig.bones[b].arm;
        if let Some(arm) = owner.arm {
            d = arm.inverse() * d;
        }
        d.set_loc(Vec3::ZERO);
        d
    }
}
