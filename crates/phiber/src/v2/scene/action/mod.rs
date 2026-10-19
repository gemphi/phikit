//! The session's actions — the Action Editor's `template_action` (browse
//! every action in the file, New, Unlink), its slot selector, the
//! sidebar's Action panel (Manual Frame Range, Cyclic) and the keys I
//! inserts — as keys over the file (`ctx.scene.assigned`,
//! `.made_actions`, `.action_ranges`, `.keyed`), so Undo steps them and a
//! save writes them (phiphx `write/action.rs`, `write/keys.rs`, checked in
//! Blender by `blender_opens_saves`), and the NLA stacks Push Down, Stash
//! and the NLA editor edit (`.nla`, `nla.rs`, `nla_editor.rs`, written by
//! `write/nla.rs`).
//!
//! `set.rs` makes the scene the rest of the app reads out of them: the
//! file's model with the new actions listed, each action's users as
//! assigned and each object moving by the action and slot it has now.
//! `rule.rs` is the operator; `keys.rs` the inserts, `keying.rs` I.

pub mod autokey;
pub mod channel;
pub mod clean;
pub mod dope;
pub mod keying;
pub mod keys;
pub mod nla;
pub mod nla_editor;
pub mod preview;
pub mod reshape;
pub mod retime;
pub mod rule;
pub mod set;

use retime::BKeyStep;

use crate::phiphx::scene::{SceneAction, SceneModel, SceneNla};
use crate::v2::state::{BState, SceneKey};

const SEP: char = '|';
const IS: char = '=';
const AND: char = '#';

/// An object's action as the session left it: `action` through `slot`
/// (its identifier); an empty `action` is Unlink, an empty `slot` none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BActionPick {
    pub object: String,
    pub action: String,
    pub slot: String,
}

/// An action's Manual Frame Range (`None` when off) and Cyclic Animation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BActionRange {
    pub action: String,
    pub range: Option<(f32, f32)>,
    pub cyclic: bool,
}

/// Every action edit the session made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSceneActions {
    pub assigned: Vec<BActionPick>,
    /// `(name, source)`: a copy of `source`, or empty when it is empty.
    pub made: Vec<(String, String)>,
    pub ranges: Vec<BActionRange>,
    /// The keys inserted, moved and deleted, in order (`retime.rs`).
    pub keyed: Vec<BKeyStep>,
    /// F-curve channel properties changed — extrapolation, protect, mute
    /// (`channel.rs`).
    pub channels: Vec<channel::BChannelEdit>,
    /// Each NLA stack the session edited, whole (`nla.rs`).
    pub nla: Vec<SceneNla>,
}

impl BSceneActions {
    fn entries(state: &BState, key: SceneKey) -> Vec<(String, String)> {
        state.get(key).unwrap_or("").split(SEP).filter_map(|e| e.split_once(IS)).map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    pub fn now(state: &BState) -> Self {
        let assigned = Self::entries(state, SceneKey::Assigned)
            .into_iter()
            .map(|(object, rest)| {
                let (action, slot) = rest.split_once(AND).unwrap_or((rest.as_str(), ""));
                BActionPick { object, action: action.to_string(), slot: slot.to_string() }
            })
            .collect();
        let ranges = Self::entries(state, SceneKey::ActionRanges)
            .into_iter()
            .map(|(action, rest)| {
                let mut part = rest.split(',');
                let (start, end, cyclic) = (part.next().and_then(|v| v.parse().ok()), part.next().and_then(|v| v.parse().ok()), part.next() == Some("1"));
                BActionRange { action, range: start.zip(end), cyclic }
            })
            .collect();
        let keyed = BKeyStep::parse_all(state.get(SceneKey::Keyed).unwrap_or(""));
        let channels = channel::BChannelEdit::parse_all(state.get(SceneKey::Channels).unwrap_or(""));
        let nla = Self::parse_nla(state.get(SceneKey::Nla).unwrap_or(""));
        Self { assigned, made: Self::entries(state, SceneKey::MadeActions), ranges, keyed, channels, nla }
    }

    pub fn write(&self, state: &mut BState) {
        let join = |list: Vec<String>| list.join(&SEP.to_string());
        state.set(SceneKey::Assigned, join(self.assigned.iter().map(|p| format!("{}{IS}{}{AND}{}", p.object, p.action, p.slot)).collect()));
        state.set(SceneKey::MadeActions, join(self.made.iter().map(|(n, s)| format!("{n}{IS}{s}")).collect()));
        let range = |r: &BActionRange| r.range.map_or("-,-".to_string(), |(s, e)| format!("{s},{e}"));
        state.set(SceneKey::ActionRanges, join(self.ranges.iter().map(|r| format!("{}{IS}{},{}", r.action, range(r), u8::from(r.cyclic))).collect()));
        state.set(SceneKey::Keyed, BKeyStep::encode_all(&self.keyed));
        state.set(SceneKey::Channels, channel::BChannelEdit::encode_all(&self.channels));
        state.set(SceneKey::Nla, Self::encode_nla(&self.nla));
    }

    pub fn is_empty(&self) -> bool {
        self.assigned.is_empty() && self.made.is_empty() && self.ranges.is_empty() && self.keyed.is_empty() && self.channels.is_empty() && self.nla.is_empty()
    }

    /// Set a channel's properties, replacing any edit already made to it — one
    /// entry per `(action, slot, path, index)`, so a property does not stack
    /// with its own past.
    pub fn set_channel(&mut self, edit: channel::BChannelEdit) {
        if let Some(found) = self.channels.iter_mut().find(|e| e.id == edit.id) {
            // Keep the fields the new edit leaves unset, so Protect then Mute
            // on the same channel both stand.
            found.extend = edit.extend.or(found.extend);
            found.protect = edit.protect.or(found.protect);
            found.mute = edit.mute.or(found.mute);
        } else {
            self.channels.push(edit);
        }
    }

    /// `object` animated by `action` through `slot` from here on.
    pub fn assign(&mut self, object: &str, action: &str, slot: &str) {
        self.assigned.retain(|p| p.object != object);
        self.assigned.push(BActionPick { object: object.to_string(), action: action.to_string(), slot: slot.to_string() });
    }

    pub fn set_range(&mut self, action: &str, range: Option<(f32, f32)>, cyclic: bool) {
        self.ranges.retain(|r| r.action != action);
        self.ranges.push(BActionRange { action: action.to_string(), range, cyclic });
    }

    /// The object called `from` is called `to` from here on.
    pub fn rename(&mut self, from: &str, to: &str) {
        self.assigned.iter_mut().filter(|p| p.object == from).for_each(|p| p.object = to.to_string());
        self.nla.iter_mut().filter(|n| n.object == from).for_each(|n| n.object = to.to_string());
    }

    /// Forget `object` (deleted).
    pub fn forget(&mut self, object: &str) {
        self.assigned.retain(|p| p.object != object);
        self.nla.retain(|n| n.object != object);
    }

    /// Every action's name, the file's and the session's, sorted as
    /// Blender's browse menu lists them.
    pub fn names(&self, model: &SceneModel) -> Vec<String> {
        let mut out: Vec<String> = model.actions.iter().map(|a| a.name.clone()).chain(self.made.iter().map(|(n, _)| n.clone())).collect();
        out.sort_by_key(|n| n.to_lowercase());
        out.dedup();
        out
    }

    /// `name` made unique among `taken` as Blender names a copy
    /// (`BLI_uniquename`): `CubeAction` → `CubeAction.001`, and a copy of
    /// `CubeAction.001` is `CubeAction.002`.
    pub fn unique(name: &str, taken: &[String]) -> String {
        if !taken.iter().any(|t| t == name) {
            return name.to_string();
        }
        let base = match name.rsplit_once('.') {
            Some((base, n)) if n.len() == 3 && n.chars().all(|c| c.is_ascii_digit()) => base,
            _ => name,
        };
        (1..1000).map(|i| format!("{base}.{i:03}")).find(|n| !taken.contains(n)).unwrap_or_else(|| format!("{base}.999"))
    }

    /// The slot `action` gives `object` when assigned
    /// (`generic_slot_for_autoassign`): the one it had there before, else
    /// the one named for it (`OB` + its name), else none.
    pub fn slot_for(action: &SceneAction, object: &str, before: Option<&str>) -> String {
        let named = format!("OB{object}");
        [before.unwrap_or(""), named.as_str()]
            .into_iter()
            .find(|id| !id.is_empty() && action.slots.iter().any(|s| s.identifier == *id))
            .unwrap_or("")
            .to_string()
    }

    /// The writer's edits over the file's `model`, added to `edits`
    /// (`save::gather`): each slot the keys touched, with every curve that
    /// differs from the action before them, and the curves they emptied.
    pub fn edits(&self, model: &SceneModel, edits: crate::phiphx::blend::write::BlendEdits) -> crate::phiphx::blend::write::BlendEdits {
        use crate::phiphx::blend::write::{BlendActionAssign, BlendActionRange, BlendKeyInsert};
        let actions = self.keyed_actions(model);
        let mut keys: Vec<BlendKeyInsert> = Vec::new();
        for step in &self.keyed {
            if keys.iter().any(|k| k.action == step.action() && k.slot == step.slot()) {
                continue;
            }
            let run = |a: &SceneAction| match step.slot().is_empty() {
                true => Some(a.curves.clone()),
                false => a.slots.iter().find(|s| s.identifier == step.slot()).map(|s| a.curves[s.curves.clone()].to_vec()),
            };
            let before = self.action(step.action(), model).as_ref().and_then(run).unwrap_or_default();
            let now = actions.iter().find(|a| a.name == step.action()).and_then(run).unwrap_or_default();
            let same = |a: &crate::phiphx::blend::BFCurve, b: &crate::phiphx::blend::BFCurve| a.path == b.path && a.index == b.index;
            let curves = now.iter().filter(|c| !before.iter().any(|b| b == *c)).cloned().collect();
            let removed = before.iter().filter(|b| !now.iter().any(|c| same(b, c))).map(|b| (b.path.clone(), b.index)).collect();
            keys.push(BlendKeyInsert { action: step.action().to_string(), slot: step.slot().to_string(), curves, group: "Object Transforms".into(), removed });
        }
        let some = |s: &str| (!s.is_empty()).then(|| s.to_string());
        crate::phiphx::blend::write::BlendEdits {
            actions: self.made.clone(),
            assigned: self.assigned.iter().map(|p| BlendActionAssign { object: p.object.clone(), action: some(&p.action), slot: some(&p.slot) }).collect(),
            ranges: self.ranges.iter().map(|r| BlendActionRange { action: r.action.clone(), range: r.range, cyclic: r.cyclic }).collect(),
            keys,
            nla: self.nla_stacks(),
            ..edits
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
