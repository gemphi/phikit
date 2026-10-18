//! Clean Keyframes — `ACTION_OT_clean` (`space_action/action_edit.cc`'s
//! `actkeys_clean_exec` → `clean_action_keys`), whose work is `clean_fcurve`
//! in `animation/keyframes_general.cc`. Kept in the same ordered list as the
//! inserts, moves and reshapes (`ctx.scene.keyed`), so a clean replays over
//! the file's curves in the order it was made, Undo steps it, and a save
//! writes the curves it leaves.
//!
//! Blender's operator carries two properties and the menus set both: Key ▸
//! Clean Keyframes is `channels = False` and Channel ▸ Clean Channels is
//! `channels = True` (`space_dopesheet.py`). `clean_action_keys` turns that
//! one flag into the two `clean_fcurve` takes —
//! `clean_fcurve(ale, thresh, cleardefault = clean_chan, only_selected_keys = !clean_chan)`
//! — so Clean Keyframes simplifies the **selected** keys and never removes a
//! channel, while Clean Channels considers **every** key and drops a curve
//! left holding one key at the property's default.
//!
//! **This is a per-curve edit, not a per-column one.** Every other step here
//! names frames and acts on each curve of the slot at them (`BKeyRetime`,
//! `BKeyReshape`); a clean asks each curve its own question, so `location`'s
//! X can lose a key at frame 10 while its Y keeps one. That is why it is a
//! step of its own rather than a delete the operator works out in advance:
//! the step records the **intent** — the threshold and the frames the user
//! had selected — and the walk runs per curve at replay, so a clean replayed
//! over changed curves cleans what those curves then hold, as Blender's
//! would.

use crate::phiphx::blend::{BFCurve, BKey};
use crate::phiphx::scene::SceneAction;

/// A clean's head mark: `%action#slot#frames=threshold,channels`.
const MARK: char = '%';

/// `RNA_def_float(ot->srna, "threshold", 0.001f, …)` — the operator's
/// default. Blender lets the redo panel change it; phiber's menus have no
/// redo panel yet, so the menu fires this and the field is carried on the
/// step so one can set it later without a new encoding.
pub const DEFAULT_THRESHOLD: f32 = 0.001;

/// The keys on `action`'s curves through `slot` (all of them when `slot` is
/// empty), simplified. `frames` are the selected columns a Clean Keyframes
/// considers; a Clean Channels (`channels`) leaves it empty and considers
/// every key.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyClean {
    pub action: String,
    pub slot: String,
    pub frames: Vec<f32>,
    pub threshold: f32,
    /// `channels`: `cleardefault` — also drop a curve left with one key at
    /// the property's default value.
    pub channels: bool,
}

/// `IS_EQT(a, b, c)` (`BLI_utildefines.h`): within `c`, inclusive.
fn eqt(a: f32, b: f32, thresh: f32) -> bool {
    (a - b).abs() <= thresh
}

/// The default value of the property a curve drives, for `cleardefault`.
///
/// Blender reads it from RNA — `RNA_path_resolve_property` then
/// `RNA_property_float_get_default_index` — which answers for any property
/// in the file. phiber has no RNA default table, so this covers the
/// transform channels that make up all but the rarest action: `scale` is 1
/// on every axis and a quaternion's W is 1 (`rna_def_object`), and every
/// other animatable float phiber meets defaults to 0. Recorded divergence:
/// a curve on some other property with a non-zero default is not recognised
/// as sitting at its default, so Clean Channels keeps it rather than
/// dropping it — it errs towards keeping the user's curve, which is the
/// safe way to be wrong.
fn default_value(path: &str, index: u32) -> f32 {
    let leaf = path.rsplit(['.', ']']).find(|p| !p.is_empty()).unwrap_or(path);
    match (leaf, index) {
        ("scale" | "delta_scale", _) => 1.0,
        ("rotation_quaternion" | "delta_rotation_quaternion", 0) => 1.0,
        _ => 0.0,
    }
}

impl BKeyClean {
    /// Within half a thousandth: the columns are keyed to a thousandth.
    fn selected(&self, x: f32) -> bool {
        self.frames.iter().any(|f| (f - x).abs() < 5e-4)
    }

    /// `clean_fcurve`'s walk over one curve. The first key is always kept
    /// ("now insert first keyframe, as it should be ok"); each one after it
    /// is compared against the last key KEPT and the next key as the file
    /// has it, and is dropped when it says nothing those two do not.
    ///
    /// `only_selected` is Blender's `only_selected_keys`: an unselected key
    /// is kept exactly as it stands — and, being kept, becomes the `prev`
    /// the next comparison runs against, as it does there.
    fn clean_curve(&self, curve: &mut BFCurve, only_selected: bool) {
        if curve.keys.is_empty() || (!self.channels && curve.keys.len() == 1) {
            return;
        }
        let old: Vec<BKey> = std::mem::take(&mut curve.keys);
        curve.keys.push(old[0].clone());
        for i in 1..old.len() {
            let (cur, next) = (&old[i], old.get(i + 1));
            if only_selected && !self.selected(cur.co[0]) {
                curve.keys.push(cur.clone());
                continue;
            }
            let prev = curve.keys.last().expect("the first key went in").co;
            let t = self.threshold;
            let keep = match (next, eqt(cur.co[0], prev[0], t)) {
                // At the same time as the last key kept, and so is the next,
                // and those two differ in value: keep only the further of the
                // pair, and only if it says something `prev` does not.
                (Some(n), true) if eqt(cur.co[0], n.co[0], t) && !eqt(n.co[1], prev[1], t) => {
                    cur.co[1] > n.co[1] && !eqt(cur.co[1], prev[1], t)
                }
                // At the same time otherwise: keep it only if its value is a
                // considerable distance from the last kept one.
                (_, true) => !eqt(cur.co[1], prev[1], t),
                // A different time, with a key after it: a key that matches
                // BOTH its neighbours is the middle of a flat run and goes;
                // the run collapses to its two ends.
                (Some(n), false) => !eqt(cur.co[1], prev[1], t) || !eqt(cur.co[1], n.co[1], t),
                // The last key: keep it unless it repeats the kept value.
                (None, false) => !eqt(cur.co[1], prev[1], t),
            };
            if keep {
                curve.keys.push(cur.clone());
            }
        }
        curve.recalc_handles();
    }

    pub fn apply(&self, actions: &mut [SceneAction]) {
        let Some(action) = actions.iter_mut().find(|a| a.name == self.action) else { return };
        let run = match self.slot.is_empty() {
            true => 0..action.curves.len(),
            false => match action.slots.iter().find(|s| s.identifier == self.slot) {
                Some(slot) => slot.curves.clone(),
                None => return,
            },
        };
        // `only_selected_keys = !clean_chan`.
        let only_selected = !self.channels;
        for curve in &mut action.curves[run.clone()] {
            self.clean_curve(curve, only_selected);
        }
        // `cleardefault`: "if there is just one key in fcurve, check if it's
        // the default value and if is, remove fcurve completely" — and the
        // curves a clean emptied go with them, as a delete's do
        // (`ANIM_fcurve_delete_from_animdata`).
        let doomed = |c: &BFCurve| match (self.channels, c.keys.as_slice()) {
            (_, []) => true,
            (true, [only]) => only.co[1] == default_value(&c.path, c.index),
            _ => false,
        };
        let drop: Vec<usize> = run.rev().filter(|&i| doomed(&action.curves[i])).collect();
        for at in drop {
            action.curves.remove(at);
            for slot in &mut action.slots {
                match (slot.curves.start > at, slot.curves.contains(&at)) {
                    // Wholly after the curve that went: the run shifts down.
                    (true, _) => slot.curves = slot.curves.start - 1..slot.curves.end - 1,
                    // The run that held it: one shorter.
                    (_, true) => slot.curves.end -= 1,
                    _ => {}
                }
            }
        }
        super::keys::BKeyInsert::recount(action);
    }

    pub(super) fn is_mark(entry: &str) -> bool {
        entry.starts_with(MARK)
    }

    pub(super) fn parse(entry: &str) -> Option<Self> {
        let (head, tail) = entry.strip_prefix(MARK)?.rsplit_once('=')?;
        let mut head = head.splitn(3, '#');
        let (action, slot) = (head.next()?.to_string(), head.next()?.to_string());
        let frames = head.next()?.split(',').filter_map(|f| f.parse().ok()).collect();
        let (threshold, channels) = tail.split_once(',')?;
        Some(Self { action, slot, frames, threshold: threshold.parse().ok()?, channels: channels == "1" })
    }

    pub(super) fn encode(&self) -> String {
        let frames: Vec<String> = self.frames.iter().map(f32::to_string).collect();
        let channels = u8::from(self.channels);
        format!("{MARK}{}#{}#{}={},{channels}", self.action, self.slot, frames.join(","), self.threshold)
    }
}

#[cfg(test)]
#[path = "clean_tests.rs"]
mod tests;
