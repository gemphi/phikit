//! The operators over the active object's action — Blender's
//! `template_action` (`ctx.op.action`: pick an action, pick a slot,
//! `action.new`, `action.unlink`) and the Action panel's Manual Frame
//! Range, Cyclic, Start and End (`ctx.op.action_range`, `_cyclic`,
//! `_start`, `_end`), and the Action menu's Push Down and Stash
//! (`ctx.op.action` `push_down` / `stash`, `nla.rs`). Each is read once, made on the session's edits and
//! said in the log.

use super::BSceneActions;
use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule, val::V_TRUE};

/// What one of the keys asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum BActionOp {
    Assign(String),
    Slot(String),
    New,
    Unlink,
    Range(bool),
    Cyclic(bool),
    Start(f32),
    End(f32),
    /// `action.push_down` (`stash` false) or `action.stash`.
    Nla { stash: bool },
}

impl BActionOp {
    const KEYS: [OpKey; 5] = [OpKey::Action, OpKey::ActionRange, OpKey::ActionCyclic, OpKey::ActionStart, OpKey::ActionEnd];

    pub fn parse(key: OpKey, value: &str) -> Option<Self> {
        let number = || value.trim().parse::<f32>().ok();
        Some(match key {
            OpKey::ActionRange => Self::Range(value == V_TRUE),
            OpKey::ActionCyclic => Self::Cyclic(value == V_TRUE),
            OpKey::ActionStart => Self::Start(number()?),
            OpKey::ActionEnd => Self::End(number()?),
            _ => match value.split_once(':') {
                Some(("assign", name)) => Self::Assign(name.to_string()),
                Some(("slot", id)) => Self::Slot(id.to_string()),
                _ if value == "new" => Self::New,
                _ if value == "unlink" => Self::Unlink,
                _ if value == "push_down" => Self::Nla { stash: false },
                _ if value == "stash" => Self::Nla { stash: true },
                _ => return None,
            },
        })
    }
}

impl BSceneActions {
    /// The active object's action and slot now, when it has one
    /// (`context.active_action`).
    pub fn active(state: &BState) -> Option<(String, String)> {
        let object = state.get(CtxKey::Object).filter(|o| !o.is_empty())?;
        crate::v2::scene::source::with_loaded(|model| Self::now(state).of(object, model?))
    }

    /// Manual Frame Range (`None` when off) and Cyclic of `action` now.
    pub fn range_of(&self, action: &str, model: &SceneModel) -> (Option<(f32, f32)>, bool) {
        match self.ranges.iter().find(|r| r.action == action) {
            Some(r) => (r.range, r.cyclic),
            None => self.action(action, model).map_or((None, false), |a| (a.manual_range, a.cyclic)),
        }
    }

    /// Make `op` on `object`'s action; what the log says, or `None` when
    /// there is nothing to do.
    pub fn operate(&mut self, op: &BActionOp, object: &str, model: &SceneModel) -> Option<String> {
        let current = self.of(object, model);
        if let BActionOp::Assign(name) = op {
            let action = self.action(name, model)?;
            let before = match &current {
                Some((a, slot)) if a == name => Some(slot.clone()),
                _ => model.actions.iter().find(|a| a.name == *name).and_then(|a| a.slot_of(object)).map(|s| s.identifier.clone()),
            };
            let slot = Self::slot_for(&action, object, before.as_deref());
            self.assign(object, name, &slot);
            return Some(format!("{object} uses {name}"));
        }
        if let BActionOp::Nla { stash } = op {
            return self.lay_nla(object, *stash, model);
        }
        if matches!(op, BActionOp::New) {
            let taken = self.names(model);
            let (name, source, slot) = match &current {
                Some((action, slot)) => (Self::unique(action, &taken), action.clone(), slot.clone()),
                None => (Self::unique("Action", &taken), String::new(), String::new()),
            };
            self.made.push((name.clone(), source));
            self.assign(object, &name, &slot);
            return Some(format!("New action {name}"));
        }
        let (action, slot) = current?;
        let (range, cyclic) = self.range_of(&action, model);
        let span = self.action(&action, model).map_or((1.0, 1.0), |a| (a.frame_start, a.frame_end));
        match op {
            BActionOp::Slot(id) => {
                self.assign(object, &action, id);
                return Some(if id.is_empty() { format!("{object}: no slot") } else { format!("{object} animates as {}", id.get(2..).unwrap_or(id)) });
            }
            BActionOp::Unlink => {
                self.assign(object, "", &slot);
                return Some(format!("Unlinked {action}"));
            }
            // Turning the range on starts it at the keys' span, as Blender does.
            BActionOp::Range(on) => self.set_range(&action, on.then(|| range.unwrap_or(span)), cyclic),
            BActionOp::Cyclic(on) => self.set_range(&action, range, *on),
            // `rna_Action_frame_start_set`: the end follows a start past it,
            // and the start an end before it.
            BActionOp::Start(v) => self.set_range(&action, range.map(|(_, e)| (*v, e.max(*v))), cyclic),
            BActionOp::End(v) => self.set_range(&action, range.map(|(s, _)| (s.min(*v), *v)), cyclic),
            BActionOp::Assign(_) | BActionOp::New | BActionOp::Nla { .. } => {}
        }
        Some(format!("{action}: frame range"))
    }
}

/// `ctx.op.action*` → the edit, on the active object.
pub struct BActionOpRule;

impl BRule for BActionOpRule {
    fn wants(&self, path: &BPath) -> bool {
        BActionOp::KEYS.iter().any(|k| *path == k.path())
    }
    fn after(&self, path: &BPath, value: &str, state: &mut BState) {
        let Some(key) = BActionOp::KEYS.into_iter().find(|k| *path == k.path()) else { return };
        state.remove(key);
        let (Some(op), Some(object)) = (BActionOp::parse(key, value), state.get(CtxKey::Object).filter(|o| !o.is_empty()).map(str::to_string)) else { return };
        let mut edits = BSceneActions::now(state);
        let said = crate::v2::scene::source::with_loaded(|model| model.and_then(|m| edits.operate(&op, &object, m)));
        if let Some(said) = said {
            edits.write(state);
            log::push(said);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keys_read_as_operators() {
        assert_eq!(BActionOp::parse(OpKey::Action, "assign:Spin"), Some(BActionOp::Assign("Spin".into())));
        assert_eq!(BActionOp::parse(OpKey::Action, "slot:OBCube"), Some(BActionOp::Slot("OBCube".into())));
        assert_eq!(BActionOp::parse(OpKey::Action, "new"), Some(BActionOp::New));
        assert_eq!(BActionOp::parse(OpKey::Action, "stash"), Some(BActionOp::Nla { stash: true }));
        assert_eq!(BActionOp::parse(OpKey::ActionRange, "true"), Some(BActionOp::Range(true)));
        assert_eq!(BActionOp::parse(OpKey::ActionStart, " 12 "), Some(BActionOp::Start(12.0)));
        assert_eq!(BActionOp::parse(OpKey::ActionEnd, "x"), None);
    }
}
