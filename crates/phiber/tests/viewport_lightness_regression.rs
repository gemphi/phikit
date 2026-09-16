//! Regression for the lighter, flat semantic View3D viewport.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 13.6 (viewport reads as a lighter mid-gray)

use phiber::v2::{
    state::BState,
    theme::{BPaletteName, BSurfaceTone, resolve_ui_style, ui_style_defaults},
};

#[test]
fn semantic_viewport_is_flat_opaque_and_lighter_across_palettes() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(name), &BState::new());
        let base = style.colors.bg_viewport;
        let viewport = style.viewport_surface();
        let tone = style.surface(BSurfaceTone::Viewport);

        assert_eq!((viewport.0, viewport.1), (base.0, base.1), "{name:?}");
        assert_eq!(viewport.3, 1.0, "{name:?} viewport must stay opaque");
        assert_eq!(tone.top, viewport, "{name:?}");
        assert_eq!(tone.bottom, viewport, "{name:?}");
        assert!(!tone.enabled, "{name:?} viewport must remain flat");

        let expected = (base.2 + style.effects.viewport_lightness_delta * 100.0).min(100.0);
        assert!((viewport.2 - expected).abs() < 1.0e-5, "{name:?}");
        assert!(viewport.2 >= style.colors.bg_app.2 + 18.0, "{name:?}");
        assert!(viewport.2 <= style.colors.bg_app.2 + 55.0, "{name:?}");
        assert!((style.colors.text.2 - viewport.2).abs() >= 25.0, "{name:?}");
    }
}

#[test]
fn blender_dark_viewport_remains_a_readable_neutral_mid_gray() {
    let style = resolve_ui_style(ui_style_defaults(BPaletteName::BlenderDark), &BState::new());
    let viewport = style.viewport_surface();
    assert!((55.0..=65.0).contains(&viewport.2), "got L={}", viewport.2);
    assert!(viewport.1 <= 12.0, "got saturation {}", viewport.1);
}
