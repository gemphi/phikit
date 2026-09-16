//! Profile modal set - static spec arrays and label/icon mappings.

use super::BProfileSpec;
use super::def::*;

pub static ALL: &[BProfileType] = &[
    BProfileType::SignIn,
    BProfileType::Preferences,
    BProfileType::ThemeSettings,
    BProfileType::KeyboardShortcuts,
    BProfileType::SignOut,
];

pub static ALL_SPECS: &[BProfileSpec] = &[
    BProfileSpec::new(BProfileType::SignIn),
    BProfileSpec::new(BProfileType::Preferences),
    BProfileSpec::new(BProfileType::ThemeSettings),
    BProfileSpec::new(BProfileType::KeyboardShortcuts),
    BProfileSpec::new(BProfileType::SignOut),
];

pub static LABELS: &[&str] = &[
    PROFILE_SIGN_IN_LABEL,
    PROFILE_PREFERENCES_LABEL,
    PROFILE_THEME_SETTINGS_LABEL,
    PROFILE_KEYBOARD_SHORTCUTS_LABEL,
    PROFILE_SIGN_OUT_LABEL,
];
pub static ICONS: &[&str] = &[
    PROFILE_SIGN_IN_ICON,
    PROFILE_PREFERENCES_ICON,
    PROFILE_THEME_SETTINGS_ICON,
    PROFILE_KEYBOARD_SHORTCUTS_ICON,
    PROFILE_SIGN_OUT_ICON,
];
