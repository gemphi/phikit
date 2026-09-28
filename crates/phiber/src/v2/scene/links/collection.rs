//! The scene's collection tree as it stands: the file's, with the
//! Outliner's nestings applied (`outliner_collection_drop` — a collection
//! dropped on another becomes its last child, `BKE_collection_move`). A
//! node keeps the path the file has for it: that is the id its row, its
//! toggles and the objects moved into it key on, so nesting a collection
//! carries its subtree along and leaves everything that names it pointing
//! at it still.

use crate::phiphx::scene::{SceneCollection, SceneModel};

/// One collection and the ones under it now.
#[derive(Clone, Debug, PartialEq)]
pub struct BCollectionNode {
    pub name: String,
    /// Its path in the file (`Scene Collection/Props`) — its id.
    pub path: String,
    pub children: Vec<BCollectionNode>,
}

impl BCollectionNode {
    /// The file's tree, one root per scene collection.
    pub fn file(model: &SceneModel) -> Vec<Self> {
        model.collections.iter().map(|c| Self::of(c, "")).collect()
    }

    /// The tree now: the file's with `nested` replayed in order. A step
    /// that no longer fits — a collection into itself, or a scene
    /// collection moved — is passed over.
    pub fn now(model: &SceneModel, nested: &[(String, String)]) -> Vec<Self> {
        let mut roots = Self::file(model);
        for (child, parent) in nested {
            for root in &mut roots {
                root.nest(child, parent);
            }
        }
        roots
    }

    fn of(c: &SceneCollection, parent: &str) -> Self {
        let path = if parent.is_empty() { c.name.clone() } else { format!("{parent}/{}", c.name) };
        let children = c.children.iter().map(|k| Self::of(k, &path)).collect();
        Self { name: c.name.clone(), path, children }
    }

    /// Can the collection at `child` go into the one at `parent`? Not a
    /// scene collection, not into itself or anything under it.
    pub fn fits(&self, child: &str, parent: &str) -> bool {
        self.path != child && self.find(parent).is_some() && self.find(child).is_some_and(|c| c.find(parent).is_none())
    }

    /// Put the collection at `child` into the one at `parent`, last.
    pub fn nest(&mut self, child: &str, parent: &str) -> bool {
        if !self.fits(child, parent) {
            return false;
        }
        let Some(node) = self.take(child) else { return false };
        match self.find_mut(parent) {
            Some(at) => {
                at.children.push(node);
                true
            }
            None => false,
        }
    }

    /// The node at `path`, this one or under it.
    pub fn find(&self, path: &str) -> Option<&Self> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|k| k.find(path))
    }

    fn find_mut(&mut self, path: &str) -> Option<&mut Self> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter_mut().find_map(|k| k.find_mut(path))
    }

    fn take(&mut self, path: &str) -> Option<Self> {
        match self.children.iter().position(|k| k.path == path) {
            Some(i) => Some(self.children.remove(i)),
            None => self.children.iter_mut().find_map(|k| k.take(path)),
        }
    }

    /// The path of the collection holding the one at `path`.
    pub fn parent_of(&self, path: &str) -> Option<&str> {
        match self.children.iter().any(|k| k.path == path) {
            true => Some(&self.path),
            false => self.children.iter().find_map(|k| k.parent_of(path)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(path: &str, children: Vec<BCollectionNode>) -> BCollectionNode {
        BCollectionNode { name: path.rsplit('/').next().unwrap().into(), path: path.into(), children }
    }

    #[test]
    fn a_nested_collection_takes_its_subtree_and_keeps_its_file_path() {
        let (props, lamps, rim) = ("Scene Collection/Props", "Scene Collection/Lamps", "Scene Collection/Lamps/Rim");
        let mut root = node("Scene Collection", vec![node(props, vec![]), node(lamps, vec![node(rim, vec![])])]);
        assert!(root.nest(lamps, props));
        assert_eq!(root.parent_of(lamps), Some(props));
        assert_eq!(root.parent_of(rim), Some(lamps), "the subtree comes along");
        assert!(root.find(props).unwrap().find(rim).is_some());
        // Into itself, under itself, or the scene collection anywhere: refused.
        assert!(!root.nest(props, rim) && !root.nest(lamps, lamps) && !root.nest("Scene Collection", props));
        assert!(root.nest(lamps, "Scene Collection"), "back to the top");
        assert_eq!(root.children.iter().map(|k| k.name.as_str()).collect::<Vec<_>>(), ["Props", "Lamps"]);
    }
}
