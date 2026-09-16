//! Node graph canvas — shared by Node/Shader/Compositor/GeoNode hosts.
//! Blender ref: `space_node/space_node.cc` WINDOW region draw.

pub mod item;
pub mod list;

pub use item::BNodeItem;
pub use list::BNodeList;
