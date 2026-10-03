//! The Image and UV editors' sidebar opens Blender's panels.
//!
//! Their three tabs — Image, Tool, View — were on the strip and opened
//! nothing: `BSidebarBody::None` on all three, which is a row of buttons
//! that do nothing at all. `sidebar::image` is the table
//! (`bl_ui/space_image.py`), grouped by `bl_category` the way Blender
//! groups the N panel.
//!
//! The two editors are one space in two modes (`SpaceImageEditor`), so they
//! share one tab set and one table. A test for that, because giving them a
//! table each is how they would drift.

use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::image;
use phiber::v2::part::sidebar::panel::BSidebarPanels;
use phiber::v2::part::sidebar::set::{BSidebarBody, tabs_for};
use phiber::v2::state::{BPath, BState};
use phiber::v2::view::BView;

/// The card titles the sidebar shows for `host`'s tab `tab`.
fn titles(host: BHostType, tab: &str) -> Vec<String> {
    let mut state = BState::new();
    // The tab a host's sidebar is showing, as `BHostState` spells it:
    // `ctx.<host>.sidebar.tab`.
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

#[test]
fn both_editors_share_one_tab_set() {
    // One space, two modes: a table each is how they drift apart.
    assert_eq!(tabs_for(BHostType::Image), tabs_for(BHostType::Uv));
    assert_eq!(
        tabs_for(BHostType::Image).iter().map(|t| t.label).collect::<Vec<_>>(),
        vec!["Image", "Tool", "View"],
        "Blender's `bl_category` names"
    );
}

#[test]
fn no_tab_opens_nothing_any_more() {
    for tab in tabs_for(BHostType::Image) {
        assert_ne!(tab.body, BSidebarBody::None, "the {} tab opens nothing", tab.label);
    }
}

#[test]
fn the_image_tab_is_the_datablock() {
    // `IMAGE_PT_image_properties` draws `template_image`, which is a
    // shorthand for a fixed set of properties; this tree declares its rows.
    assert_eq!(image::IMAGE.len(), 1);
    let keys: Vec<_> = image::IMAGE[0].props.iter().map(|p| p.key).collect();
    assert_eq!(
        keys,
        vec![
            "image.source",
            "image.filepath",
            "image.colorspace_settings.name",
            "image.alpha_mode",
            "image.use_view_as_render",
        ]
    );
}

#[test]
fn the_view_tab_is_blenders_three_panels_in_blenders_order() {
    let titles: Vec<_> = image::VIEW.iter().map(|s| s.title).collect();
    assert_eq!(titles, vec!["Display", "2D Cursor", "UDIM Grid"]);
}

#[test]
fn the_sidebar_builds_those_cards_for_both_editors() {
    for host in [BHostType::Image, BHostType::Uv] {
        assert_eq!(titles(host, "Image"), vec!["Image"], "{host:?} Image tab");
        assert_eq!(
            titles(host, "View"),
            vec!["Display", "2D Cursor", "UDIM Grid"],
            "{host:?} View tab"
        );
        assert_eq!(titles(host, "Tool"), vec!["Active Tool"], "{host:?} Tool tab");
    }
}

#[test]
fn the_tool_tab_is_the_viewports_own_active_tool() {
    // Blender's `IMAGE_PT_tools_active` is the same panel the 3D viewport's
    // Tool tab shows. One table, reached from both.
    let image_tool = tabs_for(BHostType::Image).iter().find(|t| t.label == "Tool").unwrap().body;
    let view3d_tool = tabs_for(BHostType::View3D).iter().find(|t| t.label == "Tool").unwrap().body;
    assert_eq!(image_tool, view3d_tool, "one Active Tool panel, not a copy each");
}
