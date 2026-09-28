//! [`BSceneTree`]: the scene's membership as it stands — the file's
//! collections with the session's renames, deletions, added objects (in
//! the scene collection, where `object.add` puts them), duplicates (beside
//! their source), moves and links applied, the collections nested as the
//! Outliner left them, and each object's parent. The Outliner draws it,
//! and a collection's toggles hide what it holds now.

use super::{BCollectionNode, BSceneLinks};
use crate::phiphx::scene::{SceneCollection, SceneModel};
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::state::BState;
use std::collections::HashMap;

/// The scene's collections and parents as they stand now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSceneTree {
    /// Collection path → `(name, Object.type)` of the objects directly in it.
    members: HashMap<String, Vec<(String, String)>>,
    parents: HashMap<String, String>,
    /// The collection tree, nestings applied.
    pub collections: Vec<BCollectionNode>,
}

impl BSceneTree {
    /// `(path, name, Object.type)` of everything the file puts in `c` and under it.
    fn walk(c: &SceneCollection, parent: &str, model: &SceneModel, out: &mut Vec<(String, String, String)>) {
        let path = if parent.is_empty() { c.name.clone() } else { format!("{parent}/{}", c.name) };
        for &i in &c.object_indices {
            if let Some(o) = model.objects.get(i) {
                out.push((path.clone(), o.name.clone(), model.object_types.get(i).cloned().unwrap_or_else(|| "MESH".into())));
            }
        }
        for n in &c.extra_names {
            let ty = if model.cameras.iter().any(|k| &k.name == n) {
                "CAMERA"
            } else if model.empties.iter().any(|k| &k.name == n) {
                "EMPTY"
            } else {
                "LIGHT"
            };
            out.push((path.clone(), n.clone(), ty.into()));
        }
        for k in &c.children {
            Self::walk(k, &path, model, out);
        }
    }

    pub fn now(state: &BState, model: &SceneModel) -> Self {
        let edit = BSceneEdit::now(state);
        let links = BSceneLinks::now(state);
        let now = |file_name: &str| edit.renamed.iter().find(|(_, old)| old == file_name).map_or(file_name.to_string(), |(new, _)| new.clone());
        let mut members: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut flat = Vec::new();
        for c in &model.collections {
            Self::walk(c, "", model, &mut flat);
        }
        for (path, name, ty) in flat.iter().filter(|(_, name, _)| !edit.deleted.contains(name)) {
            members.entry(path.clone()).or_default().push((now(name), ty.clone()));
        }
        let root = model.collections.first().map(|c| c.name.clone()).unwrap_or_default();
        for (copy, source) in &edit.copied {
            let home = members.iter().find(|(_, list)| list.iter().any(|(n, _)| n == source)).map(|(p, list)| (p.clone(), list.iter().find(|(n, _)| n == source).map(|(_, t)| t.clone())));
            let (path, ty) = home.map_or((root.clone(), None), |(p, t)| (p, t));
            members.entry(path).or_default().push((copy.clone(), ty.unwrap_or_else(|| "MESH".into())));
        }
        for (name, _) in &edit.added {
            members.entry(root.clone()).or_default().push((name.clone(), "MESH".into()));
        }
        let type_of = |members: &HashMap<String, Vec<(String, String)>>, object: &str| members.values().flatten().find(|(n, _)| n == object).map(|(_, t)| t.clone());
        for (object, path) in &links.moved {
            let Some(ty) = type_of(&members, object) else { continue };
            for list in members.values_mut() {
                list.retain(|(n, _)| n != object);
            }
            members.entry(path.clone()).or_default().push((object.clone(), ty));
        }
        // A link keeps the object where it was and puts it here as well.
        for (object, path) in &links.linked {
            let Some(ty) = type_of(&members, object) else { continue };
            let list = members.entry(path.clone()).or_default();
            if !list.iter().any(|(n, _)| n == object) {
                list.push((object.clone(), ty));
            }
        }
        let mut parents: HashMap<String, String> = model.parents.iter().map(|(c, p)| (now(c), now(p))).collect();
        parents.extend(links.parented.iter().cloned());
        Self { members, parents, collections: BCollectionNode::now(model, &links.nested) }
    }

    /// `(name, Object.type)` of the objects directly in the collection at `path`.
    pub fn members(&self, path: &str) -> &[(String, String)] {
        self.members.get(path).map_or(&[], Vec::as_slice)
    }

    /// The tree holding the collection at `path`.
    pub fn root_of(&self, path: &str) -> Option<&BCollectionNode> {
        self.collections.iter().find(|root| root.find(path).is_some())
    }

    pub fn parent_of(&self, name: &str) -> Option<&str> {
        self.parents.get(name).map(String::as_str)
    }

    /// Is `ancestor` above `name` in the parent chain (or `name` itself)?
    pub fn descends_from(&self, name: &str, ancestor: &str) -> bool {
        let mut at = Some(name);
        for _ in 0..256 {
            match at {
                Some(n) if n == ancestor => return true,
                Some(n) => at = self.parent_of(n),
                None => return false,
            }
        }
        false
    }
}
