//! The Image editor's menus — `IMAGE_MT_editor_menus`: View and Image in
//! View mode; the UV editor adds Select and UV. Structure: which menus;
//! their entries are `set.rs`.

pub mod set;

use super::super::item::{BMenuSpec, set::m};

/// `IMAGE_MT_editor_menus` in View mode.
pub static IMAGE: &[BMenuSpec] = &[m("View", set::VIEW), m("Image", set::IMAGE)];
/// The UV editor's menus (`show_uvedit`).
pub static UV: &[BMenuSpec] = &[m("View", set::VIEW), m("Select", set::SELECT), m("Image", set::IMAGE), m("UV", set::UV)];
