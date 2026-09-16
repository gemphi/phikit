//! Window menu spec - struct, impl.
//!
//! def.rs  - constants
//! set.rs  - static `MenuItem` arrays
//! mod.rs  - `MenuSpecProvider` impl

pub mod def;
pub mod set;

use super::{BMenu, MenuSpec, MenuSpecProvider};

pub struct WindowMenu;

impl MenuSpecProvider for WindowMenu {
    fn idx() -> BMenu {
        BMenu::Window
    }
    fn spec() -> MenuSpec {
        MenuSpec {
            idx: BMenu::Window,
            items: &set::ITEMS,
        }
    }
}
