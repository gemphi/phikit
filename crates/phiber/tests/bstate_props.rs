//! Properties 15 & 16 — `BState` set/get round-trip and V_FALSE
//! idempotence.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 51.6, 51.7, 51.8
//!
//! - Set then get returns the written value.
//! - Writing V_FALSE twice leaves the map identical to writing it once.

use phiber::v2::state::{BPath, BState, V_EMPTY, V_FALSE, V_TRUE};
use proptest::prelude::*;

fn arb_segment() -> impl Strategy<Value = &'static str> {
    // Static string pool so BPath::owned can hold the leaked references.
    static POOL: &[&str] = &[
        "space", "menu", "File", "Edit", "Render", "Window", "Help", "modal", "theme", "zone",
        "left", "right", "bottom", "outliner", "view3d", "buts", "scene",
    ];
    (0..POOL.len()).prop_map(|i| POOL[i])
}

fn arb_path() -> impl Strategy<Value = BPath> {
    prop::collection::vec(arb_segment(), 1..=4).prop_map(BPath::owned)
}

fn arb_value() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(V_TRUE),
        Just(V_FALSE),
        Just(V_EMPTY),
        Just("hello"),
        Just("world")
    ]
}

proptest! {
    #[test]
    fn set_then_get_round_trip(path in arb_path(), value in arb_value()) {
        let mut s = BState::new();
        s.set(&path, value);
        prop_assert_eq!(s.get(&path), Some(value));
    }

    #[test]
    fn v_false_is_idempotent(path in arb_path()) {
        let mut once = BState::new();
        once.set(&path, V_FALSE);
        let mut twice = BState::new();
        twice.set(&path, V_FALSE);
        twice.set(&path, V_FALSE);
        let a: Vec<_> = { let mut v: Vec<String> = once.iter_keys().cloned().collect(); v.sort(); v };
        let b: Vec<_> = { let mut v: Vec<String> = twice.iter_keys().cloned().collect(); v.sort(); v };
        prop_assert_eq!(a, b);
    }

    #[test]
    fn absent_key_reads_none(path in arb_path()) {
        let s = BState::new();
        prop_assert_eq!(s.get(&path), None);
    }
}
