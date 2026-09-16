//! Help menu spec - struct, impl.
//!
//! def.rs  - constants
//! set.rs  - static `MenuItem` arrays
//! mod.rs  - `MenuSpecProvider` impl

pub mod def;
pub mod set;

use super::{BMenu, MenuSpec, MenuSpecProvider};

pub struct HelpMenu;

impl MenuSpecProvider for HelpMenu {
    fn idx() -> BMenu {
        BMenu::Help
    }
    fn spec() -> MenuSpec {
        MenuSpec {
            idx: BMenu::Help,
            items: &set::ITEMS,
        }
    }
}
