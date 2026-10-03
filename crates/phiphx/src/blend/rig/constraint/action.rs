//! The Action constraint (`actcon_get_tarmat`, `actcon_evaluate`): another
//! action's pose of this bone, at a frame picked by one of the target's
//! channels (or a fixed evaluation time), mixed into the owner.

use super::{BCob, BConstraint, BConstraintHost, BTarget};
use crate::blend::anim::fcurve::BFCurve;
use crate::blend::object::transform::ObjectTransformRaw;
use crate::blend::rig::driver::{BRnaPath, BRnaStep};
use crate::blend::rig::math::{BEulerOrder, BMat4};
use crate::blend::rig::solve::{BChannelValue, BPoseWork};
use crate::blend::rig::BONE_CONNECTED;
use glam::Mat4;

/// `ACTCON_BONE_USE_OBJECT_ACTION`, `ACTCON_USE_EVAL_TIME`.
const USE_OBJECT_ACTION: u32 = 1 << 0;
const USE_EVAL_TIME: u32 = 1 << 1;
/// `ACTCON_MIX_*` (`ACTCON_MIX_AFTER_FULL` is the rest).
const MIX_AFTER: u32 = 1;
const MIX_BEFORE: u32 = 2;
const MIX_BEFORE_FULL: u32 = 3;
const MIX_AFTER_SPLIT: u32 = 4;
const MIX_BEFORE_SPLIT: u32 = 5;
const MIX_REPLACE: u32 = 6;

/// `bActionConstraint`: `channel` the target channel (`type`: 0–2 its
/// rotation in degrees, 10–12 its scale, 20–22 its location), mapped from
/// `min..max` onto the action's `start..end`.
#[derive(Clone, Debug, PartialEq)]
pub struct BActionData {
    pub channel: i16,
    pub start: f32,
    pub end: f32,
    pub min: f32,
    pub max: f32,
    pub flag: u32,
    pub mix: u32,
    pub eval_time: f32,
    /// Whether the constraint names an action at all.
    pub has_action: bool,
    /// The action's pose-bone curves, for the slot the constraint names.
    pub curves: Vec<BFCurve>,
}

impl BPoseWork<'_> {
    /// `actcon_get_tarmat` then `actcon_evaluate`.
    pub(super) fn action(&self, cob: &mut BCob, bone: usize, con: &BConstraint, target: &BTarget, data: &BActionData) {
        let use_time = data.flag & USE_EVAL_TIME != 0;
        // An action applied to the rig object rather than the bone is not
        // evaluated here (`what_does_obaction` on a work object).
        if (*target == BTarget::None && !use_time) || data.flag & USE_OBJECT_ACTION != 0 {
            return;
        }
        let ct = match data.has_action {
            true => match self.action_frame(cob, con, target, data) {
                Some(t) => self.action_pose(bone, data, t),
                None => return,
            },
            false => Mat4::IDENTITY,
        };
        let own = cob.matrix;
        cob.matrix = match data.mix {
            MIX_REPLACE => ct,
            MIX_BEFORE_FULL => ct * own,
            MIX_AFTER => BCob::aligned_scale(&own, &ct),
            MIX_BEFORE => BCob::aligned_scale(&ct, &own),
            MIX_AFTER_SPLIT => BCob::split_channels(&own, &ct),
            MIX_BEFORE_SPLIT => BCob::split_channels(&ct, &own),
            _ => own * ct,
        };
    }

    /// The action frame the target's channel (or the evaluation time)
    /// picks.
    fn action_frame(&self, cob: &BCob, con: &BConstraint, target: &BTarget, data: &BActionData) -> Option<f32> {
        let s = if data.flag & USE_EVAL_TIME != 0 {
            data.eval_time
        } else {
            let m = BPoseWork::target_matrix(self, target, &self.owner(cob), con.tarspace, con.headtail)?;
            let (vec, axis) = match data.channel {
                c if c < 10 => (BEulerOrder(1).of_mat4(&m) * 180.0 / std::f32::consts::PI, c),
                c if c < 20 => (m.size(), c - 10),
                c => (m.loc(), c - 20),
            };
            let range = data.max - data.min;
            if range == 0.0 || !(0..=2).contains(&axis) { 0.0 } else { (vec[axis as usize] - data.min) / range }
        };
        Some(s.clamp(0.0, 1.0) * (data.end - data.start) + data.start)
    }

    /// The bone's channels as the action has them at `frame`, on a fresh
    /// channel with the bone's rotation mode (`BKE_pchan_calc_mat`).
    fn action_pose(&self, b: usize, data: &BActionData, frame: f32) -> Mat4 {
        let bone = &self.rig.bones[b];
        let mut raw = ObjectTransformRaw { rotmode: self.state.raw[b].rotmode, ..Default::default() };
        for c in data.curves.iter().filter(|c| !c.muted()) {
            let Some(path) = BRnaPath::parse(&c.path) else { continue };
            if let [BRnaStep::Field(p), BRnaStep::Field(bs), BRnaStep::Key(name), BRnaStep::Field(property)] = path.0.as_slice()
                && p == "pose"
                && bs == "bones"
                && *name == bone.name
                && let Some(v) = c.eval(frame)
            {
                raw.set(property, c.index as usize, v);
            }
        }
        Self::channel_matrix(&raw, bone.flag & BONE_CONNECTED != 0)
    }
}
