//! What a constraint sees outside its owner ([`BSceneView`]): other
//! objects' world matrices, local spaces and rotation modes, and other
//! rigs' poses — and the object half of `BKE_constraint_mat_convertspace`
//! ([`BObjectSpace`]).

use super::constraint::BTarget;
use crate::blend::curve::BPathAt;
use super::math::BMat4;
use super::space::{BRigSpace, BSpaceOwner, SPACE_CUSTOM, SPACE_LOCAL, SPACE_WORLD};
use glam::{Mat4, Vec3};

/// The scene around a constraint's owner, by object block address.
pub trait BSceneView {
    /// An object's world matrix (`object_to_world`), when the scene has it.
    fn world(&self, address: u64) -> Option<Mat4>;

    /// What the object's local space is taken against
    /// ([`BObjectSpace::basis_of`]); `None` reads its local space as world.
    fn basis(&self, _address: u64) -> Option<Mat4> {
        None
    }

    /// The object's `rotmode`: a target's `rotOrder`.
    fn rotmode(&self, _address: u64) -> i16 {
        1
    }

    /// An armature object's rig, posed at this frame.
    fn rig(&self, _address: u64) -> Option<BRigSpace<'_>> {
        None
    }

    /// A curve object's path, at this frame's evaluation time.
    fn path(&self, _address: u64) -> Option<BPathAt<'_>> {
        None
    }

    /// `constraint_target_to_mat4` for a target outside the owner's rig: an
    /// object, or a bone of another rig — the object itself when it has no
    /// such bone, as Blender falls back.
    fn target(&self, target: &BTarget, owner: &BSpaceOwner, space: u8, headtail: f32) -> Option<Mat4> {
        let object = |a: u64| {
            let world = self.world(a).unwrap_or(Mat4::IDENTITY);
            BObjectSpace { basis: self.basis(a), custom: owner.custom }.convert(world, SPACE_WORLD, space)
        };
        match target {
            BTarget::Object(a) => Some(object(*a)),
            BTarget::Bone(a, name) => match self.rig(*a).and_then(|r| r.rig.bone(name).map(|b| (r, b))) {
                Some((rig, b)) => Some(rig.bone_target(b, owner, space, headtail)),
                None => Some(object(*a)),
            },
            BTarget::None | BTarget::Rig(_) => None,
        }
    }

    /// A target's `rotOrder` outside the owner's rig: the object's rotation
    /// mode, or the bone channel's.
    fn target_rot_order(&self, target: &BTarget) -> i16 {
        match target {
            BTarget::Object(a) => self.rotmode(*a),
            BTarget::Bone(a, name) => match self.rig(*a).and_then(|r| r.rig.bone(name).map(|b| r.rig.bones[b].raw.rotmode)) {
                Some(mode) => mode,
                None => self.rotmode(*a),
            },
            BTarget::None | BTarget::Rig(_) => 1,
        }
    }

    /// Another rig's bone deform in world space (the Armature constraint).
    fn target_deform(&self, target: &BTarget) -> Option<Mat4> {
        let BTarget::Bone(a, name) = target else { return None };
        let rig = self.rig(*a)?;
        rig.rig.bone(name).map(|b| rig.deform(b))
    }
}

/// A closure from address to world matrix is a view with nothing else.
impl<F: Fn(u64) -> Option<Mat4>> BSceneView for F {
    fn world(&self, address: u64) -> Option<Mat4> {
        self(address)
    }
}

/// A view of nothing: no other object is in the scene.
pub struct BEmptyScene;

impl BSceneView for BEmptyScene {
    fn world(&self, _address: u64) -> Option<Mat4> {
        None
    }
}

/// An object's spaces for a constraint: world; local, against `basis`; and
/// the custom space's object (`custom`, its world matrix).
pub struct BObjectSpace {
    pub basis: Option<Mat4>,
    pub custom: Mat4,
}

impl BObjectSpace {
    /// What an object's local space is taken against: its parent's world
    /// through `parentinv`, or — with no parent — its own local matrix with
    /// the axes normalized and no location ("local to the owner's
    /// transform-property-rotated axes").
    pub fn basis_of(parent: Option<(Mat4, Mat4)>, local: Mat4) -> Mat4 {
        match parent {
            Some((world, parentinv)) => world * parentinv,
            None => {
                let mut m = local;
                m.normalize_ex();
                m.set_loc(Vec3::ZERO);
                m
            }
        }
    }

    /// The object branch of `BKE_constraint_mat_convertspace`: the pose
    /// spaces are a bone's, and leave an object's matrix as it is.
    pub fn convert(&self, mat: Mat4, from: u8, to: u8) -> Mat4 {
        if from == to {
            return mat;
        }
        let basis = self.basis.unwrap_or(Mat4::IDENTITY);
        match from {
            SPACE_WORLD => match to {
                SPACE_LOCAL => basis.inverse_safe() * mat,
                SPACE_CUSTOM => self.custom.inverse_safe() * mat,
                _ => mat,
            },
            SPACE_LOCAL => {
                let m = basis * mat;
                if to == SPACE_CUSTOM { self.custom.inverse_safe() * m } else { m }
            }
            SPACE_CUSTOM => {
                let m = self.custom * mat;
                if to != SPACE_WORLD { self.convert(m, SPACE_WORLD, to) } else { m }
            }
            _ => mat,
        }
    }
}
