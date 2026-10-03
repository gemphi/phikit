//! I — `anim.keyframe_insert` over the 3D Viewport (Object ▸ Animation ▸
//! Insert Keyframe) and Insert Keyframes over the Dope Sheet and the Graph
//! editor (`action.keyframe_insert` / `graph.keyframe_insert`, All
//! Channels) — made on the session's edits as `keys.rs` inserts.
//!
//! In the viewport, I keys each selected object's Location, Rotation (as
//! its rotation mode keeps it) and Scale — the default Key Channels — at
//! their values now: where its curves put it at the frame, or where the
//! Transform panel or G / R / S left it. An object with no action gets
//! `<Object>Action` (`id_action_ensure`), one with no slot the slot named
//! for it (`assign_action_ensure_slot_for_keying`); from then on its
//! animation drives it, as in Blender after a key. Over an animation
//! editor, Insert Keyframes keys every curve of the shown objects' slots
//! at its value now.

use super::BSceneActions;
use super::keys::{BKeyInsert, BKeyed};
use crate::phiphx::blend::ObjectTransformRaw;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::transform::BTransforms;
use crate::v2::state::{BKey as _, BPath, BState, OpKey, log, rule::BRule};
use glam::{EulerRot, Quat};

/// Which insert I asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BKeying {
    /// Location, Rotation, Scale (the viewport's I).
    Transforms,
    /// Every curve the object's slot has (an animation editor's I).
    Channels,
}

/// What an object's values are now: the model, the session's edits over
/// it, the frame.
pub struct BKeyNow<'a> {
    pub model: &'a SceneModel,
    pub edit: BSceneEdit,
    pub transforms: BTransforms,
    pub frame: f32,
}

impl BKeyNow<'_> {
    /// The object's transform properties now: its rest values with its
    /// curves at the frame written over them (a copy stands as its source),
    /// then the Transform panel's edit over those.
    pub fn raw(&self, object: &str) -> ObjectTransformRaw {
        let file = self.edit.renamed.iter().find(|(new, _)| new == object).map_or(object, |(_, old)| old.as_str());
        let source = self.edit.copied.iter().find(|(copy, _)| copy == file).map_or(file, |(_, from)| from.as_str());
        let mut raw = self.model.motion.named(source).map(|m| m.raw_at(self.frame)).unwrap_or_default();
        if let Some(t) = self.transforms.get(object) {
            let (loc, rot, scale) = t.to_blend();
            let quat = Quat::from_euler(EulerRot::ZYX, rot[2], rot[1], rot[0]);
            (raw.loc, raw.size) = (loc, scale);
            match raw.rotmode {
                0 => raw.quat = [quat.w, quat.x, quat.y, quat.z],
                -1 => {
                    let (axis, angle) = quat.to_axis_angle();
                    (raw.rot_axis, raw.rot_angle) = (axis.into(), angle);
                }
                _ => raw.rot = rot,
            }
        }
        raw
    }

    /// Location, the rotation its mode keeps, Scale — in Blender's order.
    pub fn transforms(raw: &ObjectTransformRaw) -> Vec<BKeyed> {
        let each = |path: &str, values: &[f32]| values.iter().enumerate().map(|(i, v)| BKeyed { path: path.to_string(), index: i as u32, value: *v }).collect::<Vec<_>>();
        let rotation = match raw.rotmode {
            0 => each("rotation_quaternion", &raw.quat),
            -1 => each("rotation_axis_angle", &[raw.rot_angle, raw.rot_axis[0], raw.rot_axis[1], raw.rot_axis[2]]),
            _ => each("rotation_euler", &raw.rot),
        };
        [each("location", &raw.loc), rotation, each("scale", &raw.size)].concat()
    }
}

impl BSceneActions {
    /// The action and slot a key on `object` goes in, made or given as
    /// Blender does when there is none.
    pub(super) fn keyed_slot(&mut self, object: &str, model: &SceneModel) -> (String, String) {
        let named = format!("OB{object}");
        match self.of(object, model) {
            Some((action, slot)) if !slot.is_empty() => (action, slot),
            Some((action, _)) => {
                let slot = self.action(&action, model).map(|a| Self::slot_for(&a, object, None)).filter(|s| !s.is_empty()).unwrap_or(named);
                self.assign(object, &action, &slot);
                (action, slot)
            }
            None => {
                let action = Self::unique(&format!("{object}Action"), &self.names(model));
                self.made.push((action.clone(), String::new()));
                self.assign(object, &action, &named);
                (action, named)
            }
        }
    }

    /// I on `objects`: the inserts made; the objects keyed.
    pub fn key(&mut self, how: BKeying, objects: &[String], now: &BKeyNow) -> Vec<String> {
        let mut keyed = Vec::new();
        for object in objects {
            let raw = now.raw(object);
            let insert = match how {
                BKeying::Transforms => {
                    let (action, slot) = self.keyed_slot(object, now.model);
                    BKeyInsert { action, slot, frame: now.frame, channels: BKeyNow::transforms(&raw) }
                }
                BKeying::Channels => {
                    let Some((action, slot)) = self.of(object, now.model).filter(|(_, s)| !s.is_empty()) else { continue };
                    let Some(found) = self.action(&action, now.model) else { continue };
                    let Some(run) = found.slots.iter().find(|s| s.identifier == slot).map(|s| s.curves.clone()) else { continue };
                    let values = BKeyNow::transforms(&raw);
                    let value = |path: &str, index: u32, curve: &crate::phiphx::blend::BFCurve| {
                        values.iter().find(|v| v.path == path && v.index == index).map_or_else(|| curve.evaluate(now.frame), |v| v.value)
                    };
                    let channels = found.curves[run].iter().map(|c| BKeyed { path: c.path.clone(), index: c.index, value: value(&c.path, c.index, c) }).collect();
                    BKeyInsert { action, slot, frame: now.frame, channels }
                }
            };
            if !insert.channels.is_empty() {
                self.keyed.push(super::retime::BKeyStep::Insert(insert));
                keyed.push(object.clone());
            }
        }
        keyed
    }
}

/// `ctx.op.keyframe` → the inserts, on the selected objects.
pub struct BKeyframeRule;

impl BRule for BKeyframeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Keyframe.path()
    }
    fn after(&self, _: &BPath, value: &str, state: &mut BState) {
        state.remove(OpKey::Keyframe);
        let how = match value {
            "insert" => BKeying::Transforms,
            "channels" => BKeying::Channels,
            "delete" | "clear" => return Self::delete(state, value == "clear"),
            _ => return,
        };
        let objects = crate::v2::scene::select::BSelection::now(state).list;
        let mut edits = BSceneActions::now(state);
        let mut transforms = BTransforms::now(state);
        let keyed = crate::v2::scene::source::with_loaded(|model| {
            let Some(model) = model else { return Vec::new() };
            let frame = crate::v2::scene::motion::shown_frame(state).unwrap_or(model.timeline.current_frame as f32);
            let now = BKeyNow { model, edit: BSceneEdit::now(state), transforms: transforms.clone(), frame };
            let keyed = edits.key(how, &objects, &now);
            // Keyed where it stands, a file object is driven by its curves
            // from here on; one the session made has none to drive it yet.
            if how == BKeying::Transforms {
                keyed.iter().filter(|o| now.moves(o)).for_each(|o| transforms.forget(o));
            }
            keyed.into_iter().map(|o| (o, frame)).collect::<Vec<_>>()
        });
        let frame = keyed.first().map_or(0.0, |(_, f)| *f);
        let keyed: Vec<String> = keyed.into_iter().map(|(o, _)| o).collect();
        let said = match keyed.as_slice() {
            [] if objects.is_empty() => "Nothing selected to key".to_string(),
            [] => "Nothing to key".to_string(),
            [one] => format!("Keyed {one} at frame {frame}"),
            many => format!("Keyed {} objects at frame {frame}", many.len()),
        };
        if !keyed.is_empty() {
            edits.write(state);
            transforms.write(state);
        }
        log::push(said);
    }
}

impl BKeyframeRule {
    /// Alt I, `anim.keyframe_delete_v3d`: each selected object's keys on the
    /// shown frame; with `all`, Clear Keyframes (`anim.keyframes_clear_v3d`):
    /// every key it has. A curve left empty goes (`retime.rs`).
    fn delete(state: &mut BState, all: bool) {
        use super::dope::{BKeySelection, BKeyTarget};
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return };
        let frame = crate::v2::scene::motion::shown_frame(state).unwrap_or(model.timeline.current_frame as f32);
        let mut picks = BKeySelection::default();
        for object in crate::v2::scene::select::BSelection::now(state).list {
            let columns = BKeyTarget::columns(&model.actions, &object);
            columns.into_iter().filter(|c| all || (c - frame).abs() < 5e-4).for_each(|c| picks.0.push(super::dope::BKeyPick { object: object.clone(), frame: c }));
        }
        let mut objects: Vec<&str> = picks.0.iter().map(|p| p.object.as_str()).collect();
        objects.dedup();
        let said = match objects.len() {
            0 if all => "No keyframes to clear".to_string(),
            0 => "No keyframes removed from the selected objects at this frame".to_string(),
            n if all => format!("Cleared the keyframes of {n} object(s)"),
            n => format!("{n} object(s) successfully had {} keyframe column(s) removed", picks.0.len()),
        };
        if !picks.0.is_empty() {
            let mut edits = BSceneActions::now(state);
            edits.keyed.extend(BKeyTarget::steps(&model.actions, &picks, None));
            edits.write(state);
        }
        log::push(said);
    }
}

impl BKeyNow<'_> {
    /// Does the file move this object (so its curves can drive it)?
    pub(super) fn moves(&self, object: &str) -> bool {
        let file = self.edit.renamed.iter().find(|(new, _)| new == object).map_or(object, |(_, old)| old.as_str());
        self.model.motion.named(file).is_some()
    }
}
