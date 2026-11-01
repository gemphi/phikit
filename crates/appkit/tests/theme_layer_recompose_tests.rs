//! Property test for theme layer recompose.
//!
//! Feature: macos-visual-polish, Property 6: Theme layer selection triggers recompose with correct values
//! **Validates: Requirements 9.4, 15.1, 15.2, 15.3**
//!
//! For any valid (variant, mode, material) triple from the known sets, after calling
//! the corresponding `set_variant`/`set_mode`/`set_material` on `ThemeRegistry`, the
//! current theme's color and geometry maps SHALL contain the values defined by that
//! layer's `apply_*` function.
//!
//! NOTE: `ThemeRegistry` is a global singleton. Tests serialize through the lock,
//! and we restore the registry to a known state at the end of each property case.
//! Run with: cargo test --test theme_layer_recompose_tests -- --test-threads=1

use proptest::prelude::*;
use rust_appkit::theme::{keys, registry};

// The valid layer values come from the theme keys module.
// Variant layer (a.k.a. "brand" in the design doc / "Scheme" in the appearance menu).
const VARIANTS: &[&str] = &[
    keys::variant::APPLE,
    keys::variant::WINDOWS,
    keys::variant::TESLA,
    keys::variant::SWIFT,
    keys::variant::BARBIE,
    keys::variant::FENTY,
    keys::variant::CYBERPUNK,
    keys::variant::CHANEL,
];

const MODES: &[&str] = &[keys::mode::DARK, keys::mode::LIGHT, keys::mode::OMBRE];

const MATERIALS: &[&str] = &[
    keys::material::STANDARD,
    keys::material::GLASS,
    keys::material::METAL,
    keys::material::ONYX,
    keys::material::LACE,
    keys::material::WETLAND,
    keys::material::PLATINUM,
    keys::material::HOLOGRAPHIC,
];

/// Apply a (variant, mode, material) triple to the global registry and return the
/// resolved current theme tokens we want to inspect.
fn apply_triple_and_snapshot(
    variant: &str,
    mode: &str,
    material: &str,
) -> (
    String, // current_variant
    String, // current_mode
    String, // current_material
    rust_appkit::NSColor, // CLR_BG_WINDOW
    f32,    // RADIUS_PANEL
    f32,    // RADIUS_BUTTON
) {
    let mut reg = registry::get_registry().lock().unwrap();
    // The order matches the design's data flow: mode → variant → material layers.
    reg.set_mode(mode);
    reg.set_variant(variant);
    reg.set_material(material);

    let current = reg.current();
    let bg_window = current
        .colors
        .get(keys::CLR_BG_WINDOW)
        .copied()
        .expect("CLR_BG_WINDOW must be present after recompose");
    let radius_panel = current
        .layout
        .geometry
        .get(keys::RADIUS_PANEL)
        .copied()
        .expect("RADIUS_PANEL must be present after recompose");
    let radius_button = current
        .layout
        .geometry
        .get(keys::RADIUS_BUTTON)
        .copied()
        .expect("RADIUS_BUTTON must be present after recompose");

    (
        reg.current_variant().to_string(),
        reg.current_mode().to_string(),
        reg.current_material().to_string(),
        bg_window,
        radius_panel,
        radius_button,
    )
}

/// Compute the expected resolved theme by composing the same layers off-registry.
/// This mirrors what `recompose` does internally and gives us a ground-truth
/// reference to compare the registry's `current()` against.
fn compose_reference(
    variant: &str,
    mode: &str,
    material: &str,
) -> (rust_appkit::NSColor, f32, f32) {
    let reg = registry::get_registry().lock().unwrap();
    let theme = reg.compose(variant, mode, material);
    let bg_window = theme
        .colors
        .get(keys::CLR_BG_WINDOW)
        .copied()
        .expect("reference theme should have CLR_BG_WINDOW");
    let radius_panel = theme
        .layout
        .geometry
        .get(keys::RADIUS_PANEL)
        .copied()
        .expect("reference theme should have RADIUS_PANEL");
    let radius_button = theme
        .layout
        .geometry
        .get(keys::RADIUS_BUTTON)
        .copied()
        .expect("reference theme should have RADIUS_BUTTON");
    (bg_window, radius_panel, radius_button)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// **Validates: Requirements 9.4, 15.1, 15.2, 15.3**
    ///
    /// Property 6: Theme layer selection triggers recompose with correct values.
    ///
    /// For every (variant, mode, material) triple drawn from the known valid sets,
    /// after calling the corresponding setters on `ThemeRegistry`:
    ///   - The registry reports the new layer selections via `current_*()`.
    ///   - The current theme's color map contains the value the layers prescribe
    ///     (specifically `CLR_BG_WINDOW`, which mode and material both write to).
    ///   - The current theme's geometry map contains the value the material layer
    ///     prescribes (specifically `RADIUS_PANEL` and `RADIUS_BUTTON`).
    /// We compare against an independently composed reference theme so the assertion
    /// is layer-aware: it follows the precedence base → mode → variant → material
    /// without hardcoding values for every triple.
    #[test]
    fn prop_theme_layer_recompose_yields_expected_values(
        variant_idx in 0usize..VARIANTS.len(),
        mode_idx in 0usize..MODES.len(),
        material_idx in 0usize..MATERIALS.len(),
    ) {
        let variant = VARIANTS[variant_idx];
        let mode = MODES[mode_idx];
        let material = MATERIALS[material_idx];

        let (
            current_variant,
            current_mode,
            current_material,
            bg_window,
            radius_panel,
            radius_button,
        ) = apply_triple_and_snapshot(variant, mode, material);

        // 1. The registry's reported state matches the inputs.
        prop_assert_eq!(
            &current_variant, variant,
            "current_variant should equal the variant we set"
        );
        prop_assert_eq!(
            &current_mode, mode,
            "current_mode should equal the mode we set"
        );
        prop_assert_eq!(
            &current_material, material,
            "current_material should equal the material we set"
        );

        // 2. The current theme's tokens equal the values an independent compose
        //    of the same triple produces. This proves recompose actually applied
        //    the layers (not just stored their names).
        let (ref_bg, ref_radius_panel, ref_radius_button) = compose_reference(variant, mode, material);

        prop_assert_eq!(
            bg_window, ref_bg,
            "CLR_BG_WINDOW after recompose must equal the value composed from \
             variant={} mode={} material={}",
            variant, mode, material
        );
        prop_assert!(
            (radius_panel - ref_radius_panel).abs() < f32::EPSILON,
            "RADIUS_PANEL after recompose ({}) must equal reference ({}) for \
             variant={} mode={} material={}",
            radius_panel, ref_radius_panel, variant, mode, material
        );
        prop_assert!(
            (radius_button - ref_radius_button).abs() < f32::EPSILON,
            "RADIUS_BUTTON after recompose ({}) must equal reference ({}) for \
             variant={} mode={} material={}",
            radius_button, ref_radius_button, variant, mode, material
        );

        // Reset to a known-good baseline so other tests in the suite see a
        // predictable starting state (apple.dark.standard).
        {
            let mut reg = registry::get_registry().lock().unwrap();
            reg.set_mode(keys::mode::DARK);
            reg.set_variant(keys::variant::APPLE);
            reg.set_material(keys::material::STANDARD);
        }
    }
}
