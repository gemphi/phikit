//! Menu specs - shared enums, structs, trait, and aggregate functions.
//!
//! Menu types are categorized into subdirectories:
//!   menubar/   - top-level dropdown menus (File, Edit, Render, Window, Help)
//!   sidebar/   - context menus for sidebar/outliner areas
//!   switcher/  - area type switcher menu items
//!   spacebar/  - workspace tab menu items
//!
//! Structure:
//!   def.rs  - types, enums, constants
//!   set.rs  - per-group static `MenuItem` arrays and lookup helpers
//!   mod.rs  - module re-exports and `BMenu -> MenuSpec` dispatch

mod def;
pub use def::*;

pub mod header;
pub mod menubar;
pub mod sidebar;
pub mod spacebar;
pub mod switcher;

impl BMenu {
    pub fn spec(self) -> MenuSpec {
        match self {
            BMenu::File => menubar::file::FileMenu::spec(),
            BMenu::Edit => menubar::edit::EditMenu::spec(),
            BMenu::Render => menubar::render::RenderMenu::spec(),
            BMenu::Window => menubar::window::WindowMenu::spec(),
            BMenu::Help => menubar::help::HelpMenu::spec(),
        }
    }
}

// Store → UI conversion lives in the UI layer (e.g. buikit or phiber/bui)
// so that this module stays UI-agnostic.
