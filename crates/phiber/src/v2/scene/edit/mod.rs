//! Edits to the scene since the file was read: objects added, deleted,
//! duplicated and renamed. The loaded model stays what the file said; the
//! edits are four keys (`ctx.scene.added` / `deleted` / `copied` /
//! `renamed`) applied over its object list, so Undo is a state step and a
//! reload is a clean slate.
//!
//! Blender ref: `object.add` / `mesh.primitive_*_add` (at the 3D cursor —
//! the origin here — named after the primitive, `.001` when taken),
//! `object.delete`, `object.duplicate` (in place), `object.hide_view_set`,
//! `object.hide_view_clear` and the rename popup (`TOPBAR_PT_name`, F2).

mod rule;
mod shape;
pub use rule::BSceneEditRule;
pub use shape::{SHAPES, primitive_mesh, unique_name};

use crate::phiphx::scene::IDENTITY_M4;
use crate::v2::scene::BObject3D;
use crate::v2::state::{BState, SceneKey};

/// Entries within a key; name and value within an entry.
const SEP: char = '|';
const IS: char = '=';

/// What has changed since the file was read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BSceneEdit {
    /// `(name, shape slug)` of every added primitive.
    pub added: Vec<(String, String)>,
    pub deleted: Vec<String>,
    /// `(copy, source)` of every duplicate.
    pub copied: Vec<(String, String)>,
    /// `(new, old)` of every rename, `old` being the name the object had in
    /// the file (or was added / duplicated under).
    pub renamed: Vec<(String, String)>,
}

fn pairs(raw: Option<&str>) -> Vec<(String, String)> {
    raw.unwrap_or("")
        .split(SEP)
        .filter_map(|entry| entry.split_once(IS).map(|(a, b)| (a.to_string(), b.to_string())))
        .collect()
}

fn join_pairs(list: &[(String, String)]) -> String {
    list.iter().map(|(a, b)| format!("{a}{IS}{b}")).collect::<Vec<_>>().join(&SEP.to_string())
}

impl BSceneEdit {
    pub fn now(state: &BState) -> Self {
        Self {
            added: pairs(state.get(SceneKey::Added)),
            deleted: state.get(SceneKey::Deleted).unwrap_or("").split(SEP).filter(|n| !n.is_empty()).map(str::to_string).collect(),
            copied: pairs(state.get(SceneKey::Copied)),
            renamed: pairs(state.get(SceneKey::Renamed)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty() && self.copied.is_empty() && self.renamed.is_empty()
    }

    pub fn write(&self, state: &mut BState) {
        state.set(SceneKey::Added, join_pairs(&self.added));
        state.set(SceneKey::Deleted, self.deleted.join(&SEP.to_string()));
        state.set(SceneKey::Copied, join_pairs(&self.copied));
        state.set(SceneKey::Renamed, join_pairs(&self.renamed));
    }

    /// The object now called `from` is called `to` from here on. An added
    /// or duplicated object is renamed in its own entry; a file object gets
    /// a rename entry, updated in place when renamed again, and dropped
    /// when it gets its file name back.
    pub fn rename(&mut self, from: &str, to: &str) {
        if let Some(entry) = self.added.iter_mut().find(|(name, _)| name == from) {
            entry.0 = to.to_string();
        } else if let Some(entry) = self.copied.iter_mut().find(|(copy, _)| copy == from) {
            entry.0 = to.to_string();
        } else if let Some(i) = self.renamed.iter().position(|(new, _)| new == from) {
            if self.renamed[i].1 == to {
                self.renamed.remove(i);
            } else {
                self.renamed[i].0 = to.to_string();
            }
        } else {
            self.renamed.push((to.to_string(), from.to_string()));
        }
    }

    /// One number for the memo that keys the object list.
    pub fn fingerprint(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut h);
        h.finish()
    }

    /// The file's objects with the edits applied: deletions removed,
    /// duplicates beside their sources, added primitives at the origin.
    pub fn apply(&self, mut objects: Vec<BObject3D>) -> Vec<BObject3D> {
        if self.is_empty() {
            return objects;
        }
        objects.retain(|object| !self.deleted.iter().any(|gone| gone == object.name()));
        for (copy, source) in &self.copied {
            if let Some(original) = objects.iter().find(|object| object.name() == source).cloned() {
                objects.push(original.renamed(copy));
            }
        }
        for (name, shape) in &self.added {
            if let Some(mesh) = primitive_mesh(shape) {
                objects.push(BObject3D::Loaded {
                    name: name.clone(),
                    mesh,
                    location: [0.0; 3],
                    scale: [1.0; 3],
                    matrix_world: IDENTITY_M4,
                    materials: Vec::new(),
                    material_indices: Default::default(),
                    uvs: Default::default(),
                });
            }
        }
        for (new, old) in &self.renamed {
            if let Some(i) = objects.iter().position(|object| object.name() == old) {
                let object = objects.remove(i);
                objects.insert(i, object.renamed(new));
            }
        }
        objects
    }
}

impl std::hash::Hash for BSceneEdit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.added.hash(state);
        self.deleted.hash(state);
        self.copied.hash(state);
        self.renamed.hash(state);
    }
}

impl BObject3D {
    /// The same object under another name.
    pub fn renamed(mut self, to: &str) -> Self {
        match &mut self {
            BObject3D::Primitive { name, .. } | BObject3D::Loaded { name, .. } => *name = to.to_string(),
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::state::{BKey as _, CtxKey, OpKey};
    use crate::v2::view::prim::BAction;

    fn edit(state: &mut BState, op: &str) {
        apply_action(state, &BAction::Set(OpKey::SceneEdit.path(), op.into()));
    }

    fn names(state: &BState) -> Vec<String> {
        BSceneEdit::now(state).apply(super::super::current_scene(state, None)).iter().map(|o| o.name().to_string()).collect()
    }

    #[test]
    fn add_delete_and_duplicate_change_the_list_the_viewport_draws() {
        crate::v2::scene::source::clear_loaded_scene();
        crate::v2::scene::source::clear_scene_objects();
        let mut state = BState::new();
        edit(&mut state, "add:cube");
        assert!(names(&state).contains(&"Cube.001".to_string()), "the default Cube is taken: {:?}", names(&state));
        assert_eq!(state.get(CtxKey::Object), Some("Cube.001"), "the new object is active");
        edit(&mut state, "add:monkey");
        assert!(names(&state).contains(&"Suzanne".to_string()));
        // Suzanne is selected; duplicate it, then delete the copy.
        edit(&mut state, "duplicate");
        assert!(names(&state).contains(&"Suzanne.001".to_string()));
        assert_eq!(state.get(CtxKey::Object), Some("Suzanne.001"));
        edit(&mut state, "delete");
        assert!(!names(&state).contains(&"Suzanne.001".to_string()));
        assert!(state.get(CtxKey::Object).is_none(), "nothing is active after a delete");
        // Delete a file object: it goes to the deleted list.
        state.set(CtxKey::Object, "Sphere");
        super::super::select::BSelection { list: vec!["Sphere".into()] }.write(&mut state);
        edit(&mut state, "delete");
        assert!(!names(&state).contains(&"Sphere".to_string()));
        assert_eq!(BSceneEdit::now(&state).deleted, ["Sphere"]);
        assert!(state.get(OpKey::SceneEdit).is_none());
    }

    #[test]
    fn a_rename_follows_the_object_whatever_it_was_and_undoes_itself_on_the_way_back() {
        crate::v2::scene::source::clear_loaded_scene();
        crate::v2::scene::source::clear_scene_objects();
        let mut state = BState::new();
        let mut edit = BSceneEdit::now(&state);
        edit.rename("Cube", "Crate");
        edit.rename("Crate", "Box");
        assert_eq!(edit.renamed, [("Box".to_string(), "Cube".to_string())], "one entry, updated in place");
        edit.write(&mut state);
        assert!(names(&state).contains(&"Box".to_string()) && !names(&state).contains(&"Cube".to_string()));
        edit.rename("Box", "Cube");
        assert!(edit.renamed.is_empty(), "back to its file name: no entry");
        // Added and duplicated objects are renamed in their own entries.
        edit.added.push(("Suzanne".into(), "monkey".into()));
        edit.copied.push(("Sphere.001".into(), "Sphere".into()));
        edit.rename("Suzanne", "Monkey");
        edit.rename("Sphere.001", "Ball");
        assert_eq!((edit.added[0].0.as_str(), edit.copied[0].0.as_str(), edit.renamed.len()), ("Monkey", "Ball", 0));
        edit.write(&mut state);
        let list = names(&state);
        assert!(list.contains(&"Monkey".to_string()) && list.contains(&"Ball".to_string()), "{list:?}");
    }
}
