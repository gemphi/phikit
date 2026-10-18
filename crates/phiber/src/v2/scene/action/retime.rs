//! Keys the Dope Sheet moved or deleted — `transform.transform` in
//! `TIME_TRANSLATE` (G, or a drag on a key) and `action.delete` (X) — kept
//! in the same ordered list as the keys I inserted (`ctx.scene.keyed`), so
//! an insert, a move and an insert again replay as they were made, Undo
//! steps them, and a save writes the curves they leave.
//!
//! A move carries every key at the moved frames on the slot's curves, with
//! its handles, by whole frames (the Dope Sheet snaps to frames by
//! default). A key left standing where a moved one lands is dropped, the
//! moved one kept (`posttrans_fcurve_deletedupes`); then each curve is
//! sorted and its handles worked out again (`BKE_fcurve_handles_recalc`).
//! A delete takes the keys at the frames off, and a curve left with no key
//! goes with them (`ANIM_fcurve_delete_from_animdata`).

use super::keys::BKeyInsert;
use crate::phiphx::scene::SceneAction;

/// A retime's head: `~action#slot#frames=by`, `=x` for a delete.
const MARK: char = '~';
const DELETE: &str = "x";

/// The keys at `frames` on `action`'s curves through `slot` (all of them
/// when `slot` is empty), moved `by` frames — or deleted, with no `by`.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyRetime {
    pub action: String,
    pub slot: String,
    pub frames: Vec<f32>,
    pub by: Option<f32>,
}

impl BKeyRetime {
    /// Within half a thousandth: the columns are keyed to a thousandth.
    fn hit(&self, x: f32) -> bool {
        self.frames.iter().any(|f| (f - x).abs() < 5e-4)
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
        for curve in &mut action.curves[run.clone()] {
            let (taken, kept): (Vec<_>, Vec<_>) = curve.keys.drain(..).partition(|k| self.hit(k.co[0]));
            curve.keys = kept;
            if taken.is_empty() {
                continue;
            }
            if let Some(by) = self.by {
                let moved: Vec<_> = taken.into_iter().map(|mut k| {
                    for point in [&mut k.h1, &mut k.co, &mut k.h2] {
                        point[0] += by;
                    }
                    k
                }).collect();
                curve.keys.retain(|k| !moved.iter().any(|m| (m.co[0] - k.co[0]).abs() < 5e-4));
                curve.keys.extend(moved);
                curve.keys.sort_by(|a, b| a.co[0].total_cmp(&b.co[0]));
            }
            curve.recalc_handles();
        }
        // The curves left empty, last first, each slot's run closing up.
        let empty: Vec<usize> = run.rev().filter(|&i| action.curves[i].keys.is_empty()).collect();
        for at in empty {
            action.curves.remove(at);
            for slot in &mut action.slots {
                if slot.curves.start > at {
                    slot.curves = slot.curves.start - 1..slot.curves.end - 1;
                } else if slot.curves.contains(&at) {
                    slot.curves.end -= 1;
                }
            }
        }
        BKeyInsert::recount(action);
    }

    fn parse(entry: &str) -> Option<Self> {
        let (head, by) = entry.strip_prefix(MARK)?.rsplit_once('=')?;
        let mut head = head.splitn(3, '#');
        let (action, slot) = (head.next()?.to_string(), head.next()?.to_string());
        let frames = head.next()?.split(',').filter_map(|f| f.parse().ok()).collect();
        let by = if by == DELETE { None } else { Some(by.parse().ok()?) };
        Some(Self { action, slot, frames, by })
    }

    fn encode(&self) -> String {
        let frames: Vec<String> = self.frames.iter().map(f32::to_string).collect();
        let by = self.by.map_or(DELETE.to_string(), |b| b.to_string());
        format!("{MARK}{}#{}#{}={by}", self.action, self.slot, frames.join(","))
    }
}

/// One edit to the keys, in the order it was made.
#[derive(Clone, Debug, PartialEq)]
pub enum BKeyStep {
    Insert(BKeyInsert),
    Retime(BKeyRetime),
    Reshape(super::reshape::BKeyReshape),
    /// `ACTION_OT_clean` — a per-curve simplification rather than a
    /// per-column edit, which is why it is its own step (`clean.rs`).
    Clean(super::clean::BKeyClean),
}

impl BKeyStep {
    pub fn action(&self) -> &str {
        match self {
            Self::Insert(i) => &i.action,
            Self::Retime(r) => &r.action,
            Self::Reshape(r) => &r.action,
            Self::Clean(c) => &c.action,
        }
    }

    pub fn slot(&self) -> &str {
        match self {
            Self::Insert(i) => &i.slot,
            Self::Retime(r) => &r.slot,
            Self::Reshape(r) => &r.slot,
            Self::Clean(c) => &c.slot,
        }
    }

    pub fn apply(&self, actions: &mut [SceneAction]) {
        match self {
            Self::Insert(i) => i.apply(actions),
            Self::Retime(r) => r.apply(actions),
            Self::Reshape(r) => r.apply(actions),
            Self::Clean(c) => c.apply(actions),
        }
    }

    /// The steps as `ctx.scene.keyed` spells them: an insert as before
    /// (`keys.rs`), a retime marked `~`, a reshape marked `^`, a clean
    /// marked `%`.
    pub fn parse_all(raw: &str) -> Vec<Self> {
        let one = |e: &str| match e {
            _ if e.starts_with(MARK) => BKeyRetime::parse(e).map(Self::Retime),
            _ if super::reshape::BKeyReshape::is_mark(e) => super::reshape::BKeyReshape::parse(e).map(Self::Reshape),
            _ if super::clean::BKeyClean::is_mark(e) => super::clean::BKeyClean::parse(e).map(Self::Clean),
            _ => BKeyInsert::parse(e).map(Self::Insert),
        };
        raw.split('|').filter(|e| !e.is_empty()).filter_map(one).collect()
    }

    pub fn encode_all(list: &[Self]) -> String {
        let one = |s: &Self| match s {
            Self::Insert(i) => i.encode(),
            Self::Retime(r) => r.encode(),
            Self::Reshape(r) => r.encode(),
            Self::Clean(c) => c.encode(),
        };
        list.iter().map(one).collect::<Vec<_>>().join("|")
    }
}

#[cfg(test)]
#[path = "retime_tests.rs"]
mod tests;
