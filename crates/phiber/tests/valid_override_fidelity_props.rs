//! Feature: phiber-resolved-ui-style-settings, Property 2
//! Validates: Requirements 2.3, 13.2.

use phiber::v2::{
    state::BState,
    theme::{
        BFontFamily, BPaletteName, BShadowStyle, BStyleParser, BStyleValue, UI_STYLE_DESCRIPTORS,
        resolve_ui_style, ui_style_defaults,
    },
};
use proptest::prelude::*;

#[derive(Clone, Debug)]
struct ValidOverride {
    descriptor_index: usize,
    value: BStyleValue,
    encoded: String,
}

fn encode(value: BStyleValue) -> String {
    match value {
        BStyleValue::Number(value) => value.to_string(),
        BStyleValue::Bool(value) => value.to_string(),
        BStyleValue::Color((h, s, l, a)) => format!("{h},{s},{l},{a}"),
        BStyleValue::Shadow(shadow) => shadow.slug().to_string(),
        BStyleValue::Font(family) => family.slug().to_string(),
    }
}

fn number_strategy(index: usize, min: f32, max: f32) -> BoxedStrategy<ValidOverride> {
    prop_oneof![Just(min), Just(max), min..=max]
        .prop_map(move |number| {
            let value = BStyleValue::Number(number);
            ValidOverride {
                descriptor_index: index,
                value,
                encoded: encode(value),
            }
        })
        .boxed()
}
fn arb_valid_override() -> impl Strategy<Value = ValidOverride> {
    (0usize..UI_STYLE_DESCRIPTORS.len()).prop_flat_map(|index| {
        match UI_STYLE_DESCRIPTORS[index].parser {
            BStyleParser::Number { min, max } => number_strategy(index, min, max),
            BStyleParser::Bool => any::<bool>()
                .prop_map(move |flag| {
                    let value = BStyleValue::Bool(flag);
                    ValidOverride {
                        descriptor_index: index,
                        value,
                        encoded: encode(value),
                    }
                })
                .boxed(),
            BStyleParser::ColorHsla => {
                (0.0f32..=360.0, 0.0f32..=100.0, 0.0f32..=100.0, 0.0f32..=1.0)
                    .prop_map(move |color| {
                        let value = BStyleValue::Color(color);
                        ValidOverride {
                            descriptor_index: index,
                            value,
                            encoded: encode(value),
                        }
                    })
                    .boxed()
            }
            BStyleParser::Shadow => {
                proptest::sample::select(BShadowStyle::ALL.to_vec())
                    .prop_map(move |shadow| {
                        let value = BStyleValue::Shadow(shadow);
                        ValidOverride {
                            descriptor_index: index,
                            value,
                            encoded: encode(value),
                        }
                    })
                    .boxed()
            }
            BStyleParser::Font => {
                proptest::sample::select(BFontFamily::ALL.to_vec())
                    .prop_map(move |family| {
                        let value = BStyleValue::Font(family);
                        ValidOverride {
                            descriptor_index: index,
                            value,
                            encoded: encode(value),
                        }
                    })
                    .boxed()
            }
        }
    })
}

fn arb_palette() -> impl Strategy<Value = BPaletteName> {
    (0usize..BPaletteName::ALL.len()).prop_map(|index| BPaletteName::ALL[index])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 2: Valid overrides are applied exactly.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 2
    /// **Validates: Requirements 2.3, 13.2**
    #[test]
    fn valid_override_is_exact_and_unrelated_fields_resolve_independently(
        palette in arb_palette(),
        generated in arb_valid_override(),
    ) {
        let defaults = ui_style_defaults(palette);
        let baseline = resolve_ui_style(defaults, &BState::new());
        let descriptor = &UI_STYLE_DESCRIPTORS[generated.descriptor_index];
        let mut state = BState::new();
        state.set(descriptor.key, &generated.encoded);

        prop_assert_eq!(
            descriptor.parse_and_validate(&generated.encoded),
            Ok(generated.value),
            "generator emitted an invalid value for {}",
            descriptor.target,
        );
        let resolved = resolve_ui_style(defaults, &state);
        prop_assert_eq!(descriptor.value(&resolved), generated.value);

        for unrelated in UI_STYLE_DESCRIPTORS {
            if unrelated.target != descriptor.target {
                prop_assert_eq!(
                    unrelated.value(&resolved),
                    unrelated.value(&baseline),
                    "override for {} changed unrelated field {}",
                    descriptor.target,
                    unrelated.target,
                );
            }
        }
    }
}
