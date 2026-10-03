//! `BKE_constraints_solve` for any owner: each constraint of its stack in
//! turn, in its own space, blended into the owner by its influence
//! (`interp_m4_m4m4`). What a bone and an object differ in — where the
//! targets are, what the owner's spaces are — is its [`BConstraintHost`].

use super::{BConstraint, BConstraintKind, BCob, BTarget, CONSTRAINT_DISABLE, CONSTRAINT_OFF, CONSTRAINT_SPACEONCE};
use crate::blend::rig::math::BMat4;
use crate::blend::rig::space::{BSpaceOwner, SPACE_CUSTOM, SPACE_WORLD};
use crate::blend::rig::view::BSceneView;
use glam::Mat4;

/// Who a constraint stack belongs to (`bConstraintOb`'s `ob` and `pchan`).
pub trait BConstraintHost {
    /// The owner as a space conversion serves it.
    fn owner(&self, cob: &BCob) -> BSpaceOwner;
    /// `constraint_target_to_mat4`: a target in `space`.
    fn target_matrix(&self, target: &BTarget, owner: &BSpaceOwner, space: u8, headtail: f32) -> Option<Mat4>;
    /// `BKE_constraint_mat_convertspace` for the owner itself.
    fn convert_owner(&self, cob: &BCob, mat: Mat4, from: u8, to: u8) -> Mat4;
    /// A target's `rotOrder`.
    fn rot_order_of(&self, target: &BTarget) -> i16;
    /// A target bone's deform in world space (the Armature constraint).
    fn deform_of(&self, target: &BTarget) -> Option<Mat4>;
    /// What a Set Inverse still pending in the file multiplies in: the rig
    /// object's world for a bone, nothing for an object.
    fn inverse_base(&self) -> Mat4;
    /// The kinds only this host evaluates (the Action constraint on a bone).
    fn own_kind(&self, _con: &BConstraint, _cob: &mut BCob) {}
    /// The scene around the owner.
    fn scene(&self) -> &dyn BSceneView;
}

/// One owner's constraints as a frame has them: influence and mute as
/// animated, and the Armature constraints' target weights (empty: as
/// saved).
pub struct BConstraintStack<'a> {
    pub list: &'a [BConstraint],
    pub influence: &'a [f32],
    pub muted: &'a [bool],
    pub weights: &'a [Vec<f32>],
}

impl BConstraintStack<'_> {
    /// `BKE_constraints_solve`: `cob.matrix` in world space goes in and
    /// comes out.
    pub fn solve(&self, host: &impl BConstraintHost, cob: &mut BCob) {
        for (i, con) in self.list.iter().enumerate() {
            let influence = self.influence.get(i).copied().unwrap_or(con.influence);
            let muted = self.muted.get(i).copied().unwrap_or(con.flag & CONSTRAINT_OFF != 0);
            if con.flag & CONSTRAINT_DISABLE != 0 || muted || influence == 0.0 {
                continue;
            }
            if matches!(con.kind, BConstraintKind::Ik(_) | BConstraintKind::Other(_)) {
                continue;
            }
            cob.custom = Self::custom_space(host, con);
            let old = cob.matrix;
            cob.matrix = host.convert_owner(cob, cob.matrix, SPACE_WORLD, con.ownspace);
            self.evaluate(host, con, i, cob);
            // Child Of sets `CONSTRAINT_SPACEONCE` itself when it has a
            // target and clears it when it has none (`childof_evaluate`).
            let once = match &con.kind {
                BConstraintKind::ChildOf { target, .. } => *target != BTarget::None,
                _ => con.flag & CONSTRAINT_SPACEONCE != 0,
            };
            if !once {
                cob.matrix = host.convert_owner(cob, cob.matrix, con.ownspace, SPACE_WORLD);
            }
            if influence < 1.0 {
                cob.matrix = old.interp(&cob.matrix, influence);
            }
        }
    }

    /// `BKE_constraint_custom_object_space_init`.
    pub(super) fn custom_space(host: &impl BConstraintHost, con: &BConstraint) -> Mat4 {
        if con.ownspace != SPACE_CUSTOM && con.tarspace != SPACE_CUSTOM {
            return Mat4::IDENTITY;
        }
        let owner = BSpaceOwner { arm: None, custom: Mat4::IDENTITY };
        host.target_matrix(&con.space, &owner, SPACE_WORLD, 0.0).unwrap_or(Mat4::IDENTITY)
    }

    /// One constraint's `evaluate_constraint`, its target in `tarspace`.
    fn evaluate(&self, host: &impl BConstraintHost, con: &BConstraint, index: usize, cob: &mut BCob) {
        let owner = host.owner(cob);
        let target = |t: &BTarget| host.target_matrix(t, &owner, con.tarspace, con.headtail);
        match &con.kind {
            BConstraintKind::CopyTransforms { target: t, mix, flag } => {
                if let Some(ct) = target(t) {
                    BCob::copy_transforms(cob, ct, *mix, *flag);
                }
            }
            BConstraintKind::CopyLocation { target: t, flag } => {
                if let Some(ct) = target(t) {
                    BCob::copy_location(cob, ct, *flag);
                }
            }
            BConstraintKind::CopyRotation { target: t, flag, order, mix } => {
                if let Some(ct) = target(t) {
                    BCob::copy_rotation(cob, ct, *flag, *order, *mix);
                }
            }
            BConstraintKind::CopyScale { target: t, flag, power } => {
                if let Some(ct) = target(t) {
                    BCob::copy_scale(cob, ct, *flag, *power);
                }
            }
            BConstraintKind::DampedTrack { target: t, track } => {
                if let Some(ct) = target(t) {
                    let v = ct.loc() - cob.matrix.loc();
                    BCob::damped_track(&mut cob.matrix, v, *track);
                }
            }
            BConstraintKind::StretchTo { target: t, .. } => {
                if let Some(ct) = target(t) {
                    BCob::stretch_to(cob, ct, &con.kind);
                }
            }
            BConstraintKind::LimitRotation { min, max, flag, order } => BCob::limit_rotation(cob, *min, *max, *flag, *order),
            BConstraintKind::LimitDistance { target: t, dist, soft, flag, mode } => {
                if let Some(ct) = target(t) {
                    BCob::limit_distance(cob, ct, *dist, *soft, *flag, *mode);
                }
            }
            BConstraintKind::Armature { targets, .. } => {
                let saved: Vec<f32> = targets.iter().map(|t| t.1).collect();
                let weights = self.weights.get(index).map_or(saved.as_slice(), Vec::as_slice);
                BCob::armature(cob, targets, weights, |t| host.deform_of(t));
            }
            BConstraintKind::ChildOf { target: t, flag, invmat } => {
                if let Some(ct) = target(t) {
                    BCob::child_of(cob, ct, host.rot_order_of(t), *flag, *invmat, host.inverse_base());
                }
            }
            BConstraintKind::TrackTo { target: t, track, up, flags } => {
                if let Some(ct) = target(t) {
                    BCob::track_to(cob, ct, *track, *up, *flags);
                }
            }
            BConstraintKind::LockedTrack { target: t, track, lock } => {
                if let Some(ct) = target(t) {
                    BCob::locked_track(cob, ct, *track, *lock);
                }
            }
            BConstraintKind::LimitLocation { min, max, flag } => BCob::limit_location(cob, *min, *max, *flag),
            BConstraintKind::LimitScale { min, max, flag } => BCob::limit_scale(cob, *min, *max, *flag),
            BConstraintKind::MaintainVolume { axis, mode, volume } => BCob::maintain_volume(cob, *axis, *mode, *volume),
            BConstraintKind::Transformation { target: t, data } => {
                if let Some(ct) = target(t) {
                    data.evaluate(cob, ct);
                }
            }
            BConstraintKind::Floor { target: t, axis, offset, flag } => {
                if let Some(ct) = target(t) {
                    BCob::floor(cob, ct, *axis, *offset, *flag);
                }
            }
            BConstraintKind::Pivot { target: t, offset, axis, flag } => BCob::pivot(cob, target(t), *offset, *axis, *flag),
            BConstraintKind::FollowPath { target: BTarget::Object(a), flag, offset, offset_fac, track, up } => {
                let scene = host.scene();
                let on_path = scene.path(*a).and_then(|p| p.follow_matrix(*flag, *offset, *offset_fac, *track, *up));
                if let (Some(world), Some(m)) = (scene.world(*a), on_path) {
                    BCob::follow_path(cob, world * m, *flag);
                }
            }
            BConstraintKind::FollowPath { .. } => {}
            BConstraintKind::ClampTo { target: BTarget::Object(a), axis, cyclic } => {
                let scene = host.scene();
                if let (Some(world), Some(at)) = (scene.world(*a), scene.path(*a)) {
                    BCob::clamp_to(cob, world, at.path, *axis, *cyclic);
                }
            }
            BConstraintKind::ClampTo { .. } => {}
            BConstraintKind::Action { .. } => host.own_kind(con, cob),
            BConstraintKind::Ik(_) | BConstraintKind::Other(_) => {}
        }
    }
}
