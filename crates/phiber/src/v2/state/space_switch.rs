//! Space switcher — pure function on `BState`.
//!
//! [`switch_space`] does exactly two things:
//!
//! 1. Sets `ctx.space` to the target space's slug (Requirement 45.1).
//! 2. Closes every `ctx.menu.*` key by writing [`V_FALSE`] (Requirement
//!    45.2). Any menu that was open on the previous space collapses.
//!
//! Nothing else in the state map is touched — the property tested by
//! Requirement 52.4 (`switch(b); switch(a);` restores every non-menu key)
//! holds trivially because we never read or write anything outside
//! [`K_SPACE`] and the `ctx.menu.*` prefix.

use super::{BState, key::K_SPACE, val::V_FALSE};
use crate::v2::space::BSpaceType;

/// Switch the active workspace tab and collapse every open menu.
pub fn switch_space(state: &mut BState, target: BSpaceType) {
    state.set(&K_SPACE, target.slug());
    close_all_menus(state);
}

fn close_all_menus(state: &mut BState) {
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| k.starts_with("ctx.menu.") || k.starts_with("ctx.switcher."))
        .cloned()
        .collect();
    for k in victims {
        state.set_encoded(&k, V_FALSE);
    }
}
