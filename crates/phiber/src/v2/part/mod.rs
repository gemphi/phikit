//! Part — the leaf. Every variant lives in its own subfolder with item + list.

pub mod item;
pub mod list;
pub mod set;
pub mod spec;
pub mod r#type;

// One subfolder per BPartType variant.
pub mod activity;
pub mod assetshelf;
pub mod channels;
pub mod custom;
pub mod execute;
pub mod hud;
pub mod menubar;
pub mod modebar;
pub mod navbar;
pub mod panel;
pub mod sidebar;
pub mod spacebar;
pub mod statusbar;
pub mod strip;
pub mod switcher;
pub mod toolbar;
pub mod toolprops;
pub mod zonebar;

pub use item::BPartItem;
pub use list::BPartList;
pub use set::SET;
pub use spec::BPartSpec;
pub use r#type::BPartType;
