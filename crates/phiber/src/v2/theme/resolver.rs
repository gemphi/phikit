//! Pure resolution of canonical UI-style defaults and runtime overrides.

use super::{BResolvedUiStyle, BUiStyleDefaults, UI_STYLE_DESCRIPTORS};
use crate::v2::state::BState;

/// Resolves one complete style snapshot from immutable defaults and state.
///
/// Each descriptor is considered exactly once. Missing or rejected values
/// leave that descriptor's copied default untouched; valid values are applied
/// only to the descriptor's target field.
pub fn resolve_ui_style(defaults: &BUiStyleDefaults, state: &BState) -> BResolvedUiStyle {
    let mut resolved = BResolvedUiStyle::from(defaults);

    for descriptor in UI_STYLE_DESCRIPTORS {
        let Some(raw) = state.get(descriptor.key) else {
            continue;
        };
        let Ok(value) = descriptor.parse_and_validate(raw) else {
            continue;
        };
        let applied = descriptor.apply_value(&mut resolved, value);
        debug_assert!(applied, "descriptor parser and writer types must agree");
    }

    resolved.normalize_resize_geometry();
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::{
        BPaletteName, BShadowStyle,
        state::key::ui_style as key,
        theme::{BStyleValue, ui_style_defaults},
    };

    fn state_snapshot(state: &BState) -> Vec<(String, String)> {
        let mut snapshot: Vec<_> = state
            .iter_keys()
            .map(|key| {
                (
                    key.clone(),
                    state.get_encoded(key).unwrap_or_default().to_owned(),
                )
            })
            .collect();
        snapshot.sort();
        snapshot
    }

    #[test]
    fn missing_overrides_copy_every_completed_palette_default() {
        let state = BState::new();

        for palette in BPaletteName::ALL.iter().copied() {
            let defaults = ui_style_defaults(palette);
            assert_eq!(
                resolve_ui_style(defaults, &state),
                BResolvedUiStyle::from(defaults)
            );
        }
    }

    #[test]
    fn valid_overrides_are_applied_through_canonical_descriptors() {
        let defaults = ui_style_defaults(BPaletteName::AppleDark);
        let mut state = BState::new();
        state.set(&key::RADIUS_MD, "12.5");
        state.set(&key::COLOR_ACCENT, "210, 50, 40, 0.75");
        state.set(&key::EFFECT_GRADIENTS, "false");
        state.set(&key::EFFECT_SURFACE_SHADOW, "none");

        let resolved = resolve_ui_style(defaults, &state);

        assert_eq!(resolved.radii.md, 12.5);
        assert_eq!(resolved.colors.accent, (210.0, 50.0, 40.0, 0.75));
        assert!(!resolved.effects.gradients);
        assert_eq!(resolved.effects.surface_shadow, BShadowStyle::None);
        assert_eq!(resolved.radii.lg, defaults.radii.lg);
        for (target, expected) in [
            ("radii.md", BStyleValue::Number(12.5)),
            (
                "colors.accent",
                BStyleValue::Color((210.0, 50.0, 40.0, 0.75)),
            ),
            ("effects.gradients", BStyleValue::Bool(false)),
            (
                "effects.surface_shadow",
                BStyleValue::Shadow(BShadowStyle::None),
            ),
        ] {
            let descriptor = UI_STYLE_DESCRIPTORS
                .iter()
                .find(|descriptor| descriptor.target == target)
                .expect("canonical descriptor");
            assert_eq!(descriptor.value(&resolved), expected);
        }
    }

    #[test]
    fn invalid_numeric_overrides_fall_back_per_field() {
        let defaults = ui_style_defaults(BPaletteName::BlenderDark);
        let mut state = BState::new();
        state.set(&key::RADIUS_XS, "not-a-number");
        state.set(&key::TYPOGRAPHY_BODY, "NaN");
        state.set(&key::SPACING_GAP_XS, "-1");
        state.set(&key::STROKE_BORDER, "17");
        state.set(&key::RADIUS_SM, "3.5");

        let resolved = resolve_ui_style(defaults, &state);

        assert_eq!(resolved.radii.xs, defaults.radii.xs);
        assert_eq!(resolved.typography.body, defaults.typography.body);
        assert_eq!(resolved.spacing.gap_xs, defaults.spacing.gap_xs);
        assert_eq!(resolved.strokes.border, defaults.strokes.border);
        assert_eq!(resolved.radii.sm, 3.5);
    }

    #[test]
    fn conflicting_resize_overrides_are_normalized_without_mutating_inputs() {
        let defaults = *ui_style_defaults(BPaletteName::TokyoNight);
        let defaults_before = defaults;
        let mut state = BState::new();
        state.set(&key::RESIZE_VISIBLE_SEAM, "64");
        state.set(&key::RESIZE_HIT_WIDTH, "2");
        state.set(&key::RESIZE_GAP, "12");
        state.set(&key::RESIZE_INSET, "-42");
        let state_before = state_snapshot(&state);

        let resolved = resolve_ui_style(&defaults, &state);

        assert_eq!(resolved.resize.visible_seam, 64.0);
        assert_eq!(resolved.resize.hit_width, 64.0);
        assert_eq!(resolved.resize.gap, 12.0);
        assert_eq!(resolved.resize.inset, -42.0);
        assert_eq!(defaults, defaults_before);
        assert_eq!(state_snapshot(&state), state_before);
    }
}
