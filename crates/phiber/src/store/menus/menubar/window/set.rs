//! Window menu sets - labels, icons, shortcuts.

use super::def::*;
use crate::store::menus::MenuItem;

pub static ITEMS: &[MenuItem] = &[
    MenuItem::node(WINDOW_NEW_WINDOW, WINDOW_NEW_WINDOW_ICON),
    MenuItem::node(WINDOW_NEW_MAIN_WINDOW, WINDOW_NEW_MAIN_WINDOW_ICON),
    MenuItem::separator(),
    MenuItem::node(WINDOW_TOGGLE_FULLSCREEN, WINDOW_TOGGLE_FULLSCREEN_ICON)
        .with_shortcut(WINDOW_TOGGLE_FULLSCREEN_SHORTCUT),
    MenuItem::separator(),
    MenuItem::node(WINDOW_NEXT_WORKSPACE, WINDOW_NEXT_WORKSPACE_ICON)
        .with_shortcut(WINDOW_NEXT_WORKSPACE_SHORTCUT),
    MenuItem::node(WINDOW_PREVIOUS_WORKSPACE, WINDOW_PREVIOUS_WORKSPACE_ICON)
        .with_shortcut(WINDOW_PREVIOUS_WORKSPACE_SHORTCUT),
    MenuItem::separator(),
    MenuItem::node(WINDOW_SHOW_STATUS_BAR, WINDOW_SHOW_STATUS_BAR_ICON),
    MenuItem::separator(),
    MenuItem::node(
        WINDOW_SAVE_SCREENSHOT_WHOLE,
        WINDOW_SAVE_SCREENSHOT_WHOLE_ICON,
    ),
    MenuItem::node(
        WINDOW_SAVE_SCREENSHOT_AREA,
        WINDOW_SAVE_SCREENSHOT_AREA_ICON,
    ),
];
