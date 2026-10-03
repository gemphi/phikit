//! The four animation editors filter and search from their headers.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered on the right
//! (`part::search`, `part::filter`). The Dope Sheet, the Graph editor, the
//! Drivers editor and the NLA had none of the second and none of the third —
//! four headers with an empty middle and an empty right end, and the Dope
//! Sheet is the editor where filtering by name is how you find a channel in
//! a rig.
//!
//! Blender declares it once: `dopesheet_filter` over one `DopeSheet` struct,
//! drawn by `DOPESHEET_PT_filters`, `GRAPH_PT_filters` and `NLA_PT_filters`.
//! So there is one table and one key scope here, and these pin that — four
//! copies of it is how they would drift.

use phiber::v2::host::BHostType;
use phiber::v2::part::filter::{dope, filters_channels, has_filter};
use phiber::v2::part::search::search_key;
use phiber::v2::state::val::{V_FALSE, V_TRUE};
use phiber::v2::state::{BKey as _, BState, DopeKey};
use phiber::v2::view::BView;

const EDITORS: [BHostType; 4] =
    [BHostType::Action, BHostType::Ipo, BHostType::Drivers, BHostType::Nla];

/// The popover the header's control opens, when it is open.
fn opened(state: &BState, is_nla: bool) -> Option<Vec<BView>> {
    match dope::view(state, is_nla) {
        BView::Popover { body, open, .. } => open.then_some(body),
        _ => None,
    }
}

#[test]
fn all_four_declare_a_filter_and_a_search() {
    for host in EDITORS {
        assert!(filters_channels(host), "{host:?} filters channels");
        assert!(has_filter(host), "{host:?} has a header filter");
        assert_eq!(
            search_key(host),
            Some(DopeKey::Search.path()),
            "{host:?} searches by `DopeSheet.filter_text`"
        );
    }
}

#[test]
fn they_share_one_search_key() {
    // One `DopeSheet` in Blender per space, but the same struct and the same
    // panel: narrowing the Dope Sheet and then opening the Graph editor must
    // not show everything again.
    let keys: Vec<_> = EDITORS.iter().map(|host| search_key(*host)).collect();
    assert!(keys.windows(2).all(|pair| pair[0] == pair[1]), "four editors, one key: {keys:?}");
    assert_eq!(DopeKey::Search.path().encode(), "ctx.dopesheet.filter_text");
}

#[test]
fn a_host_that_does_not_filter_channels_is_unaffected() {
    for host in [BHostType::Time, BHostType::View3D, BHostType::Outliner, BHostType::Text] {
        assert!(!filters_channels(host), "{host:?} has no channels to filter");
    }
    // And the ones that already searched keep their own key.
    assert_ne!(search_key(BHostType::Outliner), Some(DopeKey::Search.path()));
    assert_ne!(search_key(BHostType::Buts), Some(DopeKey::Search.path()));
}

#[test]
fn the_control_is_shut_until_it_is_clicked() {
    assert!(opened(&BState::new(), false).is_none(), "shut to start with");
    let mut state = BState::new();
    state.set(&DopeKey::FilterOpen.path(), V_TRUE);
    assert!(opened(&state, false).is_some(), "the header's control opens it");
}

#[test]
fn the_popover_is_blenders_boxes_and_the_nla_swaps_one() {
    let mut state = BState::new();
    state.set(&DopeKey::FilterOpen.path(), V_TRUE);
    let dope_rows = opened(&state, false).expect("the open popover");
    let nla_rows = opened(&state, true).expect("the NLA's");
    assert_eq!(dope_rows.len(), nla_rows.len(), "the NLA swaps a box, it does not add one");
    assert_eq!(dope::TYPES.len(), 23, "every type `space_dopesheet.py` lays out");
    assert_eq!(dope::ERRORS.key, "show_only_errors");
    assert_eq!(dope::MISSING_NLA.key, "show_missing_nla");
}

#[test]
fn an_untouched_box_answers_the_way_a_fresh_blender_does() {
    // The type boxes ship ON: defaulting them off would empty the channel
    // list for anyone who never opens the popover, which is worse than having
    // no filter at all.
    let state = BState::new();
    for spec in &dope::TYPES {
        assert!(dope::is_on(&state, spec), "{} should ship on", spec.key);
    }
    assert!(dope::is_on(&state, &dope::INLINE[0]), "Only Show Selected ships on");
    assert!(!dope::is_on(&state, &dope::INLINE[1]), "Show Hidden ships off");
}

#[test]
fn a_box_the_user_turned_off_stays_off() {
    let mut state = BState::new();
    state.set(&dope::key(&dope::TYPES[0]), V_FALSE);
    assert!(!dope::is_on(&state, &dope::TYPES[0]), "an explicit false beats the default");
    assert_eq!(
        DopeKey::Filter("show_transforms").path().encode(),
        "ctx.dopesheet.show_transforms"
    );
}

#[test]
fn searching_does_not_open_the_filter() {
    // The middle of the header and its right end are two controls; one key
    // for both would have typing in the search box open the popover.
    let mut state = BState::new();
    state.set(&DopeKey::Search.path(), "spine");
    assert!(opened(&state, false).is_none());
    assert_ne!(DopeKey::Search.path(), DopeKey::FilterOpen.path());
}
