//! The application menus' content.
//!
//! Ground truth: `bl_ui/space_topbar.py` (TOPBAR_MT_* classes) and
//! `docs/blender-manual/manual/interface/window_system/topbar.rst`. Nested
//! sub-menus (New / Recover / Import / Export / External Data / Clean Up /
//! Defaults / System) use `BMenuKind::Sub`, which recurses without limit.
//!
//! Every entry does something: it opens a modal or the file browser, fires
//! an operator (`fire`, consumed by a rule or by the app — `app/op.rs`),
//! flips a checkable (`tog`), or — where the operator is not here yet —
//! reports itself as not available (`menubar/op.rs`), so the status bar
//! always answers the click.

use super::super::item::set::*;
use super::super::item::{BMenuItem, BMenuSpec};
use crate::v2::state::{BKey as _, BPath, OpKey, ScreenKey, ZoneKey};
use crate::v2::state::val::{
    V_MODAL_ABOUT, V_MODAL_COMMAND, V_MODAL_HELP, V_MODAL_SETTINGS, V_MODAL_SPLASH,
};
use crate::v2::zone::BZoneType;

pub static NIL: &[BMenuItem] = &[];

/// The keys the checkable entries flip.
fn k_autopack() -> BPath {
    ScreenKey::Autopack.path()
}
fn k_lock_modes() -> BPath {
    ScreenKey::LockObjectModes.path()
}
fn k_lock_interface() -> BPath {
    ScreenKey::LockInterface.path()
}
/// The status bar is the bottom zone of every workspace (`space/*/set.rs`).
fn k_status_bar() -> BPath {
    ZoneKey::Shown(BZoneType::Bottom).path()
}

// ── File > New sub-menu: Blender's app templates; phiber starts a file in
//    one of its workspaces (`wm.read_homefile` with `app_template`). ──────────
static NEW: &[BMenuItem] = &[
    fire("General", "Ctrl N", OpKey::NewFile, "Layout"),
    fire("Shading", "", OpKey::NewFile, "Shading"),
    fire("UV Lights", "", OpKey::NewFile, "UvEditing"),
    fire("Animation", "", OpKey::NewFile, "Animation"),
    fire("Rendering", "", OpKey::NewFile, "Rendering"),
    fire("Compositing", "", OpKey::NewFile, "Compositing"),
    fire("Geometry Nodes", "", OpKey::NewFile, "GeometryNodes"),
    fire("Scripting", "", OpKey::NewFile, "Scripting"),
];

// ── File > Recover sub-menu ────────────────────────────────────────────────
static RECOVER: &[BMenuItem] = &[op("Recover Last Session", ""), op("Recover Auto Save", "")];

// ── File > Import sub-menu (bl_ui/space_topbar.py TOPBAR_MT_file_import).
//    Every one opens the browser; what is read depends on the extension
//    picked (`scene/import.rs`: OBJ, glTF, USD). ────────────────────────────
static IMPORT: &[BMenuItem] = &[
    op_file("Alembic (.abc)", "", "import"),
    op_file("Universal Scene Description (.usd*)", "", "import"),
    op_file("Wavefront (.obj)", "", "import"),
    op_file("Stanford PLY (.ply)", "", "import"),
    op_file("Filmbox (.fbx)", "", "import"),
    op_file("X3D Extensible 3D (.x3d/.wrl)", "", "import"),
    op_file("Motion Capture (.bvh)", "", "import"),
    op_file("glTF 2.0 (.glb/.gltf)", "", "import"),
    op_file("STL (.stl)", "", "import"),
    op_file("Scalable Vector Graphics (.svg)", "", "import"),
];

// ── File > Export sub-menu (TOPBAR_MT_file_export). The formats phiber
//    writes name themselves (`scene/export.rs`); the rest report. ────────────
static EXPORT: &[BMenuItem] = &[
    op("Alembic (.abc)", ""),
    op_export("Universal Scene Description (.usda)", "usda"),
    op("Grease Pencil as SVG", ""),
    op("Grease Pencil as PDF", ""),
    op_export("Wavefront (.obj)", "obj"),
    op_export("Stanford PLY (.ply)", "ply"),
    op("Filmbox (.fbx)", ""),
    op("X3D (.x3d)", ""),
    op("Motion Capture (.bvh)", ""),
    op_export("glTF 2.0 (.gltf)", "gltf"),
    op_export("STL (.stl)", "stl"),
];

// ── File > External Data sub-menu ──────────────────────────────────────────
static EXTERNAL_DATA: &[BMenuItem] = &[
    tog("Automatically Pack Resources", k_autopack),
    op("Pack Resources", ""),
    op("Unpack Resources", ""),
    op("Pack Linked Libraries", ""),
    op("Unpack Linked Libraries", ""),
    SEP,
    op("Make Paths Relative", ""),
    op("Make Paths Absolute", ""),
    op("Report Missing Files", ""),
    op("Find Missing Files", ""),
];

// ── File > Clean Up sub-menu ───────────────────────────────────────────────
static CLEAN_UP: &[BMenuItem] = &[op("Purge Unused Data", ""), op("Manage Unused Data", "")];

// ── File > Defaults sub-menu ───────────────────────────────────────────────
static DEFAULTS: &[BMenuItem] = &[op("Save Startup File", ""), fire("Load Factory Settings", "", OpKey::Factory, "")];

// ── Phiber > System sub-menu (TOPBAR_MT_blender_system) ────────────────────
static SYSTEM: &[BMenuItem] = &[op("Save System Info...", ""), fire("Load Factory Settings", "", OpKey::Factory, "")];

// ── Top-level Topbar menus ─────────────────────────────────────────────────

// Blender ref: `TOPBAR_MT_blender` in `bl_ui/space_topbar.py`.
static BLENDER_ITEMS: &[BMenuItem] = &[
    op_modal("Splash Screen", "", V_MODAL_SPLASH),
    op_modal("About Phiber", "", V_MODAL_ABOUT),
    SEP,
    op("Install Application Template...", ""),
    SEP,
    sub("System", SYSTEM),
];

// Blender ref: `TOPBAR_MT_file` in `bl_ui/space_topbar.py`.
static FILE_ITEMS: &[BMenuItem] = &[
    sub("New", NEW),
    op_file("Open...", "Ctrl O", "open"),
    RECENT,
    fire("Revert", "", OpKey::Revert, ""),
    sub("Recover", RECOVER),
    SEP,
    fire("Save", "Ctrl S", OpKey::Save, ""),
    op_file("Save As...", "Shift Ctrl S", "save_as"),
    op_file("Save Copy...", "", "save_copy"),
    fire("Save Incremental", "Ctrl Alt S", OpKey::Save, "incremental"),
    SEP,
    op_file("Link...", "", "link"),
    op_file("Append...", "", "append"),
    op("Data Previews", ""),
    SEP,
    sub("Import", IMPORT),
    sub("Export", EXPORT),
    op_export("Export All Collections", "gltf"),
    SEP,
    sub("External Data", EXTERNAL_DATA),
    sub("Clean Up", CLEAN_UP),
    SEP,
    sub("Defaults", DEFAULTS),
    SEP,
    fire("Quit", "Ctrl Q", OpKey::Quit, ""),
];

// Blender ref: `TOPBAR_MT_edit` in `bl_ui/space_topbar.py`.
static EDIT_ITEMS: &[BMenuItem] = &[
    fire("Undo", "Ctrl Z", OpKey::Undo, ""),
    fire("Redo", "Shift Ctrl Z", OpKey::Redo, ""),
    HISTORY,
    SEP,
    op("Adjust Last Operation...", "F9"),
    op("Repeat Last", "Shift R"),
    op("Repeat History...", ""),
    SEP,
    op_modal("Menu Search...", "F3", V_MODAL_COMMAND),
    op_modal("Operator Search...", "", V_MODAL_COMMAND),
    SEP,
    fire("Rename Active Item...", "F2", OpKey::Rename, ""),
    op("Batch Rename...", "Ctrl F2"),
    SEP,
    tog("Lock Object Modes", k_lock_modes),
    SEP,
    op_modal("Preferences...", "Ctrl ,", V_MODAL_SETTINGS),
];

// Blender ref: `TOPBAR_MT_render` in `bl_ui/space_topbar.py`.
static RENDER_ITEMS: &[BMenuItem] = &[
    fire("Render Image", "F12", OpKey::Render, "image"),
    fire("Render Animation", "Ctrl F12", OpKey::Render, "animation"),
    fire("Render Turnaround", "", OpKey::Render, "turnaround"),
    SEP,
    op_file("Render Audio...", "", "mixdown"),
    SEP,
    fire("View Render", "F11", OpKey::Render, "view"),
    fire("View Animation", "Ctrl F11", OpKey::Render, "play"),
    SEP,
    tog("Lock Interface", k_lock_interface),
];

// Blender ref: `TOPBAR_MT_window` in `bl_ui/space_topbar.py`.
static WINDOW_ITEMS: &[BMenuItem] = &[
    op("New Window", ""),
    op("New Main Window", ""),
    SEP,
    fire("Toggle Window Fullscreen", "", OpKey::Fullscreen, ""),
    SEP,
    fire("Next Workspace", "Ctrl Page Down", OpKey::SpaceStep, "+1"),
    fire("Previous Workspace", "Ctrl Page Up", OpKey::SpaceStep, "-1"),
    SEP,
    tog_shown("Show Status Bar", k_status_bar),
    SEP,
    op("Save Screenshot...", ""),
    op("Save Screenshot (Editor)...", ""),
];

// Blender ref: `TOPBAR_MT_help` in `bl_ui/space_topbar.py`.
static HELP_ITEMS: &[BMenuItem] = &[
    op_modal("Manual", "", V_MODAL_HELP),
    op_url("Support", "https://www.blender.org/support/"),
    op_url("User Communities", "https://www.blender.org/community/"),
    op_url("Get Involved", "https://www.blender.org/get-involved/"),
    op_url("Release Notes", "https://developer.blender.org/docs/release_notes/"),
    SEP,
    op_url("Report a Bug", "https://github.com/phiace/phiace/issues/new"),
    op("Save System Info...", ""),
];

// ── Per-host top-level menu tables ─────────────────────────────────────────

pub static TOPBAR: &[BMenuSpec] = &[
    marked("Phiber", "phiber", BLENDER_ITEMS),
    m("File", FILE_ITEMS),
    m("Edit", EDIT_ITEMS),
    m("Render", RENDER_ITEMS),
    m("Window", WINDOW_ITEMS),
    m("Help", HELP_ITEMS),
];
