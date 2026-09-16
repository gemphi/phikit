//! Property 8 — Space switch round-trip.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 45.1, 45.3, 52.4
//!
//! After `switch_space(state, target)`:
//!   - `ctx.space` equals `target.slug()`.
//!   - Every `ctx.menu.*` key is V_FALSE.
//!   - Every non-menu, non-space key is unchanged from the input.

use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BPath, BState, K_SPACE, V_FALSE, switch_space};
use proptest::prelude::*;

fn arb_space() -> impl Strategy<Value = BSpaceType> {
    (0..BSpaceType::ALL.len()).prop_map(|i| BSpaceType::ALL[i])
}

fn arb_key() -> impl Strategy<Value = BPath> {
    static SEGMENTS: &[&str] = &["scene", "object", "view_layer", "buts", "outliner", "theme"];
    prop::collection::vec((0..SEGMENTS.len()).prop_map(|i| SEGMENTS[i]), 1..=3)
        .prop_map(BPath::owned)
}

proptest! {
    #[test]
    fn switch_writes_slug_and_closes_menus(target in arb_space(), seed in prop::collection::vec((arb_key(), any::<u8>()), 0..8)) {
        let mut state = BState::new();
        for (k, v) in &seed {
            state.set(k, format!("v{v}"));
        }
        // Pre-open a top-level menu so we can observe it closing.
        state.set(&BPath::owned(vec!["menu", "File"]), "true");

        // Snapshot every non-menu key before the switch.
        let before: Vec<(String, String)> = state
            .iter_keys()
            .filter(|k| !k.starts_with("ctx.menu.") && k.as_str() != "ctx.space")
            .map(|k| (k.clone(), state.get_encoded(k).unwrap_or("").to_string()))
            .collect();

        switch_space(&mut state, target);

        // Space slug matches.
        prop_assert_eq!(state.get(&K_SPACE), Some(target.slug()));

        // Every menu key is V_FALSE.
        for k in state.iter_keys() {
            if k.starts_with("ctx.menu.") {
                prop_assert!(
                    state.get_encoded(k) == Some(V_FALSE),
                    "menu key {} not closed after switch_space", k
                );
            }
        }

        // Non-menu, non-space keys are preserved bit-for-bit.
        for (k, v) in &before {
            prop_assert_eq!(state.get_encoded(k), Some(v.as_str()));
        }
    }
}
