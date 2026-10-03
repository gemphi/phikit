//! The Timeline's right column is a sidebar, not empty space.
//!
//! The Timeline is a mode of the Dope Sheet (`SPACE_ACTION`), and the Dope
//! Sheet's Center region carries `RGN_TYPE_UI` on its right. The Timeline's
//! declared a Center pane and nothing else, so the column down its right
//! edge was blank while every other timing editor beside it had a strip
//! there.
//!
//! What the strip shows is the transport's own two panels, and it shows them
//! by calling the very functions the header's popovers call
//! (`playback::panel::body`, `::snap_body`). That is the point of the
//! `Playback` body variant: one set of rows, two ways to reach them, so a
//! setting cannot read one way in the popover and another in the sidebar.

use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::panel::BSidebarPanels;
use phiber::v2::part::sidebar::set::{BSidebarBody, tabs_for};
use phiber::v2::pane::BPaneType;
use phiber::v2::part::BPartType;
use phiber::v2::region::BRegionType;
use phiber::v2::state::BState;
use phiber::v2::view::BView;

/// The titles the sidebar's cards carry, for the tab showing.
fn titles(state: &BState) -> Vec<String> {
    BSidebarPanels::new(BHostType::Time)
        .views(state)
        .iter()
        .filter_map(|view| match view {
            BView::Card(card) => card.title().map(str::to_string),
            _ => None,
        })
        .collect()
}

#[test]
fn the_timeline_declares_a_sidebar_where_its_right_column_was() {
    let center = BHostType::Time
        .spec()
        .regions
        .iter()
        .find(|region| region.kind == BRegionType::Center)
        .expect("the Timeline's content region");
    let right = center
        .panes
        .iter()
        .find(|pane| pane.kind == BPaneType::Right)
        .expect("the Timeline's right column");
    assert_eq!(right.parts, &[BPartType::Sidebar], "the column is the sidebar part, not a new one");
}

#[test]
fn it_is_the_same_part_the_dope_sheet_puts_there() {
    // The Timeline is a mode of the Dope Sheet: the same region, the same
    // part. A second spelling of "sidebar" for this one host is the thing
    // this test exists to stop.
    for host in [BHostType::Time, BHostType::Action] {
        let center = host.spec().regions.iter().find(|r| r.kind == BRegionType::Center).expect("a content region");
        let right = center.panes.iter().find(|p| p.kind == BPaneType::Right).expect("a right column");
        assert_eq!(right.parts, &[BPartType::Sidebar], "{host:?}");
    }
}

#[test]
fn its_tab_is_the_transports_own_panels() {
    let tabs = tabs_for(BHostType::Time);
    assert_eq!(tabs.len(), 1, "one tab: the transport's");
    assert_eq!(tabs[0].label, "Playback");
    assert_eq!(tabs[0].body, BSidebarBody::Playback);
}

#[test]
fn the_sidebar_lists_both_of_the_transports_panels() {
    let state = BState::new();
    assert_eq!(titles(&state), vec!["Playback", "Playhead Snapping"]);
}

#[test]
fn a_panel_here_lays_out_the_rows_the_popover_lays_out() {
    use phiber::v2::part::playback::panel;
    let state = BState::new();
    let cards = BSidebarPanels::new(BHostType::Time).views(&state);
    assert_eq!(cards.len(), 2);
    // A panel opens by default, as every other sidebar panel does, and what
    // it draws is the popover's rows — the same function, not a copy that
    // can drift from it.
    let rows = |view: &BView| match view {
        BView::Card(card) => card.body.len(),
        _ => panic!("a card"),
    };
    assert_eq!(rows(&cards[0]), panel::body(&state).len(), "Playback draws the popover's rows");
    assert_eq!(rows(&cards[1]), panel::snap_body(&state).len(), "so does Playhead Snapping");
    assert!(panel::body(&state).len() > 1, "and there are rows to draw");
}

#[test]
fn no_other_host_picked_up_a_sidebar_by_accident() {
    // `tabs_for` is a match, so a host with no entry answers with no tabs —
    // and a host with no tabs must not be given a strip of nothing.
    for host in [BHostType::Topbar, BHostType::Statusbar, BHostType::Buts, BHostType::Outliner] {
        assert!(tabs_for(host).is_empty(), "{host:?} has no sidebar tabs");
    }
}
