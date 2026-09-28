//! A collection's object rows. The Outliner is a table: what reads as a tree
//! is each row's `depth`. With Object Children on (`use_filter_children`) a
//! child's row comes straight after its parent's — after the parent's data
//! row — one level deeper, and the child is not listed again at its own
//! collection's level; the parent's disclosure arrow folds both away. A
//! search lists the matches flat, as the rows are then all at one level.
//!
//! Blender ref: `tree_display_view_layer.cc` (`ObjectsChildrenBuilder`).

use super::layer::Ctx;
use super::tree::{BOutlineRow, BRowKind, intern_name, is_expanded_or, summarise};
use super::type_set::BOutlineIcon;

/// Deeper than any real hierarchy; stops a malformed parent loop.
const MAX_DEPTH: u16 = 64;

/// The rows of `objects` (name, icon) at `depth`: the ones without a listed
/// parent, each followed by its children.
pub(super) fn push(rows: &mut Vec<BOutlineRow>, objects: Vec<(String, &'static str)>, depth: u16, cx: &Ctx) {
    let nest = cx.filter.children && !cx.filtering;
    let listed = |name: &str| objects.iter().any(|(n, _)| n == name);
    for (name, icon) in &objects {
        let parent_listed = nest && cx.tree.parent_of(name).is_some_and(listed);
        if !parent_listed {
            push_object(rows, name, icon, depth, &objects, nest, cx);
        }
    }
}

fn push_object(rows: &mut Vec<BOutlineRow>, name: &str, icon: &'static str, depth: u16, all: &[(String, &'static str)], nest: bool, cx: &Ctx) {
    let hit = cx.filter.hits(name);
    if cx.filtering && !hit {
        return;
    }
    let kids: Vec<&(String, &'static str)> = if nest && depth < MAX_DEPTH {
        all.iter().filter(|(n, _)| cx.tree.parent_of(n) == Some(name)).collect()
    } else {
        Vec::new()
    };
    let oid = intern_name(name);
    let open = !cx.filtering && is_expanded_or(cx.s, oid, false);
    let has_data = cx.filter.contents && icon == BOutlineIcon::for_object_type("MESH");
    let folds = has_data || !kids.is_empty();
    let summary = if folds && !open {
        summarise(has_data.then_some("mesh").into_iter().chain(kids.iter().map(|(_, icon)| *icon)))
    } else {
        Vec::new()
    };
    rows.push(BOutlineRow {
        id: oid,
        kind: BRowKind::Object,
        depth,
        icon,
        label: name.to_string(),
        expanded: folds.then_some(open),
        object: Some(name.to_string()),
        matched: cx.filtering && hit,
        summary,
    });
    if !open {
        return;
    }
    if has_data {
        rows.push(BOutlineRow {
            id: intern_name(&format!("{name}/data")),
            kind: BRowKind::Data,
            depth: depth + 1,
            icon: "mesh",
            label: name.to_string(),
            expanded: None,
            object: None,
            matched: false,
            summary: Vec::new(),
        });
    }
    for (kid, kid_icon) in kids {
        push_object(rows, kid, kid_icon, depth + 1, all, nest, cx);
    }
}
