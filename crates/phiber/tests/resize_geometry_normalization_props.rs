//! Feature: phiber-resolved-ui-style-settings, Property 9
//! **Validates: Requirements 7.5, 13.6**

use phiber::v2::{
    state::{BState, key::ui_style as key},
    theme::{BPaletteName, BResizeStyle, resolve_ui_style, ui_style_defaults},
};
use proptest::{prelude::*, test_runner::TestCaseResult};

fn arb_valid_resize_overrides() -> impl Strategy<Value = BResizeStyle> {
    (0.0f32..=128.0).prop_flat_map(|visible_seam| {
        (
            Just(visible_seam),
            visible_seam..=256.0,
            0.0f32..=128.0,
            -256.0f32..=256.0,
            0.0f32..=512.0,
            0.0f32..=4096.0,
        )
            .prop_map(
                |(visible_seam, hit_width, gap, inset, snap_threshold, min_extent)| BResizeStyle {
                    visible_seam,
                    hit_width,
                    gap,
                    inset,
                    snap_threshold,
                    min_extent,
                },
            )
    })
}

fn arb_conflicting_resize_overrides() -> impl Strategy<Value = BResizeStyle> {
    (f32::MIN_POSITIVE..=128.0).prop_flat_map(|visible_seam| {
        (
            Just(visible_seam),
            0.0f32..visible_seam,
            0.0f32..=128.0,
            -256.0f32..=256.0,
            0.0f32..=512.0,
            0.0f32..=4096.0,
        )
            .prop_map(
                |(visible_seam, hit_width, gap, inset, snap_threshold, min_extent)| BResizeStyle {
                    visible_seam,
                    hit_width,
                    gap,
                    inset,
                    snap_threshold,
                    min_extent,
                },
            )
    })
}

fn resolve_resize(overrides: BResizeStyle) -> BResizeStyle {
    let mut state = BState::new();
    for (key, value) in [
        (&key::RESIZE_VISIBLE_SEAM, overrides.visible_seam),
        (&key::RESIZE_HIT_WIDTH, overrides.hit_width),
        (&key::RESIZE_GAP, overrides.gap),
        (&key::RESIZE_INSET, overrides.inset),
        (&key::RESIZE_SNAP_THRESHOLD, overrides.snap_threshold),
        (&key::RESIZE_MIN_EXTENT, overrides.min_extent),
    ] {
        state.set(key, &value.to_string());
    }

    resolve_ui_style(ui_style_defaults(BPaletteName::AppleDark), &state).resize
}

fn approximately_equal(left: f32, right: f32) -> bool {
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= f32::EPSILON * 8.0 * scale
}

fn assert_usable(requested: BResizeStyle) -> TestCaseResult {
    let resolved = resolve_resize(requested);

    for (name, value) in [
        ("visible_seam", resolved.visible_seam),
        ("hit_width", resolved.hit_width),
        ("gap", resolved.gap),
        ("inset", resolved.inset),
        ("snap_threshold", resolved.snap_threshold),
        ("min_extent", resolved.min_extent),
    ] {
        prop_assert!(value.is_finite(), "resolved resize {name} was not finite");
    }
    for (name, value) in [
        ("visible_seam", resolved.visible_seam),
        ("hit_width", resolved.hit_width),
        ("gap", resolved.gap),
        ("snap_threshold", resolved.snap_threshold),
        ("min_extent", resolved.min_extent),
    ] {
        prop_assert!(value >= 0.0, "resolved resize {name} was negative");
    }

    prop_assert_eq!(resolved.visible_seam, requested.visible_seam);
    prop_assert_eq!(resolved.gap, requested.gap);
    prop_assert_eq!(resolved.inset, requested.inset);
    prop_assert_eq!(resolved.snap_threshold, requested.snap_threshold);
    prop_assert_eq!(resolved.min_extent, requested.min_extent);
    prop_assert_eq!(
        resolved.hit_width,
        requested.hit_width.max(requested.visible_seam)
    );
    prop_assert!(resolved.hit_width >= resolved.visible_seam);

    // The overlay starts at the following sibling's edge plus the resolved
    // inset. Both resize renderers center the seam within this translated hit
    // band, so the two transparent margins must remain equal for every gap and
    // signed inset accepted by the descriptor policy. Derive those margins in
    // the overlay's local frame to avoid cancellation at large coordinates.
    let following_edge = resolved.gap;
    let overlay_start = following_edge + resolved.inset;
    let overlay_end = overlay_start + resolved.hit_width;
    let available_margin = resolved.hit_width - resolved.visible_seam;
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
        prop_assert!(valid.hit_width >= valid.visible_seam);
        prop_assert!(conflicting.hit_width < conflicting.visible_seam);
        assert_usable(valid)?;
        assert_usable(conflicting)?;
    }
}
