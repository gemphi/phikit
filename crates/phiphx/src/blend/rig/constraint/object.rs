//! An object's constraints (`BKE_object_where_is_calc`'s last step,
//! `CONSTRAINT_OBTYPE_OBJECT`): the object's world after its parent goes
//! in, its constrained world comes out. Its targets are other objects and
//! other rigs' bones, read through the scene around it; its local space is
//! its parent's world through `parentinv`, or its own rotation.

use super::{BCob, BConstraint, BConstraintHost, BConstraintKind, BConstraintStack, BTarget};
use crate::blend::anim::fcurve::BFCurve;
use crate::blend::rig::driver::{BRnaPath, BRnaStep};
use crate::blend::rig::space::BSpaceOwner;
use crate::blend::rig::view::{BObjectSpace, BSceneView};
use crate::blend::rig::space::{SPACE_CUSTOM, SPACE_WORLD};
use glam::{Mat4, Vec3};

/// One object's constraints being solved.
pub struct BObjectWork<'a> {
    /// What the owner's local space is taken against
    /// ([`BObjectSpace::basis_of`]).
    pub basis: Mat4,
    /// The scene around it, solved as far as its targets.
    pub scene: &'a dyn BSceneView,
}

impl BObjectWork<'_> {
    /// `constraints` over an object standing at `world` with rotation mode
    /// `rotmode`, `curves` its action's (for animated influences and
    /// mutes): its world after them.
    pub fn solve(&self, constraints: &[BConstraint], curves: &[BFCurve], frame: f32, world: Mat4, rotmode: i16) -> Mat4 {
        let (influence, muted) = Self::settings(constraints, curves, frame);
        let mut cob = BCob { bone: None, matrix: world, rot_order: if rotmode > 0 { rotmode } else { 1 }, custom: Mat4::IDENTITY };
        BConstraintStack { list: constraints, influence: &influence, muted: &muted, weights: &[] }.solve(self, &mut cob);
        cob.matrix
    }

    /// Where each open constraint's relationship line runs from
    /// (`overlay_relation.hh`): every target's place as the constraint
    /// reads it (`get_target_matrix`, in its target space) — a Follow Path
    /// curve's point on its path, an Armature constraint's bones at their
    /// heads — and its custom space's object. Clamp To reports no target
    /// matrix, and a folded constraint draws no line, as in Blender.
    pub fn target_points(&self, constraints: &[BConstraint]) -> Vec<Vec3> {
        let mut out = Vec::new();
        for con in constraints.iter().filter(|c| c.expanded) {
            let custom = BConstraintStack::custom_space(self, con);
            let owner = BSpaceOwner { arm: None, custom };
            match &con.kind {
                BConstraintKind::FollowPath { target: BTarget::Object(a), flag, offset, offset_fac, track, up } => {
                    let on_path = self.scene.path(*a).and_then(|p| p.follow_matrix(*flag, *offset, *offset_fac, *track, *up));
                    out.extend(self.scene.world(*a).zip(on_path).map(|(world, m)| (world * m).w_axis.truncate()));
                }
                BConstraintKind::ClampTo { .. } | BConstraintKind::FollowPath { .. } | BConstraintKind::Action { .. } => {}
                BConstraintKind::Armature { targets, .. } => {
                    out.extend(targets.iter().filter_map(|t| self.target_matrix(&t.0, &owner, SPACE_WORLD, 0.0)).map(|m| m.w_axis.truncate()));
                }
                kind => out.extend(kind.targets().into_iter().filter_map(|t| self.target_matrix(t, &owner, con.tarspace, con.headtail)).map(|m| m.w_axis.truncate())),
            }
            if (con.ownspace == SPACE_CUSTOM || con.tarspace == SPACE_CUSTOM) && con.space != BTarget::None {
                out.push(custom.w_axis.truncate());
            }
        }
        out
    }

    /// Each constraint's influence and mute at `frame`: as saved, or as a
    /// curve on `constraints["Name"].influence` / `.mute` has it (a
    /// boolean on at one half, `ANIMSYS_FLOAT_AS_BOOL`).
    fn settings(constraints: &[BConstraint], curves: &[BFCurve], frame: f32) -> (Vec<f32>, Vec<bool>) {
        let mut influence: Vec<f32> = constraints.iter().map(|c| c.influence).collect();
        let mut muted: Vec<bool> = constraints.iter().map(|c| c.flag & super::CONSTRAINT_OFF != 0).collect();
        for c in curves.iter().filter(|c| !c.muted()) {
            let Some(path) = BRnaPath::parse(&c.path) else { continue };
            let [BRnaStep::Field(field), BRnaStep::Key(name), BRnaStep::Field(property)] = path.0.as_slice() else { continue };
            let (Some(i), Some(v)) = (constraints.iter().position(|con| con.name == *name), c.eval(frame)) else { continue };
            match property.as_str() {
                "influence" if field == "constraints" => influence[i] = v.clamp(0.0, 1.0),
                "mute" if field == "constraints" => muted[i] = v >= 0.5,
                _ => {}
            }
        }
        (influence, muted)
    }
}

impl BConstraintHost for BObjectWork<'_> {
    fn owner(&self, cob: &BCob) -> BSpaceOwner {
        BSpaceOwner { arm: None, custom: cob.custom }
    }

    fn target_matrix(&self, target: &BTarget, owner: &BSpaceOwner, space: u8, headtail: f32) -> Option<Mat4> {
        self.scene.target(target, owner, space, headtail)
    }

    fn convert_owner(&self, cob: &BCob, mat: Mat4, from: u8, to: u8) -> Mat4 {
        BObjectSpace { basis: Some(self.basis), custom: cob.custom }.convert(mat, from, to)
    }

    fn rot_order_of(&self, target: &BTarget) -> i16 {
        self.scene.target_rot_order(target)
    }

    fn deform_of(&self, target: &BTarget) -> Option<Mat4> {
        self.scene.target_deform(target)
    }

    fn inverse_base(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    fn scene(&self) -> &dyn BSceneView {
        self.scene
    }
}
