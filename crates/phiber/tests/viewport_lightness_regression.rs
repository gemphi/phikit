//! Regression for the flat semantic View3D viewport.
//!
//! The canvas is the palette's own viewport tone, with no lift applied: it
//! has to stay dark enough for the floor grid and the axes to read against
//! it. It still has to sit clearly above the app chrome, and clearly below
//! the text drawn over it.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 13.6

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
        // The canvas sits clearly inward of the window shade: lighter in a
        // dark theme, darker in a light one.
        let lift = if style.colors.ladder.light { style.colors.bg_app.2 - viewport.2 } else { viewport.2 - style.colors.bg_app.2 };
        assert!((12.0..=55.0).contains(&lift), "{name:?}: lift={lift}");
        assert!((style.colors.text.2 - viewport.2).abs() >= 25.0, "{name:?}");
    }
}

#[test]
fn blender_dark_viewport_is_blenders_neutral_grey_seed() {
    let style = resolve_ui_style(ui_style_defaults(BPaletteName::Graphite), &BState::new());
    let viewport = style.viewport_surface();
    // `#3D3D3D`, Blender's viewport grey, is the seed and so the canvas shade.
    assert!((viewport.2 - 23.9).abs() < 0.2, "got L={}", viewport.2);
    assert_eq!(viewport.1, 0.0, "got saturation {}", viewport.1);
    assert_eq!(style.colors.ladder.step_of(viewport), Some(10));
}

#[test]
fn the_grid_carries_enough_alpha_to_read_on_the_canvas() {
    // The grid is drawn on a canvas that is a near neighbour of it in tone,
    // so the alpha is what separates them. Lifting the canvas instead washed
    // the grid out, which is the opposite of what it was meant to fix.
    for name in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(name), &BState::new());
        assert!(
            style.effects.viewport_grid_alpha >= 0.14,
            "{name:?} grid alpha {} is too faint to read",
            style.effects.viewport_grid_alpha
        );
    }
}
