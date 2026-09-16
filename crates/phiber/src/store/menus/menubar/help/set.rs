//! Help menu sets - labels, icons, shortcuts.

use super::def::*;
use crate::store::menus::MenuItem;

// System info submenu
static SYSTEM_ITEMS: &[MenuItem] = &[
    MenuItem::node(HELP_SAVE_SYSTEM_INFO, HELP_SAVE_SYSTEM_INFO_ICON),
    MenuItem::node(HELP_RELOAD_SCRIPTS, HELP_RELOAD_SCRIPTS_ICON),
    MenuItem::node(HELP_MEMORY_STATISTICS, HELP_MEMORY_STATISTICS_ICON),
    MenuItem::node(HELP_DEBUG_MENU, HELP_DEBUG_MENU_ICON),
    MenuItem::node(HELP_REDRAW_TIMER, HELP_REDRAW_TIMER_ICON),
    MenuItem::node(HELP_CLEAN_UP_SPACE_DATA, HELP_CLEAN_UP_SPACE_DATA_ICON),
];

// Top-level Help menu
pub static ITEMS: &[MenuItem] = &[
    MenuItem::node(HELP_MANUAL, HELP_MANUAL_ICON).with_shortcut(HELP_MANUAL_SHORTCUT),
    MenuItem::node(HELP_TUTORIALS, HELP_TUTORIALS_ICON),
    MenuItem::node(HELP_SUPPORT, HELP_SUPPORT_ICON),
    MenuItem::node(HELP_EXAMPLES, HELP_EXAMPLES_ICON),
    MenuItem::node(HELP_LINKS, HELP_LINKS_ICON),
    MenuItem::node(HELP_RELEASE_NOTES, HELP_RELEASE_NOTES_ICON),
    MenuItem::separator(),
    MenuItem::node(HELP_ABOUT, HELP_ABOUT_ICON),
    MenuItem::separator(),
    MenuItem::node(HELP_SYSTEM, HELP_SYSTEM_ICON).with_items(SYSTEM_ITEMS),
    MenuItem::separator(),
    MenuItem::node(HELP_OPERATOR_CHEAT_SHEET, HELP_OPERATOR_CHEAT_SHEET_ICON),
];
