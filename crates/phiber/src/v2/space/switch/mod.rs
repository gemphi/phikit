//! Switching space — pure function on `BState`.
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
//! [`CtxKey::Space.path()`] and the `ctx.menu.*` prefix.

use crate::v2::state::{BState, CtxKey, menu::close_overlays};
use crate::v2::space::BSpaceType;

/// Switch the active workspace tab and collapse every open menu.
pub fn switch_space(state: &mut BState, target: BSpaceType) {
    state.set(CtxKey::Space, target.slug());
    close_overlays(state);
}
