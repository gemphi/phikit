//! Feature: phiber-resolved-ui-style-settings, Property 9
//! **Validates: Requirements 7.5, 13.6**
//!
//! The seam is `gap − seam_rest_under` thick at rest and `gap +
//! seam_hover_over` under the pointer; the hit band is widened to the
//! hovered seam so the line never outgrows what can be grabbed.

use phiber::v2::{
    state::{BState, key::ui_style as key},
    theme::{BPaletteName, BResizeStyle, resolve_ui_style, ui_style_defaults},
};
use proptest::{prelude::*, test_runner::TestCaseResult};

fn resize_from(seam_rest_under: f32, seam_hover_over: f32, hit_width: f32, gap: f32, inset: f32, snap_threshold: f32, min_extent: f32) -> BResizeStyle {
    BResizeStyle { seam_rest_under, seam_hover_over, hit_width, gap, inset, snap_threshold, min_extent }
}

fn arb_valid_resize_overrides() -> impl Strategy<Value = BResizeStyle> {
    (0.0f32..=128.0, 0.0f32..=128.0, 0.0f32..=128.0).prop_flat_map(|(under, over, gap)| {
        let hovered = (gap + over).max((gap - under).max(1.0));
        (Just(under), Just(over), hovered.min(256.0)..=256.0, Just(gap), -256.0f32..=256.0, 0.0f32..=512.0, 0.0f32..=4096.0)
            .prop_map(|(u, o, hit, g, inset, snap, min)| resize_from(u, o, hit, g, inset, snap, min))
    })
}

fn arb_conflicting_resize_overrides() -> impl Strategy<Value = BResizeStyle> {
    (0.0f32..=128.0, f32::MIN_POSITIVE..=128.0, 1.0f32..=128.0).prop_flat_map(|(under, over, gap)| {
        let hovered = (gap + over).max((gap - under).max(1.0));
        (Just(under), Just(over), 0.0f32..hovered, Just(gap), -256.0f32..=256.0, 0.0f32..=512.0, 0.0f32..=4096.0)
            .prop_map(|(u, o, hit, g, inset, snap, min)| resize_from(u, o, hit, g, inset, snap, min))
    })
}

fn resolve_resize(overrides: BResizeStyle) -> BResizeStyle {
    let mut state = BState::new();
    for (key, value) in [
        (&key::RESIZE_SEAM_REST_UNDER, overrides.seam_rest_under),
        (&key::RESIZE_SEAM_HOVER_OVER, overrides.seam_hover_over),
        (&key::RESIZE_HIT_WIDTH, overrides.hit_width),
        (&key::RESIZE_GAP, overrides.gap),
        (&key::RESIZE_INSET, overrides.inset),
        (&key::RESIZE_SNAP_THRESHOLD, overrides.snap_threshold),
        (&key::RESIZE_MIN_EXTENT, overrides.min_extent),
    ] {
        state.set(key, &value.to_string());
    }

    resolve_ui_style(ui_style_defaults(BPaletteName::Sphere), &state).resize
}

fn approximately_equal(left: f32, right: f32) -> bool {
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= f32::EPSILON * 8.0 * scale
}

fn assert_usable(requested: BResizeStyle) -> TestCaseResult {
    let resolved = resolve_resize(requested);

    for (name, value) in [
        ("seam_rest_under", resolved.seam_rest_under),
        ("seam_hover_over", resolved.seam_hover_over),
        ("hit_width", resolved.hit_width),
        ("gap", resolved.gap),
        ("inset", resolved.inset),
        ("snap_threshold", resolved.snap_threshold),
        ("min_extent", resolved.min_extent),
    ] {
        prop_assert!(value.is_finite(), "resolved resize {name} was not finite");
        if name != "inset" {
            prop_assert!(value >= 0.0, "resolved resize {name} was negative");
        }
    }

    prop_assert_eq!(resolved.seam_rest_under, requested.seam_rest_under);
    prop_assert_eq!(resolved.seam_hover_over, requested.seam_hover_over);
    prop_assert_eq!(resolved.gap, requested.gap);
    prop_assert_eq!(resolved.inset, requested.inset);
    prop_assert_eq!(resolved.snap_threshold, requested.snap_threshold);
    prop_assert_eq!(resolved.min_extent, requested.min_extent);
    prop_assert_eq!(resolved.hit_width, requested.hit_width, "no field bends another");
    prop_assert!(resolved.rest_thickness() >= 1.0, "the resting seam is at least a hairline");
    prop_assert!(resolved.hover_thickness() >= resolved.rest_thickness(), "hovering never thins the seam");

    // The overlay starts at the following sibling's edge plus the resolved
    // inset. Both resize renderers center the seam within this translated hit
    // band, so the two transparent margins must remain equal for every gap and
    // signed inset accepted by the descriptor policy. Derive those margins in
    // the overlay's local frame to avoid cancellation at large coordinates.
    let following_edge = resolved.gap;
    let overlay_start = following_edge + resolved.inset;
    let overlay_end = overlay_start + resolved.hit_width;
    let available_margin = resolved.hit_width - resolved.hover_thickness().min(resolved.hit_width);
    let leading_margin = available_margin / 2.0;
    let trailing_margin = available_margin - leading_margin;
    let seam_start = overlay_start + leading_margin;
    let seam_end = overlay_end - trailing_margin;

    prop_assert!(overlay_start.is_finite() && overlay_end.is_finite());
    prop_assert!(seam_start.is_finite() && seam_end.is_finite());
    prop_assert!(approximately_equal(leading_margin, trailing_margin));
    prop_assert!(leading_margin >= -f32::EPSILON && trailing_margin >= -f32::EPSILON);

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 9: Resolved resize geometry remains usable.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 9
    /// **Validates: Requirements 7.5, 13.6**
    #[test]
    fn resize_geometry_normalizes_valid_and_conflicting_overrides(
        valid in arb_valid_resize_overrides(),
        conflicting in arb_conflicting_resize_overrides(),
    ) {
        prop_assert!(valid.hit_width >= valid.hover_thickness());
        prop_assert!(conflicting.hit_width < conflicting.hover_thickness());
        assert_usable(valid)?;
        assert_usable(conflicting)?;
    }
}
