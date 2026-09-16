//! Modal constants — labels, widths, and type arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BModalType {
    Settings,
    Help,
    Profile,
    FileBrowser,
}

pub const SETTINGS_LABEL: &str = "Settings";
pub const HELP_LABEL: &str = "Help";
pub const PROFILE_LABEL: &str = "Profile";
pub const FILE_BROWSER_LABEL: &str = "File Browser";

pub const SETTINGS_ICON: &str = "settings";
pub const HELP_ICON: &str = "help-circle";
pub const PROFILE_ICON: &str = "user";
pub const FILE_BROWSER_ICON: &str = "folder-open";

pub const SETTINGS_WIDTH: f32 = 640.0;
pub const HELP_WIDTH: f32 = 480.0;
pub const PROFILE_WIDTH: f32 = 400.0;
pub const FILE_BROWSER_WIDTH: f32 = 720.0;
