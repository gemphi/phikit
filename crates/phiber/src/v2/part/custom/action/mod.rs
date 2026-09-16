//! Animation editor canvas — shared by Action/Nla/Ipo/Drivers hosts.
//! Blender ref: `space_action`, `space_nla`, `space_graph`.

pub mod item;
pub mod list;

pub use item::BActionItem;
pub use list::BActionList;
