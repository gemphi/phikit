//! Edit menu spec - struct, impl.
//!
//! def.rs  - constants
//! set.rs  - static `MenuItem` arrays
//! mod.rs  - `MenuSpecProvider` impl

pub mod def;
pub mod set;

use super::{BMenu, MenuSpec, MenuSpecProvider};

pub struct EditMenu;

impl MenuSpecProvider for EditMenu {
    fn idx() -> BMenu {
        BMenu::Edit
    }
    fn spec() -> MenuSpec {
        MenuSpec {
            idx: BMenu::Edit,
            items: &set::ITEMS,
        }
    }
}
