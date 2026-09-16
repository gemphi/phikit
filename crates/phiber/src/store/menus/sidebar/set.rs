//! Sidebar menu sets - outliner, settings, and activity bar arrays.

use super::def::*;
use crate::store::hosts::BHostType;
use crate::store::menus::MenuItem;

// â”€â”€ Outliner â”€â”€
pub static OUTLINER_ITEMS: &[MenuItem] = &[
    MenuItem::node(OUTLINER_EXPAND_ALL, OUTLINER_EXPAND_ALL_ICON),
    MenuItem::node(OUTLINER_COLLAPSE_ALL, OUTLINER_COLLAPSE_ALL_ICON),
    MenuItem::separator(),
    MenuItem::node(OUTLINER_SELECT_HIERARCHY, OUTLINER_SELECT_HIERARCHY_ICON),
    MenuItem::node(OUTLINER_RENAME, OUTLINER_RENAME_ICON),
];

// â”€â”€ Settings â”€â”€
pub static SETTINGS_ITEMS: &[MenuItem] = &[
    MenuItem::node(SETTINGS_PREFERENCES, SETTINGS_PREFERENCES_ICON),
    MenuItem::node(SETTINGS_KEYMAP, SETTINGS_KEYMAP_ICON),
    MenuItem::separator(),
    MenuItem::node(SETTINGS_THEME, SETTINGS_THEME_ICON),
    MenuItem::node(SETTINGS_SYSTEM, SETTINGS_SYSTEM_ICON),
];

// â”€â”€ Activity Bar â”€â”€
pub static ACTIVITY_TOP: &[MenuItem] = &[
    MenuItem::node(ACTIVITY_TOGGLE_RAIL, ACTIVITY_TOGGLE_RAIL_ICON),
    MenuItem::node(ACTIVITY_HOME, ACTIVITY_HOME_ICON),
    MenuItem::node(ACTIVITY_SEARCH, ACTIVITY_SEARCH_ICON),
    MenuItem::node(ACTIVITY_RECENT, ACTIVITY_RECENT_ICON),
];

pub static ACTIVITY_CENTER: &[MenuItem] = &[
    MenuItem::node(ACTIVITY_MENU, ACTIVITY_MENU_ICON),
    MenuItem::node(ACTIVITY_SPACES, ACTIVITY_SPACES_ICON),
    MenuItem::node(ACTIVITY_MORE, ACTIVITY_MORE_ICON),
];

pub static ACTIVITY_BOTTOM: &[MenuItem] = &[
    MenuItem::node(ACTIVITY_USER, ACTIVITY_USER_ICON),
    MenuItem::node(ACTIVITY_SETTINGS, ACTIVITY_SETTINGS_ICON),
];

/// Select the most relevant sidebar menu for a host.
pub fn with_idx(host: BHostType) -> &'static [MenuItem] {
    match host {
        BHostType::Outliner => OUTLINER_ITEMS,
        BHostType::Buts => SETTINGS_ITEMS,
        _ => ACTIVITY_CENTER,
    }
}
