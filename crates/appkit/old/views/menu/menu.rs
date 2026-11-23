//! Menu and MenuBar Definitions
//!
//! this file defines the Menu and MenuBar structures used to build
//! hierarchical navigation for the application. it handles collection
//! of menu items and provides convenience methods for common additions.

use super::item::MenuItem;
use std::sync::Arc;

/// A collection of menu items representing a dropdown or group.
///
/// tracks a list of items and handles their lifecycle.
/// mirrors Apple's tiered approach to menu organization.
#[derive(Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    /// Creates a new Menu with the given title.
    /// initializes an empty list of items.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    /// Appends a pre-configured MenuItem to the menu.
    /// manages internal ordering of items.
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Adds a simple action item to the menu.
    /// uses "---" as a shortcut for adding separators.
    pub fn add(&mut self, title: &str, action: impl Fn() + Send + Sync + 'static) {
        if title == "---" {
            self.items.push(MenuItem::separator());
        } else {
            self.items.push(MenuItem::new(title, Arc::new(action)));
        }
    }

    /// Adds an action item with a visual icon.
    /// enhances item discoverability in the UI.
    pub fn add_with_icon(
        &mut self,
        title: &str,
        icon: &str,
        action: impl Fn() + Send + Sync + 'static,
    ) {
        self.items
            .push(MenuItem::new(title, Arc::new(action)).with_icon(icon));
    }

    /// Adds an action item with a keyboard shortcut.
    /// provides quick access hints to the user.
    pub fn add_with_shortcut(
        &mut self,
        title: &str,
        shortcut: &str,
        action: impl Fn() + Send + Sync + 'static,
    ) {
        self.items
            .push(MenuItem::new(title, Arc::new(action)).with_shortcut(shortcut));
    }

    /// Adds an action item with both icon and shortcut.
    /// used for high-visibility primary actions.
    pub fn add_icon_with_shortcut(
        &mut self,
        title: &str,
        icon: &str,
        shortcut: &str,
        action: impl Fn() + Send + Sync + 'static,
    ) {
        self.items.push(
            MenuItem::new(title, Arc::new(action))
                .with_icon(icon)
                .with_shortcut(shortcut),
        );
    }

    /// Adds a non-interactive section header.
    /// useful for grouping related items within the menu.
    pub fn add_section(&mut self, title: &str) {
        self.items.push(MenuItem::section(title));
    }

    /// Adds a nested submenu to the menu.
    /// mirrors Apple's hierarchical navigation pattern.
    pub fn add_submenu(&mut self, menu: Menu) {
        self.items.push(MenuItem::submenu(menu));
    }

    /// Adds a visual separator line.
    /// improves organization and readability of the menu content.
    pub fn add_separator(&mut self) {
        self.items.push(MenuItem::separator());
    }
}

/// A container for the top-level menus in an application.
///
/// tracks the list of primary dropdowns displayed in the menu bar.
/// handles addition and retrieval of top-level menu groups.
#[derive(Clone)]
pub struct MenuBar {
    pub menus: Vec<Menu>,
}

impl MenuBar {
    /// Creates a new empty MenuBar.
    /// ready to be populated with top-level Menu objects.
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Appends a top-level menu to the bar.
    /// manages the collection of primary application dropdowns.
    pub fn add(&mut self, menu: Menu) {
        self.menus.push(menu);
    }

    /// Alias for add() to match Apple naming patterns.
    /// provides flexibility for different API usage styles.
    pub fn add_menu(&mut self, menu: Menu) {
        self.menus.push(menu);
    }
}
