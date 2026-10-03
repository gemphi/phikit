//! A sidebar that is open paints the column it opened.
//!
//! `part::sidebar` builds the panels and hands them to the renderer on
//! `BRailView::panels`. `view/draw/rail.rs` laid out the strip's entries and
//! never read that field, so every sidebar in the app was half-wired: N set
//! the state, the strip redrew with the tab active, and the column was built
//! and thrown away. `draw::sidebar::render_at` and
//! `BPanelEdge::beside_strip` had been written for it and were reached from
//! nowhere.
//!
//! `every_file_is_reachable` catches a *file* nothing declares. This is the
//! same failure one level down: a field nothing reads. The compiler is
//! silent about both, so a test has to ask.
//!
//! It asks of every host that declares sidebar tabs, not just the viewport —
//! one host wired and the rest not is how this got missed.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::set::tabs_for;
use phiber::v2::part::sidebar::show::BSidebarShown;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::val::V_TRUE;
use phiber::v2::state::{BState, CtxKey, ModalKey};

/// Every host that offers a sidebar at all.
fn hosts_with_a_sidebar() -> Vec<BHostType> {
    BHostType::ALL.iter().copied().filter(|host| !tabs_for(*host).is_empty()).collect()
}

#[test]
fn the_renderer_reads_the_field_the_part_fills_in() {
    // The part sets `panels`; the renderer has to use it. A source check,
    // because a field that is written and never read compiles perfectly.
    let rail = include_str!("../src/v2/view/draw/rail.rs");
    assert!(
        rail.contains("rail.panels"),
        "the rail renderer must read `BRailView::panels` — the part built them for it"
    );
    assert!(
        rail.contains("BSidebar(panels)"),
        "and draw them as the sidebar column, not as more strip entries"
    );
}

/// The sidebar tabs that still open nothing, host by tab.
///
/// A whole-host list let a partly-filled host pass: give one of its three
/// tabs a body and the host came off the list while the other two stayed
/// empty. So the backlog names the pair, and it may only shrink — writing a
/// tab's panels is what takes its line out.
///
/// What is left is mostly `draw_buttons_ext` and its kin: a panel whose rows
/// come from the active node, strip or channel rather than from a table, so
/// declaring it would mean inventing rows. Those need their own mechanism,
/// not a table.
const NO_PANELS_YET: &[(BHostType, &str)] = &[
    (BHostType::Shader, "Options"),
    (BHostType::Node, "Options"),
    (BHostType::GeoNode, "Attributes"),
    (BHostType::Seq, "Strip"),
    (BHostType::Seq, "Modifiers"),
    (BHostType::Seq, "Preview"),
    (BHostType::Clip, "Track"),
    (BHostType::Clip, "Solve"),
    (BHostType::Nla, "Edited Action"),
    (BHostType::Nla, "Item"),
    (BHostType::Ipo, "View"),
    (BHostType::Ipo, "F-Curve"),
    (BHostType::Ipo, "Modifiers"),
    (BHostType::Ipo, "Drivers"),
    (BHostType::Drivers, "View"),
    (BHostType::Drivers, "F-Curve"),
    (BHostType::Drivers, "Modifiers"),
    (BHostType::Drivers, "Drivers"),
];

#[test]
fn a_sidebar_tab_with_nothing_behind_it_is_named_in_the_backlog() {
    // A tab that opens nothing is a button that does nothing. That is
    // allowed only while the pair is named above, and the list may only
    // shrink — so this fails BOTH ways: on a new empty tab, and on a tab
    // that has its panels now and is still listed.
    use phiber::v2::part::sidebar::set::BSidebarBody;
    let mut unlisted = Vec::new();
    let mut done = Vec::new();
    for host in hosts_with_a_sidebar() {
        for tab in tabs_for(host) {
            let empty = tab.body == BSidebarBody::None;
            let listed = NO_PANELS_YET.contains(&(host, tab.label));
            match (empty, listed) {
                (true, false) => unlisted.push((host, tab.label)),
                (false, true) => done.push((host, tab.label)),
                _ => {}
            }
        }
    }
    assert!(unlisted.is_empty(), "these tabs open nothing and are not in the backlog: {unlisted:?}");
    assert!(done.is_empty(), "these tabs have their panels now — take them off NO_PANELS_YET: {done:?}");
}

#[test]
fn the_backlog_names_no_tab_twice_and_no_tab_that_does_not_exist() {
    // A stale line would make the list look shorter than it is.
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for (host, label) in NO_PANELS_YET {
        let pair = (host.slug(), *label);
        assert!(!seen.contains(&pair), "{host:?} {label:?} is listed twice");
        seen.push(pair);
        assert!(
            tabs_for(*host).iter().any(|tab| tab.label == *label),
            "{host:?} has no {label:?} tab any more — drop the line"
        );
    }
}

#[gpui::test]
async fn opening_the_viewports_sidebar_paints_its_column(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(CtxKey::Object, "Cube");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("panel-sidebar-column").is_none(),
        "shut until opened, as Blender's startup file has it"
    );

    app.update(cx, |app, cx| {
        app.state.set(&BSidebarShown(BHostType::View3D).key(), V_TRUE);
        cx.notify();
    });
    cx.run_until_parked();
    let column = cx.debug_bounds("panel-sidebar-column").expect("the column the sidebar opened");
    let strip = cx.debug_bounds("tab-view_layer-view_layer").expect("the tab strip");
    // The rail is pinned to the editor's right edge, so its panels stand to
    // its left — the strip is the outer edge, the column the inner one.
    assert!(
        f32::from(column.origin.x + column.size.width) <= f32::from(strip.origin.x),
        "the column stands left of the strip: {column:?} vs {strip:?}"
    );
    assert!(f32::from(column.size.width) > 0.0, "and it has a width");
}
