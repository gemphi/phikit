//! The 3D Viewport's menus — `VIEW3D_MT_view`, `VIEW3D_MT_select_object`,
//! `VIEW3D_MT_add` / `VIEW3D_MT_mesh_add`, `VIEW3D_MT_object` — and the
//! View and Select menus the node editors share. Every entry fires an
//! operator (`scene/view_op.rs`, `scene/select.rs`, `scene/edit.rs`) or
//! reports itself.

use super::super::item::set::*;
use super::super::item::{BMenuItem, BMenuSpec};
use crate::v2::state::OpKey;

// ── 3D Viewport (`VIEW3D_MT_view`, `_select_object`, `_add`, `_object`) ──
static VIEW3D_VIEWPOINT: &[BMenuItem] = &[
    op("Camera", "Numpad 0"),
    SEP,
    fire("Top", "Numpad 7", OpKey::View3d, "viewpoint:top"),
    fire("Bottom", "Ctrl Numpad 7", OpKey::View3d, "viewpoint:bottom"),
    SEP,
    fire("Front", "Numpad 1", OpKey::View3d, "viewpoint:front"),
    fire("Back", "Ctrl Numpad 1", OpKey::View3d, "viewpoint:back"),
    SEP,
    fire("Right", "Numpad 3", OpKey::View3d, "viewpoint:right"),
    fire("Left", "Ctrl Numpad 3", OpKey::View3d, "viewpoint:left"),
];
static VIEW3D_NAVIGATION: &[BMenuItem] = &[
    fire("Orbit Left", "Numpad 4", OpKey::View3d, "orbit_left"),
    fire("Orbit Right", "Numpad 6", OpKey::View3d, "orbit_right"),
    fire("Orbit Up", "Numpad 8", OpKey::View3d, "orbit_up"),
    fire("Orbit Down", "Numpad 2", OpKey::View3d, "orbit_down"),
    SEP,
    fire("Zoom In", "Numpad +", OpKey::View3d, "zoom_in"),
    fire("Zoom Out", "Numpad -", OpKey::View3d, "zoom_out"),
];
pub static VIEW3D_VIEW: &[BMenuItem] = &[
    op("Toolbar", "T"),
    // `space_data.show_region_ui`: the sidebar region beside the tab strip.
    tog_keyed("Sidebar", "N", crate::v2::part::sidebar::show::BSidebarShown::view3d_key),
    SEP,
    fire("Frame Selected", "Numpad .", OpKey::View3d, "frame_selected"),
    fire("Frame All", "Home", OpKey::View3d, "frame_all"),
    fire("Perspective/Orthographic", "Numpad 5", OpKey::View3d, "persportho"),
    SEP,
    sub("Viewpoint", VIEW3D_VIEWPOINT),
    sub("Navigation", VIEW3D_NAVIGATION),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
/// The node editors' View menu (`NODE_MT_view`).
pub static EDITOR_VIEW: &[BMenuItem] = &[
    op("Frame All", "Home"),
    op("Frame Selected", "Numpad ."),
    SEP,
    op("Toggle Maximize Area", "Ctrl Space"),
];
pub static SELECT_ITEMS: &[BMenuItem] = &[
    fire("All", "A", OpKey::Select, "all"),
    fire("None", "Alt A", OpKey::Select, "none"),
    fire("Invert", "Ctrl I", OpKey::Select, "invert"),
    SEP,
    fire("Box Select", "B", OpKey::Select, "box"),
    fire("Circle Select", "C", OpKey::Select, "circle"),
];
/// `VIEW3D_MT_mesh_add`: the primitives phiber builds (`scene/edit.rs`).
static VIEW3D_ADD_MESH: &[BMenuItem] = &[
    fire("Plane", "", OpKey::SceneEdit, "add:plane"),
    fire("Cube", "", OpKey::SceneEdit, "add:cube"),
    fire("Circle", "", OpKey::SceneEdit, "add:circle"),
    fire("UV Sphere", "", OpKey::SceneEdit, "add:uv_sphere"),
    fire("Ico Sphere", "", OpKey::SceneEdit, "add:ico_sphere"),
    fire("Cylinder", "", OpKey::SceneEdit, "add:cylinder"),
    fire("Cone", "", OpKey::SceneEdit, "add:cone"),
    fire("Torus", "", OpKey::SceneEdit, "add:torus"),
    SEP,
    fire("Monkey", "", OpKey::SceneEdit, "add:monkey"),
];
pub static VIEW3D_ADD: &[BMenuItem] = &[
    sub("Mesh", VIEW3D_ADD_MESH),
    op("Curve", ""),
    op("Camera", ""),
    op("Light", ""),
];
static OBJECT_SHOW_HIDE: &[BMenuItem] = &[
    fire("Show Hidden Objects", "Alt H", OpKey::SceneEdit, "unhide"),
    fire("Hide Selected", "H", OpKey::SceneEdit, "hide"),
];
/// `VIEW3D_MT_transform_object`'s head: the three modal transforms.
static OBJECT_TRANSFORM: &[BMenuItem] = &[
    fire("Move", "G", OpKey::Transform, "translate"),
    fire("Rotate", "R", OpKey::Transform, "rotate"),
    fire("Scale", "S", OpKey::Transform, "resize"),
];
pub static OBJECT_ITEMS: &[BMenuItem] = &[
    sub("Transform", OBJECT_TRANSFORM),
    SEP,
    op("Shade Auto Smooth", ""),
    SEP,
    fire("Duplicate Objects", "Shift D", OpKey::SceneEdit, "duplicate"),
    fire("Delete", "X", OpKey::SceneEdit, "delete"),
    SEP,
    sub("Show/Hide", OBJECT_SHOW_HIDE),
];
pub static VIEW3D: &[BMenuSpec] = &[
    m("View", VIEW3D_VIEW),
    m("Select", SELECT_ITEMS),
    m("Add", VIEW3D_ADD),
    m("Object", OBJECT_ITEMS),
];
