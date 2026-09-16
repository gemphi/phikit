//! Property coverage for immutable resolved-style inputs.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 5
//! **Validates: Requirements 2.6**

use phiber::v2::{
    state::BState,
    theme::{
        BPaletteName, BResolvedUiStyle, BStyleParser, BUiStyleDefaults, UI_STYLE_DESCRIPTORS,
        resolve_ui_style, ui_style_defaults,
    },
};
use proptest::prelude::*;

type ValueSample = (usize, u16, u16, u16, u16);

fn arb_value_samples() -> impl Strategy<Value = Vec<ValueSample>> {
    prop::collection::vec(
        (
            0usize..UI_STYLE_DESCRIPTORS.len(),
            any::<u16>(),
            any::<u16>(),
            any::<u16>(),
            any::<u16>(),
        ),
        0..=UI_STYLE_DESCRIPTORS.len() * 2,
    )
}

fn valid_raw(parser: BStyleParser, a: u16, b: u16, c: u16, d: u16) -> String {
    match parser {
        BStyleParser::Number { min, max } => {
            let fraction = a as f32 / u16::MAX as f32;
            (min + (max - min) * fraction).to_string()
        }
        BStyleParser::Bool => (a % 2 == 0).to_string(),
        BStyleParser::ColorHsla => format!(
            "{},{},{},{}",
            a as f32 / u16::MAX as f32 * 360.0,
            b as f32 / u16::MAX as f32 * 100.0,
            c as f32 / u16::MAX as f32 * 100.0,
            d as f32 / u16::MAX as f32,
        ),
        BStyleParser::Shadow => match a % 2 {
            0 => "none".to_owned(),
            _ => "extra_large".to_owned(),
        },
    }
}
fn generated_defaults(palette_index: usize, samples: &[ValueSample]) -> BUiStyleDefaults {
    let base = *ui_style_defaults(BPaletteName::ALL[palette_index]);
    let mut generated = BResolvedUiStyle::from(&base);

    for &(index, a, b, c, d) in samples {
        let descriptor = &UI_STYLE_DESCRIPTORS[index];
        let value = descriptor
            .parse_and_validate(&valid_raw(descriptor.parser, a, b, c, d))
            .expect("generated descriptor value must be valid");
        assert!(descriptor.apply_value(&mut generated, value));
    }
    generated.resize.hit_width = generated
        .resize
        .hit_width
        .max(generated.resize.visible_seam);

    BUiStyleDefaults {
        colors: generated.colors,
        radii: generated.radii,
        typography: generated.typography,
        spacing: generated.spacing,
        dimensions: generated.dimensions,
        strokes: generated.strokes,
        resize: generated.resize,
        effects: generated.effects,
        roles: generated.roles,
    }
}

fn generated_state(samples: &[ValueSample]) -> BState {
    let mut state = BState::new();
    for &(index, a, b, c, d) in samples {
        let descriptor = &UI_STYLE_DESCRIPTORS[index];
        state.set(descriptor.key, valid_raw(descriptor.parser, a, b, c, d));
    }
    state
}

fn state_snapshot(state: &BState) -> Vec<(String, String)> {
    let mut snapshot: Vec<_> = state
        .iter_keys()
        .map(|key| {
            (
                key.clone(),
                state
                    .get_encoded(key)
                    .expect("iterated key must exist")
                    .to_owned(),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 5: Style resolution is pure.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 5
    /// **Validates: Requirements 2.6**
    #[test]
    fn style_resolution_is_pure(
        palette_index in 0usize..BPaletteName::ALL.len(),
        default_samples in arb_value_samples(),
        state_samples in arb_value_samples(),
    ) {
        let defaults = generated_defaults(palette_index, &default_samples);
        let state = generated_state(&state_samples);
        let defaults_before = defaults.clone();
        let state_before = state.clone();

        let _resolved = resolve_ui_style(&defaults, &state);

        prop_assert_eq!(defaults, defaults_before);
        prop_assert_eq!(state_snapshot(&state), state_snapshot(&state_before));
    }
}
