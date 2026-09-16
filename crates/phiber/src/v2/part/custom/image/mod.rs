//! Image/UV canvas — shared by Image and Uv hosts.
//! Blender ref: `space_image/space_image.cc` WINDOW region draw.

pub mod item;
pub mod list;

pub use item::BImageItem;
pub use list::BImageList;
