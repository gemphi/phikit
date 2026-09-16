//! Menu state context sync helpers.
#![allow(dead_code)]

use super::MenuState;

/// Close any open menus when the active context changes (e.g. switching spaces).
pub fn sync_context_change(state: &mut MenuState) {
    state.close_all();
}
