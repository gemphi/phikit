//! Edit menu sets - labels, icons, shortcuts.

use super::def::*;
use crate::store::menus::MenuItem;

pub static ITEMS: &[MenuItem] = &[
    MenuItem::node(EDIT_UNDO, EDIT_UNDO_ICON).with_shortcut(EDIT_UNDO_SHORTCUT),
    MenuItem::node(EDIT_REDO, EDIT_REDO_ICON).with_shortcut(EDIT_REDO_SHORTCUT),
    MenuItem::node(EDIT_UNDO_HISTORY, EDIT_UNDO_HISTORY_ICON),
    MenuItem::separator(),
    MenuItem::node(EDIT_MENU_SEARCH, EDIT_MENU_SEARCH_ICON)
        .with_shortcut(EDIT_MENU_SEARCH_SHORTCUT),
    MenuItem::node(EDIT_OPERATOR_SEARCH, EDIT_OPERATOR_SEARCH_ICON),
    MenuItem::separator(),
    MenuItem::node(EDIT_RENAME_ACTIVE, EDIT_RENAME_ACTIVE_ICON)
        .with_shortcut(EDIT_RENAME_ACTIVE_SHORTCUT),
    MenuItem::node(EDIT_BATCH_RENAME, EDIT_BATCH_RENAME_ICON)
        .with_shortcut(EDIT_BATCH_RENAME_SHORTCUT),
    MenuItem::separator(),
    MenuItem::node(EDIT_LOCK_OBJECT_MODES, EDIT_LOCK_OBJECT_MODES_ICON),
    MenuItem::separator(),
    MenuItem::node(EDIT_PREFERENCES, EDIT_PREFERENCES_ICON)
        .with_shortcut(EDIT_PREFERENCES_SHORTCUT),
];
