//! Modal set - static spec arrays, labels, icons, and widths.

use super::BModalSpec;
use super::def::*;

pub static ALL: &[BModalType] = &[
    BModalType::Settings,
    BModalType::Help,
    BModalType::Profile,
    BModalType::FileBrowser,
];

pub static ALL_SPECS: &[BModalSpec] = &[
    BModalSpec {
        idx: BModalType::Settings,
        items: &[],
    },
    BModalSpec {
        idx: BModalType::Help,
        items: &[],
    },
    BModalSpec {
        idx: BModalType::Profile,
        items: &[],
    },
    BModalSpec {
        idx: BModalType::FileBrowser,
        items: &[],
    },
];

pub static LABELS: &[&str] = &[
    SETTINGS_LABEL,
    HELP_LABEL,
    PROFILE_LABEL,
    FILE_BROWSER_LABEL,
];
pub static ICONS: &[&str] = &[SETTINGS_ICON, HELP_ICON, PROFILE_ICON, FILE_BROWSER_ICON];
pub static WIDTHS: &[f32] = &[
    SETTINGS_WIDTH,
    HELP_WIDTH,
    PROFILE_WIDTH,
    FILE_BROWSER_WIDTH,
];
