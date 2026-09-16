//! Per-host header menu sets - static `MenuItem` arrays with multi-level nested submenus.
//!
//! Each host type gets its own header menu tree matching Blender's editor headers.

use super::def::*;
use crate::store::menus::MenuItem;

// ── View3D View submenu ──
static VIEW3D_VIEW_REGIONS: &[MenuItem] = &[
    MenuItem::check("Camera", "camera", false),
    MenuItem::check("3D Cursor", "crosshair", false),
    MenuItem::check("Render Region", "crop", false),
    MenuItem::check("Composite", "layers", false),
];

static VIEW3D_VIEW_AREA: &[MenuItem] = &[
    MenuItem::node("Duplicate Area", "copy"),
    MenuItem::node("Quad View", "grid"),
    MenuItem::node("Horizontal Split", "split"),
    MenuItem::node("Vertical Split", "layout"),
    MenuItem::node("Join Area", "merge"),
    MenuItem::node("Swap Area", "arrow_left"),
    MenuItem::node("Toggle Fullscreen", "maximize"),
];

pub static VIEW3D_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View Node", "node"),
    MenuItem::node("Adjust Last Operation", "sliders"),
    MenuItem::separator(),
    MenuItem::node("View Regions", "eye").with_items(VIEW3D_VIEW_REGIONS),
    MenuItem::node("Render Region", "crop"),
    MenuItem::separator(),
    MenuItem::node("Area", "layout_grid").with_items(VIEW3D_VIEW_AREA),
    MenuItem::separator(),
    MenuItem::check("Toolbar", "toolbar", true),
    MenuItem::check("Sidebar", "sidebar", true),
    MenuItem::check("Status Bar", "footer", true),
];

// ── View3D Select submenu ──
pub static VIEW3D_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
    MenuItem::separator(),
    MenuItem::node("Box Select", "select_box"),
    MenuItem::node("Circle Select", "select_circle"),
    MenuItem::node("Lasso Select", "select_lasso"),
    MenuItem::separator(),
    MenuItem::node("Select Grouped", "group"),
    MenuItem::node("Select Pattern", "search"),
    MenuItem::node("Select Random", "dices"),
];

// ── View3D Add Mesh submenu ──
static VIEW3D_ADD_MESH: &[MenuItem] = &[
    MenuItem::node("Plane", "square"),
    MenuItem::node("Cube", "cube"),
    MenuItem::node("Circle", "circle"),
    MenuItem::node("UV Sphere", "sphere"),
    MenuItem::node("Icosphere", "blob"),
    MenuItem::node("Cylinder", "cylinder"),
    MenuItem::node("Cone", "cone"),
    MenuItem::node("Torus", "torus"),
];

// ── View3D Add Curve submenu ──
static VIEW3D_ADD_CURVE: &[MenuItem] = &[
    MenuItem::node("Bezier Curve", "pen_tool"),
    MenuItem::node("Circle", "circle"),
    MenuItem::node("Nurbs Curve", "spline"),
    MenuItem::node("Path", "route"),
];

pub static VIEW3D_ADD_ITEMS: &[MenuItem] = &[
    MenuItem::node("Mesh", "cube").with_items(VIEW3D_ADD_MESH),
    MenuItem::node("Curve", "pen_tool").with_items(VIEW3D_ADD_CURVE),
    MenuItem::node("Surface", "surface"),
    MenuItem::node("Metaball", "blob"),
    MenuItem::node("Text", "type"),
    MenuItem::node("Armature", "bone"),
    MenuItem::node("Empty", "circle_off"),
];

// ── View3D Object submenu ──
static VIEW3D_OBJECT_TRANSFORM: &[MenuItem] = &[
    MenuItem::node("Grab", "move"),
    MenuItem::node("Rotate", "rotate"),
    MenuItem::node("Scale", "scale"),
    MenuItem::node("Transform", "transform"),
];

static VIEW3D_OBJECT_SHADING: &[MenuItem] = &[
    MenuItem::node("Smooth Shading", "smooth"),
    MenuItem::node("Flat Shading", "faceted"),
    MenuItem::node("Auto Smooth", "settings"),
];

pub static VIEW3D_OBJECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("Transform", "transform").with_items(VIEW3D_OBJECT_TRANSFORM),
    MenuItem::node("Apply", "check"),
    MenuItem::node("Parent", "link"),
    MenuItem::node("Shading", "material").with_items(VIEW3D_OBJECT_SHADING),
    MenuItem::node("Duplicate", "copy"),
    MenuItem::node("Delete", "trash"),
];

// ── View3D Header Menu Tree ──
pub static VIEW3D_ITEMS: &[MenuItem] = &[
    MenuItem::node(VIEW3D_VIEW, VIEW3D_VIEW_ICON).with_items(VIEW3D_VIEW_ITEMS),
    MenuItem::node(VIEW3D_SELECT, VIEW3D_SELECT_ICON).with_items(VIEW3D_SELECT_ITEMS),
    MenuItem::node(VIEW3D_ADD, VIEW3D_ADD_ICON).with_items(VIEW3D_ADD_ITEMS),
    MenuItem::node(VIEW3D_OBJECT, VIEW3D_OBJECT_ICON).with_items(VIEW3D_OBJECT_ITEMS),
];

// ── Outliner Multi-Level Submenus ──
static OUTLINER_VIEW_REGIONS: &[MenuItem] = &[
    MenuItem::check("Filter Panel", "filter", true),
    MenuItem::check("Search Bar", "search", true),
];

static OUTLINER_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("Expand All", "maximize-2"),
    MenuItem::node("Collapse All", "minimize-2"),
    MenuItem::separator(),
    MenuItem::node("View Regions", "eye").with_items(OUTLINER_VIEW_REGIONS),
];

static OUTLINER_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
    MenuItem::separator(),
    MenuItem::node("Select Hierarchy", "folder-tree"),
];

static OUTLINER_OBJECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("Hide Selected", "eye-off"),
    MenuItem::node("Show Hidden", "eye"),
    MenuItem::separator(),
    MenuItem::node("Delete", "trash"),
];

static OUTLINER_COLLECTION_ITEMS: &[MenuItem] = &[
    MenuItem::node("New Collection", "folder-plus"),
    MenuItem::node("Duplicate Collection", "copy"),
    MenuItem::node("Delete Collection", "trash"),
];

pub static OUTLINER_ITEMS: &[MenuItem] = &[
    MenuItem::node(OUTLINER_VIEW, OUTLINER_VIEW_ICON).with_items(OUTLINER_VIEW_ITEMS),
    MenuItem::node(OUTLINER_SELECT, OUTLINER_SELECT_ICON).with_items(OUTLINER_SELECT_ITEMS),
    MenuItem::node(OUTLINER_OBJECT, OUTLINER_OBJECT_ICON).with_items(OUTLINER_OBJECT_ITEMS),
    MenuItem::node(OUTLINER_COLLECTION, OUTLINER_COLLECTION_ICON)
        .with_items(OUTLINER_COLLECTION_ITEMS),
];

// ── Properties (Buts) Multi-Level Submenus ──
static BUTS_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("Adjust Last Operation", "sliders"),
    MenuItem::separator(),
    MenuItem::check("Sidebar", "sidebar", true),
];

static BUTS_OBJECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("Copy To Selected", "copy"),
    MenuItem::node("Paste To Selected", "clipboard"),
];

static BUTS_EDIT_ITEMS: &[MenuItem] = &[MenuItem::node("Preferences...", "settings")];

pub static BUTS_ITEMS: &[MenuItem] = &[
    MenuItem::node(BUTS_VIEW, BUTS_VIEW_ICON).with_items(BUTS_VIEW_ITEMS),
    MenuItem::node(BUTS_OBJECT, BUTS_OBJECT_ICON).with_items(BUTS_OBJECT_ITEMS),
    MenuItem::node(BUTS_EDIT, BUTS_EDIT_ICON).with_items(BUTS_EDIT_ITEMS),
];

// ── Animation Editors Multi-Level Submenus ──
static ANIM_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View All", "maximize-2"),
    MenuItem::node("Frame Selected", "target"),
    MenuItem::separator(),
    MenuItem::check("Play Animation", "play", false),
];

static ANIM_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
    MenuItem::separator(),
    MenuItem::node("Box Select", "select_box"),
];

static ANIM_MARKER_ITEMS: &[MenuItem] = &[
    MenuItem::node("Add Marker", "bookmark-plus"),
    MenuItem::node("Duplicate Marker", "copy"),
    MenuItem::node("Delete Marker", "trash"),
];

static ANIM_CHANNEL_ITEMS: &[MenuItem] = &[
    MenuItem::node("Group Channels", "folder"),
    MenuItem::node("Ungroup Channels", "folder-minus"),
    MenuItem::separator(),
    MenuItem::node("Hide Selected", "eye-off"),
];

static ANIM_KEY_ITEMS: &[MenuItem] = &[
    MenuItem::node("Insert Keyframe", "key"),
    MenuItem::node("Delete Keyframe", "trash"),
    MenuItem::separator(),
    MenuItem::node("Interpolation Mode", "trending-up"),
];

pub static ANIM_ITEMS: &[MenuItem] = &[
    MenuItem::node(ANIM_VIEW, ANIM_VIEW_ICON).with_items(ANIM_VIEW_ITEMS),
    MenuItem::node(ANIM_SELECT, ANIM_SELECT_ICON).with_items(ANIM_SELECT_ITEMS),
    MenuItem::node(ANIM_MARKER, ANIM_MARKER_ICON).with_items(ANIM_MARKER_ITEMS),
    MenuItem::node(ANIM_CHANNEL, ANIM_CHANNEL_ICON).with_items(ANIM_CHANNEL_ITEMS),
    MenuItem::node(ANIM_KEY, ANIM_KEY_ICON).with_items(ANIM_KEY_ITEMS),
];

// ── Node Editors Multi-Level Submenus ──
static NODE_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View All", "maximize-2"),
    MenuItem::node("Frame Selected", "target"),
    MenuItem::separator(),
    MenuItem::check("Backdrop", "image", true),
];

static NODE_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
    MenuItem::separator(),
    MenuItem::node("Select Linked", "link"),
];

static NODE_ADD_INPUT: &[MenuItem] = &[
    MenuItem::node("RGB", "palette"),
    MenuItem::node("Value", "sliders"),
    MenuItem::node("Texture Coordinate", "grid"),
];

static NODE_ADD_ITEMS: &[MenuItem] = &[
    MenuItem::node("Input", "log-in").with_items(NODE_ADD_INPUT),
    MenuItem::node("Output", "log-out"),
    MenuItem::node("Shader", "circle"),
    MenuItem::node("Color", "palette"),
    MenuItem::node("Vector", "navigation"),
    MenuItem::node("Converter", "cpu"),
];

static NODE_NODE_ITEMS: &[MenuItem] = &[
    MenuItem::node("Make Group", "folder-plus"),
    MenuItem::node("Ungroup", "folder-minus"),
    MenuItem::separator(),
    MenuItem::node("Mute", "volume-x"),
    MenuItem::node("Duplicate", "copy"),
    MenuItem::node("Delete", "trash"),
];

pub static NODE_ITEMS: &[MenuItem] = &[
    MenuItem::node(NODE_VIEW, NODE_VIEW_ICON).with_items(NODE_VIEW_ITEMS),
    MenuItem::node(NODE_SELECT, NODE_SELECT_ICON).with_items(NODE_SELECT_ITEMS),
    MenuItem::node(NODE_ADD, NODE_ADD_ICON).with_items(NODE_ADD_ITEMS),
    MenuItem::node(NODE_NODE, NODE_NODE_ICON).with_items(NODE_NODE_ITEMS),
];

// ── Image / UV Editor Multi-Level Submenus ──
static IMAGE_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View All", "maximize-2"),
    MenuItem::node("Zoom In", "zoom-in"),
    MenuItem::node("Zoom Out", "zoom-out"),
];

static IMAGE_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
];

static IMAGE_IMAGE_ITEMS: &[MenuItem] = &[
    MenuItem::node("New", "file-plus"),
    MenuItem::node("Open...", "folder-open"),
    MenuItem::node("Save", "save"),
    MenuItem::node("Save As...", "save-all"),
];

static IMAGE_UV_ITEMS: &[MenuItem] = &[
    MenuItem::node("Unwrap", "box"),
    MenuItem::node("Smart UV Project", "cpu"),
    MenuItem::node("Pack Islands", "package"),
];

pub static IMAGE_ITEMS: &[MenuItem] = &[
    MenuItem::node(IMAGE_VIEW, IMAGE_VIEW_ICON).with_items(IMAGE_VIEW_ITEMS),
    MenuItem::node(IMAGE_SELECT, IMAGE_SELECT_ICON).with_items(IMAGE_SELECT_ITEMS),
    MenuItem::node(IMAGE_IMAGE, IMAGE_IMAGE_ICON).with_items(IMAGE_IMAGE_ITEMS),
    MenuItem::node(IMAGE_UV, IMAGE_UV_ICON).with_items(IMAGE_UV_ITEMS),
];

// ── Text / Console / Info Multi-Level Submenus ──
static TEXT_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("Top of File", "arrow-up"),
    MenuItem::node("Bottom of File", "arrow-down"),
    MenuItem::separator(),
    MenuItem::check("Line Numbers", "list", true),
    MenuItem::check("Word Wrap", "align-left", true),
];

static TEXT_EDIT_ITEMS: &[MenuItem] = &[
    MenuItem::node("Undo", "rotate-ccw"),
    MenuItem::node("Redo", "rotate-cw"),
    MenuItem::separator(),
    MenuItem::node("Cut", "scissors"),
    MenuItem::node("Copy", "copy"),
    MenuItem::node("Paste", "clipboard"),
];

static TEXT_FORMAT_ITEMS: &[MenuItem] = &[
    MenuItem::node("Indent", "indent"),
    MenuItem::node("Unindent", "outdent"),
    MenuItem::node("Toggle Comment", "code"),
];

pub static TEXT_ITEMS: &[MenuItem] = &[
    MenuItem::node(TEXT_VIEW, TEXT_VIEW_ICON).with_items(TEXT_VIEW_ITEMS),
    MenuItem::node(TEXT_EDIT, TEXT_EDIT_ICON).with_items(TEXT_EDIT_ITEMS),
    MenuItem::node(TEXT_FORMAT, TEXT_FORMAT_ICON).with_items(TEXT_FORMAT_ITEMS),
];

// ── File / Asset Browser Submenus ──
static FILE_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("Sort By Name", "sort-asc"),
    MenuItem::node("Sort By Date", "calendar"),
    MenuItem::node("Sort By Size", "hard-drive"),
];

static FILE_SELECT_ITEMS: &[MenuItem] = &[
    MenuItem::node("All", "check"),
    MenuItem::node("None", "x"),
    MenuItem::node("Invert", "refresh_cw"),
];

pub static FILE_ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_VIEW, FILE_VIEW_ICON).with_items(FILE_VIEW_ITEMS),
    MenuItem::node(FILE_SELECT, FILE_SELECT_ICON).with_items(FILE_SELECT_ITEMS),
];

// ── Video Sequencer Submenus ──
static SEQ_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View All", "maximize-2"),
    MenuItem::node("Zoom In", "zoom-in"),
    MenuItem::node("Zoom Out", "zoom-out"),
];

static SEQ_SELECT_ITEMS: &[MenuItem] =
    &[MenuItem::node("All", "check"), MenuItem::node("None", "x")];

static SEQ_ADD_ITEMS: &[MenuItem] = &[
    MenuItem::node("Movie", "film"),
    MenuItem::node("Sound", "music"),
    MenuItem::node("Image/Sequence", "image"),
];

static SEQ_STRIP_ITEMS: &[MenuItem] = &[
    MenuItem::node("Cut Strips", "scissors"),
    MenuItem::node("Duplicate Strips", "copy"),
    MenuItem::node("Delete Strips", "trash"),
];

pub static SEQ_ITEMS: &[MenuItem] = &[
    MenuItem::node(SEQ_VIEW, SEQ_VIEW_ICON).with_items(SEQ_VIEW_ITEMS),
    MenuItem::node(SEQ_SELECT, SEQ_SELECT_ICON).with_items(SEQ_SELECT_ITEMS),
    MenuItem::node(SEQ_ADD, SEQ_ADD_ICON).with_items(SEQ_ADD_ITEMS),
    MenuItem::node(SEQ_STRIP, SEQ_STRIP_ICON).with_items(SEQ_STRIP_ITEMS),
];

// ── Movie Clip Editor Submenus ──
static CLIP_VIEW_ITEMS: &[MenuItem] = &[
    MenuItem::node("View All", "maximize-2"),
    MenuItem::node("Zoom In", "zoom-in"),
];

static CLIP_SELECT_ITEMS: &[MenuItem] =
    &[MenuItem::node("All", "check"), MenuItem::node("None", "x")];

static CLIP_CLIP_ITEMS: &[MenuItem] = &[
    MenuItem::node("Open Clip", "folder-open"),
    MenuItem::node("Reload Clip", "refresh-cw"),
];

static CLIP_TRACKING_ITEMS: &[MenuItem] = &[
    MenuItem::node("Add Track Marker", "plus"),
    MenuItem::node("Track Forward", "play"),
    MenuItem::node("Track Backwards", "rewind"),
];

pub static CLIP_ITEMS: &[MenuItem] = &[
    MenuItem::node(CLIP_VIEW, CLIP_VIEW_ICON).with_items(CLIP_VIEW_ITEMS),
    MenuItem::node(CLIP_SELECT, CLIP_SELECT_ICON).with_items(CLIP_SELECT_ITEMS),
    MenuItem::node(CLIP_CLIP, CLIP_CLIP_ICON).with_items(CLIP_CLIP_ITEMS),
    MenuItem::node(CLIP_TRACKING, CLIP_TRACKING_ICON).with_items(CLIP_TRACKING_ITEMS),
];

// ── Default ──
pub static DEFAULT_ITEMS: &[MenuItem] = &[MenuItem::node(DEFAULT_VIEW, DEFAULT_VIEW_ICON)];
