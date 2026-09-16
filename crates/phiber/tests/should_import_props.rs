//! Property 6.3 — should_import trigger gate.
//!
//! Feature: blend-file-mesh-loading, Property 6.3: should_import trigger gate
//! Validates: Requirements 1.2, 1.5, 1.6
//!
//! `should_import(prev, selected)` is `true` iff `selected` ends in `.blend`
//! (case-insensitive) AND differs as a raw string from `prev`.

use phiber::v2::app::import::should_import;
use phiber::v2::scene::is_blend_path;
use proptest::prelude::*;

/// Strategy: an arbitrary ASCII string, possibly empty.
fn arb_path() -> impl Strategy<Value = String> {
    proptest::string::string_regex("([a-zA-Z0-9/_.-]{0,32})?").expect("path regex valid")
}

/// Strategy: a string that ends with `.blend` in some case variant.
fn arb_blend_path() -> impl Strategy<Value = String> {
    (
        proptest::string::string_regex("[a-zA-Z0-9/_]{1,24}").expect("prefix regex valid"),
        prop_oneof![
            Just(".blend"),
            Just(".BLEND"),
            Just(".Blend"),
            Just(".bLend"),
            Just(".blEnd"),
        ],
    )
        .prop_map(|(prefix, suffix)| format!("{prefix}{suffix}"))
}

/// Strategy: a string that does NOT end with `.blend` (any case).
fn arb_non_blend_path() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zA-Z0-9/_.-]{0,32}").expect("non-blend regex valid")
        .prop_filter("must not end in .blend", |s| !is_blend_path(s))
}

proptest! {
    // Feature: blend-file-mesh-loading, Property 6.3: should_import trigger gate

    /// A new .blend path (differing from prev) always triggers import.
    #[test]
    fn new_blend_triggers_import(
        prev in arb_path(),
        selected in arb_blend_path(),
    ) {
        // Only trigger when the paths differ (raw string comparison).
        prop_assume!(prev != selected);
        let result = should_import(Some(prev.as_str()), Some(selected.as_str()));
        prop_assert!(result, "expected true for new blend path {selected:?} (prev={prev:?})");
    }

    /// The same .blend path as prev never triggers import (idempotent).
    #[test]
    fn same_blend_as_prev_does_not_trigger(path in arb_blend_path()) {
        let result = should_import(Some(path.as_str()), Some(path.as_str()));
        prop_assert!(!result, "expected false for same path {path:?}");
    }

    /// A non-.blend selection never triggers import, regardless of prev.
    #[test]
    fn non_blend_never_triggers(
        prev in prop::option::of(arb_blend_path()),
        selected in arb_non_blend_path(),
    ) {
        let prev_ref = prev.as_deref();
        let result = should_import(prev_ref, Some(selected.as_str()));
        prop_assert!(!result, "non-blend {selected:?} should never trigger import");
    }

    /// None selected never triggers import.
    #[test]
    fn none_selected_never_triggers(prev in prop::option::of(arb_path())) {
        let result = should_import(prev.as_deref(), None);
        prop_assert!(!result, "None selected should never trigger import");
    }

    /// Logical definition: result == (is_blend_path(sel) && prev != Some(sel)).
    #[test]
    fn matches_logical_definition(
        prev in prop::option::of(arb_path()),
        selected in prop::option::of(arb_path()),
    ) {
        let expected = match &selected {
            Some(sel) => is_blend_path(sel) && prev.as_deref() != Some(sel.as_str()),
            None => false,
        };
        let actual = should_import(prev.as_deref(), selected.as_deref());
        prop_assert_eq!(actual, expected, "{}", format!(
            "logical definition mismatch: prev={prev:?}, selected={selected:?}"
        ));
    }
}

// --- Deterministic unit cases ---

#[test]
fn none_none_is_false() {
    assert!(!should_import(None, None));
}

#[test]
fn none_prev_with_blend_is_true() {
    assert!(should_import(None, Some("/first/load.blend")));
}

#[test]
fn same_blend_path_is_false() {
    assert!(!should_import(
        Some("/dir/model.blend"),
        Some("/dir/model.blend")
    ));
}

#[test]
fn different_blend_path_is_true() {
    assert!(should_import(
        Some("/old/scene.blend"),
        Some("/new/model.blend")
    ));
}

#[test]
fn non_blend_extension_is_false() {
    assert!(!should_import(None, Some("notes.txt")));
    assert!(!should_import(None, Some("readme.md")));
    assert!(!should_import(None, Some("")));
}

#[test]
fn case_insensitive_blend_suffix() {
    assert!(should_import(None, Some("MODEL.BLEND")));
    assert!(should_import(None, Some("scene.Blend")));
    assert!(should_import(None, Some("mesh.bLeNd")));
}

#[test]
fn blend_exactly_the_word_without_path_is_not_blend() {
    // "blend" alone has no dot suffix -> false.
    assert!(!should_import(None, Some("blend")));
}
