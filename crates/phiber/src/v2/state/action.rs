//! Pure application of a rendered [`BAction`] to the single hierarchical state.
//!
//! This file only dispatches: a `Set` stores its value and hands the key to
//! `state::rule`, where every follow-up lives with the level it belongs to.
//! Nothing here compares a path — `tests/rule_budget.rs` keeps it that way.

use super::{
    BState, apply_open, apply_toggle, edit, menu, rule,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::prim::BAction;

/// Apply the same state transition used by `BPhiberApp`, without GPUI.
/// The app adds only `cx.notify()` after this transform.
pub fn apply_action(state: &mut BState, action: &BAction) {
    match action {
        BAction::Set(path, value) => {
            edit::before_set(state, path, value);
            state.set(path, value);
            rule::after_set(state, path, value);
            // Any committed `Set` dismisses every deployed overlay.
            menu::close_overlays(state);
        }
        BAction::Toggle(path) if super::MenuKey::owns(path) => apply_toggle(state, path),
        BAction::Toggle(path) => {
            let next = if state.get(path) == Some(V_TRUE) { V_FALSE } else { V_TRUE };
            state.set(path, next);
        }
        BAction::Open(path) => apply_open(state, path),
        BAction::Reset(path) => state.remove(path),
    }
}
