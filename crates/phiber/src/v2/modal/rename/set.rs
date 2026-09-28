//! The rename popup's rules: opening it on the active object, and applying
//! the name it was given.

use crate::v2::part::outliner::key::k_restrict;
use crate::v2::part::outliner::set::TOGGLES;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::select::BSelection;
use crate::v2::state::{BKey as _, AppKey, BPath, BState, CtxKey, ModalKey, OpKey, OutlinerKey, log, rule::BRule, val::{V_EMPTY, V_MODAL_RENAME}};

/// `TOPBAR_PT_name`'s label, and `marker.rename`'s.
pub const TITLE: &str = "Rename Active Item";
pub const MARKER_TITLE: &str = "Rename Marker";

/// `ctx.op.rename` (any value): the popup opens on the active object with
/// the caret in the field. Without an active object it says so instead.
pub struct BRenameOpenRule;

impl BRule for BRenameOpenRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Rename.path()
    }
    fn after(&self, _: &BPath, subject: &str, state: &mut BState) {
        state.remove(OpKey::Rename);
        state.remove(AppKey::RenameMarker);
        // `marker.rename` (F2 over the Timeline): the selected marker, else
        // the one at the playhead.
        if subject == "marker" {
            let markers = crate::v2::scene::marker::BMarkerList::now(state);
            let frame = crate::v2::scene::marker::BMarkerList::selected(state).first().copied().unwrap_or_else(|| crate::v2::state::key::int_or(state, CtxKey::Frame, 1));
            let Some(name) = markers.at(frame).map(str::to_string) else {
                log::push("No marker to rename".to_string());
                return;
            };
            state.set(AppKey::RenameMarker, frame.to_string());
            state.set(AppKey::RenameText, name);
            state.set(ModalKey::Open, V_MODAL_RENAME);
            crate::v2::state::edit::start(state, AppKey::RenameText);
            return;
        }
        let Some(active) = state.get(CtxKey::Object).filter(|n| !n.is_empty()).map(str::to_string) else {
            log::push("No active object to rename".to_string());
            return;
        };
        state.set(AppKey::RenameText, active);
        state.set(ModalKey::Open, V_MODAL_RENAME);
        crate::v2::state::edit::start(state, AppKey::RenameText);
    }
}

/// `ctx.rename.text` committed (Return in the field, or OK): the active
/// object takes the name, unless it is empty, unchanged or taken.
pub struct BRenameRule;

impl BRule for BRenameRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == AppKey::RenameText.path()
    }
    fn after(&self, _: &BPath, to: &str, state: &mut BState) {
        if state.get(ModalKey::Open) != Some(V_MODAL_RENAME) {
            return;
        }
        let to = to.trim().to_string();
        if let Some(frame) = state.get(AppKey::RenameMarker).and_then(|f| f.parse::<i32>().ok()) {
            state.remove(AppKey::RenameMarker);
            state.set(ModalKey::Open, V_EMPTY);
            crate::v2::state::edit::stop(state);
            let mut markers = crate::v2::scene::marker::BMarkerList::now(state);
            if let Some(entry) = markers.list.iter_mut().find(|(f, _)| *f == frame).filter(|_| !to.is_empty()) {
                log::push(format!("Renamed marker '{}' to '{to}'", entry.1));
                entry.1 = to;
                markers.write(state);
            }
            return;
        }
        let Some(from) = state.get(CtxKey::Object).filter(|n| !n.is_empty()).map(str::to_string) else { return };
        state.set(ModalKey::Open, V_EMPTY);
        // The popup is gone: so is the caret, whichever way the name came.
        crate::v2::state::edit::stop(state);
        if to.is_empty() || to == from {
            return;
        }
        let taken = crate::v2::scene::scene_objects(state).iter().any(|object| object.name() == to);
        if taken {
            log::push(format!("Cannot rename: '{to}' is taken"));
            return;
        }
        let mut edit = BSceneEdit::now(state);
        edit.rename(&from, &to);
        edit.write(state);
        let mut links = crate::v2::scene::links::BSceneLinks::now(state);
        links.rename(&from, &to);
        links.write(state);
        let mut transforms = crate::v2::scene::transform::BTransforms::now(state);
        transforms.rename(&from, &to);
        transforms.write(state);
        let mut actions = crate::v2::scene::action::BSceneActions::now(state);
        actions.rename(&from, &to);
        actions.write(state);
        state.set(CtxKey::Object, to.clone());
        let mut selection = BSelection::now(state);
        for name in &mut selection.list {
            if *name == from {
                *name = to.clone();
            }
        }
        selection.write(state);
        carry_row_keys(state, &from, &to);
        log::push(format!("Renamed '{from}' to '{to}'"));
    }
}

/// The Outliner row's toggles and fold go with the object.
fn carry_row_keys(state: &mut BState, from: &str, to: &str) {
    for toggle in TOGGLES {
        let old = k_restrict(from, toggle.state_key);
        if let Some(value) = state.get(&old).map(str::to_string) {
            state.set(&k_restrict(to, toggle.state_key), value);
            state.remove(&old);
        }
    }
    let old = OutlinerKey::Expanded(crate::v2::part::outliner::tree::intern_name(from)).path();
    if let Some(value) = state.get(&old).map(str::to_string) {
        state.set(&OutlinerKey::Expanded(crate::v2::part::outliner::tree::intern_name(to)).path(), value);
        state.remove(&old);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::state::val::V_TRUE;
    use crate::v2::view::prim::BAction;

    #[test]
    fn f2_opens_on_the_active_object_and_return_renames_it_everywhere() {
        crate::v2::scene::source::clear_loaded_scene();
        crate::v2::scene::source::clear_scene_objects();
        let mut state = BState::new();
        apply_action(&mut state, &BAction::Set(OpKey::Rename.path(), String::new()));
        assert_ne!(state.get(ModalKey::Open), Some(V_MODAL_RENAME), "nothing active: nothing to rename");
        state.set(CtxKey::Object, "Cube");
        state.set(&k_restrict("Cube", "hide_render"), V_TRUE);
        BSelection { list: vec!["Cube".into(), "Gem".into()] }.write(&mut state);
        apply_action(&mut state, &BAction::Set(OpKey::Rename.path(), String::new()));
        assert_eq!(state.get(ModalKey::Open), Some(V_MODAL_RENAME));
        assert_eq!(state.get(AppKey::RenameText), Some("Cube"), "the field holds the name");
        assert!(crate::v2::state::edit::is_editing(&state, AppKey::RenameText), "the caret is in it");
        // A taken name is refused; the popup closes either way.
        apply_action(&mut state, &BAction::Set(AppKey::RenameText.path(), "Gem".into()));
        assert_eq!(state.get(CtxKey::Object), Some("Cube"));
        assert_eq!(state.get(ModalKey::Open), Some(""));
        apply_action(&mut state, &BAction::Set(OpKey::Rename.path(), String::new()));
        apply_action(&mut state, &BAction::Set(AppKey::RenameText.path(), "Crate".into()));
        assert_eq!(state.get(CtxKey::Object), Some("Crate"), "the active object follows");
        assert_eq!(BSelection::now(&state).list, ["Crate", "Gem"], "so does the selection");
        assert_eq!(state.get(&k_restrict("Crate", "hide_render")), Some(V_TRUE), "and the row's toggles");
        let names: Vec<String> = crate::v2::scene::scene_objects(&state).iter().map(|o| o.name().to_string()).collect();
        assert!(names.contains(&"Crate".to_string()) && !names.contains(&"Cube".to_string()), "{names:?}");
        // The field's own typing does not rename: only a commit does.
        state.set(AppKey::RenameText, "Half");
        assert_eq!(state.get(CtxKey::Object), Some("Crate"));
    }
}
