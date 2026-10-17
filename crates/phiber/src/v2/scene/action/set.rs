//! The scene with the session's actions in it — what the Dope Sheet, the
//! Graph and NLA editors, the Timeline's keys and the viewport read in
//! place of the file's model once an action was made, assigned or retimed:
//!
//! - New actions join the list: a copy carries its source's curves and
//!   slots (`BKE_id_copy`), an empty one nothing (`BKE_action_add`).
//! - The keys I inserted go in, in order (`keys.rs`), making the slots and
//!   curves they need; whoever those actions drive moves by them.
//! - Each action's users and bindings are who has it now.
//! - Each assigned object moves by its action's slot, or not at all with
//!   no slot or no action, as Blender evaluates `AnimData`.
//! - The NLA stacks the session edited stand in the file's (`nla.rs`);
//!   whoever has one plays its NLA under its action.
//! - The NLA editor's action line names the action an object has now.
//!
//! Built once per edit and model (`BActionScene`), so a frame costs a
//! pointer comparison.

use super::{BActionRange, BSceneActions};
use crate::phiphx::scene::{SceneAction, SceneModel, SceneNla};
use crate::v2::state::BState;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// The last scene built: from which model, for which edits.
    static BUILT: RefCell<Option<(Arc<SceneModel>, BSceneActions, Arc<SceneModel>)>> = const { RefCell::new(None) };
}

/// The session's action edits over a loaded model.
pub struct BActionScene;

impl BActionScene {
    /// `model` with the session's actions in — the same `Arc` when there
    /// are none, the one built last when nothing changed.
    pub fn over(model: Arc<SceneModel>, state: &BState) -> Arc<SceneModel> {
        let edits = BSceneActions::now(state);
        if edits.is_empty() {
            return model;
        }
        if let Some(hit) = BUILT.with(|cell| cell.borrow().as_ref().filter(|(m, e, _)| Arc::ptr_eq(m, &model) && *e == edits).map(|(_, _, built)| Arc::clone(built))) {
            return hit;
        }
        let built = Arc::new(edits.apply(&model));
        BUILT.with(|cell| *cell.borrow_mut() = Some((model, edits, Arc::clone(&built))));
        built
    }

    /// The file's model under `model`, when `model` is one `over` built.
    pub fn raw(model: Arc<SceneModel>) -> Arc<SceneModel> {
        BUILT.with(|cell| cell.borrow().as_ref().filter(|(_, _, built)| Arc::ptr_eq(built, &model)).map(|(raw, _, _)| Arc::clone(raw))).unwrap_or(model)
    }
}

impl BSceneActions {
    /// The action called `name` as the session has it: the file's, or a
    /// made one (a copy of its source, renamed, with no users yet).
    pub fn action(&self, name: &str, model: &SceneModel) -> Option<SceneAction> {
        if let Some(action) = model.actions.iter().find(|a| a.name == name) {
            return Some(action.clone());
        }
        let (_, source) = self.made.iter().find(|(n, _)| n == name)?;
        let base = match source.is_empty() {
            true => SceneAction::default(),
            false => self.action(source, model).unwrap_or_default(),
        };
        Some(SceneAction { name: name.to_string(), users: Vec::new(), bindings: Vec::new(), ..base })
    }

    /// `(action, slot identifier)` `object` has now, when it has an action.
    pub fn of(&self, object: &str, model: &SceneModel) -> Option<(String, String)> {
        if let Some(pick) = self.assigned.iter().find(|p| p.object == object) {
            return (!pick.action.is_empty()).then(|| (pick.action.clone(), pick.slot.clone()));
        }
        model.actions.iter().find(|a| a.used_by(object)).map(|a| (a.name.clone(), a.slot_of(object).map_or(String::new(), |s| s.identifier.clone())))
    }

    /// Every action as the session has it: the file's and the made ones,
    /// with the keys inserted in.
    pub fn keyed_actions(&self, model: &SceneModel) -> Vec<SceneAction> {
        let mut actions = model.actions.clone();
        actions.extend(self.made.iter().filter_map(|(name, _)| self.action(name, model)));
        for insert in &self.keyed {
            insert.apply(&mut actions);
        }
        // Channel properties last, over the curves the key steps leave
        // (`channel.rs`).
        for edit in &self.channels {
            edit.apply(&mut actions);
        }
        actions
    }

    /// `model` with these edits in.
    pub fn apply(&self, model: &SceneModel) -> SceneModel {
        let mut out = model.clone();
        out.actions = self.keyed_actions(model);
        // Laid before the assignments, which name each action line after.
        self.lay(&mut out);
        for pick in &self.assigned {
            for action in &mut out.actions {
                action.users.retain(|u| *u != pick.object);
                action.bindings.retain(|(o, _)| *o != pick.object);
            }
            let action = out.actions.iter_mut().find(|a| a.name == pick.action && !pick.action.is_empty());
            let curves = match action {
                Some(action) => {
                    let handle = action.slots.iter().find(|s| s.identifier == pick.slot && !pick.slot.is_empty()).map(|s| s.handle);
                    action.users.push(pick.object.clone());
                    action.bindings.push((pick.object.clone(), handle.unwrap_or(i32::MIN)));
                    handle.map_or(Vec::new(), |h| action.curves_of(h).to_vec())
                }
                None => Vec::new(),
            };
            if let Some(motion) = out.motion.motion.list.iter_mut().find(|m| m.name == pick.object) {
                motion.curves = curves;
            }
            let line = (!pick.action.is_empty()).then(|| pick.action.clone());
            match out.nla.iter_mut().find(|n| n.object == pick.object) {
                Some(nla) => nla.action = line,
                None if line.is_some() => out.nla.push(SceneNla { object: pick.object.clone(), action: line, ..Default::default() }),
                None => {}
            }
        }
        // Whoever an action with new keys drives moves by them.
        let SceneModel { actions, motion, .. } = &mut out;
        for action in actions.iter().filter(|a| self.keyed.iter().any(|k| k.action() == a.name)) {
            for (object, handle) in &action.bindings {
                if let Some(m) = motion.motion.list.iter_mut().find(|m| m.name == *object) {
                    m.curves = action.curves_of(*handle).to_vec();
                }
            }
        }
        for BActionRange { action, range, cyclic } in &self.ranges {
            if let Some(a) = out.actions.iter_mut().find(|a| a.name == *action) {
                a.manual_range = *range;
                a.cyclic = *cyclic;
            }
        }
        out
    }
}
