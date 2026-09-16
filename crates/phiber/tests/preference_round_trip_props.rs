//! Feature: phiber-resolved-ui-style-settings, Property 10
//! Validates: Requirements 9.3, 9.4, 9.5.

use phiber::v2::{
    part::panel::prefs::{BPreferenceControls, BPreferenceEditor, BPreferenceRow, SECTIONS},
    state::{BState, action::apply_action},
    theme::{
        BPaletteName, BShadowStyle, BStyleParser, BStyleValue, UI_STYLE_DESCRIPTORS,
        resolve_ui_style, ui_style_defaults,
    },
    view::BAction,
};
use proptest::prelude::*;

fn arb_palette() -> impl Strategy<Value = BPaletteName> {
    (0usize..BPaletteName::ALL.len()).prop_map(|index| BPaletteName::ALL[index])
}

fn generated_value(
    row: BPreferenceRow,
    number_fraction: f32,
    flag: bool,
    color: (f32, f32, f32, f32),
    extra_large_shadow: bool,
) -> BStyleValue {
    match row.parser() {
        BStyleParser::Number { min, max } => {
            let value = if number_fraction <= 0.0 {
                min
            } else if number_fraction >= 1.0 {
                max
            } else {
                min + (max - min) * number_fraction
            };
            BStyleValue::Number(value)
        }
        BStyleParser::Bool => BStyleValue::Bool(flag),
        BStyleParser::ColorHsla => BStyleValue::Color(color),
        BStyleParser::Shadow => BStyleValue::Shadow(if extra_large_shadow {
            BShadowStyle::ExtraLarge
        } else {
            BShadowStyle::None
        }),
    }
}

fn display_matches(controls: BPreferenceControls, row: BPreferenceRow, value: BStyleValue) -> bool {
    let encoded = row
        .descriptor()
        .format_value(value)
        .expect("generated value must be canonical");
    match controls.editor {
        BPreferenceEditor::Field(field) => {
            let expected = match row.metadata().unit {
                Some(unit) => format!("{encoded} {unit}"),
                None => encoded,
            };
            field.text() == expected
        }
        BPreferenceEditor::Selector(selector) => selector.selected() == encoded,
        BPreferenceEditor::Toggle(toggle) => {
            matches!(value, BStyleValue::Bool(expected) if toggle.checked() == expected)
        }
    }
}

fn prepare_state(
    row: BPreferenceRow,
    value: BStyleValue,
    state: &mut BState,
    defaults: &phiber::v2::theme::BUiStyleDefaults,
) {
    if let BStyleValue::Bool(flag) = value {
        state.set(row.key(), (!flag).to_string());
    }

    // A generated hit width below the default seam is valid on its own but is
    // normalized as a conflicting pair. Keep the companion seam compatible so
    // this property observes the exact value committed by the Preferences UI.
    if row.target() == "resize.hit_width" {
        let BStyleValue::Number(hit_width) = value else {
            unreachable!("resize.hit_width must be numeric")
        };
        let seam = UI_STYLE_DESCRIPTORS
            .iter()
            .find(|descriptor| descriptor.target == "resize.visible_seam")
            .expect("visible seam descriptor must exist");
        let seam_value = BStyleValue::Number(defaults.resize.visible_seam.min(hit_width));
        state.set(
            seam.key,
            seam.format_value(seam_value)
                .expect("compatible seam must be canonical"),
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 10: Preference editing round-trips through canonical state.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 10
    /// **Validates: Requirements 9.3, 9.4, 9.5**
    #[test]
    fn preference_edit_and_reset_round_trip_for_every_exposed_descriptor(
        palette in arb_palette(),
        number_fraction in 0.0f32..=1.0,
        flag in any::<bool>(),
        color in (0.0f32..=360.0, 0.0f32..=100.0, 0.0f32..=100.0, 0.0f32..=1.0),
        extra_large_shadow in any::<bool>(),
    ) {
        let defaults = ui_style_defaults(palette);
        let mut row_count = 0;

        for (index, row) in SECTIONS.iter().flat_map(|section| section.rows()).enumerate() {
            row_count += 1;
            let descriptor = &UI_STYLE_DESCRIPTORS[index];
            prop_assert!(
                std::ptr::eq(row.descriptor(), descriptor),
                "preference row {} is not backed by its canonical descriptor",
                row.target(),
            );

            let value = generated_value(
                row,
                number_fraction,
                flag,
                color,
                extra_large_shadow,
            );
            let encoded = descriptor
                .format_value(value)
                .expect("generated value must pass descriptor validation");
            let mut state = BState::new();
            prepare_state(row, value, &mut state, defaults);

            let controls = BPreferenceControls::for_row(row, &state, defaults);
            let edit = match controls.editor {
                BPreferenceEditor::Field(field) => field
                    .commit(&encoded)
                    .expect("generated field value must be accepted"),
                BPreferenceEditor::Selector(selector) => selector
                    .select(&encoded)
                    .expect("generated choice must be accepted"),
                BPreferenceEditor::Toggle(toggle) => toggle.action(),
            };
            match &edit {
                BAction::Set(key, written) => {
                    prop_assert_eq!(key, row.key(), "edit used a non-canonical key");
                    prop_assert_eq!(written, &encoded, "edit changed the canonical value");
                }
                action => prop_assert!(false, "preference edit emitted {action:?}"),
            }

            apply_action(&mut state, &edit);
            prop_assert_eq!(state.get(row.key()), Some(encoded.as_str()));
            let resolved = resolve_ui_style(defaults, &state);
            prop_assert_eq!(descriptor.value(&resolved), value);

            let edited_controls = BPreferenceControls::for_row(row, &state, defaults);
            prop_assert!(edited_controls.reset.enabled());
            prop_assert!(
                display_matches(edited_controls, row, value),
                "edited display did not round-trip {}",
                row.target(),
            );

            let reset = edited_controls.reset.action();
            match &reset {
                BAction::Reset(key) => {
                    prop_assert_eq!(key, row.key(), "reset used a non-canonical key");
                }
                action => prop_assert!(false, "preference reset emitted {action:?}"),
            }
            apply_action(&mut state, &reset);

            prop_assert_eq!(state.get(row.key()), None);
            let fallback = descriptor.default_value(defaults);
            let reset_resolved = resolve_ui_style(defaults, &state);
            prop_assert_eq!(descriptor.value(&reset_resolved), fallback);
            prop_assert!(
                display_matches(
                    BPreferenceControls::for_row(row, &state, defaults),
                    row,
                    fallback,
                ),
                "reset display did not return {} to its effective default",
                row.target(),
            );
        }

        prop_assert_eq!(row_count, UI_STYLE_DESCRIPTORS.len());
    }
}
