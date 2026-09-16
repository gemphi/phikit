//! Pure application of a rendered [`BAction`] to the single hierarchical state.

use super::{
    BState, apply_open, apply_toggle,
    key::{K_MODAL, P_MENU},
    val::{V_EMPTY, V_FALSE, V_TRUE},
};
use crate::v2::view::prim::BAction;

/// Apply the same state transition used by `BPhiberApp`, without GPUI.
/// The app adds only `cx.notify()` after this transform.
pub fn apply_action(state: &mut BState, action: &BAction) {
    match action {
        BAction::Set(path, value) => {
            state.set(path, value);
            // A newly selected File Browser column starts ascending. A
            // repeated click uses the dedicated direction toggle instead.
            if path == &crate::v2::part::custom::file::keys::k_sort() {
                state.set(
                    &crate::v2::part::custom::file::keys::k_sort_invert(),
                    V_FALSE,
                );
            }
            // File-browser navigation intents are reduced here: the Back /
            // Forward / Up / Refresh buttons and every path write go through
            // `file_nav`, which owns the folder history.
            if super::file_nav::handles(path) {
                super::file_nav::apply(state, path, value);
            }
            // Picking a file is a commit: record it in the welcome window's
            // recent list and dismiss whatever picker was on screen (the file
            // browser modal, or the splash itself).
            if path.encode() == "ctx.file.selected" && !value.is_empty() {
                crate::v2::modal::splash::recent::push_recent(state, value);
                state.set(&K_MODAL, V_EMPTY);
            }
            close_all_menus(state);
        }
        BAction::Toggle(path) if path.segs.first() == Some(&P_MENU) => {
            apply_toggle(state, path);
        }
        BAction::Toggle(path) => {
            let next = if state.get(path) == Some(V_TRUE) {
                V_FALSE
            } else {
                V_TRUE
            };
            state.set(path, next);
        }
        BAction::Open(path) => apply_open(state, path),
        BAction::Reset(path) => state.remove(path),
    }
}

/// Any committed `Set` dismisses every deployed overlay — the menubar
/// popovers AND the per-area editor-type switcher popovers. Without the
/// second prefix, choosing an editor type left its own popover on screen.
fn close_all_menus(state: &mut BState) {
    let keys: Vec<String> = state
        .iter_keys()
        .filter(|key| key.starts_with("ctx.menu.") || key.starts_with("ctx.switcher."))
        .cloned()
        .collect();
    for key in keys {
        state.set_encoded(&key, V_FALSE);
    }
}
