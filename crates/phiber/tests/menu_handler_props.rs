//! Properties 5, 6, 7 — Menu handler semantics.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 43.1, 43.3, 43.4, 43.5, 43.7, 44.5, 52.1–52.3
//!
//! - Property 5 (top-close): after any sequence of top-level toggles,
//!   at most one `ctx.menu.<X>` at depth 2 is V_TRUE.
//! - Property 6 (sibling-close preserves ancestor): opening a nested key
//!   leaves the parent chain intact.
//! - Property 7 (idempotent open): calling `apply_open(k)` twice with no
//!   intervening writes produces the same state as one call.

use phiber::v2::state::{BPath, BState, V_TRUE, apply_open, apply_toggle, k_menu, k_menu_path};
use proptest::prelude::*;

const TOPS: &[&str] = &["Blender", "File", "Edit", "Render", "Window", "Help"];
/// Host slug every property here scopes its keys to.
const H: &str = "topbar";
const SUBS: &[&str] = &["Import", "Export", "Recover", "Defaults", "New"];

fn arb_top() -> impl Strategy<Value = &'static str> {
    (0..TOPS.len()).prop_map(|i| TOPS[i])
}
fn arb_sub() -> impl Strategy<Value = &'static str> {
    (0..SUBS.len()).prop_map(|i| SUBS[i])
}

// Property 5 — at most one top-level menu is open after any toggle
// sequence starting from the initial (all-closed) state.
proptest! {
    #[test]
    fn top_close_last_open_wins(actions in prop::collection::vec(arb_top(), 0..12)) {
        let mut state = BState::new();
        for top in &actions {
            let key = k_menu(H, top);
            apply_toggle(&mut state, &key);
        }

        let open_tops: Vec<_> = TOPS
            .iter()
            .copied()
            .filter(|t| state.get(&k_menu(H, t)) == Some(V_TRUE))
            .collect();
        prop_assert!(open_tops.len() <= 1, "expected ≤1 open top, got {:?}", open_tops);
    }
}

// Property 6 — opening a nested key leaves the parent chain intact.
proptest! {
    #[test]
    fn sibling_close_preserves_ancestor(top in arb_top(), a in arb_sub(), b in arb_sub()) {
        prop_assume!(a != b);
        let mut state = BState::new();
        // Open the parent first, then a nested child, then a sibling.
        apply_toggle(&mut state, &k_menu(H, top));
        apply_open(&mut state, &k_menu_path(H, &[top, a]));
        apply_open(&mut state, &k_menu_path(H, &[top, b]));

        // Parent stays open.
        prop_assert_eq!(state.get(&k_menu(H, top)), Some(V_TRUE));
        // Last-opened sibling wins; the earlier sibling closes.
        prop_assert_eq!(state.get(&k_menu_path(H, &[top, b])), Some(V_TRUE));
    }
}

// Property 7 — `apply_open(k); apply_open(k);` is equivalent to a single
// call: the second call must not disturb any state.
proptest! {
    #[test]
    fn open_is_idempotent(top in arb_top(), sub in arb_sub()) {
        let key = k_menu_path(H, &[top, sub]);

        let mut once = BState::new();
        apply_toggle(&mut once, &k_menu(H, top));
        apply_open(&mut once, &key);

        let mut twice = BState::new();
        apply_toggle(&mut twice, &k_menu(H, top));
        apply_open(&mut twice, &key);
        apply_open(&mut twice, &key);

        let sort = |s: &BState| -> Vec<(String, String)> {
            let mut v: Vec<(String, String)> = s
                .iter_keys()
                .map(|k| (k.clone(), s.get_encoded(k).unwrap_or("").to_string()))
                .collect();
            v.sort();
            v
        };
        prop_assert_eq!(sort(&once), sort(&twice));
    }
}

#[test]
fn menubar_active_helper_starts_false() {
    let state = BState::new();
    assert!(!phiber::v2::state::is_menubar_active(&state));
}

#[test]
fn opening_top_makes_menubar_active() {
    let mut state = BState::new();
    apply_toggle(&mut state, &k_menu(H, "File"));
    assert!(phiber::v2::state::is_menubar_active(&state));
}

/// Regression: two hosts that both declare a menu with the same label must
/// not share one open/closed flag. Opening "View" in the View3D header used
/// to deploy the Outliner's, Properties' and Image editor's "View" popovers
/// at the same time, because the key was `ctx.menu.View` for all of them.
#[test]
fn same_label_in_two_hosts_opens_independently() {
    let mut state = BState::new();
    apply_toggle(&mut state, &k_menu("view3d", "View"));

    assert_eq!(state.get(&k_menu("view3d", "View")), Some(V_TRUE));
    assert_ne!(state.get(&k_menu("outliner", "View")), Some(V_TRUE));
    assert_ne!(state.get(&k_menu("buts", "View")), Some(V_TRUE));
}

/// Only one top-level menu is deployed app-wide: opening a menu in a second
/// host closes the one already open in the first.
#[test]
fn opening_a_menu_in_another_host_closes_the_first() {
    let mut state = BState::new();
    apply_toggle(&mut state, &k_menu("view3d", "View"));
    apply_toggle(&mut state, &k_menu("outliner", "View"));

    assert_ne!(state.get(&k_menu("view3d", "View")), Some(V_TRUE));
    assert_eq!(state.get(&k_menu("outliner", "View")), Some(V_TRUE));
}

/// Hover-switching is armed per host, so an open menu in one editor header
/// does not make another editor's header hover-sensitive.
#[test]
fn menubar_active_is_scoped_per_host() {
    let mut state = BState::new();
    apply_toggle(&mut state, &k_menu("view3d", "View"));

    assert!(phiber::v2::state::is_menubar_active_in(&state, "view3d"));
    assert!(!phiber::v2::state::is_menubar_active_in(&state, "outliner"));
}

fn _use_bpath(_: BPath) {}
