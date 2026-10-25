//! Regression for the flat semantic View3D viewport.
//!
//! The canvas is the palette's own viewport tone, with no lift applied: it
//! has to stay far enough from the floor grid and the axes for them to read
//! against it. It still has to sit clearly above the app chrome, and clearly
//! below the text drawn over it.
//!
//! "Far enough", not "dark enough": every assertion here is about separation,
//! and each holds with a ladder read from white as well as from black. The
//! mode the app opens in is Light now (`BThemeMode::DEFAULT`); these tests
//! read the dark ladder by name (`ui_style_defaults`) because the dark values
//! are the ones with a Blender number to check against.
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

#[test]
fn the_sweep_lift_is_never_painted_as_an_opacity() {
    // `BView3D::render` packs the sweep lift into the alpha of the canvas
    // colour, because that is what the tracer's kernel reads `Camera::sky.a`
    // as: how much lighter the left edge runs (`wgsl/util.wgsl`, `sky_at`).
    // It is a lightness delta, not an opacity — so the value carrying it may
    // only ever reach the tracer.
    //
    // It used to reach two more places. The camera frame's `TH_BACK` outline
    // and the line under the box / circle / lasso were painted with it, and
    // painted means the alpha IS an opacity: about 0.07 in a theme with a
    // sweep, and exactly 0.0 in every flat one — Graphite, Silver, Apple,
    // Ember, Cyberpunk, where `gradients: false` zeroes the lift.
    // Those two lines were invisible in six of eleven themes, and choosing a
    // flat theme is exactly when a camera frame still has to be there.
    let src = include_str!("../src/v2/view/draw/canvas/view3d/mod.rs");
    let lift = "vp.a = ctx.style.viewport_sweep_lift();";
    assert!(src.contains(lift), "the tracer's sky still carries the lift");
    assert_eq!(src.matches(lift).count(), 1, "in one place");
    // The two paint sites ask for the surface itself, which `viewport_surface`
    // hands over opaque, rather than reusing the tracer's value.
    assert!(src.contains("back: to_hsla(ctx.style.viewport_surface())"), "the camera frame paints the opaque shade");
    assert!(
        src.contains("BGestureInk::now(ctx.state, to_hsla(ctx.style.viewport_surface())"),
        "and so does the gesture ink"
    );
    assert!(!src.contains("back: vp"), "neither takes the tracer's sky");
    assert!(!src.contains("BGestureInk::now(ctx.state, vp"), "neither takes the tracer's sky");
    // And the thing that makes this matter: a flat theme's lift is zero, so
    // an opacity taken from it is full transparency.
    for name in [BPaletteName::Apple, BPaletteName::Graphite] {
        let style = resolve_ui_style(ui_style_defaults(name), &BState::new());
        assert_eq!(style.viewport_sweep_lift(), 0.0, "{name:?} is flat");
        assert_eq!(style.viewport_surface().3, 1.0, "but its canvas shade is opaque");
    }
}

#[test]
fn a_final_render_stands_in_the_same_light_as_the_viewport() {
    // F12 built its `BRayLook` from two fixed greys, `hsla(0, 0, 0.14)` for
    // the sky and `hsla(0, 0, 0.3)` for the grid, so a render came back on a
    // near-black ground under every palette while the viewport beside it
    // followed the theme. Same two tokens now, and through `resolve_ui_style`
    // so a colour edited in Preferences reaches a render too.
    let src = include_str!("../src/v2/app/op/render.rs");
    assert!(src.contains("style.viewport_surface()"), "the render's sky is the canvas shade");
    assert!(src.contains("style.viewport_grid_stroke()"), "and its grid is the grid stroke");
    assert!(src.contains("resolve_ui_style"), "with the overrides on it");
    assert!(!src.contains("hsla(0.0, 0.0, 0.14"), "no fixed sky");
    assert!(!src.contains("hsla(0.0, 0.0, 0.3,"), "no fixed grid");
    // The one fixed colour left there is Blender's: a Principled default base
    // of 0.8 grey, for a render with no material to read.
    assert!(src.contains("hsla(0.0, 0.0, 0.8"), "Blender's default base colour stays");
}
