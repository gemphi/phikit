//! File menu spec - struct, impl.
//!
//! def.rs  - constants
//! set.rs  - static `MenuItem` arrays
//! mod.rs  - `MenuSpecProvider` impl

pub mod def;
pub mod set;

use super::{BMenu, MenuSpec, MenuSpecProvider};

pub struct FileMenu;

impl MenuSpecProvider for FileMenu {
    fn idx() -> BMenu {
        BMenu::File
    }
    fn spec() -> MenuSpec {
        MenuSpec {
            idx: BMenu::File,
            items: &set::ITEMS,
        }
    }
}
