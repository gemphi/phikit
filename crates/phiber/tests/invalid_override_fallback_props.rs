//! Property coverage for rejected numeric UI-style overrides.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 3
//! **Validates: Requirements 2.4**

use phiber::v2::{
    BPaletteName, BResolvedUiStyle, BState, BStyleParseError, BStyleParser, BStyleValue,
    UI_STYLE_DESCRIPTORS, resolve_ui_style, ui_style_defaults,
};
use proptest::{
    prelude::*,
    test_runner::{TestCaseError, TestCaseResult},
};

fn assert_all_numeric_fields_valid(style: &BResolvedUiStyle) -> TestCaseResult {
    for descriptor in UI_STYLE_DESCRIPTORS {
        let BStyleParser::Number { min, max } = descriptor.parser else {
            continue;
        };
        let BStyleValue::Number(value) = descriptor.value(style) else {
            return Err(TestCaseError::fail(format!(
                "numeric descriptor {} did not read a number",
                descriptor.target
            )));
        };
        prop_assert!(
            value.is_finite() && value >= min && value <= max,
            "{} resolved to invalid {}; expected finite [{}, {}]",
            descriptor.target,
            value,
            min,
            max,
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 3: Invalid overrides fall back safely.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 3
    /// **Validates: Requirements 2.4**
    #[test]
    fn invalid_numeric_overrides_retain_defaults_and_validity(
        malformed_suffix in any::<u64>(),
        non_finite_variant in 0usize..3,
        out_of_bounds_delta in 0.25f32..1024.0,
        palette_index in 0usize..BPaletteName::ALL.len(),
    ) {
        let defaults = ui_style_defaults(BPaletteName::ALL[palette_index]);
        let expected = BResolvedUiStyle::from(defaults);
        let malformed = format!("not-a-number-{malformed_suffix}");
        let non_finite = ["NaN", "inf", "-inf"][non_finite_variant];
        for descriptor in UI_STYLE_DESCRIPTORS {
            let BStyleParser::Number { min, max } = descriptor.parser else {
                continue;
            };
            let invalid_cases = [
                (malformed.clone(), BStyleParseError::Malformed),
                (non_finite.to_owned(), BStyleParseError::NonFinite),
                ((min - out_of_bounds_delta).to_string(), BStyleParseError::OutOfBounds),
                ((max + out_of_bounds_delta).to_string(), BStyleParseError::OutOfBounds),
            ];

            for (raw, expected_error) in invalid_cases {
                prop_assert_eq!(
                    descriptor.parse_and_validate(&raw),
                    Err(expected_error),
                    "{} did not reject {:?} as {:?}",
                    descriptor.target,
                    raw,
                    expected_error,
                );

                let mut state = BState::new();
                state.set(descriptor.key, raw.clone());
                let resolved = resolve_ui_style(defaults, &state);

                prop_assert_eq!(
                    descriptor.value(&resolved),
                    descriptor.default_value(defaults),
                    "{} did not retain its target default for {:?}",
                    descriptor.target,
                    raw,
                );
                prop_assert_eq!(
                    &resolved,
                    &expected,
                    "invalid override changed resolved style for {} with {:?}",
                    descriptor.target,
                    raw,
                );
                assert_all_numeric_fields_valid(&resolved)?;
            }
        }
    }
}
