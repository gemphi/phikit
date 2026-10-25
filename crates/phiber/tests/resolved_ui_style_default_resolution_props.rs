//! Generated coverage for total default UI-style resolution.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 1
//! **Validates: Requirements 1.7, 2.2, 2.7, 12.1, 13.1**

use phiber::v2::{
    state::BState,
    theme::{
        BPaletteName, BRadiusScale, BResolvedUiStyle, BShadowStyle, BUiStyleDefaults,
        UI_STYLE_DESCRIPTORS, resolve_ui_style, ui_style_defaults,
    },
};
use proptest::prelude::*;

fn arb_valid_defaults() -> impl Strategy<Value = BUiStyleDefaults> {
    (
        0usize..BPaletteName::ALL.len(),
        (
            0.0f32..8.0,
            0.125f32..8.0,
            0.125f32..8.0,
            0.125f32..8.0,
            0.125f32..8.0,
        ),
        0.75f32..1.25,
        0.5f32..1.5,
        0.5f32..1.5,
        (0.5f32..2.0, 0.5f32..2.0, 0.5f32..2.0),
        (
            0.5f32..16.0,
            0.0f32..16.0,
            0.0f32..16.0,
            -48.0f32..0.0,
            0.0f32..64.0,
            32.0f32..256.0,
        ),
        0.5f32..1.5,
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                palette_index,
                (xs, sm_step, md_step, lg_step, xl_step),
                typography_scale,
                spacing_scale,
                dimension_scale,
                (hairline, border, grid),
                (seam_over, hit_extra, gap, inset, snap_threshold, min_extent),
                effect_scale,
                gradients,
                shadow,
            )| {
                let mut defaults = *ui_style_defaults(BPaletteName::ALL[palette_index]);
                let sm = xs + sm_step;
                let md = sm + md_step;
                let lg = md + lg_step;
                defaults.radii = BRadiusScale {
                    xs,
                    sm,
                    md,
                    lg,
                    xl: lg + xl_step,
                };

                defaults.typography.body *= typography_scale;
                defaults.typography.header *= typography_scale;
                defaults.typography.caption *= typography_scale;

                defaults.spacing.gap_xs *= spacing_scale;
                defaults.spacing.gap_sm *= spacing_scale;
                defaults.spacing.gap_md *= spacing_scale;
                defaults.spacing.gap_lg *= spacing_scale;
                defaults.spacing.pad_xs *= spacing_scale;
                defaults.spacing.pad_sm *= spacing_scale;
                defaults.spacing.pad_md *= spacing_scale;
                defaults.spacing.pad_lg *= spacing_scale;

                defaults.dimensions.header_height *= dimension_scale;
                defaults.dimensions.footer_height *= dimension_scale;
                defaults.dimensions.row_height *= dimension_scale;
                defaults.dimensions.button_height *= dimension_scale;
                defaults.dimensions.scrubbing_height *= dimension_scale;
                defaults.dimensions.asset_shelf_height *= dimension_scale;
                defaults.dimensions.bottom_strip_height *= dimension_scale;
                defaults.dimensions.bottom_panel_height *= dimension_scale;
                defaults.dimensions.menu_width *= dimension_scale;
                defaults.dimensions.status_log_height *= dimension_scale;
                defaults.dimensions.banner_height *= dimension_scale;
                defaults.dimensions.activity_width *= dimension_scale;
                defaults.dimensions.toolbar_width *= dimension_scale;
                defaults.dimensions.sidebar_width *= dimension_scale;
                defaults.dimensions.sidebar_min_width *= dimension_scale;
                defaults.dimensions.panel_width *= dimension_scale;
                defaults.dimensions.navbar_width *= dimension_scale;
                defaults.dimensions.channels_width *= dimension_scale;
                defaults.dimensions.tool_props_width *= dimension_scale;
                defaults.dimensions.area_left_width *= dimension_scale;
                defaults.dimensions.area_right_width *= dimension_scale;
                defaults.dimensions.file_date_column_width *= dimension_scale;
                defaults.dimensions.file_size_column_width *= dimension_scale;
                defaults.dimensions.indent_unit *= dimension_scale;
                defaults.dimensions.restriction_column_width *= dimension_scale;
                defaults.dimensions.gizmo_button_size *= dimension_scale;
                defaults.dimensions.gizmo_icon_size *= dimension_scale;
                defaults.dimensions.object_icon_size *= dimension_scale;
                defaults.dimensions.object_glyph_size *= dimension_scale;

                defaults.strokes.hairline = hairline;
                defaults.strokes.border = border;
                defaults.strokes.grid = grid;
                defaults.resize.seam_hover_over = seam_over;
                defaults.resize.gap = gap;
                defaults.resize.hit_width = defaults.resize.hover_thickness() + hit_extra;
                defaults.resize.inset = inset;
                defaults.resize.snap_threshold = snap_threshold;
                defaults.resize.min_extent = min_extent;

                defaults.effects.gradients = gradients;
                defaults.effects.gradient_lift *= effect_scale;
                defaults.effects.sheen_lift *= effect_scale;
                defaults.effects.selected_hover_lift *= effect_scale;
                defaults.effects.footer_lift *= effect_scale;
                defaults.effects.sidebar_lift *= effect_scale;
                defaults.effects.card_lift *= effect_scale;
                defaults.effects.section_lift *= effect_scale;
                defaults.effects.strip_recess *= effect_scale;
                defaults.effects.strip_gradient_lift *= effect_scale;
                defaults.effects.tab_accent_mix *= effect_scale;
                defaults.effects.tab_saturation_lift *= effect_scale;
                defaults.effects.tab_top_lift *= effect_scale;
                defaults.effects.viewport_lightness_delta *= effect_scale;
                defaults.effects.viewport_grid_alpha *= effect_scale;
                defaults.effects.axis_alpha *= effect_scale;
                defaults.effects.node_grid_lift *= effect_scale;
                defaults.effects.node_grid_alpha *= effect_scale;
                defaults.effects.image_light_lift *= effect_scale;
                defaults.effects.image_dark_lift *= effect_scale;
                defaults.effects.surface_shadow = if shadow {
                    BShadowStyle::ExtraLarge
                } else {
                    BShadowStyle::None
                };
                defaults
            },
        )
}

fn arb_irrelevant_state_entries() -> impl Strategy<Value = Vec<(String, String)>> {
    let suffix = proptest::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .expect("irrelevant state-key regex is valid");
    let value = proptest::string::string_regex("[ -~]{0,32}")
        .expect("irrelevant state-value regex is valid");

    prop_oneof![
        3 => Just(Vec::new()),
        7 => prop::collection::vec((suffix, value), 1..=8),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 1: Default resolution is total.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 1
    /// **Validates: Requirements 1.7, 2.2, 2.7, 12.1, 13.1**
    #[test]
    fn total_default_resolution_returns_every_complete_domain(
        defaults in arb_valid_defaults(),
        irrelevant_entries in arb_irrelevant_state_entries(),
    ) {
        let mut state = BState::new();
        for (suffix, value) in irrelevant_entries {
            state.set_encoded(&format!("ctx.property_one.{suffix}"), value);
        }

        let resolved = resolve_ui_style(&defaults, &state);

        prop_assert_eq!(resolved.colors, defaults.colors);
        prop_assert_eq!(resolved.radii, defaults.radii);
        prop_assert_eq!(resolved.typography, defaults.typography);
        prop_assert_eq!(resolved.spacing, defaults.spacing);
        prop_assert_eq!(resolved.dimensions, defaults.dimensions);
        prop_assert_eq!(resolved.strokes, defaults.strokes);
        prop_assert_eq!(resolved.resize, defaults.resize);
        prop_assert_eq!(resolved.effects, defaults.effects);
        prop_assert_eq!(resolved.roles, defaults.roles);
        prop_assert_eq!(resolved, BResolvedUiStyle::from(&defaults));

        for descriptor in UI_STYLE_DESCRIPTORS {
            prop_assert_eq!(
                descriptor.value(&resolved),
                descriptor.default_value(&defaults),
                "resolved descriptor {} did not retain its complete default",
                descriptor.target,
            );
        }
    }
}
