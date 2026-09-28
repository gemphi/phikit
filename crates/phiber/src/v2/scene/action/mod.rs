//! The session's actions — the Action Editor's `template_action` (browse
//! every action in the file, New, Unlink), its slot selector, and the
//! sidebar's Action panel (Manual Frame Range, Cyclic) — as three keys over
//! the file (`ctx.scene.assigned`, `.made_actions`, `.action_ranges`), so
//! Undo steps them and a save writes them (phiphx `write/action.rs`,
//! checked in Blender by `blender_opens_saves`).
//!
//! `set.rs` makes the scene the rest of the app reads out of them: the
//! file's model with the new actions listed, each action's users as
//! assigned and each object moving by the action and slot it has now.
//! `rule.rs` is the operator.

pub mod rule;
pub mod set;

use crate::phiphx::scene::{SceneAction, SceneModel};
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
        Self { assigned, made: Self::entries(state, SceneKey::MadeActions), ranges }
    }

    pub fn write(&self, state: &mut BState) {
        let join = |list: Vec<String>| list.join(&SEP.to_string());
        state.set(SceneKey::Assigned, join(self.assigned.iter().map(|p| format!("{}{IS}{}{AND}{}", p.object, p.action, p.slot)).collect()));
        state.set(SceneKey::MadeActions, join(self.made.iter().map(|(n, s)| format!("{n}{IS}{s}")).collect()));
        let range = |r: &BActionRange| r.range.map_or("-,-".to_string(), |(s, e)| format!("{s},{e}"));
        state.set(SceneKey::ActionRanges, join(self.ranges.iter().map(|r| format!("{}{IS}{},{}", r.action, range(r), u8::from(r.cyclic))).collect()));
    }

    pub fn is_empty(&self) -> bool {
        self.assigned.is_empty() && self.made.is_empty() && self.ranges.is_empty()
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
    }

    /// Forget `object` (deleted).
    pub fn forget(&mut self, object: &str) {
        self.assigned.retain(|p| p.object != object);
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

    /// The writer's edits, added to `edits` (`save::gather`).
    pub fn edits(&self, edits: crate::phiphx::blend::write::BlendEdits) -> crate::phiphx::blend::write::BlendEdits {
        use crate::phiphx::blend::write::{BlendActionAssign, BlendActionRange};
        let some = |s: &str| (!s.is_empty()).then(|| s.to_string());
        crate::phiphx::blend::write::BlendEdits {
            actions: self.made.clone(),
            assigned: self.assigned.iter().map(|p| BlendActionAssign { object: p.object.clone(), action: some(&p.action), slot: some(&p.slot) }).collect(),
            ranges: self.ranges.iter().map(|r| BlendActionRange { action: r.action.clone(), range: r.range, cyclic: r.cyclic }).collect(),
            ..edits
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edits_round_trip_through_their_keys_and_names_copy_as_blender_does() {
        let mut s = BState::new();
        let mut edits = BSceneActions::default();
        edits.assign("Cube", "CubeAction.001", "OBCube");
        edits.assign("Camera", "", "");
        edits.made.push(("CubeAction.001".into(), "CubeAction".into()));
        edits.set_range("CubeAction.001", Some((5.0, 30.0)), true);
        edits.set_range("Spin", None, false);
        edits.write(&mut s);
        assert_eq!(BSceneActions::now(&s), edits);
        let taken = vec!["CubeAction".to_string(), "CubeAction.001".to_string()];
        assert_eq!(BSceneActions::unique("CubeAction", &taken), "CubeAction.002");
        assert_eq!(BSceneActions::unique("CubeAction.001", &taken), "CubeAction.002");
        assert_eq!(BSceneActions::unique("Action", &taken), "Action");
    }
}
