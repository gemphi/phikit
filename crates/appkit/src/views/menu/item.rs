//! Menu Item Definition
//!
//! this file defines the MenuItem struct representing a single entry in a menu.
//! it supports titles, actions, icons, shortcuts, and nested submenus.
//! mirrors Apple's modular pattern for hierarchical menu structures.

use super::{Menu, MenuAction};
use std::sync::Arc;

/// A single item in a menu or menu bar.
///
/// tracks interactive state, visual identity, and potential submenus.
/// provides factory methods for standard item types like actions and separators.
#[derive(Clone)]
pub struct MenuItem {
    pub title: String,
    pub action: Option<MenuAction>,
    pub enabled: bool,
    pub is_section: bool,
    pub icon: Option<String>,
    pub submenu: Option<Box<Menu>>,
    pub shortcut: Option<String>,
}

impl MenuItem {
    /// Creates a new action item with a title and closure.
    /// initializes the item as enabled and without any special styling.
    pub fn new(title: &str, action: MenuAction) -> Self {
        Self {
            title: title.to_string(),
            action: Some(action),
            enabled: true,
            is_section: false,
            icon: None,
            submenu: None,
            shortcut: None,
        }
    }

    /// Assigns a visual icon to the menu item.
    /// allows the item to be identified by a symbol in the UI.
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Assigns a keyboard shortcut to the menu item.
    /// provides a hint to the user about quick activation keys.
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    /// Creates a visual separator item.
    /// used to group related items within a single menu.
    pub fn separator() -> Self {
        Self {
            title: String::new(),
            action: None,
            enabled: false,
            is_section: false,
            icon: None,
            submenu: None,
            shortcut: None,
        }
    }

    /// Creates a non-interactive section header.
    /// provides a labeled group title within a complex menu.
    pub fn section(title: &str) -> Self {
        Self {
            title: title.to_string(),
            action: None,
            enabled: false,
            is_section: true,
            icon: None,
            submenu: None,
            shortcut: None,
        }
    }

    /// Creates an item that opens a nested submenu.
    /// mirrors Apple's hierarchical navigation pattern for menus.
    pub fn submenu(menu: Menu) -> Self {
        Self {
            title: menu.title.clone(),
            action: None,
            enabled: true,
            is_section: false,
            icon: None,
            submenu: Some(Box::new(menu)),
            shortcut: None,
        }
    }

    /// Convenience factory for simple closure-based actions.
    /// wraps the provided function in an Arc for thread-safe sharing.
    pub fn action<F: Fn() + Send + Sync + 'static>(title: &str, action: F) -> Self {
        Self {
            title: title.to_string(),
            action: Some(Arc::new(action)),
            enabled: true,
            is_section: false,
            icon: None,
            submenu: None,
            shortcut: None,
        }
    }

    /// Creates a submenu item with a custom label different from the menu title.
    /// provides flexibility in how nested hierarchies are presented to the user.
    pub fn submenu_with_label(title: &str, menu: Menu) -> Self {
        Self {
            title: title.to_string(),
            action: None,
            enabled: true,
            is_section: false,
            icon: None,
            submenu: Some(Box::new(menu)),
            shortcut: None,
        }
    }
}
