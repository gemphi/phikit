//! View Layer rows for a **loaded** `.blend` — the file's real collection
//! tree, as Blender's Outliner draws it (`tree_display_view_layer.cc`):
//!
//! ```text
//! Scene Collection
//!   ├ <child collections, recursively>   (collections before objects)
//!   └ <objects in this collection>        (sorted when SO_SKIP_SORT_ALPHA is off)
//!        └ <object data>                  (mesh rows, collapsed by default)
//! ```
//!
//! Search keeps a collection when any descendant matches and force-opens it,
//! matching `outliner_filter_tree` + `TSE_CHILDSEARCH`.

use super::filter::BOutlineFilter;
use super::type_set::BOutlineIcon;
use super::tree::BOutlineRow;
use super::tree::BRowKind;
use super::tree::intern_name;
use super::tree::summarise;
use super::tree::is_expanded;
use super::tree::sort_alpha;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::links::{BCollectionNode, BSceneTree};
use crate::v2::state::BState;

/// What a flatten reads: the state, the scene's membership as it stands
/// now (`scene::links::BSceneTree` — the file with the session's edits),
/// the filter.
pub(super) struct Ctx<'a> {
    pub s: &'a BState,
    pub tree: BSceneTree,
    pub filter: &'a BOutlineFilter,
    pub filtering: bool,
}

/// Every object name (mesh + extras) under `c`, recursively.
fn any_hit(c: &BCollectionNode, cx: &Ctx) -> bool {
    names(&c.path, cx).iter().any(|(n, _)| cx.filter.hits(n)) || c.children.iter().any(|k| any_hit(k, cx))
}

/// `(name, icon)` of the objects directly in the collection at `path` that
/// the Filter popover lets through (`BOutlineFilter::shows`: Objects, the
/// type boxes, the object state).
fn names(path: &str, cx: &Ctx) -> Vec<(String, &'static str)> {
    let mut out: Vec<(String, &'static str)> = cx
        .tree
        .members(path)
        .iter()
        .filter(|(name, ty)| cx.filter.shows(cx.s, name, ty))
        .map(|(name, ty)| (name.clone(), BOutlineIcon::for_object_type(ty)))
        .collect();
    if sort_alpha(cx.s) {
        out.sort_by_key(|(n, _)| n.to_lowercase());
    }
    out
}

/// The state id of a collection at `path` (names joined by `/` from the scene
/// collection down). The row's expansion, and its eye / camera toggles, key
/// on this — the same id `seed_view_layer` writes the file's flags to.
pub fn collection_id(path: &str) -> &'static str {
    intern_name(&format!("col/{path}"))
}

/// The kinds under a closed collection, all the way down: its child
/// collections first, then every object by type (`outliner_draw_iconrow`
/// recurses into a closed element's subtree).
fn kinds_under(c: &BCollectionNode, cx: &Ctx, out: &mut Vec<&'static str>) {
    out.extend(c.children.iter().map(|_| "collection"));
    for (_, icon) in names(&c.path, cx) {
        out.push(icon);
    }
    for k in &c.children {
        kinds_under(k, cx, out);
    }
}

fn push_collection(rows: &mut Vec<BOutlineRow>, c: &BCollectionNode, depth: u16, cx: &Ctx) {
    if cx.filtering && !any_hit(c, cx) {
        return;
    }
    let id = collection_id(&c.path);
    let open = cx.filtering || is_expanded(cx.s, id);
    let summary = if open {
        Vec::new()
    } else {
        let mut kinds = Vec::new();
        kinds_under(c, cx, &mut kinds);
        summarise(kinds)
    };
    rows.push(BOutlineRow {
        id,
        kind: if depth == 0 { BRowKind::SceneCollection } else { BRowKind::Collection },
        depth,
        icon: if depth == 0 { "scene" } else { "collection" },
        label: c.name.clone(),
        expanded: Some(open),
        // A collection row carries the eye and the camera too
        // (`outliner_draw_restrictbuts`, `TSE_LAYER_COLLECTION`): hiding it
        // hides every level under it. Toggles used to stop at the leaves.
        object: Some(id.to_string()),
        matched: false,
        summary,
    });
    if !open {
        return;
    }
    if !cx.filter.collections {
        // Collections off: every object in the tree, straight under the
        // Scene Collection (`use_filter_collection`).
        let mut all = Vec::new();
        every_object(c, cx, &mut all);
        if sort_alpha(cx.s) {
            all.sort_by_key(|(n, _)| n.to_lowercase());
        }
        super::object_rows::push(rows, all, depth + 1, cx);
        return;
    }
    for k in &c.children {
        push_collection(rows, k, depth + 1, cx);
    }
    super::object_rows::push(rows, names(&c.path, cx), depth + 1, cx);
}

fn every_object(c: &BCollectionNode, cx: &Ctx, out: &mut Vec<(String, &'static str)>) {
    for object in names(&c.path, cx) {
        if !out.iter().any(|(name, _)| *name == object.0) {
            out.push(object);
        }
    }
    for k in &c.children {
        every_object(k, cx, out);
    }
}

/// Rows for the loaded scene, or `None` when the model carries no tree
/// (e.g. a `.phx` scene) so the caller falls back to the flat list.
pub fn rows(s: &BState, model: &SceneModel, filter: &BOutlineFilter) -> Option<Vec<BOutlineRow>> {
    if model.collections.is_empty() {
        return None;
    }
    let cx = Ctx { s, tree: BSceneTree::now(s, model), filter, filtering: filter.searching() };
    let mut out = Vec::new();
    for c in &cx.tree.collections {
        push_collection(&mut out, c, 0, &cx);
    }
    Some(out)
}

/// Every object a collection above it hides — its own toggle, or an
/// ancestor's — in the loaded model, where `hidden` reads the toggle of a
/// collection row. `LAYER_COLLECTION_HIDE`, `COLLECTION_HIDE_RENDER` and
/// `COLLECTION_HIDE_SELECT` all inherit down the tree (as it stands, the
/// Outliner's nestings in), and an excluded collection
/// (`LAYER_COLLECTION_EXCLUDE`, the checkbox) is out of every pass. An
/// object in two collections is hidden only when both hide it: its base
/// takes the flags of every collection that holds it
/// (`BKE_layer_collection_sync`). This is the set a pass subtracts before
/// it draws, renders or picks.
pub fn hidden_by_collections(s: &BState, model: &SceneModel, hidden: fn(&BState, &str) -> bool) -> std::collections::HashSet<String> {
    type Names = std::collections::HashSet<String>;
    fn walk(c: &BCollectionNode, inherited: bool, s: &BState, tree: &BSceneTree, hidden: fn(&BState, &str) -> bool, out: &mut (Names, Names)) {
        let id = collection_id(&c.path);
        let hides = inherited || hidden(s, id) || super::key::is_excluded(s, id);
        let set = if hides { &mut out.0 } else { &mut out.1 };
        set.extend(tree.members(&c.path).iter().map(|(name, _)| name.clone()));
        for k in &c.children {
            walk(k, hides, s, tree, hidden, out);
        }
    }
    // What each collection holds now, the session's moves and additions in.
    let tree = BSceneTree::now(s, model);
    let mut out = (Names::new(), Names::new());
    for c in &tree.collections {
        walk(c, false, s, &tree, hidden, &mut out);
    }
    let (hid, shown) = out;
    hid.difference(&shown).cloned().collect()
}
