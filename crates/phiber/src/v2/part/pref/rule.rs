//! Picking a theme puts *that* theme in force — every value of it.
//!
//! Blender ref: `preferences.reset_default_theme` and a preset's XML load
//! (`bpy.ops.script.execute_preset` → `UI_theme_init_userdef`): choosing a
//! theme replaces every theme value, so nothing of the last one stays.
//! Here a colour row edited under one theme was an override in
//! `ctx.ui_style.*`, and an override outlives a `Set` of `theme` — a header
//! and a viewport nudged under Moss stayed Moss under Barbie. This rule
//! clears the overrides when the theme or its mode changes, so the window
//! resolves the chosen theme's own values and never two themes at once.

use crate::v2::state::{BKey as _, BPath, BState, AppKey, rule::BRule};

/// The prefix every style override encodes under (`key::ui_style`).
const OVERRIDE_PREFIX: &str = "ctx.ui_style.";

/// `theme` / `theme_mode` → every `ui_style` override cleared.
pub struct BThemePickRule;

impl BRule for BThemePickRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == AppKey::Theme.path() || *path == AppKey::ThemeMode.path()
    }
    fn after(&self, _: &BPath, _: &str, state: &mut BState) {
        clear_overrides(state);
    }
}

/// Remove every style override, leaving the theme's defaults in force.
pub fn clear_overrides(state: &mut BState) {
    let stale: Vec<String> = state.iter_keys().filter(|key| key.starts_with(OVERRIDE_PREFIX)).cloned().collect();
    for key in stale {
        state.remove_encoded(&key);
    }
}

/// `wm.read_factory_settings` for the preferences: the default theme in its
/// default mode with nothing overridden — `File ▸ Defaults ▸ Load Factory
/// Settings` and `Preferences ▸ Load Factory Preferences`.
pub struct BFactoryRule;

impl BRule for BFactoryRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == crate::v2::state::OpKey::Factory.path()
    }
    fn after(&self, _: &BPath, _: &str, state: &mut BState) {
        clear_overrides(state);
        state.remove(AppKey::Theme);
        state.remove(AppKey::ThemeMode);
        state.remove(AppKey::UseGradients);
        crate::v2::state::log::push("Loaded the factory preferences".to_string());
        state.remove(crate::v2::state::OpKey::Factory);
    }
}
