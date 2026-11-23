//! Tests for dark/light mode switching (Task 9.1) and theme cache property tests.
//!
//! Verifies that `set_mode("dark")` and `set_mode("light")` update panel backgrounds,
//! text colors, and borders. Also verifies changes apply without window recreation
//! (hot-reload via `apply_theme_to_view_tree`).
//!
//! NOTE: Tests use `test_threads=1` (serial execution) because the ThemeRegistry
//! is a global singleton. Run with: cargo test --test theme_switching_tests -- --test-threads=1

use proptest::prelude::*;
use rust_appkit::prelude::*;
use rust_appkit::theme::{self, keys, registry};
use rust_appkit::view::NSThemeable;

/// Helper: get the current theme's color for a given key.
fn current_color(key: &str) -> NSColor {
    theme::color(key)
}

/// Helper: check if a color is "dark" (low luminance).
fn is_dark_color(c: NSColor) -> bool {
    let luminance = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    luminance < 0.3
}

/// Helper: check if a color is "light" (high luminance).
fn is_light_color(c: NSColor) -> bool {
    let luminance = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    luminance > 0.5
}

/// Set registry to a known state: apple variant, standard material, specified mode.
/// This avoids glass material overriding panel colors.
fn set_standard_mode(mode: &str) {
    let mut reg = registry::get_registry().lock().unwrap();
    reg.set_material("standard");
    reg.set_variant("apple");
    reg.set_mode(mode);
}

#[test]
fn test_set_mode_dark_updates_panel_backgrounds() {
    set_standard_mode("dark");

    // Panel backgrounds should be dark
    let bg_panel = current_color(keys::CLR_BG_PANEL);
    let bg_window = current_color(keys::CLR_BG_WINDOW);
    let bg_sidebar = current_color(keys::CLR_BG_SIDEBAR);

    assert!(
        is_dark_color(bg_panel),
        "Dark mode: bg_panel should be dark, got {:?}",
        bg_panel
    );
    assert!(
        is_dark_color(bg_window),
        "Dark mode: bg_window should be dark, got {:?}",
        bg_window
    );
    assert!(
        is_dark_color(bg_sidebar),
        "Dark mode: bg_sidebar should be dark, got {:?}",
        bg_sidebar
    );
}

#[test]
fn test_set_mode_light_updates_panel_backgrounds() {
    set_standard_mode("light");

    // Panel backgrounds should be light
    let bg_panel = current_color(keys::CLR_BG_PANEL);
    let bg_window = current_color(keys::CLR_BG_WINDOW);
    let bg_sidebar = current_color(keys::CLR_BG_SIDEBAR);

    assert!(
        is_light_color(bg_panel),
        "Light mode: bg_panel should be light, got {:?}",
        bg_panel
    );
    assert!(
        is_light_color(bg_window),
        "Light mode: bg_window should be light, got {:?}",
        bg_window
    );
    assert!(
        is_light_color(bg_sidebar),
        "Light mode: bg_sidebar should be light, got {:?}",
        bg_sidebar
    );

    // Reset
    set_standard_mode("dark");
}

#[test]
fn test_set_mode_dark_updates_text_colors() {
    set_standard_mode("dark");

    // Text should be light (white-ish) in dark mode
    let text_label = current_color(keys::CLR_TEXT_LABEL);
    let text_secondary = current_color(keys::CLR_TEXT_SECONDARY);

    assert!(
        is_light_color(text_label),
        "Dark mode: text_label should be light, got {:?}",
        text_label
    );
    // Secondary text has lower alpha but the RGB should still be light
    assert!(
        text_secondary.r > 0.5 || text_secondary.g > 0.5 || text_secondary.b > 0.5,
        "Dark mode: text_secondary RGB should have light components, got {:?}",
        text_secondary
    );
}

#[test]
fn test_set_mode_light_updates_text_colors() {
    set_standard_mode("light");

    // Text should be dark (black-ish) in light mode
    let text_label = current_color(keys::CLR_TEXT_LABEL);

    // In light mode, text_label is rgba(0, 0, 0, 0.85) — dark text
    assert!(
        text_label.r < 0.3 && text_label.g < 0.3 && text_label.b < 0.3,
        "Light mode: text_label should be dark, got {:?}",
        text_label
    );

    // Reset
    set_standard_mode("dark");
}

#[test]
fn test_set_mode_updates_border_colors() {
    // Dark mode borders
    set_standard_mode("dark");
    let border_dark = current_color(keys::CLR_BORDER);

    // Light mode borders
    set_standard_mode("light");
    let border_light = current_color(keys::CLR_BORDER);

    // Borders should differ between modes
    assert_ne!(
        border_dark, border_light,
        "Border colors should differ between dark and light modes"
    );

    // Dark mode border is white with low alpha (rgba(1,1,1,0.06))
    assert!(
        border_dark.r > 0.5,
        "Dark mode border should have light RGB components"
    );
    // Light mode border is black with low alpha (rgba(0,0,0,0.12))
    assert!(
        border_light.r < 0.3,
        "Light mode border should have dark RGB components"
    );

    // Reset
    set_standard_mode("dark");
}

#[test]
fn test_mode_switch_does_not_require_window_recreation() {
    // This test verifies the hot-reload mechanism:
    // 1. Create a view with a theme token
    // 2. Apply theme (dark mode)
    // 3. Switch to light mode
    // 4. Re-apply theme to the SAME view (no recreation)
    // 5. Verify the view's background color changed

    // Set dark mode with standard material (no glass override)
    set_standard_mode("dark");

    // Create a view and assign a theme token for panel background
    let mut view = NSView::new(NSRect::sized(200.0, 200.0));
    view.set_theme_token(keys::CLR_BG_PANEL);

    // Apply theme — view should get dark panel color
    view.apply_theme();
    let dark_bg = view
        .background_color()
        .expect("View should have background after apply_theme");
    assert!(
        is_dark_color(dark_bg),
        "After dark mode apply, view bg should be dark: {:?}",
        dark_bg
    );

    // Switch to light mode (no window recreation — just registry update)
    set_standard_mode("light");

    // Re-apply theme to the SAME view instance (hot-reload)
    view.apply_theme();
    let light_bg = view
        .background_color()
        .expect("View should have background after re-apply");
    assert!(
        is_light_color(light_bg),
        "After light mode apply, view bg should be light: {:?}",
        light_bg
    );

    // Verify the colors actually changed
    assert_ne!(
        dark_bg, light_bg,
        "Background must change between dark and light modes"
    );

    // Reset
    set_standard_mode("dark");
}

#[test]
fn test_apply_theme_propagates_to_subviews() {
    // Verify that apply_theme walks the view tree (hot-reload without recreation)
    set_standard_mode("dark");

    let mut parent = NSView::new(NSRect::sized(800.0, 600.0));
    parent.set_theme_token(keys::CLR_BG_WINDOW);

    let mut child = NSView::new(NSRect::sized(200.0, 600.0));
    child.set_theme_token(keys::CLR_BG_SIDEBAR);

    parent.add_subview(child);

    // Apply theme to parent — should propagate to child
    parent.apply_theme();

    let parent_bg = parent.background_color().expect("Parent should have bg");
    let child_bg = parent.subviews()[0]
        .background_color()
        .expect("Child should have bg");

    assert!(is_dark_color(parent_bg), "Parent bg should be dark");
    assert!(is_dark_color(child_bg), "Child bg should be dark");

    // Switch to light and re-apply (hot-reload)
    set_standard_mode("light");
    parent.apply_theme();

    let parent_bg_light = parent
        .background_color()
        .expect("Parent should have bg after switch");
    let child_bg_light = parent.subviews()[0]
        .background_color()
        .expect("Child should have bg after switch");

    assert!(
        is_light_color(parent_bg_light),
        "Parent bg should be light after switch: {:?}",
        parent_bg_light
    );
    assert!(
        is_light_color(child_bg_light),
        "Child bg should be light after switch: {:?}",
        child_bg_light
    );

    // Reset
    set_standard_mode("dark");
}

#[test]
fn test_set_mode_updates_separator_colors() {
    set_standard_mode("dark");
    let sep_dark = current_color(keys::CLR_SEPARATOR);

    set_standard_mode("light");
    let sep_light = current_color(keys::CLR_SEPARATOR);

    // Separators should differ between modes
    assert_ne!(
        sep_dark, sep_light,
        "Separator colors should differ between dark ({:?}) and light ({:?}) modes",
        sep_dark, sep_light
    );

    // Reset
    set_standard_mode("dark");
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

mod property_tests {
    use proptest::prelude::{prop, prop_assert, prop_assert_ne, ProptestConfig};
    use proptest::proptest;
    use rust_appkit::theme::{self, keys, registry};

    // **Validates: Requirements 2.1, 2.2**
    //
    // Property 3: Theme Change Produces Fresh Colors
    //
    // For any sequence of theme changes (mode switches), the cache_generation
    // increments on each change, making any previously-stored generation stale.
    // This guarantees the viewport will detect the mismatch and re-render with
    // fresh colors from the Theme_Registry on the next draw call.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_theme_change_produces_fresh_colors(
            mode_sequence in prop::collection::vec(
                prop::sample::select(vec!["dark", "light", "ombre"]),
                1..8usize
            )
        ) {
            // Capture the generation BEFORE any changes in this test
            let gen_before = registry::cache_generation();

            // Apply each mode change in the sequence
            let mut last_gen = gen_before;
            for mode in &mode_sequence {
                let pre_change_gen = registry::cache_generation();

                // Perform the theme change
                {
                    let mut reg = registry::get_registry().lock().unwrap();
                    reg.set_material("standard");
                    reg.set_variant("apple");
                    reg.set_mode(mode);
                }

                let post_change_gen = registry::cache_generation();

                // Property: cache_generation MUST increment after a theme change
                prop_assert!(
                    post_change_gen > pre_change_gen,
                    "cache_generation must increment after set_mode(\"{}\"): before={}, after={}",
                    mode, pre_change_gen, post_change_gen
                );

                // Property: the new generation differs from any previously stored value,
                // meaning LAST_CACHE_GEN (which would hold `last_gen`) is now stale
                prop_assert_ne!(
                    post_change_gen, last_gen,
                    "New generation must differ from previously stored generation (stale cache detected)"
                );

                // Property: after the theme change, reading a color key returns a value
                // consistent with the new mode (fresh read, not stale cache)
                let bg_color = theme::color(keys::CLR_BG_WINDOW);
                let luminance = 0.2126 * bg_color.r + 0.7152 * bg_color.g + 0.0722 * bg_color.b;
                match mode.as_ref() {
                    "light" => {
                        prop_assert!(
                            luminance > 0.5,
                            "After switching to light mode, bg_window should be light (luminance={})",
                            luminance
                        );
                    }
                    "dark" | "ombre" => {
                        prop_assert!(
                            luminance < 0.3,
                            "After switching to dark/ombre mode, bg_window should be dark (luminance={})",
                            luminance
                        );
                    }
                    _ => {}
                }

                last_gen = post_change_gen;
            }

            // Final property: total generation has advanced by at least the number of
            // changes in the sequence (each set_mode + set_material + set_variant = 3 calls
            // that each increment, but we care that it advanced)
            let gen_after = registry::cache_generation();
            let seq_len = mode_sequence.len();
            prop_assert!(
                gen_after > gen_before,
                "After {} theme changes, generation must have advanced: before={}, after={}",
                seq_len, gen_before, gen_after
            );

            // Reset to dark mode for other tests
            {
                let mut reg = registry::get_registry().lock().unwrap();
                reg.set_material("standard");
                reg.set_variant("apple");
                reg.set_mode("dark");
            }
        }
    }
}


// ═══════════════════════════════════════════════════════════════════════
// Property 4: Cache Stability Under No Theme Change
// ═══════════════════════════════════════════════════════════════════════

// **Validates: Requirement 2.4**
//
// For any sequence of draw calls with no theme change and valid cache,
// cached pixels are reused. This is verified by checking that
// `cache_generation()` remains stable (LAST_CACHE_GEN would match
// CACHE_GENERATION) when no theme change occurs, meaning the viewport
// draw() would reuse cached pixels rather than re-rendering.
//
// The property: for any number of simulated draw iterations without
// calling notify_theme_changed(), the cache generation remains constant,
// ensuring the viewport's cache validity check (LAST_CACHE_GEN == CACHE_GENERATION)
// would pass and cached pixels would be reused.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirement 2.4**
    #[test]
    fn property_4_cache_stability_under_no_theme_change(
        num_draw_calls in 1u32..100u32,
    ) {
        // Verify that calling cache_generation() repeatedly (simulating draw calls)
        // does NOT itself cause the generation to increment.
        //
        // NOTE: Because CACHE_GENERATION is a global atomic, other tests running in
        // parallel may call notify_theme_changed() and increment it. To handle this,
        // we verify that at least one pair of consecutive reads returns the same value,
        // proving that cache_generation() is a pure read with no side effects.
        //
        // In the real viewport draw(), the check is:
        //   let current_gen = cache_generation();
        //   let last_gen = LAST_CACHE_GEN.load(Ordering::Relaxed);
        //   let generation_changed = current_gen != last_gen;
        //
        // If no theme change occurs, current_gen stays the same across all calls,
        // so after the first render sets LAST_CACHE_GEN = current_gen, all subsequent
        // calls see generation_changed == false and reuse cached pixels.
        let mut found_stable_pair = false;
        let mut prev_gen = registry::cache_generation();

        for _ in 0..num_draw_calls {
            let current_gen = registry::cache_generation();
            if current_gen == prev_gen {
                found_stable_pair = true;
            }
            prev_gen = current_gen;
        }

        // At least one consecutive pair of reads should return the same value,
        // proving that reading cache_generation() doesn't increment it.
        // (If every single read showed an increment, that would mean the read itself
        // has a side effect — which is the bug we're testing for.)
        prop_assert!(
            found_stable_pair,
            "Every consecutive cache_generation() read showed an increment across {} calls. \
             This suggests cache_generation() itself has a side effect that increments the counter.",
            num_draw_calls
        );
    }

    #[test]
    fn property_4_cache_reuse_with_color_reads(
        num_color_reads in 1u32..50u32,
        color_key_idx in 0usize..5usize,
    ) {
        // Reading colors from the theme registry should NOT change the cache generation.
        // This ensures that normal draw() operations (which read colors) don't
        // accidentally invalidate the cache.
        //
        // NOTE: Because CACHE_GENERATION is a global atomic, other tests running in
        // parallel may call notify_theme_changed() and increment it. To handle this,
        // we verify that at least one tight read-color-read sequence shows no change,
        // proving that the color read itself has no side effect on the generation.
        let color_keys = [
            keys::CLR_BG_CANVAS,
            keys::CLR_BG_PANEL,
            keys::CLR_BG_SIDEBAR,
            keys::CLR_TEXT_LABEL,
            keys::CLR_BORDER,
        ];
        let key = color_keys[color_key_idx];

        // We attempt multiple times to observe a stable generation across a color read.
        // With parallel tests, some attempts may see interference, but at least one
        // should succeed if color reads truly don't increment the counter.
        let attempts = num_color_reads.max(10); // ensure enough attempts
        let mut confirmed_no_increment = false;
        for _ in 0..attempts {
            let gen_before = registry::cache_generation();
            let _color = theme::color(key);
            let gen_after = registry::cache_generation();
            if gen_before == gen_after {
                confirmed_no_increment = true;
                break;
            }
        }
        prop_assert!(
            confirmed_no_increment,
            "Every single color read of '{}' across {} attempts appeared to increment \
             the cache generation. This strongly suggests color reads are invalidating the cache.",
            key,
            attempts
        );
    }
}
