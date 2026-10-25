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
//! - [`apply_toggle`] is what a CLICK arrives as, at any depth, and it flips:
//!   an open key closes (with everything open under it), a shut one opens. At
//!   depth 3 (`ctx.menu.<host>.<X>`) it adds *top-close* — every other
//!   top-level menu, in ANY host, is set to [`V_FALSE`] first. Requirement
//!   43.1, 43.5.
//!
//! - [`apply_open`] is what a HOVER arrives as, at depth ≥ 2, and it only
//!   opens: every sibling under `key`'s parent prefix is closed, then `key`
//!   is set to [`V_TRUE`]. Idempotent — if `key` is already [`V_TRUE`] the
//!   state is left untouched (Requirement 43.3), which is what keeps a
//!   submenu up while the pointer crosses the row that opened it.
//!   Requirement 43.4.
//!
//! - [`hover_open_if_menubar_active`] wraps [`apply_toggle`] but only
//!   dispatches when at least one top-level `ctx.menu.*` key is currently
//!   `V_TRUE`. This is how the menubar starts cycling on hover only
//!   after a click has opened one menu. Requirement 43.2, 44.1.
//!
//! Property tests in `crates/phiber/tests/menu_handler_props.rs` exercise
//! Requirements 52.1–52.3 (Properties 5, 6, 7).

use super::{
    BPath, BState, MenuKey,
    val::{V_FALSE, V_TRUE},
};

/// Toggle a menu key. Top-level toggles top-close every peer first.
///
/// A toggle closes what is open at EVERY depth. This used to fall through to
/// [`apply_open`] for a nested key, and `apply_open` returns early when the
/// key is already `V_TRUE` — so clicking the row that had opened a submenu
/// did nothing, while clicking the menubar label above it closed its menu.
/// One of the two was wrong, and it was the nested one: a click is a toggle
/// wherever it lands. Hover still arrives as [`apply_open`], so hovering an
/// open submenu leaves it open.
pub fn apply_toggle(state: &mut BState, key: &BPath) {
    let top = MenuKey::is_top_level(key);
    if top {
        close_all_top_level_except(state, &key.encode());
        close_panels(state);
    }
    if state.get(key) == Some(V_TRUE) {
        close(state, key);
        return;
    }
    if top {
        state.set(key, V_TRUE);
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
        .any(|k| MenuKey::is_top_level_encoded(k) && state.get_encoded(k) == Some(V_TRUE))
}

/// True iff at least one top-level menu is open *within this host*. Hover
/// switching is per-host: moving the pointer across one header's labels must
/// not reach into another host's menubar.
pub fn is_menubar_active_in(state: &BState, host: &str) -> bool {
    let prefix = MenuKey::host_prefix(host);
    state.iter_keys().any(|k| {
        k.strip_prefix(&prefix)
            .is_some_and(|rest| !rest.contains('.'))
            && state.get_encoded(k) == Some(V_TRUE)
    })
}

/// Close every deployed overlay — the menubar popovers AND the per-area
/// editor-type switcher popovers. Without the second prefix, choosing an
/// editor type left its own popover on screen.
pub fn close_overlays(state: &mut BState) {
    let menus = MenuKey::prefix();
    let switchers = super::AreaKey::switcher_prefix();
    // And every enum picker. A dropdown stayed open after its own value was
    // chosen: the item wrote the value and nothing closed the list, so you
    // picked GPU Compute and the menu sat there over the panel. Blender's
    // closes the moment you choose, and so does every menu here already —
    // the picker was the one overlay this sweep did not name.
    let pickers = super::ButsKey::dropdown_prefix();
    let keys: Vec<String> = state
        .iter_keys()
        // And every colour picker (`<colour key>.picker`, `BCtrl::picker_key`):
        // they were the one popover no sweep named, so a picker stayed up
        // after its colour was set, reset or another was opened.
        .filter(|k| k.starts_with(&menus) || k.starts_with(&switchers) || k.starts_with(&pickers) || is_colour_picker(k))
        .cloned()
        .collect();
    for key in keys {
        state.set_encoded(&key, V_FALSE);
    }
}

/// Whether an encoded key is a colour picker's open flag.
pub fn is_colour_picker(encoded: &str) -> bool {
    encoded.ends_with(".picker")
}

/// Open one colour picker: every other picker closes first, so at most one is
/// ever up — opening a second used to stack it over the first.
pub fn toggle_colour_picker(state: &mut BState, path: &crate::v2::state::BPath) {
    let opening = state.get(path) != Some(V_TRUE);
    let encoded = path.encode();
    let others: Vec<String> = state.iter_keys().filter(|k| is_colour_picker(k) && **k != encoded).cloned().collect();
    for key in others {
        state.set_encoded(&key, V_FALSE);
    }
    state.set(path, if opening { V_TRUE } else { V_FALSE });
}

/// Close every header panel popover (`MenuKey::Panel`): a menu opening
/// takes them down, as Blender's popovers go when a menu opens.
pub fn close_panels(state: &mut BState) {
    let mark = MenuKey::panel_mark();
    let keys: Vec<String> = state.iter_keys().filter(|k| k.contains(mark) && state.get_encoded(k) == Some(V_TRUE)).cloned().collect();
    for key in keys {
        state.set_encoded(&key, V_FALSE);
    }
}

/// Close `key` and everything opened underneath it. A menu that closes while
/// a submenu of its is open must take that submenu with it — otherwise the
/// submenu's key stays `V_TRUE` and the popover reappears, stacked over the
/// next menu the user opens.
pub fn close(state: &mut BState, key: &BPath) {
    close_encoded(state, &key.encode());
}

fn close_encoded(state: &mut BState, encoded: &str) {
    let descendants = format!("{encoded}.");
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| {
            (k.as_str() == encoded || k.starts_with(&descendants))
                && state.get_encoded(k) == Some(V_TRUE)
        })
        .cloned()
        .collect();
    for k in victims {
        state.set_encoded(&k, V_FALSE);
    }
    state.set_encoded(encoded, V_FALSE);
}

/// Close every open top-level menu in every host except `keep` (an encoded
/// key). Cross-host: only one menubar popover is ever deployed at a time.
fn close_all_top_level_except(state: &mut BState, keep: &str) {
    let victims: Vec<String> = state
        .iter_keys()
        .filter(|k| {
            MenuKey::is_top_level_encoded(k) && k.as_str() != keep && state.get_encoded(k) == Some(V_TRUE)
        })
        .cloned()
        .collect();
    for k in victims {
        close_encoded(state, &k);
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
        close_encoded(state, &k);
    }
}

#[cfg(test)]
mod colour_picker_tests {
    use super::*;
    use crate::v2::state::BPath;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn picker(name: &'static str) -> BPath {
        BPath::owned(vec!["ui_style", "colors", name, "picker"])
    }

    #[test]
    fn one_colour_picker_at_a_time_and_a_set_closes_it() {
        let mut state = BState::new();
        apply_action(&mut state, &BAction::Toggle(picker("accent")));
        assert_eq!(state.get(&picker("accent")), Some(V_TRUE));
        // Opening a second closes the first.
        apply_action(&mut state, &BAction::Toggle(picker("text")));
        assert_eq!(state.get(&picker("text")), Some(V_TRUE));
        assert_ne!(state.get(&picker("accent")), Some(V_TRUE), "the first picker closed");
        // A committed write anywhere takes it down, as it does every overlay.
        apply_action(&mut state, &BAction::Set(BPath::owned(vec!["ui_style", "colors", "text"]), "0, 0, 80, 1".into()));
        assert_ne!(state.get(&picker("text")), Some(V_TRUE), "a set closed the picker");
    }
}
