//! Auto keying — the record button (`ToolSettings.autokey_mode`,
//! `animrig::autokeyframe_object`): with it on, a G, R or S kept in the
//! viewport keys what it changed, at the shown frame, as I would.
//!
//! Read off Blender 5.0.1 (`bpy`, `transform.translate / rotate / resize`
//! with auto keying on, over the factory scene):
//! - only the channels of the transform keyed: G keys Location, R the
//!   rotation the object's mode keeps, S Scale — each of their three (or
//!   four) curves;
//! - **Only Insert Needed** is on for auto keying by default
//!   (`use_auto_keyframe_insert_needed`): a curve already holding the value
//!   at the frame gets no key, a curve with no keys always does
//!   (`animrig::new_key_needed`) — so a second G along X keys X alone;
//! - **Replace** (`REPLACE_KEYS`) only replaces a key standing on the frame;
//!   with none there, or no curve, nothing is keyed.
//!
//! A channel group the session's edits had already moved away from the
//! file's values is keyed with the transform's, so the object stands where
//! it was left once its curves drive it again.

use super::BSceneActions;
use super::keying::BKeyNow;
use super::keys::{BKeyInsert, BKeyed};
use super::retime::BKeyStep;
use crate::phiphx::blend::BFCurve;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::transform::BTransforms;
use crate::v2::scene::xform::BTransformMode;
use crate::v2::state::BState;

/// The record button's mode (`AUTOKEY_MODE_NORMAL` / `_EDITKEYS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BAutoKey {
    AddReplace,
    Replace,
}

impl BAutoKey {
    /// The record button's mode, while it is on.
    pub fn now(s: &BState) -> Option<Self> {
        let p = crate::v2::part::playback::settings::now(s);
        p.auto_key.then_some(if p.auto_key_replace { Self::Replace } else { Self::AddReplace })
    }

    /// Which transform channels `path` is among.
    fn group(path: &str) -> &'static str {
        match path {
            "location" => "location",
            "scale" => "scale",
            p if p.starts_with("rotation") => "rotation",
            _ => "",
        }
    }

    /// `new_key_needed`: no keys yet, a key on the frame with another
    /// value, or the curve's value there some other.
    fn needed(curve: Option<&BFCurve>, frame: f32, value: f32) -> bool {
        let Some(curve) = curve.filter(|c| !c.keys.is_empty()) else { return true };
        if let Some(key) = Self::on(curve, frame) {
            return key != value;
        }
        let now = curve.evaluate(frame);
        (now - value).abs() > f32::EPSILON.max(now.abs().max(value.abs()) * 32.0 * f32::EPSILON)
    }

    /// The value of the key standing on `frame` (`BEZT_BINARYSEARCH_THRESH`).
    fn on(curve: &BFCurve, frame: f32) -> Option<f32> {
        curve.keys.iter().find(|k| (k.co[0] - frame).abs() < 0.01).map(|k| k.co[1])
    }

    /// A transform `mode` was kept on `objects`: key them, when the record
    /// button is on. Returns the objects keyed.
    pub fn transformed(s: &mut BState, mode: BTransformMode, objects: &[String]) -> Vec<String> {
        let Some(auto) = Self::now(s) else { return Vec::new() };
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return Vec::new() };
        let group = match mode {
            BTransformMode::Translate => "location",
            BTransformMode::Rotate => "rotation",
            BTransformMode::Resize => "scale",
        };
        let frame = crate::v2::scene::motion::shown_frame(s).unwrap_or(model.timeline.current_frame as f32);
        let (mut edits, mut transforms) = (BSceneActions::now(s), BTransforms::now(s));
        let edit = BSceneEdit::now(s);
        let now = BKeyNow { model: &model, edit: edit.clone(), transforms: transforms.clone(), frame };
        let file = BKeyNow { model: &model, edit, transforms: BTransforms::default(), frame };
        // The curves as the session has them: an action I made this session
        // is not in the file's list.
        let actions = edits.keyed_actions(&model);
        let mut keyed = Vec::new();
        for object in objects {
            let (with, without) = (BKeyNow::transforms(&now.raw(object)), BKeyNow::transforms(&file.raw(object)));
            let moved = |c: &BKeyed| without.iter().any(|w| w.path == c.path && w.index == c.index && (w.value - c.value).abs() > 1e-6);
            let groups: Vec<&str> = ["location", "rotation", "scale"].into_iter().filter(|g| *g == group || with.iter().any(|c| Self::group(&c.path) == *g && moved(c))).collect();
            let curves: Vec<BFCurve> = edits
                .of(object, &model)
                .and_then(|(action, slot)| {
                    let a = actions.iter().find(|a| a.name == action)?;
                    let run = a.slots.iter().find(|x| x.identifier == slot)?.curves.clone();
                    Some(a.curves[run].to_vec())
                })
                .unwrap_or_default();
            let curve = |c: &BKeyed| curves.iter().find(|f| f.path == c.path && f.index == c.index);
            let channels: Vec<BKeyed> = with
                .iter()
                .filter(|c| groups.contains(&Self::group(&c.path)))
                .filter(|c| Self::needed(curve(c), frame, c.value))
                .filter(|c| auto == Self::AddReplace || curve(c).and_then(|f| Self::on(f, frame)).is_some())
                .cloned()
                .collect();
            if channels.is_empty() {
                continue;
            }
            let (action, slot) = match auto {
                Self::AddReplace => edits.keyed_slot(object, &model),
                Self::Replace => match edits.of(object, &model) {
                    Some(found) if !found.1.is_empty() => found,
                    _ => continue,
                },
            };
            // Driven by its curves from here on, when they now hold every
            // value the edit had moved.
            let held = with.iter().filter(|c| moved(c)).all(|c| channels.iter().any(|k| k.path == c.path && k.index == c.index) || !Self::needed(curve(c), frame, c.value));
            if held && now.moves(object) {
                transforms.forget(object);
            }
            edits.keyed.push(BKeyStep::Insert(BKeyInsert { action, slot, frame, channels }));
            keyed.push(object.clone());
        }
        if !keyed.is_empty() {
            edits.write(s);
            transforms.write(s);
        }
        keyed
    }
}

#[cfg(test)]
#[path = "autokey_tests.rs"]
mod tests;
