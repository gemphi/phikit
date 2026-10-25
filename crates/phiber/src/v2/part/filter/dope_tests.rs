//! Tests for `dope` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

#[test]
fn every_type_box_ships_on() {
    // Defaulting them off empties the channel list for anyone who never
    // opens the popover.
    for spec in &TYPES {
        assert!(spec.default_on, "{} ships off", spec.key);
    }
}

#[test]
fn an_untouched_box_answers_the_way_a_fresh_blender_does() {
    let state = BState::new();
    assert!(is_on(&state, &TYPES[0]), "Transforms is on");
    assert!(is_on(&state, &INLINE[0]), "Only Show Selected is on");
    assert!(!is_on(&state, &INLINE[1]), "Show Hidden is off");
    assert!(!is_on(&state, &ERRORS), "Only Show Errors is off");
}

#[test]
fn a_ticked_box_overrides_the_default_either_way() {
    let mut state = BState::new();
    state.set(&TYPES[0].key(), V_FALSE);
    assert!(!is_on(&state, &TYPES[0]));
    state.set(&INLINE[1].key(), V_TRUE);
    assert!(is_on(&state, &INLINE[1]));
}

#[test]
fn the_nla_swaps_one_box_and_nothing_else() {
    let state = BState::new();
    let (dope, nla) = (groups(&state, false), groups(&state, true));
    let count = |g: &[super::super::fold::BGroup]| g.iter().map(|(_, rows)| rows.len()).sum::<usize>();
    assert_eq!(count(&dope), count(&nla), "one box swapped, not added");
    assert_eq!(count(&dope), INLINE.len() + 1 + 1 + ORDER.len() + TYPES.len());
}

#[test]
fn the_four_headings_fold_and_filter_by_type_ships_shut() {
    use crate::v2::view::BView;
    let state = BState::new();
    let cards = body(&state, false);
    assert_eq!(cards.len(), 4, "Show, Search, Order, Filters — not twenty-nine rows");
    for view in &cards {
        let BView::Card(card) = view else { panic!("a folding card per heading") };
        assert!(card.fold.is_some(), "a heading with no triangle is the long list again");
    }
    // Blender draws a type's box only when the file holds that type; the
    // twenty-three are folded away instead (see `closed`).
    let BView::Card(filters) = &cards[3] else { panic!("a card") };
    assert!(filters.body.is_empty(), "Filter by Type ships shut");
    let BView::Card(show) = &cards[0] else { panic!("a card") };
    assert_eq!(show.body.len(), INLINE.len() + 1, "the rest ship open, as Blender's do");
}

#[test]
fn opening_filter_by_type_brings_every_box_back() {
    use crate::v2::state::val::V_TRUE;
    use crate::v2::view::BView;
    let mut state = BState::new();
    state.set(&super::super::fold::expand_key(&SCOPE, "Filters"), V_TRUE);
    let cards = body(&state, false);
    let BView::Card(filters) = &cards[3] else { panic!("a card") };
    assert_eq!(filters.body.len(), KINDS.len(), "the boxes come back in their kinds");
    assert!(filters.body.iter().all(|v| matches!(v, BView::Card(_))), "each kind a folding sub-group");
}

#[test]
fn every_type_box_is_in_one_kind_and_no_kind_runs_past_eight() {
    for spec in &TYPES {
        let homes = KINDS.iter().filter(|(_, keys)| keys.contains(&spec.key)).count();
        assert_eq!(homes, 1, "{} is in {homes} kinds", spec.key);
    }
    for (name, keys) in &KINDS {
        assert!(keys.len() <= super::super::fold::MOST, "{name} has {} boxes", keys.len());
    }
}

#[test]
fn no_two_boxes_share_a_key() {
    let mut keys: Vec<&str> = INLINE.iter().chain(&ORDER).chain(&TYPES).map(|s| s.key).collect();
    keys.push(ERRORS.key);
    keys.push(MISSING_NLA.key);
    keys.push(MULTI_WORD.key);
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), keys.len(), "two boxes would fight over one key");
}
