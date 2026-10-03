//! The Properties editor searches from its header.
//!
//! Blender has had `SpaceProperties.search_filter` in the Properties header
//! since 2.90: typing narrows the active tab to the panels whose name
//! matches, and opens them. The field was missing here — the header's middle
//! sat empty in an editor that searches — and a field that filtered nothing
//! would have been worse than none, so these pin both halves.

use phiber::v2::host::BHostType;
use phiber::v2::part::search::search_key;
use phiber::v2::state::{BKey as _, BState, ButsKey, CtxKey};
use phiber::v2::view::{BView, BViewList};

/// The panel titles the Properties body lists for the active tab. The list
/// hands back one card per panel, so the titles are the cards' own.
fn panels(state: &BState) -> Vec<String> {
    phiber::v2::part::props::BPanelList::new(BHostType::Buts)
        .items(state)
        .iter()
        .filter_map(|view| match view {
            BView::Card(card) => card.title().map(str::to_string),
            _ => None,
        })
        .collect()
}

#[test]
fn the_properties_editor_has_a_search_key() {
    assert!(
        search_key(BHostType::Buts).is_some(),
        "Properties searches from its header in Blender; the host must name a key for it"
    );
    assert_eq!(search_key(BHostType::Buts), Some(ButsKey::Search.path()));
}

#[test]
fn an_empty_search_is_not_a_filter() {
    let mut state = BState::new();
    state.set(CtxKey::Object, "Cube");
    let all = panels(&state);
    state.set(&ButsKey::Search.path(), "   ");
    assert_eq!(panels(&state), all, "whitespace is not a query");
}

#[test]
fn a_search_keeps_only_the_panels_whose_name_matches() {
    let mut state = BState::new();
    state.set(CtxKey::Object, "Cube");
    let all = panels(&state);
    assert!(!all.is_empty(), "the tab lists panels to begin with");

    // Match on a word from a real panel title, so the test does not depend
    // on which tab happens to be first.
    let needle = all[0].to_lowercase();
    state.set(&ButsKey::Search.path(), &needle);
    let found = panels(&state);
    assert!(!found.is_empty(), "a panel named {needle:?} survives its own name as a query");
    assert!(
        found.iter().all(|title| title.to_lowercase().contains(&needle)),
        "every panel left matches {needle:?}: {found:?}"
    );
    assert!(found.len() <= all.len(), "a search never adds panels");
}

#[test]
fn a_search_that_matches_nothing_leaves_nothing() {
    let mut state = BState::new();
    state.set(CtxKey::Object, "Cube");
    state.set(&ButsKey::Search.path(), "zzzznotapanel");
    assert!(panels(&state).is_empty(), "no panel is named that");
}
