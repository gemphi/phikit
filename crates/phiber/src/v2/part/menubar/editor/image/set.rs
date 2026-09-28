//! The entries of the Image editor's menus, from `bl_ui/space_image.py`:
//! `IMAGE_MT_view`, `IMAGE_MT_view_zoom`, `IMAGE_MT_image`,
//! `IMAGE_MT_select`, `IMAGE_MT_uvs`. The zoom entries and Frame All write
//! the view (`image.view_zoom_ratio`, `image.view_all`); Save and Save As
//! open the file browser; the rest are Blender's operators by name.

use super::super::super::item::{BMenuItem, set::{SEP, act, op, op_file, sub}};
use crate::v2::state::{BKey as _, ImageKey};
use crate::v2::view::prim::BAction;

/// `image.view_all`: written as `fit` and resolved against the window by the
/// canvas, since the menu does not know the region's size.
pub const ZOOM_FIT: &str = "fit";

fn zoom_to(ratio: &str) -> BAction {
    BAction::Set(ImageKey::Zoom.path(), ratio.to_string())
}
fn zoom_eighth() -> BAction {
    zoom_to("0.125")
}
fn zoom_quarter() -> BAction {
    zoom_to("0.25")
}
fn zoom_half() -> BAction {
    zoom_to("0.5")
}
fn zoom_one() -> BAction {
    zoom_to("1")
}
fn zoom_two() -> BAction {
    zoom_to("2")
}
fn zoom_four() -> BAction {
    zoom_to("4")
}
fn zoom_eight() -> BAction {
    zoom_to("8")
}
fn fit() -> BAction {
    zoom_to(ZOOM_FIT)
}
/// `image.view_zoom_in` / `_out`: a step the canvas resolves against the
/// zoom in force (`image.zoom = "+"` / `"-"`).
fn zoom_in() -> BAction {
    zoom_to("+")
}
fn zoom_out() -> BAction {
    zoom_to("-")
}

/// `IMAGE_MT_view_zoom`: `image.view_zoom_ratio` at 1:8 … 8:1, then in / out.
pub static VIEW_ZOOM: &[BMenuItem] = &[
    act("12.5% (1:8)", "", zoom_eighth),
    act("25% (1:4)", "", zoom_quarter),
    act("50% (1:2)", "", zoom_half),
    act("100% (1:1)", "Numpad 1", zoom_one),
    act("200% (2:1)", "Numpad 2", zoom_two),
    act("400% (4:1)", "Numpad 4", zoom_four),
    act("800% (8:1)", "Numpad 8", zoom_eight),
    SEP,
    act("Zoom In", "Numpad +", zoom_in),
    act("Zoom Out", "Numpad -", zoom_out),
];

/// `IMAGE_MT_view`.
pub static VIEW: &[BMenuItem] = &[
    op("Toolbar", "T"),
    op("Sidebar", "N"),
    op("Tool Settings", ""),
    op("Asset Shelf", ""),
    op("Adjust Last Operation", ""),
    SEP,
    op("Update Automatically", ""),
    op("Show Metadata", ""),
    SEP,
    op("Frame Selected", "Numpad ."),
    act("Frame All", "Home", fit),
    op("Center View to Cursor", ""),
    sub("Zoom", VIEW_ZOOM),
    SEP,
    op("Set Render Region", "Ctrl B"),
    op("Clear Render Region", "Ctrl Alt B"),
    SEP,
    op("Render Slot Cycle Next", "J"),
    op("Render Slot Cycle Previous", "Alt J"),
    SEP,
    op("Show Same Material", ""),
    op("Toggle Maximize Area", "Ctrl Space"),
];

/// `IMAGE_MT_image`. A Render Result has no file of its own, so Save opens
/// the Save As browser, as Blender's does.
pub static IMAGE: &[BMenuItem] = &[
    op("New...", "Alt N"),
    op("Open...", "Alt O"),
    op("Open Cached Render", ""),
    SEP,
    op("Replace...", ""),
    op("Reload", "Alt R"),
    op("Edit Externally", ""),
    SEP,
    op("Copy", "Ctrl C"),
    op("Paste", "Ctrl V"),
    SEP,
    op_file("Save", "Alt S", "save_image"),
    op_file("Save As...", "Shift Alt S", "save_image"),
    op_file("Save a Copy...", "", "save_image"),
    op("Save Sequence", ""),
    op("Save All Images", ""),
    SEP,
    sub("Invert", INVERT),
    op("Resize", ""),
    sub("Transform", TRANSFORM),
];

/// `IMAGE_MT_image_invert` and `IMAGE_MT_image_transform`.
pub static INVERT: &[BMenuItem] = &[
    op("Invert Image Colors", ""),
    SEP,
    op("Invert Red Channel", ""),
    op("Invert Green Channel", ""),
    op("Invert Blue Channel", ""),
    op("Invert Alpha Channel", ""),
];
pub static TRANSFORM: &[BMenuItem] = &[op("Flip Horizontally", ""), op("Flip Vertically", ""), SEP, op("Rotate 90° Clockwise", ""), op("Rotate 90° Counter-Clockwise", ""), op("Rotate 180°", "")];

/// `IMAGE_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    op("All", "A"),
    op("None", "Alt A"),
    op("Invert", "Ctrl I"),
    SEP,
    op("Box Select", "B"),
    op("Circle Select", "C"),
    SEP,
    op("Select Linked", "Ctrl L"),
    op("Select Pinned", "Shift P"),
];

/// `IMAGE_MT_uvs_mirror`.
pub static MIRROR: &[BMenuItem] = &[op("X Axis", ""), op("Y Axis", "")];

/// `IMAGE_MT_uvs`.
pub static UV: &[BMenuItem] = &[
    sub("Transform", TRANSFORM),
    sub("Mirror", MIRROR),
    SEP,
    op("Unwrap", "U"),
    op("Pin", "P"),
    op("Unpin", "Alt P"),
    SEP,
    op("Pack Islands", ""),
    op("Average Islands Scale", ""),
    op("Minimize Stretch", ""),
    op("Stitch", "Alt V"),
];
