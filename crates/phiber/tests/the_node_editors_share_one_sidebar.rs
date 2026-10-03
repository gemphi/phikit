//! The four node editors share one sidebar table.
//!
//! The Shader editor, the generic Node editor, Geometry Nodes and the
//! Compositor are one space type (`SpaceNodeEditor`) with a different tree in
//! it. Blender registers most of the N panel once for all of them and polls
//! on the tree type, so `sidebar::node` is one set of tables reached from
//! four tab sets — a table each is how they would drift apart.
//!
//! All eight of those tabs opened nothing before. What is still empty is the
//! Shader and Node editors' Options tabs and Geometry Nodes' Attributes tab,
//! each of which is a per-node-type `draw_buttons_ext` rather than a table;
//! they stay named in `an_open_sidebar_paints_its_panels`.

use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::node;
use phiber::v2::part::sidebar::panel::BSidebarPanels;
use phiber::v2::part::sidebar::set::{BSidebarBody, tabs_for};
use phiber::v2::state::{BPath, BState};
use phiber::v2::view::BView;

const EDITORS: [BHostType; 4] =
    [BHostType::Shader, BHostType::Node, BHostType::GeoNode, BHostType::Compositor];

/// The card titles a host's sidebar shows for one tab.
fn titles(host: BHostType, tab: &str) -> Vec<String> {
    let mut state = BState::new();
    state.set(&BPath::owned(vec![host.slug(), "sidebar", "tab"]), tab);
    BSidebarPanels::new(host)
        .views(&state)
        .iter()
        .filter_map(|view| match view {
            BView::Card(card) => card.title().map(str::to_string),
            _ => None,
        })
        .collect()
}

/// One host's tab by label.
fn body(host: BHostType, label: &str) -> BSidebarBody {
    tabs_for(host).iter().find(|tab| tab.label == label).unwrap_or_else(|| panic!("{host:?} {label}")).body
}

#[test]
fn every_node_editor_has_a_node_tab_and_they_all_point_at_one_table() {
    let first = body(EDITORS[0], "Node");
    assert_eq!(first, BSidebarBody::Props(node::NODE));
    for host in EDITORS {
        assert_eq!(body(host, "Node"), first, "{host:?} has a Node table of its own");
    }
}

#[test]
fn the_group_tab_is_shared_by_the_three_that_have_one() {
    // The Compositor edits the scene's tree, not a group, so it has no Group
    // tab — the other three share one.
    for host in [BHostType::Shader, BHostType::Node, BHostType::GeoNode] {
        assert_eq!(body(host, "Group"), BSidebarBody::Props(node::GROUP), "{host:?}");
    }
    assert!(
        !tabs_for(BHostType::Compositor).iter().any(|tab| tab.label == "Group"),
        "the Compositor has no Group tab"
    );
}

#[test]
fn only_the_compositor_has_the_quality_panel() {
    assert_eq!(body(BHostType::Compositor, "Options"), BSidebarBody::Props(node::OPTIONS));
    // `NODE_PT_quality` polls on the compositor tree; the other editors'
    // Options tabs are a different thing and are still empty.
    for host in [BHostType::Shader, BHostType::Node] {
        assert_eq!(body(host, "Options"), BSidebarBody::None, "{host:?}");
    }
}

#[test]
fn the_node_tab_builds_the_active_node_with_its_colour_under_it() {
    for host in EDITORS {
        // One card: Color is a sub-panel of Node, not a sibling of it.
        assert_eq!(titles(host, "Node"), vec!["Node"], "{host:?} Node tab");
    }
    let color = &node::NODE[0].list[0];
    assert_eq!(color.title, "Color");
    assert!(color.closed, "Blender closes it by default");
}

#[test]
fn the_group_tab_builds_the_trees_own_card() {
    for host in [BHostType::Shader, BHostType::Node, BHostType::GeoNode] {
        assert_eq!(titles(host, "Group"), vec!["Group"], "{host:?} Group tab");
    }
}

#[test]
fn the_compositors_options_tab_builds_its_performance_card() {
    assert_eq!(titles(BHostType::Compositor, "Options"), vec!["Performance"]);
}

#[test]
fn geometry_nodes_keeps_blenders_tab_order() {
    // Group first, then Attributes, then Node — not the Shader editor's
    // order. The tables are shared; the tab sets are each host's own.
    assert_eq!(
        tabs_for(BHostType::GeoNode).iter().map(|tab| tab.label).collect::<Vec<_>>(),
        vec!["Group", "Attributes", "Node"]
    );
}
