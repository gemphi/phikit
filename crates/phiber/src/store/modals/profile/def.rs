//! Profile modal constants.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BProfileType {
    SignIn,
    Preferences,
    ThemeSettings,
    KeyboardShortcuts,
    SignOut,
}

pub const PROFILE_SIGN_IN_LABEL: &str = "Sign In";
pub const PROFILE_PREFERENCES_LABEL: &str = "Preferences";
pub const PROFILE_THEME_SETTINGS_LABEL: &str = "Theme Settings";
pub const PROFILE_KEYBOARD_SHORTCUTS_LABEL: &str = "Keyboard Shortcuts";
pub const PROFILE_SIGN_OUT_LABEL: &str = "Sign Out";

pub const PROFILE_SIGN_IN_ICON: &str = "login";
pub const PROFILE_PREFERENCES_ICON: &str = "settings";
pub const PROFILE_THEME_SETTINGS_ICON: &str = "palette";
pub const PROFILE_KEYBOARD_SHORTCUTS_ICON: &str = "keyboard";
pub const PROFILE_SIGN_OUT_ICON: &str = "logout";
