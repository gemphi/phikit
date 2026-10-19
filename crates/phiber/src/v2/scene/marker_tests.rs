//! Tests for `marker` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;
use crate::v2::state::action::apply_action;
use crate::v2::view::prim::BAction;

fn op(state: &mut BState, what: &str) {
    apply_action(state, &BAction::Set(OpKey::Marker.path(), what.into()));
}

#[test]
fn markers_are_added_at_the_playhead_jumped_between_and_deleted() {
    crate::v2::scene::source::clear_loaded_scene();
    let mut state = BState::new();
    state.set(CtxKey::Frame, "10");
    op(&mut state, "add");
    op(&mut state, "add");
    assert_eq!(BMarkerList::now(&state).list, [(10, "F_10".to_string())], "one per frame");
    state.set(CtxKey::Frame, "30");
    op(&mut state, "add");
    state.set(CtxKey::Frame, "1");
    op(&mut state, "next");
    assert_eq!(state.get(CtxKey::Frame), Some("10"));
    op(&mut state, "next");
    assert_eq!(state.get(CtxKey::Frame), Some("30"));
    op(&mut state, "next");
    assert_eq!(state.get(CtxKey::Frame), Some("30"), "no marker beyond: stays");
    op(&mut state, "prev");
    assert_eq!(state.get(CtxKey::Frame), Some("10"));
    op(&mut state, "delete");
    assert_eq!(BMarkerList::now(&state).list, [(30, "F_30".to_string())]);
    assert!(state.get(OpKey::Marker).is_none());
    assert_eq!(BMarkerList::parse("5=b|2=a").encode(), "2=a|5=b", "kept in frame order");
}

#[test]
fn markers_are_selected_moved_and_the_selection_is_what_delete_takes() {
    crate::v2::scene::source::clear_loaded_scene();
    let mut state = BState::new();
    state.set(CtxKey::Markers, "10=A|20=B|30=C");
    op(&mut state, "select:10");
    assert_eq!(BMarkerList::selected(&state), [10]);
    op(&mut state, "select+:30");
    assert_eq!(BMarkerList::selected(&state), [10, 30], "Shift extends");
    op(&mut state, "select:20");
    assert_eq!(BMarkerList::selected(&state), [20], "a plain click starts over");
    // Dragged to a free frame: the marker moves and stays selected.
    op(&mut state, "move:20:25");
    assert_eq!(BMarkerList::now(&state).list, [(10, "A".to_string()), (25, "B".to_string()), (30, "C".to_string())]);
    assert_eq!(BMarkerList::selected(&state), [25]);
    // Onto a taken frame: nothing moves.
    op(&mut state, "move:25:30");
    assert_eq!(BMarkerList::now(&state).at(25), Some("B"));
    // Delete takes the selection, not the playhead's marker.
    state.set(CtxKey::Frame, "10");
    op(&mut state, "delete");
    assert_eq!(BMarkerList::now(&state).list, [(10, "A".to_string()), (30, "C".to_string())]);
    assert!(BMarkerList::selected(&state).is_empty());
    op(&mut state, "select:none");
    op(&mut state, "delete");
    assert_eq!(BMarkerList::now(&state).list, [(30, "C".to_string())], "without a selection, the playhead's");
}

#[test]
fn duplicate_copies_the_selection_to_the_next_free_frame_and_selects_it() {
    crate::v2::scene::source::clear_loaded_scene();
    let mut state = BState::new();
    state.set(CtxKey::Markers, "10=A|11=B|20=C");
    op(&mut state, "select:10");
    op(&mut state, "duplicate");
    // Blender duplicates in place, then the modal move spreads them.
    // phiber has one marker per frame: 10's copy lands at 12 (11 taken).
    let markers = BMarkerList::now(&state).list;
    assert!(markers.contains(&(10, "A".to_string())), "original kept");
    assert!(markers.contains(&(12, "A".to_string())), "copy at next free frame");
    assert_eq!(BMarkerList::selected(&state), [12], "copy selected, original deselected");
}

#[test]
fn lock_markers_gates_edits_but_leaves_selection_and_jumps() {
    crate::v2::scene::source::clear_loaded_scene();
    let mut state = BState::new();
    state.set(CtxKey::Markers, "10=A|20=B");
    state.set(CtxKey::MarkersLocked, crate::v2::state::val::V_TRUE);
    // add is gated.
    state.set(CtxKey::Frame, "30");
    op(&mut state, "add");
    assert!(BMarkerList::now(&state).at(30).is_none(), "locked: add is a no-op");
    // delete is gated.
    op(&mut state, "select:10");
    op(&mut state, "delete");
    assert_eq!(BMarkerList::now(&state).list.len(), 2, "locked: delete is a no-op");
    // Selection and jumps still work.
    op(&mut state, "select:20");
    assert_eq!(BMarkerList::selected(&state), [20]);
    state.set(CtxKey::Frame, "1");
    op(&mut state, "next");
    assert_eq!(state.get(CtxKey::Frame), Some("10"));
}
