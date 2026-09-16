//! Feature: phiber-resolved-ui-style-settings, Property 4
//! Property coverage for live canonical preference re-resolution.
//!
//! **Validates: Requirements 2.5, 4.7, 7.4, 13.3**

use phiber::v2::{
    state::BState,
    theme::{
        BPaletteName, BRadiusToken, BShadowStyle, BStyleParser, BStyleRole, BStyleValue,
        UI_STYLE_DESCRIPTORS, resolve_ui_style, ui_style_defaults,
    },
};
use proptest::prelude::*;

#[derive(Clone, Debug)]
struct LiveOverridePair {
    descriptor_index: usize,
    first: BStyleValue,
    first_encoded: String,
    second: BStyleValue,
    second_encoded: String,
}

fn pair(index: usize, first: BStyleValue, second: BStyleValue) -> LiveOverridePair {
    let descriptor = &UI_STYLE_DESCRIPTORS[index];
    LiveOverridePair {
        descriptor_index: index,
        first,
        first_encoded: descriptor
            .format_value(first)
            .expect("generated first value must be valid"),
        second,
        second_encoded: descriptor
            .format_value(second)
            .expect("generated second value must be valid"),
    }
}

fn number_pair_strategy(index: usize, min: f32, max: f32) -> BoxedStrategy<LiveOverridePair> {
    let defaults = ui_style_defaults(BPaletteName::AppleDark);
    let (min, max) = match UI_STYLE_DESCRIPTORS[index].target {
        // Keep both independently resolved values exact after resize normalization.
        "resize.visible_seam" => (min, max.min(defaults.resize.hit_width)),
        "resize.hit_width" => (min.max(defaults.resize.visible_seam), max),
        _ => (min, max),
    };
    (min..=max, min..=max)
        .prop_filter("live values must be distinct", |(first, second)| {
            first != second
        })
        .prop_map(move |(first, second)| {
            pair(
                index,
                BStyleValue::Number(first),
                BStyleValue::Number(second),
            )
        })
        .boxed()
}

fn arb_live_override_pair() -> impl Strategy<Value = LiveOverridePair> {
    (0usize..UI_STYLE_DESCRIPTORS.len()).prop_flat_map(|index| {
        match UI_STYLE_DESCRIPTORS[index].parser {
            BStyleParser::Number { min, max } => number_pair_strategy(index, min, max),
            BStyleParser::Bool => any::<bool>()
                .prop_map(move |first| {
                    pair(index, BStyleValue::Bool(first), BStyleValue::Bool(!first))
                })
                .boxed(),
            BStyleParser::ColorHsla => {
                (0.0f32..360.0, 0.0f32..=100.0, 0.0f32..=100.0, 0.0f32..=1.0)
                    .prop_map(move |(h, s, l, a)| {
                        let second_h = if h < 180.0 { h + 180.0 } else { h - 180.0 };
                        pair(
                            index,
                            BStyleValue::Color((h, s, l, a)),
                            BStyleValue::Color((second_h, s, l, a)),
                        )
                    })
                    .boxed()
            }
            BStyleParser::Shadow => any::<bool>()
                .prop_map(move |first_is_none| {
                    let (first, second) = if first_is_none {
                        (BShadowStyle::None, BShadowStyle::ExtraLarge)
                    } else {
                        (BShadowStyle::ExtraLarge, BShadowStyle::None)
                    };
                    pair(
                        index,
                        BStyleValue::Shadow(first),
                        BStyleValue::Shadow(second),
                    )
                })
                .boxed(),
        }
    })
}

fn arb_palette() -> impl Strategy<Value = BPaletteName> {
    (0usize..BPaletteName::ALL.len()).prop_map(|index| BPaletteName::ALL[index])
}

fn radius_token(target: &str) -> Option<BRadiusToken> {
    match target {
        "radii.xs" => Some(BRadiusToken::Xs),
        "radii.sm" => Some(BRadiusToken::Sm),
        "radii.md" => Some(BRadiusToken::Md),
        "radii.lg" => Some(BRadiusToken::Lg),
        "radii.xl" => Some(BRadiusToken::Xl),
        _ => None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 4: Re-resolution reflects live preference changes.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 4
    /// **Validates: Requirements 2.5, 4.7, 7.4, 13.3**
    #[test]
    fn changed_canonical_preference_is_used_by_the_subsequent_resolution(
        palette in arb_palette(),
        generated in arb_live_override_pair(),
    ) {
        let defaults = ui_style_defaults(palette);
        let descriptor = &UI_STYLE_DESCRIPTORS[generated.descriptor_index];
        let mut state = BState::new();

        prop_assert_ne!(generated.first, generated.second);
        prop_assert_eq!(
            descriptor.parse_and_validate(&generated.first_encoded),
            Ok(generated.first),
        );
        prop_assert_eq!(
            descriptor.parse_and_validate(&generated.second_encoded),
            Ok(generated.second),
        );

        state.set(descriptor.key, &generated.first_encoded);
        let first_resolved = resolve_ui_style(defaults, &state);
        prop_assert_eq!(descriptor.value(&first_resolved), generated.first);

        state.set(descriptor.key, &generated.second_encoded);
        let second_resolved = resolve_ui_style(defaults, &state);
        prop_assert_eq!(state.get(descriptor.key), Some(generated.second_encoded.as_str()));
        prop_assert_eq!(descriptor.value(&second_resolved), generated.second);
        prop_assert_eq!(descriptor.value(&first_resolved), generated.first);

        if let Some(changed_token) = radius_token(descriptor.target) {
            for role in BStyleRole::ALL {
                if second_resolved.roles.token_for(role) == changed_token {
                    prop_assert_ne!(
                        first_resolved.radius_for(role),
                        second_resolved.radius_for(role),
                        "role {:?} did not re-resolve changed token {:?}",
                        role,
                        changed_token,
                    );
                }
            }
        }
    }
}
