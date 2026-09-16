//! Settings modal constants.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSettingsType {
    Interface,
    Editing,
    Navigation,
    AddOns,
    Themes,
    System,
}

pub const SETTINGS_INTERFACE_LABEL: &str = "Interface";
pub const SETTINGS_EDITING_LABEL: &str = "Editing";
pub const SETTINGS_NAVIGATION_LABEL: &str = "Navigation";
pub const SETTINGS_ADDONS_LABEL: &str = "Add-ons";
pub const SETTINGS_THEMES_LABEL: &str = "Themes";
pub const SETTINGS_SYSTEM_LABEL: &str = "System";

pub const ROW_DISPLAY: &str = "Display";
pub const ROW_DEFAULT: &str = "Default";
pub const ROW_RESOLUTION: &str = "Resolution";
pub const ROW_AUTO: &str = "Auto";
pub const ROW_SCALE: &str = "Scale";
pub const ROW_SCALE_VAL: &str = "1.0x";
pub const ROW_LANGUAGE: &str = "Language";
pub const ROW_ENGLISH: &str = "English";

pub const ROW_AUTOSAVE: &str = "Auto-Save";
pub const ROW_ENABLED: &str = "Enabled";
pub const ROW_UNDO_STEPS: &str = "Undo Steps";
pub const ROW_UNDO_32: &str = "32";
pub const ROW_SNAP: &str = "Snap";
pub const ROW_GRID: &str = "Grid";

pub const ROW_ORBIT: &str = "Orbit";
pub const ROW_TURNTABLE: &str = "Turntable";
pub const ROW_ZOOM: &str = "Zoom";
pub const ROW_WALK: &str = "Walk";
pub const ROW_DISABLED: &str = "Disabled";

pub const ROW_INSTALLED: &str = "Installed";
pub const ROW_ZERO: &str = "0";
pub const ROW_CHECK_UPDATES: &str = "Check for Updates";
pub const ROW_MANUAL: &str = "Manual";

pub const ROW_ACTIVE_THEME: &str = "Active Theme";
pub const ROW_DARK: &str = "Dark";
pub const ROW_ACCENT: &str = "Accent";
pub const ROW_BLUE: &str = "Blue";

pub const ROW_GPU: &str = "GPU";
pub const ROW_MEMORY: &str = "Memory";
pub const ROW_SOUND: &str = "Sound";
