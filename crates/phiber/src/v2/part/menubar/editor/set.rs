//! The per-editor menus' content, one table per host.
//!
//! Compact but real: these follow each editor's own `*_MT_editor_menus` in
//! `bl_ui`, trimmed to the entries phiber can actually act on.

use super::super::item::set::*;
use super::super::item::{BMenuItem, BMenuSpec};
use super::super::topbar::set::NIL;
use crate::v2::state::{BKey as _, BPath, CtxKey, FileKey, OpKey, ScreenKey};
use crate::v2::view::prim::BAction;

pub static OUTLINER_VIEW: &[BMenuItem] = &[
    fire("Show Hierarchy", "Home", OpKey::Outliner, "hierarchy"),
    fire("Expand One Level", "Numpad +", OpKey::Outliner, "expand"),
    fire("Collapse One Level", "Numpad -", OpKey::Outliner, "collapse"),
];
pub static PROPERTIES_VIEW: &[BMenuItem] = &[
    fire("Expand All", "", OpKey::Props, "expand"),
    fire("Collapse All", "", OpKey::Props, "collapse"),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
// Blender ref: `FILEBROWSER_MT_view` / `FILEBROWSER_MT_select`
// (`bl_ui/space_filebrowser.py:500-535`).
pub static FILE_VIEW: &[BMenuItem] = &[
    op("Source List", ""),
    op("File Path", ""),
    op("Frame Selected", "Numpad ."),
    SEP,
    op("Display Size", ""),
    op("Recursion Level", ""),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
pub static FILE_SELECT: &[BMenuItem] = &[
    op("All", "A"),
    op("None", "Alt A"),
    op("Invert", "Ctrl I"),
    SEP,
    op("Box Select", "B"),
];
// Blender ref: `ASSETBROWSER_MT_view` / `_library` / `_catalog`
// (`bl_ui/space_filebrowser.py:664-735`).
pub static ASSET_VIEW: &[BMenuItem] = &[
    op("Source List", ""),
    op("Asset Details", ""),
    op("Frame Selected", "Numpad ."),
    SEP,
    op("Display Size", ""),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
fn refresh() -> BAction {
    BAction::Set(FileKey::Nav.path(), "file.refresh".to_string())
}
pub static ASSET_LIBRARY: &[BMenuItem] = &[act("Refresh", "", refresh), op("Refresh Remote Listing", "")];
pub static ASSET_CATALOG: &[BMenuItem] = &[
    fire("Undo", "", OpKey::Undo, ""),
    fire("Redo", "", OpKey::Redo, ""),
    SEP,
    op("Save Catalogs", ""),
    op("New Catalog", ""),
];
pub static NODE_ADD: &[BMenuItem] = &[
    op("Search...", "F3"),
    op("Shader", ""),
    op("Texture", ""),
    op("Converter", ""),
];
pub static NODE_ITEMS: &[BMenuItem] = &[
    op("Duplicate", "Shift D"),
    op("Delete", "X"),
    op("Join in New Frame", "Ctrl J"),
];

pub static VIEW3D: &[BMenuSpec] = super::view3d::VIEW3D;
pub use super::view3d::{EDITOR_VIEW, SELECT_ITEMS};
pub static OUTLINER: &[BMenuSpec] = &[m("View", OUTLINER_VIEW)];
pub static BUTS: &[BMenuSpec] = &[m("View", PROPERTIES_VIEW)];
// Blender ref: `TIME_MT_view`, `marker_menu_generic`, `TIME_PT_playback` (`bl_ui/space_time.py`).
fn frame_start() -> BAction {
    BAction::Set(CtxKey::FrameStep.path(), "start".to_string())
}
pub static TIME_VIEW: &[BMenuItem] = &[
    op("Sidebar", "N"),
    op("Channels", ""),
    SEP,
    fire("Frame All", "Home", OpKey::TimeView, "time:all"),
    fire("Frame Scene Range", "", OpKey::TimeView, "time:scene"),
    act("Go to Start Frame", "Shift Left", frame_start),
    SEP,
    op("Show Markers", ""),
    op("Show Seconds", "Ctrl T"),
    op("Sync Visible Range", ""),
    SEP,
    op("Only Keyframes from Selected Channels", ""),
    op("Only Show Errors", ""),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
pub static TIME_MARKER: &[BMenuItem] = &[
    op("Lock Markers", ""),
    SEP,
    fire("Jump to Previous Marker", "", OpKey::Marker, "prev"),
    fire("Jump to Next Marker", "", OpKey::Marker, "next"),
    SEP,
    op("Bind Camera to Markers", "Ctrl B"),
    SEP,
    fire("Move Marker", "G", OpKey::Marker, "grab"),
    fire("Rename Marker", "F2", OpKey::Rename, "marker"),
    SEP,
    fire("Delete Marker", "", OpKey::Marker, "delete"),
    op("Duplicate Marker", ""),
    fire("Add Marker", "M", OpKey::Marker, "add"),
];
/// `TIME_PT_playback` as a header entry: the Timeline's menu row, and the
/// head of every other editor's transport (`playback::BPlayback`).
pub const PLAYBACK: BMenuSpec = panel("Playback", crate::v2::part::playback::panel::body);
pub static TIME: &[BMenuSpec] = &[m("View", TIME_VIEW), m("Marker", TIME_MARKER), PLAYBACK];
pub static FILE_HOST: &[BMenuSpec] = &[m("View", FILE_VIEW), m("Select", FILE_SELECT)];
pub static IMAGE: &[BMenuSpec] = super::image::IMAGE;
pub static INFO: &[BMenuSpec] = &[m("View", NIL), m("Info", NIL)];
pub static SEQ: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Add", NIL),
    m("Strip", NIL),
];
pub static TEXT: &[BMenuSpec] = &[
    m("View", NIL),
    m("Text", NIL),
    m("Edit", NIL),
    m("Select", NIL),
    m("Format", NIL),
    m("Templates", NIL),
];
pub static ACTION: &[BMenuSpec] = super::anim::ACTION;
pub static NLA: &[BMenuSpec] = super::anim::NLA;
pub static NODE: &[BMenuSpec] = &[
    m("View", EDITOR_VIEW),
    m("Select", SELECT_ITEMS),
    m("Add", NODE_ADD),
    m("Node", NODE_ITEMS),
];
pub static CONSOLE: &[BMenuSpec] = &[m("View", NIL), m("Console", NIL)];
pub static UV: &[BMenuSpec] = super::image::UV;
pub static COMPOSITOR: &[BMenuSpec] = NODE;
pub static SHADER: &[BMenuSpec] = NODE;
pub static GEONODE: &[BMenuSpec] = NODE;
pub static CLIP: &[BMenuSpec] = &[
    m("View", NIL),
    m("Clip", NIL),
    m("Playback", NIL),
    m("Track", NIL),
    m("Select", NIL),
];
pub static DRIVERS: &[BMenuSpec] = super::anim::DRIVERS;
pub static ASSET: &[BMenuSpec] = &[
    m("View", ASSET_VIEW),
    m("Select", FILE_SELECT),
    m("Library", ASSET_LIBRARY),
    m("Catalog", ASSET_CATALOG),
];
pub static SPREADSHEET: &[BMenuSpec] = &[m("View", NIL)];
pub static IPO: &[BMenuSpec] = super::anim::IPO;
// Blender ref: `USERPREF_MT_editor_menus` — `USERPREF_MT_view` and
// `USERPREF_MT_save_load` shown as "Preferences" (`bl_ui/space_userpref.py`).
fn k_auto_save_prefs() -> BPath {
    ScreenKey::AutoSavePrefs.path()
}
pub static PREFS_VIEW: &[BMenuItem] = &[op("Sidebar", ""), SEP, op("Toggle Maximize Area", "Ctrl Space")];
pub static PREFS_SAVE_LOAD: &[BMenuItem] = &[
    tog("Auto-Save Preferences", k_auto_save_prefs),
    SEP,
    op("Save Preferences", ""),
    op("Revert to Saved Preferences", ""),
    fire("Load Factory Preferences", "", OpKey::Factory, ""),
];
pub static PREFS: &[BMenuSpec] = &[m("View", PREFS_VIEW), m("Preferences", PREFS_SAVE_LOAD)];
