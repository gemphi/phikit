//! Settings modal set - static spec arrays and row data.

use super::BSettingsSpec;
use super::def::*;

pub static ALL: &[BSettingsType] = &[
    BSettingsType::Interface,
    BSettingsType::Editing,
    BSettingsType::Navigation,
    BSettingsType::AddOns,
    BSettingsType::Themes,
    BSettingsType::System,
];

pub static ALL_SPECS: &[BSettingsSpec] = &[
    BSettingsSpec::new(BSettingsType::Interface),
    BSettingsSpec::new(BSettingsType::Editing),
    BSettingsSpec::new(BSettingsType::Navigation),
    BSettingsSpec::new(BSettingsType::AddOns),
    BSettingsSpec::new(BSettingsType::Themes),
    BSettingsSpec::new(BSettingsType::System),
];

pub static LABELS: &[&str] = &[
    SETTINGS_INTERFACE_LABEL,
    SETTINGS_EDITING_LABEL,
    SETTINGS_NAVIGATION_LABEL,
    SETTINGS_ADDONS_LABEL,
    SETTINGS_THEMES_LABEL,
    SETTINGS_SYSTEM_LABEL,
];
pub static INTERFACE_ROWS: &[(&str, &str)] = &[
    (ROW_DISPLAY, ROW_DEFAULT),
    (ROW_RESOLUTION, ROW_AUTO),
    (ROW_SCALE, ROW_SCALE_VAL),
    (ROW_LANGUAGE, ROW_ENGLISH),
];
pub static EDITING_ROWS: &[(&str, &str)] = &[
    (ROW_AUTOSAVE, ROW_ENABLED),
    (ROW_UNDO_STEPS, ROW_UNDO_32),
    (ROW_SNAP, ROW_GRID),
];
pub static NAVIGATION_ROWS: &[(&str, &str)] = &[
    (ROW_ORBIT, ROW_TURNTABLE),
    (ROW_ZOOM, ROW_SCALE),
    (ROW_WALK, ROW_DISABLED),
];
pub static ADDONS_ROWS: &[(&str, &str)] =
    &[(ROW_INSTALLED, ROW_ZERO), (ROW_CHECK_UPDATES, ROW_MANUAL)];
pub static THEMES_ROWS: &[(&str, &str)] = &[(ROW_ACTIVE_THEME, ROW_DARK), (ROW_ACCENT, ROW_BLUE)];
pub static SYSTEM_ROWS: &[(&str, &str)] = &[
    (ROW_GPU, ROW_AUTO),
    (ROW_MEMORY, ROW_DEFAULT),
    (ROW_SOUND, ROW_ENABLED),
];
