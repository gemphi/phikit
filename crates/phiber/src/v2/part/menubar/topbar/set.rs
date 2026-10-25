//! The application menus' content.
//!
//! Ground truth: `bl_ui/space_topbar.py` (TOPBAR_MT_* classes) and
//! `docs/blender-manual/manual/interface/window_system/topbar.rst`. Nested
//! sub-menus (New / Recover / Import / Export / External Data / Clean Up /
//! Defaults / System) use `BMenuKind::Sub`, which recurses without limit.
//!
//! Every entry does something: it opens a modal or the file browser, fires
//! an operator (`fire`, consumed by a rule or by the app — `app/op.rs`) or
//! flips a checkable (`tog`). An entry of Blender's that phiber has nothing
//! behind is left out rather than shown and refused (`tests/menu_truth.rs`
//! records each one), and no menu holds more than eight entries: what
//! does not fit goes one level down, under a name that says what it holds.

use super::super::item::set::*;
use super::super::item::{BMenuItem, BMenuSpec};
use crate::v2::state::{BKey as _, BPath, OpKey, ScreenKey, ZoneKey};
use crate::v2::state::val::{
    V_MODAL_ABOUT, V_MODAL_COMMAND, V_MODAL_HELP, V_MODAL_SETTINGS, V_MODAL_SPLASH,
};
use crate::v2::zone::BZoneType;

pub static NIL: &[BMenuItem] = &[];

/// The keys the checkable entries flip.
fn k_lock_interface() -> BPath {
    ScreenKey::LockInterface.path()
}
/// The status bar is the bottom zone of every workspace (`space/*/set.rs`).
fn k_status_bar() -> BPath {
    ZoneKey::Shown(BZoneType::Bottom).path()
}

// ── File > New: a file started in one of phiber's workspaces
//    (`wm.read_homefile` with `app_template`). ───────────────────────────────
static NEW: &[BMenuItem] = &[
    fire("General", "Ctrl N", OpKey::NewFile, "Layout"),
    fire("Shading", "", OpKey::NewFile, "Shading"),
    fire("UV Lights", "", OpKey::NewFile, "UvEditing"),
    fire("Animation", "", OpKey::NewFile, "Animation"),
    fire("Rendering", "", OpKey::NewFile, "Rendering"),
    fire("ECS Wand", "", OpKey::NewFile, "Compositing"),
    fire("Geometry Nodes", "", OpKey::NewFile, "GeometryNodes"),
    fire("Scripting", "", OpKey::NewFile, "Scripting"),
];

// ── File > Versions: the other copies of the open file — the next numbered
//    one, a copy elsewhere, and the one on disk. ────────────────────────────
static VERSIONS: &[BMenuItem] = &[
    fire("Save Incremental", "Ctrl Alt S", OpKey::Save, "incremental"),
    op_file("Save Copy...", "", "save_copy"),
    SEP,
    fire("Revert", "", OpKey::Revert, ""),
];

// ── File > Import: the formats `scene/import.rs` reads. ───────────────────
static IMPORT: &[BMenuItem] = &[
    op_file("Universal Scene Description (.usd*)", "", "import"),
    op_file("Wavefront (.obj)", "", "import"),
    op_file("glTF 2.0 (.glb/.gltf)", "", "import"),
];

// ── File > Export: the formats `scene/export.rs` writes. ──────────────────
static EXPORT: &[BMenuItem] = &[
    op_export("Universal Scene Description (.usda)", "usda"),
    op_export("Wavefront (.obj)", "obj"),
    op_export("Stanford PLY (.ply)", "ply"),
    op_export("glTF 2.0 (.gltf)", "gltf"),
    op_export("STL (.stl)", "stl"),
    SEP,
    op_export("Export All Collections", "gltf"),
];

// ── Top-level Topbar menus ─────────────────────────────────────────────────

// Phiber's own menu: the app, not the file.
static BLENDER_ITEMS: &[BMenuItem] = &[
    op_modal("Welcome...", "", V_MODAL_SPLASH),
    op_modal("About Phiber", "", V_MODAL_ABOUT),
    SEP,
    fire("Load Factory Settings", "", OpKey::Factory, ""),
    SEP,
    fire("Quit", "Ctrl Q", OpKey::Quit, ""),
];

// Blender ref: `TOPBAR_MT_file` in `bl_ui/space_topbar.py`.
static FILE_ITEMS: &[BMenuItem] = &[
    sub("New", NEW),
    op_file("Open...", "Ctrl O", "open"),
    RECENT,
    SEP,
    fire("Save", "Ctrl S", OpKey::Save, ""),
    op_file("Save As...", "Shift Ctrl S", "save_as"),
    sub("Versions", VERSIONS),
    SEP,
    sub("Import", IMPORT),
    sub("Export", EXPORT),
];

// Blender ref: `TOPBAR_MT_edit` in `bl_ui/space_topbar.py`.
static EDIT_ITEMS: &[BMenuItem] = &[
    fire("Undo", "Ctrl Z", OpKey::Undo, ""),
    fire("Redo", "Shift Ctrl Z", OpKey::Redo, ""),
    HISTORY,
    SEP,
    op_modal("Search...", "F3", V_MODAL_COMMAND),
    fire("Rename Active Item...", "F2", OpKey::Rename, ""),
    SEP,
    op_modal("Preferences...", "Ctrl ,", V_MODAL_SETTINGS),
];

// Blender ref: `TOPBAR_MT_render` in `bl_ui/space_topbar.py`.
static RENDER_ITEMS: &[BMenuItem] = &[
    fire("Render Image", "F12", OpKey::Render, "image"),
    fire("Render Animation", "Ctrl F12", OpKey::Render, "animation"),
    fire("Render Turnaround", "", OpKey::Render, "turnaround"),
    op_file("Render Audio...", "", "mixdown"),
    SEP,
    fire("View Render", "F11", OpKey::Render, "view"),
    fire("View Animation", "Ctrl F11", OpKey::Render, "play"),
    SEP,
    tog("Lock Interface", k_lock_interface),
];

// Blender ref: `TOPBAR_MT_window` in `bl_ui/space_topbar.py`.
static WINDOW_ITEMS: &[BMenuItem] = &[
    fire("Toggle Window Fullscreen", "", OpKey::Fullscreen, ""),
    SEP,
    fire("Next Workspace", "Ctrl Page Down", OpKey::SpaceStep, "+1"),
    fire("Previous Workspace", "Ctrl Page Up", OpKey::SpaceStep, "-1"),
    SEP,
    tog_shown("Show Status Bar", k_status_bar),
    // The spaces' tabs: labelled while they fit, icons only, or always
    // labelled (`part::display`).
    live("Space Tabs", crate::v2::part::display::topbar_menu),
];

// Blender ref: `TOPBAR_MT_help` in `bl_ui/space_topbar.py`. Blender's
// pages about blender.org are not phiber's, so only the manual and the
// bug tracker stay.
static HELP_ITEMS: &[BMenuItem] = &[
    op_modal("Manual", "", V_MODAL_HELP),
    op_url("Report a Bug", "https://github.com/phiace/phiace/issues/new"),
    SEP,
    op_modal("About Phiber", "", V_MODAL_ABOUT),
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
