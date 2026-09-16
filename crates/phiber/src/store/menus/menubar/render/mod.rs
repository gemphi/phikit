//! Render menu spec - struct, impl.
//!
//! def.rs  - constants
//! set.rs  - static `MenuItem` arrays
//! mod.rs  - `MenuSpecProvider` impl

pub mod def;
pub mod set;

use super::{BMenu, MenuSpec, MenuSpecProvider};

pub struct RenderMenu;

impl MenuSpecProvider for RenderMenu {
    fn idx() -> BMenu {
        BMenu::Render
    }
    fn spec() -> MenuSpec {
        MenuSpec {
            idx: BMenu::Render,
            items: &set::ITEMS,
        }
    }
}
