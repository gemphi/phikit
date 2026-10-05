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
use super::{BNodeAdditions, place, store};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

/// One row per type whose label matches `query`, in the menu's order. Choosing a
/// row adds that type at `at` (view units), named uniquely against `taken` —
/// the graph's existing node names — over the additions already in `current`.
///
/// `BAction::Set`, not `Adjust`: adding a node is a decision, and `Set` also
/// dismisses the open menu, which is exactly what choosing a row should do.
pub fn controls(
    query: &str,
    at: (f32, f32),
    current: &BNodeAdditions,
    taken: &[&str],
    host: BHostType,
) -> Vec<BControl> {
    catalog::search(query)
        .into_iter()
        .map(|ty| {
            let value = store::encode(&place(current, taken, ty, at).added);
            BControl::new()
                .with_label(ty.label())
                .with_action(BAction::Set(NodeKey::Added(host).path(), value))
        })
        .collect()
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
    controls("", view_centre(s, host), &current, &refs, host).into_iter().map(BView::MenuItem).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_per_match_in_menu_order() {
        let rows = controls("mix", (0.0, 0.0), &BNodeAdditions::default(), &[], BHostType::Shader);
        let labels: Vec<_> = rows.iter().filter_map(|c| c.label.clone()).collect();
        assert_eq!(labels, ["Mix Shader", "Mix (Legacy)"], "both matches, catalog order");
        // Empty query is the whole menu.
        assert_eq!(controls("", (0.0, 0.0), &BNodeAdditions::default(), &[], BHostType::Shader).len(), catalog::CATALOG.len());
    }

    #[test]
    fn choosing_a_row_writes_an_addition_for_that_type() {
        // The action a row carries, decoded, is the current additions plus one
        // node of the chosen type at the given point.
        let rows = controls("emission", (3.0, 5.0), &BNodeAdditions::default(), &[], BHostType::Shader);
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
        let rows = controls("emission", (0.0, 0.0), &BNodeAdditions::default(), &["Emission"], BHostType::Shader);
        let BAction::Set(_, value) = rows[0].action.clone().unwrap() else { panic!() };
        assert_eq!(store::decode(&value)[0].name, "Emission.001");
    }

    #[test]
    fn the_header_body_is_the_whole_catalog_with_no_file_loaded() {
        // With nothing loaded there are no names taken and no view, so the body
        // is every type as a menu row — the menu a fresh editor opens.
        let rows = body(&BState::new());
        assert_eq!(rows.len(), catalog::CATALOG.len());
        assert!(matches!(rows[0], BView::MenuItem(_)), "rows are menu items");
    }

    #[test]
    fn a_row_adds_on_top_of_the_additions_already_made() {
        // Picking a second type keeps the first — the row carries the whole list.
        let first = place(&BNodeAdditions::default(), &[], catalog::find("ShaderNodeValue").unwrap(), (0.0, 0.0));
        let rows = controls("gamma", (1.0, 1.0), &first, &["Value"], BHostType::Shader);
        let BAction::Set(_, value) = rows[0].action.clone().unwrap() else { panic!() };
        let added = store::decode(&value);
        assert_eq!(added.len(), 2, "Value kept, Gamma added");
        assert_eq!(added[1].idname, "ShaderNodeGamma");
    }
}
