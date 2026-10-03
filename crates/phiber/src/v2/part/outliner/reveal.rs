//! A picked object is shown in the Outliner: the rows above it open, so its
//! row is listed, and the list scrolls it to the top unless the pick was
//! made on the list itself (`view::draw::tree::BRowWindow`).
//!
//! Blender ref: `outliner_show_active` (`outliner_open_back` opens the
//! element's parents, then the view scrolls to it).

use super::filter::BOutlineFilter;
use super::layer::collection_id;
use super::tree::intern_name;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::links::{BCollectionNode, BSceneTree};
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OutlinerKey, rule::BRule, val::V_TRUE};

/// Deeper than any real hierarchy; stops a malformed parent loop.
const MAX_DEPTH: usize = 64;

/// Picking an object (`ctx.object`) opens the rows above it and asks the
/// Outliner to show it (`ctx.outliner.reveal`, a count of the picks).
pub struct BOutlinerRevealRule;

impl BRule for BOutlinerRevealRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Object.path()
    }
    fn after(&self, _: &BPath, name: &str, state: &mut BState) {
        if name.is_empty() {
            return;
        }
        let above = crate::v2::scene::source::with_loaded(|model| model.map(|m| BOutlinerReveal::above(state, m, name)));
        for id in above.unwrap_or_default() {
            state.set(OutlinerKey::Expanded(id), V_TRUE);
        }
        let picks = state.get(OutlinerKey::Reveal).and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
        state.set(OutlinerKey::Reveal, (picks + 1).to_string());
    }
}

/// The rows above an object in the View Layer tree.
pub struct BOutlinerReveal;

impl BOutlinerReveal {
    /// The ids of the rows to open so `name` is listed: the collections
    /// from the scene collection down to the first one holding it, then
    /// the parents its row nests under — those listed at the same level.
    pub fn above(state: &BState, model: &SceneModel, name: &str) -> Vec<&'static str> {
        let tree = BSceneTree::now(state, model);
        let mut chain = Vec::new();
        if !tree.collections.iter().any(|root| Self::path_to(root, &tree, name, &mut chain)) {
            return Vec::new();
        }
        let filter = BOutlineFilter::now(state);
        let level: Vec<&str> = match (filter.collections, chain.last()) {
            (true, Some(home)) => tree.members(home).iter().map(|(n, _)| n.as_str()).collect(),
            // Collections off: every object is at one level.
            _ => chain.iter().take(1).flat_map(|root| Self::every(&tree, root)).collect(),
        };
        let mut ids: Vec<&'static str> = chain.iter().map(|path| collection_id(path)).collect();
        let mut at = tree.parent_of(name);
        for _ in 0..MAX_DEPTH {
            match at.filter(|parent| filter.children && level.contains(parent)) {
                Some(parent) => {
                    ids.push(intern_name(parent));
                    at = tree.parent_of(parent);
                }
                None => break,
            }
        }
        ids
    }

    /// Push the collection paths from `c` down to the one holding `name`.
    fn path_to(c: &BCollectionNode, tree: &BSceneTree, name: &str, chain: &mut Vec<String>) -> bool {
        chain.push(c.path.clone());
        if tree.members(&c.path).iter().any(|(n, _)| n == name) || c.children.iter().any(|k| Self::path_to(k, tree, name, chain)) {
            return true;
        }
        chain.pop();
        false
    }

    /// Every object under the collection at `path`.
    fn every<'a>(tree: &'a BSceneTree, path: &str) -> Vec<&'a str> {
        let Some(root) = tree.root_of(path).and_then(|root| root.find(path)) else { return Vec::new() };
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(c) = stack.pop() {
            out.extend(tree.members(&c.path).iter().map(|(n, _)| n.as_str()));
            stack.extend(c.children.iter());
        }
        out
    }
}
