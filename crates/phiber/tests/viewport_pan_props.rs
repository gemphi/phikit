//! Property 19 — Viewport pan composition (task 25.1).
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 13.9, 26.3 (viewport pan additivity)
//!
//! For any two pan deltas (dx1, dy1) and (dx2, dy2), applying them
//! sequentially yields the same pan position as applying their sum.

use phiber::v2::state::BState;
use proptest::prelude::*;

/// Encoded state keys for viewport pan (mirrors design.md's state table).
fn k_pan_x(host: &str) -> String {
    format!("ctx.{host}.pan.x")
}
fn k_pan_y(host: &str) -> String {
    format!("ctx.{host}.pan.y")
}

/// Apply a pan delta by reading current, adding, and writing back.
fn apply_pan(state: &mut BState, host: &str, dx: f32, dy: f32) {
    let x_key = k_pan_x(host);
    let y_key = k_pan_y(host);
    let x = state
        .get_encoded(&x_key)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y = state
        .get_encoded(&y_key)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    state.set_encoded(&x_key, format!("{}", x + dx));
    state.set_encoded(&y_key, format!("{}", y + dy));
}

fn read_pan(state: &BState, host: &str) -> (f32, f32) {
    let x = state
        .get_encoded(&k_pan_x(host))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let y = state
        .get_encoded(&k_pan_y(host))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    (x, y)
}

proptest! {
    #[test]
    fn pan_is_additive(
        dx1 in -1000.0f32..1000.0f32,
        dy1 in -1000.0f32..1000.0f32,
        dx2 in -1000.0f32..1000.0f32,
        dy2 in -1000.0f32..1000.0f32,
    ) {
        // Sequential: apply(dx1,dy1); apply(dx2,dy2)
        let mut seq = BState::new();
        apply_pan(&mut seq, "view3d", dx1, dy1);
        apply_pan(&mut seq, "view3d", dx2, dy2);
        let (sx, sy) = read_pan(&seq, "view3d");

        // Combined: apply(dx1+dx2, dy1+dy2)
        let mut combined = BState::new();
        apply_pan(&mut combined, "view3d", dx1 + dx2, dy1 + dy2);
        let (cx, cy) = read_pan(&combined, "view3d");

        prop_assert!((sx - cx).abs() < 1e-2, "x drift: {sx} vs {cx}");
        prop_assert!((sy - cy).abs() < 1e-2, "y drift: {sy} vs {cy}");
    }

    #[test]
    fn pan_zero_is_identity(dx in -500.0f32..500.0f32, dy in -500.0f32..500.0f32) {
        let mut state = BState::new();
        apply_pan(&mut state, "view3d", dx, dy);
        let (before_x, before_y) = read_pan(&state, "view3d");
        apply_pan(&mut state, "view3d", 0.0, 0.0);
        let (after_x, after_y) = read_pan(&state, "view3d");
        prop_assert!((before_x - after_x).abs() < 1e-6);
        prop_assert!((before_y - after_y).abs() < 1e-6);
    }
}

// ── Task 8.4 — Minimum extents and overlay hits post-decomposition ───────────
//
// Feature: phiber-resolved-ui-style-settings
// Validates: Requirements 13.9, 26.3, 54.6, 54.8

use phiber::v2::theme::{BPaletteName, resolve_ui_style, ui_style_defaults};

/// After renderer decomposition, the resolved style's hit_width is always
/// larger than the visible seam — the interactive band is wider than the
/// painted line.
#[test]
fn hit_width_exceeds_the_hovered_seam_across_all_palettes() {
    for &palette in BPaletteName::ALL {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        assert!(
            style.resize.hit_width > style.resize.hover_thickness(),
            "palette {palette:?}: hit_width ({}) must exceed the hovered seam ({})",
            style.resize.hit_width,
            style.resize.hover_thickness(),
        );
    }
}

/// The minimum extent from the resolved style is never larger than half the
/// minimum meaningful viewport size (300 px) — so two areas can always fit.
#[test]
fn min_extent_fits_two_areas_in_minimum_viewport() {
    const MINIMUM_VIEWPORT: f32 = 300.0;
    for &palette in BPaletteName::ALL {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        assert!(
            style.resize.min_extent <= MINIMUM_VIEWPORT / 2.0,
            "palette {palette:?}: min_extent ({}) would prevent two areas in a {}px viewport",
            style.resize.min_extent,
            MINIMUM_VIEWPORT,
        );
    }
}

// Pan state is orthogonal to resize state — applying a pan delta must not
// touch any area-resize keys, and vice versa.
proptest! {
    #[test]
    fn pan_does_not_contaminate_resize_state(
        dx in -500.0f32..500.0f32,
        dy in -500.0f32..500.0f32,
    ) {
        let mut state = BState::new();
        // Record all keys before pan.
        let before_keys: std::collections::HashSet<String> =
            state.iter_keys().cloned().collect();

        apply_pan(&mut state, "view3d", dx, dy);

        // No area-resize keys should have been touched.
        for key in state.iter_keys() {
            if key.starts_with("ctx.area_resize") || key.starts_with("ui.space") {
                prop_assert!(
                    before_keys.contains(key),
                    "pan operation unexpectedly wrote resize key `{key}`"
                );
            }
        }
    }
}
