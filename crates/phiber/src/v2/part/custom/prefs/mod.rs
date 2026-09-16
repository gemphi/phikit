//! Preferences body — settings panels for the active section.
//! Blender ref: `space_userpref/space_userpref.cc` WINDOW region.

pub mod item;
pub mod list;

pub use item::BPrefsItem;
pub use list::BPrefsList;
