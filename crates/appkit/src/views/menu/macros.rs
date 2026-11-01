//! Declarative Menu Macro
//!
//! Provides a clean, declarative way to define menus without imperative code.
//!
//! ## Example
//!
//! ```rust
//! let menu_bar = menu_bar! {
//!     "File" => [
//!         "New Project" => cmd::new_project,
//!         "Open..." => cmd::open_project,
//!         ---,  // Separator
//!         "Save" => cmd::save_project,
//!         "Save As..." => cmd::save_project_as,
//!         ---,
//!         "Quit" => cmd::quit,
//!     ],
//!     "Edit" => [
//!         "Undo" => cmd::undo,
//!         "Redo" => cmd::redo,
//!         ---,
//!         "Cut" => cmd::cut,
//!         "Copy" => cmd::copy,
//!         "Paste" => cmd::paste,
//!     ],
//!     "View" => [
//!         "Hierarchy" => cmd::toggle_hierarchy,
//!         "Inspector" => cmd::toggle_inspector,
//!         ---,
//!         "Appearance" => [
//!             "Light" => cmd::set_light_mode,
//!             "Dark" => cmd::set_dark_mode,
//!         ],
//!     ],
//! };
//! ```

/// Macro for creating a menu bar declaratively
#[macro_export]
macro_rules! menu_bar {
    // Entry point: process all menu definitions
    { $( $menu_name:expr => [ $( $item:tt ),* $(,)? ] ),* $(,)? } => {
        {
            let mut menu_bar = $crate::menu::MenuBar::new();
            $(
                let menu = $crate::menu!(@parse_menu $menu_name, [ $( $item ),* ]);
                menu_bar.add_menu(menu);
            )*
            menu_bar
        }
    };
}

/// Macro for creating individual menus
#[macro_export]
macro_rules! menu {
    // Parse a menu with items
    (@parse_menu $name:expr, [ $( $item:tt ),* ]) => {
        {
            let mut menu = $crate::menu::Menu::new($name);
            $(
                $crate::menu!(@parse_item menu, $item);
            )*
            menu
        }
    };

    // Parse menu item: action
    (@parse_item $menu:ident, $label:expr => $action:expr) => {
        let item = $crate::menu::MenuItem::action($label, move || {
            $action();
        });
        $menu.add_item(item);
    };

    // Parse menu item: separator
    (@parse_item $menu:ident, ---) => {
        $menu.add_separator();
    };

    // Parse menu item: submenu
    (@parse_item $menu:ident, $label:expr => [ $( $subitem:tt ),* $(,)? ]) => {
        {
            let submenu = $crate::menu!(@parse_menu $label, [ $( $subitem ),* ]);
            let item = $crate::menu::MenuItem::submenu($label, submenu);
            $menu.add_item(item);
        }
    };

    // Entry point for standalone menu creation
    { $name:expr => [ $( $item:tt ),* $(,)? ] } => {
        $crate::menu!(@parse_menu $name, [ $( $item ),* ])
    };
}

/// Macro for creating a context menu
#[macro_export]
macro_rules! context_menu {
    { $( $item:tt ),* $(,)? } => {
        $crate::menu!(@parse_menu "", [ $( $item ),* ])
    };
}

// Re-export macros at crate root
pub use menu_bar;
pub use menu;
pub use context_menu;
