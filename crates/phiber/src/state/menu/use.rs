//! Menu state pure-use helpers.
#![allow(dead_code)]

use super::MenuState;

/// Toggle a menu open/closed by its string key.
pub fn toggle_menu(state: &mut MenuState, key: &str) {
    state.toggle(key);
}

/// Close every open menu.
pub fn close_all(state: &mut MenuState) {
    state.close_all();
}

/// Toggle the checked state of a menu item by label.
pub fn toggle_checked(state: &mut MenuState, label: &str) {
    state.toggle_check(label);
}
