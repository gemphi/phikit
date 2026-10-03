//! The NLA stacks the session edited (`ctx.scene.nla`): each object's
//! stack as Push Down, Stash (`action.push_down`, `action.stash`, the
//! Action Editor's Action menu) and the NLA editor's operators
//! (`nla_editor.rs`) left it, kept whole — tracks and strips with their
//! flags and the file blocks they came from — so Undo steps them, the NLA
//! editor and the viewport read them, and a save writes each stack as it
//! stands (phiphx `write/nla.rs`, checked in Blender by
//! `blender_opens_saves`). phiphx lays the strips (`anim/nla_stack.rs`,
//! `anim/nla_edit.rs`) and plays them (`anim/nla_eval.rs`).
//!
//! Push Down lays the active object's action, through its slot, as the
//! clip Blender makes of it (`BKE_nlastrip_new_for_slot`: over the Manual
//! Frame Range when the action has one, else from the slot's first key to
//! its last) onto the top track; Stash puts it, muted, in "[Action Stash]"
//! at the bottom. Either way the action is unassigned.

use super::BSceneActions;
use crate::phiphx::blend::write::BlendNlaStack;
use crate::phiphx::blend::{ADT_SOLO_TRACK, BMotionClip, BMotionTrack, BNlaStrip, STRIP_CLIP};
use crate::phiphx::scene::{SceneAction, SceneModel, SceneNla};

impl BSceneActions {
    /// The stacks as `ctx.scene.nla` spells them (JSON); none when empty.
    pub fn parse_nla(raw: &str) -> Vec<SceneNla> {
        if raw.is_empty() { Vec::new() } else { serde_json::from_str(raw).unwrap_or_default() }
    }

    pub fn encode_nla(list: &[SceneNla]) -> String {
        if list.is_empty() { String::new() } else { serde_json::to_string(list).unwrap_or_default() }
    }

    /// `object`'s stack now: the session's, else the file's, else empty.
    pub fn stack_of(&self, object: &str, model: &SceneModel) -> SceneNla {
        let found = self.nla.iter().chain(&model.nla).find(|n| n.object == object).cloned();
        found.unwrap_or_else(|| SceneNla { object: object.to_string(), ..Default::default() })
    }

    /// Keep `stack` as its object's from here on.
    pub fn keep_stack(&mut self, stack: SceneNla) {
        match self.nla.iter_mut().find(|n| n.object == stack.object) {
            Some(kept) => *kept = stack,
            None => self.nla.push(stack),
        }
    }

    /// `get_frame_range_of_slot` with no Manual Frame Range: the slot's
    /// first key to its last, over every curve; `(0, 0)` with none.
    fn key_range(action: &SceneAction, slot: &str) -> (f32, f32) {
        let handle = action.slots.iter().find(|s| s.identifier == slot).map_or(0, |s| s.handle);
        let keys = action.curves_of(handle).iter().filter_map(|c| Some((c.keys.first()?.co[0], c.keys.last()?.co[0])));
        keys.reduce(|(a, b), (c, d)| (a.min(c), b.max(d))).unwrap_or((0.0, 0.0))
    }

    /// The clip `nla.actionclip_add` makes of `action` for `object`
    /// (`BKE_nlastrip_new`): through the slot it would pick
    /// (`generic_slot_for_autoassign`, else the NLA's fallback — the only
    /// slot, when it suits an object), over the Manual Frame Range or the
    /// slot's keys.
    pub fn clip_for(&self, action: &SceneAction, object: &str, model: &SceneModel) -> BNlaStrip {
        let mut slot = Self::slot_for(action, object, None);
        if let ([only], true) = (action.slots.as_slice(), slot.is_empty()) {
            if matches!(only.id_code(), "OB" | "XX" | "") {
                slot = only.identifier.clone();
            }
        }
        let handle = action.slots.iter().find(|s| s.identifier == slot).map_or(0, |s| s.handle);
        let (manual, cyclic) = self.range_of(&action.name, model);
        BNlaStrip::clip(&action.name, (handle, &slot), manual.unwrap_or_else(|| Self::key_range(action, &slot)), manual.is_some(), cyclic)
    }

    /// Push Down (`stash` false) or Stash `object`'s action; what the log
    /// says, or `None` when it has no action.
    pub fn lay_nla(&mut self, object: &str, stash: bool, model: &SceneModel) -> Option<String> {
        let (action, slot) = self.of(object, model)?;
        let (manual, cyclic) = self.range_of(&action, model);
        let found = self.action(&action, model);
        let handle = found.as_ref().and_then(|a| a.slots.iter().find(|s| s.identifier == slot)).map_or(0, |s| s.handle);
        let range = manual.unwrap_or_else(|| found.as_ref().map_or((0.0, 0.0), |a| Self::key_range(a, &slot)));
        let strip = BNlaStrip::clip(&action, (handle, &slot), range, manual.is_some(), cyclic);
        let mut stack = self.stack_of(object, model);
        // Either way the object plays no action after (`unassign_action`).
        self.assign(object, "", &slot);
        let said = match stash {
            true => match stack.stash(strip) {
                Some(_) => format!("Stashed {action}"),
                None => return Some("Action+Slot has already been stashed".into()),
            },
            false => {
                stack.push_down(strip);
                format!("Pushed {action} down")
            }
        };
        self.keep_stack(stack);
        Some(said)
    }

    /// The session's stacks in `out`'s place; then whoever has one, or a
    /// strip of an action with new keys, plays its NLA as it is now.
    pub(super) fn lay(&self, out: &mut SceneModel) {
        for stack in &self.nla {
            match out.nla.iter_mut().find(|n| n.object == stack.object) {
                Some(n) => *n = stack.clone(),
                None => out.nla.push(stack.clone()),
            }
        }
        let SceneModel { nla, motion, actions, .. } = out;
        let keyed = |strip: &BNlaStrip| strip.action.as_deref().is_some_and(|a| self.keyed.iter().any(|k| k.action() == a));
        for stack in nla.iter() {
            let edited = self.nla.iter().any(|n| n.object == stack.object);
            if !edited && !stack.tracks.iter().flat_map(|t| &t.strips).any(keyed) {
                continue;
            }
            if let Some(m) = motion.motion.list.iter_mut().find(|m| m.name == stack.object) {
                (m.nla, m.solo) = (Self::played(stack, actions), stack.flag & ADT_SOLO_TRACK != 0);
            }
        }
    }

    /// `stack`'s tracks as motion plays them (`BlendFile::motion_tracks`
    /// over the session's actions).
    fn played(stack: &SceneNla, actions: &[SceneAction]) -> Vec<BMotionTrack> {
        let curves = |s: &BNlaStrip| actions.iter().find(|a| s.action.as_deref() == Some(a.name.as_str())).map(|a| a.curves_of(s.slot).to_vec()).unwrap_or_default();
        let clip = |s: &BNlaStrip| BMotionClip { strip: s.clone(), curves: curves(s) };
        stack.tracks.iter().map(|t| BMotionTrack { flag: t.flag, clips: t.strips.iter().filter(|s| s.kind == STRIP_CLIP).map(clip).collect() }).collect()
    }

    /// What the writer lays: each stack whole.
    pub fn nla_stacks(&self) -> Vec<BlendNlaStack> {
        self.nla.iter().map(|n| BlendNlaStack { object: n.object.clone(), nla: n.clone() }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BNlaTrack, STASH_TRACK};

    #[test]
    fn the_stacks_round_trip_through_their_key_and_push_down_and_stash_lay_on_them() {
        // Over frame 0 to 1, where a clip of Spin (no keys: 0 to 1) would go.
        let strip = BNlaStrip { name: "Walk".into(), start: 0.0, end: 49.0, address: 7, ..Default::default() };
        let stack = SceneNla { object: "Cube".into(), tracks: vec![BNlaTrack { name: "Base".into(), strips: vec![strip], ..Default::default() }], ..Default::default() };
        let list = vec![stack];
        assert_eq!(BSceneActions::parse_nla(&BSceneActions::encode_nla(&list)), list);
        assert!(BSceneActions::parse_nla("").is_empty() && BSceneActions::encode_nla(&[]).is_empty());

        let mut edits = BSceneActions::default();
        edits.assign("Cube", "Spin", "OBCube");
        let model = SceneModel { actions: vec![SceneAction { name: "Spin".into(), ..Default::default() }], nla: list, ..Default::default() };
        assert_eq!(edits.lay_nla("Cube", false, &model).as_deref(), Some("Pushed Spin down"));
        let names = |e: &BSceneActions| e.stack_of("Cube", &model).tracks.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&edits), ["Base", "Spin"], "on top of the file's stack, which it keeps");
        assert_eq!(edits.stack_of("Cube", &model).tracks[0].strips[0].address, 7);
        edits.assign("Cube", "Spin", "OBCube");
        edits.lay_nla("Cube", true, &model);
        assert_eq!(names(&edits), [STASH_TRACK, "Base", "Spin"]);
        edits.assign("Cube", "Spin", "OBCube");
        assert_eq!(edits.lay_nla("Cube", true, &model).as_deref(), Some("Action+Slot has already been stashed"));
        assert_eq!(edits.of("Cube", &model), None, "unassigned all the same");
    }
}
