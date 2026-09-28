//! Where the session put objects and collections — the Outliner's drag and
//! drop (`outliner_dragdrop.cc`): an object dropped on a collection moves
//! there, or with Ctrl is linked there as well (`collection_drop_exec`);
//! one dropped on another object is parented to it (`parent_drop_exec`);
//! a collection dropped on another is nested in it. Four keys over the
//! file, as the other scene edits are (`ctx.scene.moved`, `.linked`,
//! `.parented`, `.nested`), so Undo steps them and a save writes them
//! (`save::gather` → phiphx `write/relink.rs`).
//!
//! [`BSceneTree`] (`tree.rs`) is the scene as it stands with them applied,
//! over [`BCollectionNode`] (`collection.rs`), the collection tree.

use crate::v2::state::{BState, SceneKey};

mod collection;
mod tree;
pub use self::collection::BCollectionNode;
pub use self::tree::BSceneTree;

const SEP: char = '|';
const IS: char = '=';

/// The session's moves, links, parents and nestings, by the objects' names
/// now and the collections' paths in the file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BSceneLinks {
    /// `(object, collection path)`.
    pub moved: Vec<(String, String)>,
    /// `(child, parent)`.
    pub parented: Vec<(String, String)>,
    /// `(object, collection path)`: in that collection as well.
    pub linked: Vec<(String, String)>,
    /// `(collection path, new parent's path)`, in the order they were made.
    pub nested: Vec<(String, String)>,
}

impl BSceneLinks {
    fn pairs(raw: Option<&str>) -> Vec<(String, String)> {
        raw.unwrap_or("").split(SEP).filter_map(|e| e.split_once(IS).map(|(a, b)| (a.to_string(), b.to_string()))).collect()
    }

    fn join(list: &[(String, String)]) -> String {
        list.iter().map(|(a, b)| format!("{a}{IS}{b}")).collect::<Vec<_>>().join(&SEP.to_string())
    }

    pub fn now(state: &BState) -> Self {
        Self {
            moved: Self::pairs(state.get(SceneKey::Moved)),
            parented: Self::pairs(state.get(SceneKey::Parented)),
            linked: Self::pairs(state.get(SceneKey::Linked)),
            nested: Self::pairs(state.get(SceneKey::Nested)),
        }
    }

    pub fn write(&self, state: &mut BState) {
        for (key, list) in [(SceneKey::Moved, &self.moved), (SceneKey::Parented, &self.parented), (SceneKey::Linked, &self.linked), (SceneKey::Nested, &self.nested)] {
            state.set(key, Self::join(list));
        }
    }

    /// `object` moves to `path`, out of every collection it was in — its
    /// links too (a later move replaces an earlier one).
    pub fn move_to(&mut self, object: &str, path: &str) {
        self.moved.retain(|(o, _)| o != object);
        self.linked.retain(|(o, _)| o != object);
        self.moved.push((object.to_string(), path.to_string()));
    }

    /// `object` is in `path` as well as where it is.
    pub fn link_to(&mut self, object: &str, path: &str) {
        if !self.linked.iter().any(|(o, p)| o == object && p == path) {
            self.linked.push((object.to_string(), path.to_string()));
        }
    }

    pub fn parent(&mut self, child: &str, parent: &str) {
        self.parented.retain(|(c, _)| c != child);
        self.parented.push((child.to_string(), parent.to_string()));
    }

    /// The collection at `child` goes into the one at `parent`. Kept as a
    /// history, replayed in order (`BCollectionNode::now`), as the save
    /// replays it: each step was checked against the tree it was made on.
    pub fn nest(&mut self, child: &str, parent: &str) {
        self.nested.push((child.to_string(), parent.to_string()));
    }

    /// Forget `object` (deleted): its move, its links, its parent, and it
    /// as a parent.
    pub fn forget(&mut self, object: &str) {
        self.moved.retain(|(o, _)| o != object);
        self.linked.retain(|(o, _)| o != object);
        self.parented.retain(|(c, p)| c != object && p != object);
    }

    /// The object called `from` is called `to` from here on.
    pub fn rename(&mut self, from: &str, to: &str) {
        for (a, _) in self.moved.iter_mut().chain(self.linked.iter_mut()).chain(self.parented.iter_mut()) {
            if a == from {
                *a = to.to_string();
            }
        }
        for (_, parent) in self.parented.iter_mut() {
            if parent == from {
                *parent = to.to_string();
            }
        }
    }

    /// What a save writes of them (`save::gather`). The nestings go by
    /// name — collection names are unique in a file, and a path's last
    /// segment names its collection wherever earlier nestings put it.
    pub fn edits(self) -> crate::phiphx::blend::write::BlendEdits {
        let name = |path: &str| path.rsplit('/').next().unwrap_or(path).to_string();
        let nested = self.nested.iter().map(|(child, parent)| (name(child), name(parent))).collect();
        crate::phiphx::blend::write::BlendEdits { moved: self.moved, parented: self.parented, linked: self.linked, nested, ..Default::default() }
    }
}
