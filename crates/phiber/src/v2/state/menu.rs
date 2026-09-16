//! Menu open/close handler — pure functions on `BState`.
//!
//! Two operators cover every menubar interaction:
//!
//! Every key is scoped to the host whose header owns the menubar:
//!   `ctx.menu.<host_slug>.<Label>[.<Sub>...]`
//! Without the host segment, two hosts that both declare a "View" menu (the
//! Shading workspace shows six such hosts at once) share one key, so opening
//! View in one header deploys every other header's View popover too.
//!
//! - [`apply_toggle`] at depth 3 (`ctx.menu.<host>.<X>`) implements
//!   *top-close*: every other top-level menu — in ANY host — is set to
//!   [`V_FALSE`] first, then
//!   the target is flipped. Callers pass a top-level menu key when the
//!   user clicks a menubar label. Requirement 43.1, 43.5.
//!
//! - [`apply_open`] at depth ≥ 2 implements *sibling-close*: every sibling
//!   under `key`'s parent prefix is closed, then `key` is set to
//!   [`V_TRUE`]. Idempotent — if `key` is already [`V_TRUE`] the state is
//!   left untouched (Requirement 43.3). Callers pass a nested menu key
//!   when the user hovers into a submenu. Requirement 43.4.
//!
//! - [`hover_open_if_menubar_active`] wraps [`apply_toggle`] but only
//!   dispatches when at least one top-level `ctx.menu.*` key is currently
//!   `V_TRUE`. This is how the menubar starts cycling on hover only
//!   after a click has opened one menu. Requirement 43.2, 44.1.
//!
//! Property tests in `crates/phiber/tests/menu_handler_props.rs` exercise
//! Requirements 52.1–52.3 (Properties 5, 6, 7).

use super::{
    BPath, BState,
    key::P_MENU,
    val::{V_FALSE, V_TRUE},
};

/// Toggle a menu key. Top-level toggles top-close every peer first.
pub fn apply_toggle(state: &mut BState, key: &BPath) {
    if is_top_level(key) {
        close_all_top_level_except(state, &key.encode());
        let currently_open = state.get(key) == Some(V_TRUE);
        state.set(key, if currently_open { V_FALSE } else { V_TRUE });
        return;
    }
    apply_open(state, key);
}

/// Open a menu key. No-op if already [`V_TRUE`]; otherwise closes every
/// sibling under `key`'s parent prefix and writes [`V_TRUE`].
pub fn apply_open(state: &mut BState, key: &BPath) {
    if state.get(key) == Some(V_TRUE) {
        return;
    }
    close_all_siblings(state, key);
    state.set(key, V_TRUE);
}

/// Hover into `target` only if the menubar is already open (i.e. at least
/// one top-level `ctx.menu.*` is [`V_TRUE`]).
pub fn hover_open_if_menubar_active(state: &mut BState, target: &BPath) {
    if is_menubar_active(state) {
        apply_toggle(state, target);
    }
}

/// True iff at least one top-level `ctx.menu.<host>.<X>` is [`V_TRUE`].
pub fn is_menubar_active(state: &BState) -> bool {
    state
        .iter_keys()
        .any(|k| is_top_level_encoded(k) && state.get_encoded(k) == Some(V_TRUE))
}

/// True iff at least one top-level menu is open *within this host*. Hover
/// switching is per-host: moving the pointer across one header's labels must
/// not reach into another host's menubar.
pub fn is_menubar_active_in(state: &BState, host: &str) -> bool {
    let prefix = format!("ctx.{}.{}.", P_MENU, host);
    state.iter_keys().any(|k| {
        k.strip_prefix(&prefix)
            .is_some_and(|rest| !rest.contains('.'))
            && state.get_encoded(k) == Some(V_TRUE)
    })
}

/// `ctx.menu.<host>.<label>` — exactly one dot after the `ctx.menu.` prefix.
fn is_top_level_encoded(k: &str) -> bool {
    k.strip_prefix("ctx.menu.")
        .is_some_and(|rest| rest.matches('.').count() == 1)
}

/// Build `ctx.menu.<host>.<top>` from a host slug and a top-level label.
pub fn k_menu(host: &'static str, top: &'static str) -> BPath {
    BPath::owned(vec![P_MENU, host, top])
}

/// Build `ctx.menu.<host>.<seg1>.<seg2>...` from a host slug and a segment
/// slice (the top label followed by any submenu labels).
pub fn k_menu_path(host: &'static str, labels: &[&'static str]) -> BPath {
    let mut segs: Vec<&'static str> = Vec::with_capacity(labels.len() + 2);
    segs.push(P_MENU);
    segs.push(host);
    segs.extend(labels);
    BPath::owned(segs)
}

fn is_top_level(key: &BPath) -> bool {
    key.segs.len() == 3 && key.segs[0] == P_MENU
}

/// Close every open top-level menu in every host except `keep` (an encoded
/// key). Cross-host: only one menubar popover is ever deployed at a time.
fn close_all_top_level_except(state: &mut BState, keep: &str) {
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| {
            is_top_level_encoded(k) && k.as_str() != keep && state.get_encoded(k) == Some(V_TRUE)
        })
        .cloned()
        .collect();
    for k in victims {
        state.set_encoded(&k, V_FALSE);
    }
}

fn close_all_siblings(state: &mut BState, key: &BPath) {
    let encoded = key.encode();
    let Some(dot) = encoded.rfind('.') else {
        return;
    };
    let parent_prefix = &encoded[..=dot]; // includes trailing '.'
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| {
            if !k.starts_with(parent_prefix) || *k == &encoded {
                return false;
            }
            let tail = &k[parent_prefix.len()..];
            !tail.contains('.') && state.get_encoded(k) == Some(V_TRUE)
        })
        .cloned()
        .collect();
    for k in victims {
        state.set_encoded(&k, V_FALSE);
    }
}
