//! The Dope Sheet's keys: picking them and acting on them.
//!
//! A row's diamond stands for every key of that row's channels at one frame
//! — the summary's for every listed object's, an object's for its own —
//! so a pick is an object's key column (`BKeyPick`), kept in
//! `ctx.keys.selected` (Blender keeps `BEZT_SELECTED` on each key; a column
//! is picked whole here because the rows stop at the object).
//!
//! - **Click** (`action.clickselect`): the column under the pointer alone;
//!   Shift flips it; a click on nothing deselects every key.
//! - **Box** (`action.select_box`, a drag begun on nothing): the columns
//!   between the frames and rows it spans, added to what was picked.
//! - **All / None / Invert** (A, Alt A, Ctrl I; `action.select_all`).
//! - **Move** (G, or a drag begun on a key; `transform.transform` with
//!   `TIME_TRANSLATE`): the picked columns follow the pointer by whole
//!   frames, measured from where the grab began; a click or the button's
//!   release keeps them, Escape or the right button puts them back, and the
//!   whole move is one undo step.
//! - **Delete** (X, Delete; `action.delete`).
//!
//! Moves and deletes are `retime.rs` steps over the slot each object is
//! animated through (`BKeyTarget`).

pub mod set;
#[path = "set_edit.rs"]
mod set_edit;
#[path = "set_grab.rs"]
mod set_grab;
#[path = "set_jump.rs"]
mod set_jump;

pub use set::BKeyOpRule;

use super::retime::{BKeyRetime, BKeyStep};
use crate::phiphx::scene::{SceneAction, SceneModel};
use crate::v2::state::{BState, CtxKey};

/// One object's key column.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyPick {
    pub object: String,
    pub frame: f32,
}

/// The picked key columns, `Cube@20|Camera@1`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BKeySelection(pub Vec<BKeyPick>);

impl BKeySelection {
    pub fn now(s: &BState) -> Self {
        Self::parse(s.get(CtxKey::KeySelected).unwrap_or(""))
    }

    pub(super) fn parse(raw: &str) -> Self {
        let pick = |e: &str| e.rsplit_once('@').and_then(|(o, f)| Some(BKeyPick { object: o.to_string(), frame: f.parse().ok()? }));
        Self(raw.split('|').filter_map(pick).collect())
    }

    pub(super) fn encode(&self) -> String {
        self.0.iter().map(|p| format!("{}@{}", p.object, p.frame)).collect::<Vec<_>>().join("|")
    }

    pub fn write(&self, s: &mut BState) {
        s.set(CtxKey::KeySelected, self.encode());
    }

    pub fn has(&self, object: &str, frame: f32) -> bool {
        self.0.iter().any(|p| p.object == object && (p.frame - frame).abs() < 5e-4)
    }

    pub(super) fn add(&mut self, object: &str, frame: f32) {
        if !self.has(object, frame) {
            self.0.push(BKeyPick { object: object.to_string(), frame });
        }
    }

    pub(super) fn flip(&mut self, object: &str, frame: f32) {
        match self.has(object, frame) {
            true => self.0.retain(|p| !(p.object == object && (p.frame - frame).abs() < 5e-4)),
            false => self.add(object, frame),
        }
    }
}

/// Where an object's keys are: an action it uses, the slot it is animated
/// through there (its binding, else the action's only slot, else every
/// curve), and the frames keyed on that slot's curves.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyTarget {
    pub action: String,
    pub slot: String,
    pub columns: Vec<f32>,
}

impl BKeyTarget {
    pub fn of(actions: &[SceneAction], object: &str) -> Vec<Self> {
        let target = |a: &SceneAction| {
            let bound = a.bindings.iter().find(|(o, _)| o == object).and_then(|(_, h)| a.slot(*h));
            let slot = bound.or_else(|| (a.slots.len() == 1).then(|| &a.slots[0]));
            let (slot, curves) = slot.map_or((String::new(), &a.curves[..]), |s| (s.identifier.clone(), &a.curves[s.curves.clone()]));
            let mut columns: Vec<f32> = curves.iter().flat_map(|c| c.keys.iter().map(|k| (k.co[0] * 1000.0).round() / 1000.0)).collect();
            columns.sort_by(f32::total_cmp);
            columns.dedup();
            Self { action: a.name.clone(), slot, columns }
        };
        actions.iter().filter(|a| a.used_by(object)).map(target).collect()
    }

    /// Every frame keyed on `object`'s channels.
    pub fn columns(actions: &[SceneAction], object: &str) -> Vec<f32> {
        let mut out: Vec<f32> = Self::of(actions, object).into_iter().flat_map(|t| t.columns).collect();
        out.sort_by(f32::total_cmp);
        out.dedup();
        out
    }

    /// The objects the Dope Sheet lists under its summary, in order, each
    /// with a key: with Only Show Selected on (Blender's default,
    /// `ADS_FILTER_ONLYSEL`) the selected, then the active; with it off,
    /// every animated object in the scene.
    pub fn shown(s: &BState, model: &SceneModel) -> Vec<String> {
        use crate::v2::part::filter::dope::{INLINE, is_on};
        let only_selected = INLINE.iter().find(|f| f.key == "show_only_selected").is_some_and(|f| is_on(s, f));
        let mut names: Vec<String> = if only_selected {
            let mut names = model.selection.selected.clone();
            names.extend(s.get(CtxKey::Object).map(str::to_string));
            names
        } else {
            // Every object an action is bound to — a camera or a lamp as
            // much as a mesh — in the file's order of actions.
            model.actions.iter().flat_map(|a| a.users.iter().cloned()).collect()
        };
        // Once each, first place kept: the active is usually also selected.
        let mut seen = std::collections::HashSet::new();
        names.retain(|n| seen.insert(n.clone()) && !Self::columns(&model.actions, n).is_empty());
        names
    }

    /// The steps that move (`by`) or delete (`None`) the `picks`.
    pub(in crate::v2::scene::action) fn steps(actions: &[SceneAction], picks: &BKeySelection, by: Option<f32>) -> Vec<BKeyStep> {
        let mut out: Vec<BKeyRetime> = Vec::new();
        for pick in &picks.0 {
            for t in Self::of(actions, &pick.object).into_iter().filter(|t| t.columns.iter().any(|c| (c - pick.frame).abs() < 5e-4)) {
                match out.iter_mut().find(|r| r.action == t.action && r.slot == t.slot) {
                    Some(r) if !r.frames.contains(&pick.frame) => r.frames.push(pick.frame),
                    Some(_) => {}
                    None => out.push(BKeyRetime { action: t.action, slot: t.slot, frames: vec![pick.frame], by }),
                }
            }
        }
        out.into_iter().map(BKeyStep::Retime).collect()
    }

    /// The steps that snap the `picks` to the frames `target` maps them to —
    /// one move per column, `by` its own distance, so a column already on its
    /// target stays put (`ACTION_OT_snap`).
    pub(in crate::v2::scene::action) fn snap(actions: &[SceneAction], picks: &BKeySelection, target: impl Fn(f32) -> f32) -> Vec<BKeyStep> {
        let mut out: Vec<BKeyRetime> = Vec::new();
        for pick in &picks.0 {
            let by = target(pick.frame) - pick.frame;
            if by.abs() < 5e-4 {
                continue;
            }
            for t in Self::of(actions, &pick.object).into_iter().filter(|t| t.columns.iter().any(|c| (c - pick.frame).abs() < 5e-4)) {
                out.push(BKeyRetime { action: t.action, slot: t.slot, frames: vec![pick.frame], by: Some(by) });
            }
        }
        out.into_iter().map(BKeyStep::Retime).collect()
    }

    /// The steps that reshape the `picks` — the same grouping as [`Self::steps`],
    /// one reshape per `(action, slot)` carrying the picked frames and the
    /// shape to set (Key ▸ Interpolation / Easing / Handle Type).
    pub(in crate::v2::scene::action) fn reshapes(actions: &[SceneAction], picks: &BKeySelection, shape: super::reshape::BKeyShape) -> Vec<BKeyStep> {
        let mut out: Vec<super::reshape::BKeyReshape> = Vec::new();
        for pick in &picks.0 {
            for t in Self::of(actions, &pick.object).into_iter().filter(|t| t.columns.iter().any(|c| (c - pick.frame).abs() < 5e-4)) {
                match out.iter_mut().find(|r| r.action == t.action && r.slot == t.slot) {
                    Some(r) if !r.frames.contains(&pick.frame) => r.frames.push(pick.frame),
                    Some(_) => {}
                    None => out.push(super::reshape::BKeyReshape { action: t.action, slot: t.slot, frames: vec![pick.frame], shape }),
                }
            }
        }
        out.into_iter().map(BKeyStep::Reshape).collect()
    }
}

/// A move under way: where it began (`None` until the pointer first says),
/// how many key steps came before it, and the picks it started with.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyGrab {
    pub from: Option<i32>,
    pub base: usize,
    pub picks: BKeySelection,
}

impl BKeyGrab {
    pub fn now(s: &BState) -> Option<Self> {
        let mut part = s.get(CtxKey::KeyGrab).filter(|r| !r.is_empty())?.splitn(3, '#');
        let from = part.next()?.parse().ok();
        let base = part.next()?.parse().ok()?;
        Some(Self { from, base, picks: BKeySelection::parse(part.next().unwrap_or("")) })
    }

    pub fn active(s: &BState) -> bool {
        Self::now(s).is_some()
    }

    pub(super) fn write(&self, s: &mut BState) {
        let from = self.from.map(|f| f.to_string()).unwrap_or_default();
        s.set(CtxKey::KeyGrab, format!("{from}#{}#{}", self.base, self.picks.encode()));
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "tests_more.rs"]
mod tests_more;
