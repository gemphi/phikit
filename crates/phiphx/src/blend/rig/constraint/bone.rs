//! A bone's constraints: the rig being posed as their host — its bones and
//! object as targets, the bone's spaces, the rig state's animated
//! influences, mutes and weights, and the Action constraint.

use super::{BCob, BConstraint, BConstraintHost, BConstraintKind, BConstraintStack, BTarget};
use crate::blend::rig::solve::BPoseWork;
use crate::blend::rig::space::{BSpaceOwner, SPACE_WORLD};
use crate::blend::rig::view::BSceneView;
use glam::Mat4;

impl BPoseWork<'_> {
    /// `constraint_target_to_mat4`: a target's matrix in world space, then
    /// in `space`, `headtail` along a target bone.
    pub fn target_matrix(&self, target: &BTarget, owner: &BSpaceOwner, space: u8, headtail: f32) -> Option<Mat4> {
        match target {
            BTarget::None => None,
            BTarget::Rig(None) => Some(self.convert(None, owner, self.ob, SPACE_WORLD, space)),
            BTarget::Rig(Some(b)) => Some(self.space().bone_target(*b, owner, space, headtail)),
            BTarget::Object(_) | BTarget::Bone(..) => self.others.target(target, owner, space, headtail),
        }
    }

    /// `bConstraintTarget.rotOrder`: a target bone's rotation mode, or the
    /// target object's.
    pub fn rot_order_of(&self, target: &BTarget) -> i16 {
        match target {
            BTarget::Rig(Some(b)) => self.state.raw[*b].rotmode,
            BTarget::Rig(None) => self.others.rotmode(self.rig.object),
            _ => self.others.target_rot_order(target),
        }
    }

    /// `BKE_constraints_solve` over bone `b`'s constraints: `cob.matrix` in
    /// world space goes in and comes out.
    pub fn solve_constraints(&self, cob: &mut BCob, b: usize) {
        let stack = BConstraintStack {
            list: &self.rig.bones[b].constraints,
            influence: &self.state.influence[b],
            muted: &self.state.muted[b],
            weights: &self.state.weights[b],
        };
        stack.solve(self, cob);
    }
}

impl BConstraintHost for BPoseWork<'_> {
    fn owner(&self, cob: &BCob) -> BSpaceOwner {
        BSpaceOwner { arm: cob.bone.map(|b| self.rig.bones[b].arm), custom: cob.custom }
    }

    fn target_matrix(&self, target: &BTarget, owner: &BSpaceOwner, space: u8, headtail: f32) -> Option<Mat4> {
        BPoseWork::target_matrix(self, target, owner, space, headtail)
    }

    fn convert_owner(&self, cob: &BCob, mat: Mat4, from: u8, to: u8) -> Mat4 {
        self.convert(cob.bone, &self.owner(cob), mat, from, to)
    }

    fn rot_order_of(&self, target: &BTarget) -> i16 {
        BPoseWork::rot_order_of(self, target)
    }

    fn deform_of(&self, target: &BTarget) -> Option<Mat4> {
        match target {
            BTarget::Rig(Some(b)) => Some(self.ob * self.chan[*b] * self.ob_inv),
            _ => self.others.target_deform(target),
        }
    }

    fn inverse_base(&self) -> Mat4 {
        self.ob
    }

    fn scene(&self) -> &dyn BSceneView {
        self.others
    }

    fn own_kind(&self, con: &BConstraint, cob: &mut BCob) {
        if let (Some(b), BConstraintKind::Action { target, data }) = (cob.bone, &con.kind) {
            self.action(cob, b, con, target, data);
        }
    }
}
