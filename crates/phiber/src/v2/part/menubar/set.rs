//! Menubar set — per-host top menus + full item content for the Topbar.
//!
//! Ground truth: `docs/blender-manual/manual/interface/window_system/topbar.rst`
//! and `bl_ui/space_topbar.py` (TOPBAR_MT_* classes). Nested sub-menus
//! (Recover / Import / Export / External Data / Clean Up / Defaults / About)
//! use `BMenuKind::Sub(&[BMenuItem])` — recursion is unlimited.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};
use crate::v2::state::val::{
    V_MODAL_ABOUT, V_MODAL_COMMAND, V_MODAL_FILE_BROWSER, V_MODAL_HELP, V_MODAL_SETTINGS,
    V_MODAL_SPLASH,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BMenuKind {
    Op,
    Toggle,
    Sub(&'static [BMenuItem]),
    Sep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BMenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub kind: BMenuKind,
    /// Declarative modal target: the slug of the modal this item opens when
    /// clicked (e.g. `"settings"`, `"file_browser"`). Empty for ordinary
    /// operators that just close the menu. This keeps the item's action
    /// intent in the set itself so the renderer never needs a hardcoded
    /// `(menu, label)` routing table.
    pub modal: &'static str,
}

const fn op(label: &'static str, shortcut: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Op,
        modal: "",
    }
}
/// An operator that opens a modal. `modal` is the modal slug
/// (`BModalType::slug`), carried declaratively so `leaf_action` can route it
/// without a per-label match.
const fn op_modal(
    label: &'static str,
    shortcut: &'static str,
    modal: &'static str,
) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Op,
        modal,
    }
}
const fn tog(label: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Toggle,
        modal: "",
    }
}
const fn sub(label: &'static str, items: &'static [BMenuItem]) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Sub(items),
        modal: "",
    }
}
const SEP: BMenuItem = BMenuItem {
    label: "",
    shortcut: "",
    kind: BMenuKind::Sep,
    modal: "",
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BMenuSpec {
    pub label: &'static str,
    pub items: &'static [BMenuItem],
}

impl BSpec for BMenuSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Menubar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

const fn m(label: &'static str, items: &'static [BMenuItem]) -> BMenuSpec {
    BMenuSpec { label, items }
}
static NIL: &[BMenuItem] = &[];

// ── About Blender sub-menu (Blender menu / Splash) ─────────────────────────
static ABOUT: &[BMenuItem] = &[
    op("Donate", ""),
    op("What's New", ""),
    op("Credits", ""),
    op("License", ""),
    SEP,
    op("Blender Store", ""),
    op("Blender Website", ""),
];

// ── File > Recover sub-menu ────────────────────────────────────────────────
static RECOVER: &[BMenuItem] = &[op("Recover Last Session", ""), op("Recover Auto Save", "")];

// ── File > Import sub-menu (bl_ui/space_topbar.py TOPBAR_MT_file_import) ───
static IMPORT: &[BMenuItem] = &[
    op("Alembic (.abc)", ""),
    op("Universal Scene Description (.usd*)", ""),
    op("Wavefront (.obj)", ""),
    op("Stanford PLY (.ply)", ""),
    op("Filmbox (.fbx)", ""),
    op("X3D Extensible 3D (.x3d/.wrl)", ""),
    op("Motion Capture (.bvh)", ""),
    op("glTF 2.0 (.glb/.gltf)", ""),
    op("STL (.stl)", ""),
    op("Scalable Vector Graphics (.svg)", ""),
];

// ── File > Export sub-menu (from screenshot, TOPBAR_MT_file_export) ────────
static EXPORT: &[BMenuItem] = &[
    op("Alembic (.abc)", ""),
    op("Universal Scene Description (.usd*)", ""),
    op("Grease Pencil as SVG", ""),
    op("Grease Pencil as PDF", ""),
    op("Wavefront (.obj)", ""),
    op("Stanford PLY (.ply)", ""),
    op("Filmbox (.fbx)", ""),
    op("X3D (.x3d)", ""),
    op("Motion Capture (.bvh)", ""),
    op("glTF 2.0 (.glb/.gltf)", ""),
    op("STL (.stl)", ""),
];

// ── File > External Data sub-menu ──────────────────────────────────────────
static EXTERNAL_DATA: &[BMenuItem] = &[
    tog("Automatically Pack Resources"),
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
static DEFAULTS: &[BMenuItem] = &[op("Save Startup File", ""), op("Load Factory Settings", "")];

// ── Top-level Topbar menus ─────────────────────────────────────────────────

// Blender ref: `TOPBAR_MT_blender` in `bl_ui/space_topbar.py`.
static BLENDER_ITEMS: &[BMenuItem] = &[
    op_modal("Splash Screen", "", V_MODAL_SPLASH),
    op_modal("About Blender", "", V_MODAL_ABOUT),
    SEP,
    op("Install Application Template...", ""),
    SEP,
    sub("System", &[]),
];

// Blender ref: `TOPBAR_MT_file` in `bl_ui/space_topbar.py`.
static FILE_ITEMS: &[BMenuItem] = &[
    sub("New", &[]),
    op_modal("Open...", "Ctrl O", V_MODAL_FILE_BROWSER),
    sub("Open Recent", &[]),
    op("Revert", ""),
    sub("Recover", RECOVER),
    SEP,
    op("Save", "Ctrl S"),
    op_modal("Save As...", "Shift Ctrl S", V_MODAL_FILE_BROWSER),
    op("Save Copy...", ""),
    op("Save Incremental", "Ctrl Alt S"),
    SEP,
    op("Link...", ""),
    op("Append...", ""),
    sub("Data Previews", &[]),
    SEP,
    sub("Import", IMPORT),
    sub("Export", EXPORT),
    op("Export All Collections", ""),
    SEP,
    sub("External Data", EXTERNAL_DATA),
    sub("Clean Up", CLEAN_UP),
    SEP,
    sub("Defaults", DEFAULTS),
    SEP,
    op("Quit", "Ctrl Q"),
];

// Blender ref: `TOPBAR_MT_edit` in `bl_ui/space_topbar.py`.
static EDIT_ITEMS: &[BMenuItem] = &[
    op("Undo", "Ctrl Z"),
    op("Redo", "Shift Ctrl Z"),
    sub("Undo History", &[]),
    SEP,
    op("Adjust Last Operation...", "F9"),
    op("Repeat Last", "Shift R"),
    op("Repeat History...", ""),
    SEP,
    op_modal("Menu Search...", "F3", V_MODAL_COMMAND),
    op("Operator Search...", ""),
    SEP,
    op("Rename Active Item...", "F2"),
    op("Batch Rename...", "Ctrl F2"),
    SEP,
    tog("Lock Object Modes"),
    SEP,
    op_modal("Preferences...", "Ctrl Comma", V_MODAL_SETTINGS),
];

// Blender ref: `TOPBAR_MT_render` in `bl_ui/space_topbar.py`.
static RENDER_ITEMS: &[BMenuItem] = &[
    op("Render Image", "F12"),
    op("Render Animation", "Ctrl F12"),
    SEP,
    op("Render Audio...", ""),
    SEP,
    op("View Render", ""),
    op("View Animation", ""),
    SEP,
    tog("Lock Interface"),
];

// Blender ref: `TOPBAR_MT_window` in `bl_ui/space_topbar.py`.
static WINDOW_ITEMS: &[BMenuItem] = &[
    op("New Window", ""),
    op("New Main Window", ""),
    SEP,
    op("Toggle Window Fullscreen", ""),
    SEP,
    op("Next Workspace", "Ctrl PageDown"),
    op("Previous Workspace", "Ctrl PageUp"),
    SEP,
    tog("Show Status Bar"),
    SEP,
    op("Save Screenshot...", ""),
    op("Save Screenshot (Editor)...", ""),
];

// Blender ref: `TOPBAR_MT_help` in `bl_ui/space_topbar.py`.
static HELP_ITEMS: &[BMenuItem] = &[
    op_modal("Manual", "", V_MODAL_HELP),
    op("Support", ""),
    op("User Communities", ""),
    op("Get Involved", ""),
    op("Release Notes", ""),
    SEP,
    op("Report a Bug", ""),
    op("Save System Info...", ""),
];

// ── Per-host top-level menu tables ─────────────────────────────────────────

static TOPBAR: &[BMenuSpec] = &[
    m("Blender", BLENDER_ITEMS),
    m("File", FILE_ITEMS),
    m("Edit", EDIT_ITEMS),
    m("Render", RENDER_ITEMS),
    m("Window", WINDOW_ITEMS),
    m("Help", HELP_ITEMS),
];

// Compact but real editor menus used by the Shading workspace.
static EDITOR_VIEW: &[BMenuItem] = &[
    op("Frame All", "Home"),
    op("Frame Selected", "Numpad ."),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
static SELECT_ITEMS: &[BMenuItem] = &[op("All", "A"), op("None", "Alt A"), op("Invert", "Ctrl I")];
static VIEW3D_ADD: &[BMenuItem] = &[
    op("Mesh", "Shift A"),
    op("Curve", ""),
    op("Camera", ""),
    op("Light", ""),
];
static OBJECT_ITEMS: &[BMenuItem] = &[
    op("Shade Auto Smooth", ""),
    op("Duplicate Objects", "Shift D"),
    op("Delete", "X"),
];
static OUTLINER_VIEW: &[BMenuItem] = &[
    op("Show Hierarchy", "Home"),
    op("Expand One Level", "Numpad +"),
    op("Collapse One Level", "Numpad -"),
];
static PROPERTIES_VIEW: &[BMenuItem] =
    &[op("Expand All", ""), op("Collapse All", ""), op("Area", "")];
static FILE_VIEW: &[BMenuItem] = &[op("List", ""), op("Thumbnails", ""), op("Sort by Name", "")];
static IMAGE_ITEMS: &[BMenuItem] = &[
    op("New", "Alt N"),
    op("Open...", "Alt O"),
    op("Save", "Alt S"),
];
static UV_ITEMS: &[BMenuItem] = &[op("Unwrap", "U"), op("Pin", "P"), op("Pack Islands", "")];
static NODE_ADD: &[BMenuItem] = &[
    op("Search...", "F3"),
    op("Shader", ""),
    op("Texture", ""),
    op("Converter", ""),
];
static NODE_ITEMS: &[BMenuItem] = &[
    op("Duplicate", "Shift D"),
    op("Delete", "X"),
    op("Join in New Frame", "Ctrl J"),
];

static VIEW3D: &[BMenuSpec] = &[
    m("View", EDITOR_VIEW),
    m("Select", SELECT_ITEMS),
    m("Add", VIEW3D_ADD),
    m("Object", OBJECT_ITEMS),
];
static OUTLINER: &[BMenuSpec] = &[m("View", OUTLINER_VIEW)];
static BUTS: &[BMenuSpec] = &[m("View", PROPERTIES_VIEW)];
static TIME: &[BMenuSpec] = &[
    m("View", NIL),
    m("Marker", NIL),
    m("Frame", NIL),
    m("Playback", NIL),
];
static FILE_HOST: &[BMenuSpec] = &[m("View", FILE_VIEW)];
static IMAGE: &[BMenuSpec] = &[
    m("View", EDITOR_VIEW),
    m("Select", SELECT_ITEMS),
    m("Image", IMAGE_ITEMS),
    m("UV", UV_ITEMS),
];
static INFO: &[BMenuSpec] = &[m("View", NIL), m("Info", NIL)];
static SEQ: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Add", NIL),
    m("Strip", NIL),
];
static TEXT: &[BMenuSpec] = &[
    m("View", NIL),
    m("Text", NIL),
    m("Edit", NIL),
    m("Select", NIL),
    m("Format", NIL),
    m("Templates", NIL),
];
static ACTION: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Channel", NIL),
    m("Key", NIL),
];
static NLA: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Edit", NIL),
    m("Add", NIL),
    m("Tracks", NIL),
];
static NODE: &[BMenuSpec] = &[
    m("View", EDITOR_VIEW),
    m("Select", SELECT_ITEMS),
    m("Add", NODE_ADD),
    m("Node", NODE_ITEMS),
];
static CONSOLE: &[BMenuSpec] = &[m("View", NIL), m("Console", NIL)];
static UV: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Image", NIL),
    m("UV", NIL),
];
static COMPOSITOR: &[BMenuSpec] = NODE;
static SHADER: &[BMenuSpec] = NODE;
static GEONODE: &[BMenuSpec] = NODE;
static CLIP: &[BMenuSpec] = &[
    m("View", NIL),
    m("Clip", NIL),
    m("Playback", NIL),
    m("Track", NIL),
    m("Select", NIL),
];
static DRIVERS: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Channel", NIL),
    m("Key", NIL),
];
static ASSET: &[BMenuSpec] = &[m("View", NIL), m("Select", NIL)];
static SPREADSHEET: &[BMenuSpec] = &[m("View", NIL)];
static IPO: &[BMenuSpec] = &[
    m("View", NIL),
    m("Select", NIL),
    m("Marker", NIL),
    m("Channel", NIL),
    m("Key", NIL),
];

pub fn menus_for(host: BHostType) -> &'static [BMenuSpec] {
    match host {
        BHostType::Topbar => TOPBAR,
        BHostType::View3D => VIEW3D,
        BHostType::Outliner => OUTLINER,
        BHostType::Buts => BUTS,
        BHostType::Time => TIME,
        BHostType::File => FILE_HOST,
        BHostType::Image => IMAGE,
        BHostType::Info => INFO,
        BHostType::Seq => SEQ,
        BHostType::Text => TEXT,
        BHostType::Action => ACTION,
        BHostType::Nla => NLA,
        BHostType::Node => NODE,
        BHostType::Console => CONSOLE,
        BHostType::Uv => UV,
        BHostType::Compositor => COMPOSITOR,
        BHostType::Shader => SHADER,
        BHostType::GeoNode => GEONODE,
        BHostType::Clip => CLIP,
        BHostType::Drivers => DRIVERS,
        BHostType::Asset => ASSET,
        BHostType::Spreadsheet => SPREADSHEET,
        BHostType::Ipo => IPO,
        _ => &[],
    }
}
