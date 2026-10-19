//! Tests for `input` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;
use crate::v2::state::{BKey as _, BState, FileKey};

/// A window-less app with `value` in the Search field and the caret in it,
/// as a click would leave it — enough to drive the caret keys, which need
/// no window.
fn editing(value: &str) -> BPhiberApp {
    let mut app = BPhiberApp::new();
    app.state = BState::new();
    app.state.set(&FileKey::Search.path(), value);
    edit::start(&mut app.state, &FileKey::Search.path());
    app.input.caret = BCaret::at_end(value);
    app
}

fn search(app: &BPhiberApp) -> String {
    app.state.get(&FileKey::Search.path()).unwrap_or("").to_string()
}

#[test]
fn arrows_move_the_caret_and_backspace_deletes_mid_string() {
    let mut app = editing("cube");
    assert_eq!(app.input.caret.cursor(), 4);
    app.caret_left();
    app.caret_left(); // caret between 'u' and 'b'
    assert_eq!(app.input.caret.cursor(), 2);
    assert!(app.caret_backspace()); // removes 'u'
    assert_eq!(search(&app), "cbe");
    assert_eq!(app.input.caret.cursor(), 1);
    app.caret_delete(); // removes 'b' after the caret
    assert_eq!(search(&app), "ce");
}

#[test]
fn select_all_then_delete_clears_the_field() {
    let mut app = editing("cube");
    app.caret_select_all();
    assert_eq!(app.input.caret.selected, 0..4);
    assert!(app.caret_backspace());
    assert_eq!(search(&app), "");
}

#[test]
fn shift_arrow_selects_and_home_end_jump() {
    let mut app = editing("cube");
    app.caret_home();
    assert_eq!(app.input.caret.cursor(), 0);
    app.caret_select_right();
    app.caret_select_right();
    assert_eq!(app.input.caret.selected, 0..2); // "cu" selected
    app.caret_end();
    assert_eq!(app.input.caret.cursor(), 4);
    assert!(app.input.caret.selected.is_empty(), "End collapses the selection");
}

#[test]
fn a_click_without_painted_geometry_places_the_caret_at_the_end() {
    let mut app = editing("cube");
    app.input.caret.place(1);
    app.place_caret_at_point(gpui::point(gpui::px(999.0), gpui::px(0.0)), false);
    assert_eq!(app.input.caret.cursor(), 4, "no line captured yet → caret rests at the end");
}
