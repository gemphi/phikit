//! The Properties header ends with Blender's Options panel.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered or displayed on the
//! right (`part::search`, `part::filter`). Properties had the first two and
//! an empty right end — and it is the one editor where Blender puts a panel
//! there: `PROPERTIES_PT_options`, whose single control is
//! `SpaceProperties.outliner_sync`.
//!
//! These pin both halves: that the host declares a filter at all, and that
//! what opens is the panel Blender draws, with Blender's items and Blender's
//! default.

use phiber::v2::host::BHostType;
use phiber::v2::part::filter::has_filter;
use phiber::v2::part::props::options::{self, set};
use phiber::v2::state::val::V_TRUE;
use phiber::v2::state::{BKey as _, BState, ButsKey};
use phiber::v2::view::BView;

/// The popover the header's right-hand control opens, when it is open.
fn opened(state: &BState) -> Option<Vec<BView>> {
    match options::view(state) {
        BView::Popover { body, open, .. } => open.then_some(body),
        _ => None,
    }
}

#[test]
fn the_properties_host_declares_a_filter() {
    assert!(
        has_filter(BHostType::Buts),
        "Properties carries PROPERTIES_PT_options on its header's right end"
    );
}

#[test]
fn the_control_is_shut_until_it_is_clicked() {
    let state = BState::new();
    assert!(opened(&state).is_none(), "the popover starts shut");
    let mut state = BState::new();
    state.set(&ButsKey::OptionsOpen.path(), V_TRUE);
    assert!(opened(&state).is_some(), "the header's control opens it");
}

#[test]
fn the_popover_is_blenders_panel() {
    let mut state = BState::new();
    state.set(&ButsKey::OptionsOpen.path(), V_TRUE);
    let body = opened(&state).expect("the open popover");
    assert_eq!(body.len(), set::OPTIONS.len(), "one row per option Blender draws");
    assert_eq!(set::OPTIONS.len(), 1, "Blender's panel is one control today");
    assert_eq!(set::SYNC.key, "outliner_sync");
    assert_eq!(set::SYNC.heading, "Sync with Outliner");
    assert_eq!(
        set::SYNC.items.iter().map(|(v, _)| *v).collect::<Vec<_>>(),
        vec!["ALWAYS", "NEVER", "AUTO"],
        "Blender's items, in Blender's order"
    );
}

#[test]
fn an_untouched_option_answers_the_way_a_fresh_blender_does() {
    // Not `false`, and not the first item: `AUTO` is what Blender ships,
    // and defaulting an enum to its first item would silently change
    // behaviour for anyone who never opens the panel.
    let state = BState::new();
    assert_eq!(options::value_of(&state, &set::SYNC), "AUTO");
    assert_eq!(set::SYNC.default, "AUTO");
}

#[test]
fn picking_an_item_is_what_the_state_holds() {
    let mut state = BState::new();
    state.set(&options::option_key(&set::SYNC), "NEVER");
    assert_eq!(options::value_of(&state, &set::SYNC), "NEVER");
    assert_eq!(ButsKey::Option("outliner_sync").path().encode(), "ctx.buts.options.outliner_sync");
}

#[test]
fn the_search_and_the_options_are_different_keys() {
    // The middle of the header and its right end are two controls; one key
    // for both would have typing in the search box open the popover.
    assert_ne!(ButsKey::Search.path(), ButsKey::OptionsOpen.path());
    let mut state = BState::new();
    state.set(&ButsKey::Search.path(), "cube");
    assert!(opened(&state).is_none(), "searching does not open the Options panel");
}
