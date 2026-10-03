//! `ctx.op.scene_edit`: the operators that edit the scene — add a
//! primitive, delete, duplicate, hide and unhide the selection.

use super::{BSceneEdit, SHAPES, unique_name};
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};

/// `ctx.op.scene_edit` → `add:<shape>` / `delete` / `duplicate` / `hide` / `unhide`.
pub struct BSceneEditRule;

impl BRule for BSceneEditRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::SceneEdit.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let names: Vec<String> = crate::v2::scene::source::scene_objects(state).iter().map(|o| o.name().to_string()).collect();
        let selection = crate::v2::scene::select::BSelection::now(state);
        let mut edit = BSceneEdit::now(state);
        match op.split_once(':') {
            Some(("add", shape)) => {
                let Some(&(_, label, _)) = SHAPES.iter().find(|(slug, ..)| *slug == shape) else { return };
                let name = unique_name(label, &names);
                edit.added.push((name.clone(), shape.to_string()));
                edit.write(state);
                // At the 3D cursor, aligned to the world (`object.add`'s
                // default `align='WORLD'`).
                let at = crate::v2::scene::cursor::BCursor::now(state).location;
                if at != glam::Vec3::ZERO {
                    let mut transforms = crate::v2::scene::transform::BTransforms::now(state);
                    transforms.set(&name, crate::v2::scene::transform::BTransform { location: at.into(), rotation: [0.0; 3], scale: [1.0; 3] });
                    transforms.write(state);
                }
                state.set(CtxKey::Object, name.clone());
                crate::v2::scene::select::BSelection { list: vec![name.clone()] }.write(state);
                log::push(format!("Added {name}"));
            }
            None if op == "delete" => {
                for name in &selection.list {
                    if let Some(i) = edit.added.iter().position(|(n, _)| n == name) {
                        edit.added.remove(i);
                    } else if let Some(i) = edit.copied.iter().position(|(n, _)| n == name) {
                        edit.copied.remove(i);
                    } else if !edit.deleted.contains(name) {
                        edit.deleted.push(name.clone());
                    }
                }
                edit.write(state);
                let mut links = crate::v2::scene::links::BSceneLinks::now(state);
                selection.list.iter().for_each(|name| links.forget(name));
                links.write(state);
                let mut transforms = crate::v2::scene::transform::BTransforms::now(state);
                selection.list.iter().for_each(|name| transforms.forget(name));
                transforms.write(state);
                let mut actions = crate::v2::scene::action::BSceneActions::now(state);
                selection.list.iter().for_each(|name| actions.forget(name));
                actions.write(state);
                state.remove(CtxKey::Object);
                crate::v2::scene::select::BSelection::default().write(state);
                log::push(format!("Deleted {} object(s)", selection.list.len()));
            }
            None if op == "duplicate" => {
                let mut taken = names.clone();
                let mut copies = Vec::new();
                for source in &selection.list {
                    let copy = unique_name(source, &taken);
                    taken.push(copy.clone());
                    edit.copied.push((copy.clone(), source.clone()));
                    copies.push(copy);
                }
                edit.write(state);
                if let Some(first) = copies.first() {
                    state.set(CtxKey::Object, first.clone());
                }
                crate::v2::scene::select::BSelection { list: copies.clone() }.write(state);
                log::push(format!("Duplicated {} object(s)", copies.len()));
            }
            None if op == "hide" => {
                for name in &selection.list {
                    state.set(&crate::v2::part::outliner::key::k_restrict(name, "hide_viewport"), V_TRUE);
                }
                // What is hidden is not selected (`object.hide_view_set`).
                crate::v2::scene::select::BSelection::default().write(state);
                log::push(format!("Hid {} object(s)", selection.list.len()));
            }
            None if op == "unhide" => {
                let hidden: Vec<String> = names
                    .iter()
                    .map(|name| crate::v2::part::outliner::key::k_restrict(name, "hide_viewport").encode())
                    .filter(|key| state.get_encoded(key) == Some(V_TRUE))
                    .collect();
                for key in &hidden {
                    state.set_encoded(key, V_FALSE);
                }
                log::push("Revealed the hidden objects".to_string());
            }
            _ => {}
        }
        state.remove(OpKey::SceneEdit);
    }
}

