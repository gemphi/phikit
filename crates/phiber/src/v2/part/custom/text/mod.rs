//! Text editor / Info log / Console canvas — line-based text hosts.
//! Blender ref: `space_text`, `space_info`, `space_console`.

pub mod item;
pub mod list;

pub use item::BTextItem;
pub use list::BTextList;
