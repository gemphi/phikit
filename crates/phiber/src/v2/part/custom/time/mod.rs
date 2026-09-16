//! Timeline center content — the frame ruler + scrubber.
//! Blender ref: `space_action.cc` Timeline mode WINDOW region draw.

pub mod item;
pub mod list;

pub use item::BTimeItem;
pub use list::BTimeList;
