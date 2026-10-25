//! The Add menu's rows: the searched catalog, one control per type, each one an
//! action that adds that type at the menu's location.
//!
//! Ported from Blender's `node_add_menu` (`node_add_menu.cc`): the menu is the
//! node types grouped and filtered, and choosing a row calls `nodeAddNode`.
//! Here a row is a [`BControl`] — the same row the toolbar and sidebar menus are
//! built from — and choosing it writes `NodeKey::Added`, which the overlay
//! merges into the tree next frame.
//!
//! This is the menu's DATA, not its container: the popover that holds the search
//! field and lays these rows out, and the Shift-A and link-drag triggers that
//! open it at a point, are the remaining UI. Keeping the rows here, pure and off
//! the element, is what lets the search, the naming and the add action be
//! checked without a window.

use super::catalog;
use super::{BNodeAdditions, BPendingLink, place_linked, store};
use crate::v2::host::BHostType;
use crate::v2::state::val::{V_EMPTY, V_TRUE};
use crate::v2::state::{BKey as _, BState, MenuKey, NodeKey};
use crate::v2::view::BView;
use crate::v2::view::draw::canvas::node::socket::BSocketSide;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// One menu row: choosing it adds `ty` at `at`, named uniquely against `taken`,
/// over `current`, carrying `link` so a link-drag pick creates AND wires.
fn row(ty: &dyn catalog::BShaderNode, at: (f32, f32), current: &BNodeAdditions, taken: &[&str], host: BHostType, link: Option<&BPendingLink>) -> BControl {
    let value = store::encode(&place_linked(current, taken, ty, at, link.cloned()).added);
    BControl::new().with_label(ty.label()).with_action(BAction::Set(NodeKey::Added(host).path(), value))
}

/// One row per type the menu offers, in the menu's order. Choosing a row adds
/// that type at `at` (view units), named uniquely against `taken` — the graph's
/// existing node names — over the additions already in `current`.
///
/// `link` is the pending drag when the menu was opened by dropping a link on
/// empty canvas: it narrows the types to those that can take the dragged end
/// (`search_offering`) and rides on the addition so the pick creates AND wires
/// the node in one go. `None` is the plain Add menu — every matching type, no
/// link.
///
/// `BAction::Set`, not `Adjust`: adding a node is a decision, and `Set` also
/// dismisses the open menu, which is exactly what choosing a row should do.
pub fn controls(
    query: &str,
    at: (f32, f32),
    current: &BNodeAdditions,
    taken: &[&str],
    host: BHostType,
    link: Option<&BPendingLink>,
) -> Vec<BControl> {
    let types = match link {
        // A dragged output needs a node with an input to receive it, and a
        // dragged input a node with an output.
        Some(p) => catalog::search_offering(query, if p.from_output { BSocketSide::Input } else { BSocketSide::Output }),
        None => catalog::search(query),
    };
    types.into_iter().map(|ty| row(ty, at, current, taken, host, link)).collect()
}

/// The plain Add menu, grouped into one submenu per category (`by_category`) —
/// Blender's Shift-A menu shape. Each submenu is a popover to the right, opened
/// on hover, keyed by `MenuKey::Nested` so sibling categories close each other.
fn grouped(s: &BState, at: (f32, f32), current: &BNodeAdditions, taken: &[&str], host: BHostType, link: Option<&BPendingLink>) -> Vec<BView> {
    // A link-drag menu offers only the types that can take the dragged end.
    let offered: Option<Vec<&str>> = link.map(|p| {
        catalog::search_offering("", if p.from_output { BSocketSide::Input } else { BSocketSide::Output })
            .into_iter()
            .map(|ty| ty.label())
            .collect()
    });
    catalog::by_category()
        .into_iter()
        .map(|(cat, types)| {
            let types: Vec<_> = types.into_iter().filter(|ty| offered.as_ref().is_none_or(|o| o.contains(&ty.label()))).collect();
            (cat, types)
        })
        .filter(|(_, types)| !types.is_empty())
        .map(|(cat, types)| {
            let labels = ["Add", cat.label()];
            let key = MenuKey::Nested(host.slug(), &labels).path();
            let open = s.get(&key) == Some(V_TRUE);
            let anchor = BView::MenuItem(
                BControl::new()
                    .with_label(cat.label())
                    .with_submenu(true)
                    .with_active(open)
                    .with_action(BAction::Toggle(key.clone()))
                    .with_hover(BAction::Open(key.clone())),
            );
            let body = types.into_iter().map(|ty| BView::MenuItem(row(ty, at, current, taken, host, link))).collect();
            BView::popover(anchor, body, open, BAction::Set(key, V_EMPTY.to_string()), BPopoverSide::Right)
        })
        .collect()
}

/// The two writes that open the Add menu as a link-drag search: record the
/// dragged end as the pending link, then open the panel. The release handler
/// dispatches both when a fresh link lands on empty canvas (`wire::up`).
///
/// Returned rather than dispatched so the pair can be checked without a window,
/// and so the handler stays a short loop over actions.
pub fn open_actions(host: BHostType, node: &str, socket: &str, from_output: bool) -> [BAction; 2] {
    let pend = BPendingLink { node: node.into(), socket: socket.into(), from_output };
    [
        BAction::Set(NodeKey::AddLink(host).path(), store::encode_pending(&pend)),
        BAction::Set(crate::v2::state::MenuKey::Panel(host.slug(), "Add").path(), crate::v2::state::val::V_TRUE.to_string()),
    ]
}

/// Where a new node is dropped when the Add menu is a HEADER menu rather than a
/// cursor popover: the centre of what the editor is looking at, so it lands on
/// screen. Blender places it at the cursor (Shift-A), but phiber's Add menu
/// opens from the header like its other menus, so there is no cursor to use —
/// the view centre is the honest stand-in.
fn view_centre(s: &BState, host: BHostType) -> (f32, f32) {
    let view = s
        .get(&NodeKey::View(host).path())
        .and_then(super::super::view::store::decode)
        .unwrap_or_default();
    ((view.xmin + view.xmax) / 2.0, (view.ymin + view.ymax) / 2.0)
}

/// The shader Add menu's body — the header menu's rows, built from the catalog
/// over the current graph. The panel menu (`menubar`) calls this; each row adds
/// its type at the view centre, named uniquely against the graph.
///
/// Scoped to the shader editor: it is the node tree phiber loads, and the menu's
/// rows write `NodeKey::Added(Shader)`. The compositor and geometry editors
/// share the node menubar but not a loaded tree, so they keep the placeholder
/// until phiber loads one.
pub fn body(s: &BState) -> Vec<BView> {
    let host = BHostType::Shader;
    let current = super::additions_of(s, host);
    // Every name already in the graph: the file's nodes and the added ones.
    let mut taken: Vec<String> = super::super::BNode::graph_of(s)
        .map(|g| g.nodes.iter().map(|n| n.name.clone()).collect())
        .unwrap_or_default();
    taken.extend(current.added.iter().map(|a| a.name.clone()));
    let refs: Vec<&str> = taken.iter().map(String::as_str).collect();
    let at = view_centre(s, host);
    // A pending link (a link dropped on empty canvas) makes this a link-drag
    // menu narrowed to compatible types, each pick wired to the drop. A short
    // list stays flat; a long one is grouped into the same category submenus
    // as the plain Shift-A menu, so no menu lists more than a handful of rows
    // and none runs off the window (`MAX_CHILDREN`).
    match super::pending_of(s, host) {
        Some(pend) => {
            let flat = controls("", at, &current, &refs, host, Some(&pend));
            if flat.len() <= crate::v2::part::props::pref::set::MAX_CHILDREN {
                flat.into_iter().map(BView::MenuItem).collect()
            } else {
                grouped(s, at, &current, &refs, host, Some(&pend))
            }
        }
        None => grouped(s, at, &current, &refs, host, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_per_match_in_menu_order() {
        let rows = controls("mix", (0.0, 0.0), &BNodeAdditions::default(), &[], BHostType::Shader, None);
        let labels: Vec<_> = rows.iter().filter_map(|c| c.label.clone()).collect();
        assert_eq!(labels, ["Mix Shader", "Mix (Legacy)"], "both matches, catalog order");
        // Empty query is the whole menu.
        assert_eq!(controls("", (0.0, 0.0), &BNodeAdditions::default(), &[], BHostType::Shader, None).len(), catalog::CATALOG.len());
    }

    #[test]
    fn choosing_a_row_writes_an_addition_for_that_type() {
        // The action a row carries, decoded, is the current additions plus one
        // node of the chosen type at the given point.
        let rows = controls("emission", (3.0, 5.0), &BNodeAdditions::default(), &[], BHostType::Shader, None);
        let BAction::Set(path, value) = rows[0].action.clone().unwrap() else { panic!("a row sets Added") };
        assert_eq!(path, NodeKey::Added(BHostType::Shader).path(), "it writes this host's Added key");
        let added = store::decode(&value);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].idname, "ShaderNodeEmission");
        assert_eq!(added[0].at, (3.0, 5.0));
    }

    #[test]
    fn the_new_node_is_named_uniquely_against_the_graph() {
        // An "Emission" already in the graph makes the added one "Emission.001".
        let rows = controls("emission", (0.0, 0.0), &BNodeAdditions::default(), &["Emission"], BHostType::Shader, None);
        let BAction::Set(_, value) = rows[0].action.clone().unwrap() else { panic!() };
        assert_eq!(store::decode(&value)[0].name, "Emission.001");
    }

    #[test]
    fn the_plain_header_body_is_grouped_into_category_submenus() {
        // With no pending link the Shift-A menu is grouped: one submenu popover
        // per category, not a flat list (the link-drag search is the flat one).
        let rows = body(&BState::new());
        assert_eq!(rows.len(), catalog::by_category().len(), "one submenu per category");
        assert!(rows.iter().all(|r| matches!(r, BView::Popover { .. })), "each category is a submenu");
    }

    #[test]
    fn a_row_adds_on_top_of_the_additions_already_made() {
        // Picking a second type keeps the first — the row carries the whole list.
        let first = place_linked(&BNodeAdditions::default(), &[], catalog::find("ShaderNodeValue").unwrap(), (0.0, 0.0), None);
        let rows = controls("gamma", (1.0, 1.0), &first, &["Value"], BHostType::Shader, None);
        let BAction::Set(_, value) = rows[0].action.clone().unwrap() else { panic!() };
        let added = store::decode(&value);
        assert_eq!(added.len(), 2, "Value kept, Gamma added");
        assert_eq!(added[1].idname, "ShaderNodeGamma");
    }

    #[test]
    fn a_link_drag_menu_offers_only_compatible_types_and_wires_the_pick() {
        // Opened by dropping a link from an output: the rows are only types with
        // an input, and the chosen row's addition carries the induced link.
        let pend = BPendingLink { node: "Principled BSDF".into(), socket: "BSDF".into(), from_output: true };
        let rows = controls("material", (2.0, 2.0), &BNodeAdditions::default(), &[], BHostType::Shader, Some(&pend));
        let labels: Vec<_> = rows.iter().filter_map(|c| c.label.clone()).collect();
        assert_eq!(labels, ["Material Output"], "only a matching type with an input");
        let BAction::Set(_, value) = rows[0].action.clone().unwrap() else { panic!() };
        let added = store::decode(&value);
        assert_eq!(added[0].link.as_ref().unwrap().node, "Principled BSDF", "the pick carries the wire");
        // A pure source (RGB) is never offered to a dragged output — it has no input.
        let all: Vec<_> = controls("", (0.0, 0.0), &BNodeAdditions::default(), &[], BHostType::Shader, Some(&pend))
            .iter().filter_map(|c| c.label.clone()).collect();
        assert!(!all.contains(&"Color".to_string()), "RGB (no input) is not offered");
    }
}
