//! The manual's hierarchy, whole, as one declarative set.
//!
//! Every page phiber's manual will have is here from the start, most of
//! them `Stub`. The order is Blender's own chapter order
//! (`docs/blender-manual/manual/index.rst`) so the two can be read side by
//! side, and each node names the Blender page it answers to.
//!
//! A node with `blender: ""` is phiber's own — a page Blender has no
//! counterpart for. There are few, and each says why in its title.

use super::BManualState::{Drafted, Shot, Stub};
use super::{BManualFigure, BManualNode, BManualState};
use crate::v2::host::BHostType;
use crate::v2::space::BSpaceType;

/// A page with no figure.
const fn page(slug: &'static str, title: &'static str, blender: &'static str, state: BManualState) -> BManualNode {
    BManualNode { slug, title, blender, figure: None, state, children: &[] }
}

/// A page whose figure is `space` with `host` in focus.
const fn shot(
    slug: &'static str,
    title: &'static str,
    blender: &'static str,
    space: BSpaceType,
    host: BHostType,
    state: BManualState,
) -> BManualNode {
    BManualNode { slug, title, blender, figure: Some(BManualFigure { space, host }), state, children: &[] }
}

/// A chapter or section: a page that also holds others.
const fn part(slug: &'static str, title: &'static str, blender: &'static str, children: &'static [BManualNode]) -> BManualNode {
    BManualNode { slug, title, blender, figure: None, state: Stub, children }
}

// ---------------------------------------------------------------- interface

static WINDOW_SYSTEM: &[BManualNode] = &[
    shot("introduction", "Window System", "interface/window_system/introduction.rst", BSpaceType::Layout, BHostType::Topbar, Stub),
    shot("areas", "Areas", "interface/window_system/areas.rst", BSpaceType::Layout, BHostType::View3D, Stub),
    shot("regions", "Regions", "interface/window_system/regions.rst", BSpaceType::Layout, BHostType::View3D, Stub),
    shot("tabs_panels", "Tabs & Panels", "interface/window_system/tabs_panels.rst", BSpaceType::Layout, BHostType::Buts, Stub),
    shot("topbar", "Topbar", "interface/window_system/topbar.rst", BSpaceType::Layout, BHostType::Topbar, Stub),
    shot("status_bar", "Status Bar", "interface/window_system/status_bar.rst", BSpaceType::Layout, BHostType::Statusbar, Stub),
    shot("workspaces", "Workspaces", "interface/window_system/workspaces.rst", BSpaceType::Layout, BHostType::Topbar, Stub),
    page("splash", "Splash Screen", "interface/window_system/splash.rst", Stub),
];

static INTERFACE: &[BManualNode] = &[
    part("window_system", "Window System", "interface/window_system/index.rst", WINDOW_SYSTEM),
    page("controls", "Buttons & Controls", "interface/controls/index.rst", Stub),
    page("keymap", "Keymap", "interface/keymap/index.rst", Stub),
    page("operators", "Operators", "interface/operators.rst", Stub),
    page("selecting", "Selecting", "interface/selecting.rst", Stub),
    page("undo_redo", "Undo & Redo", "interface/undo_redo.rst", Stub),
    page("tool_system", "Tool System", "interface/tool_system.rst", Stub),
    page("annotate_tool", "Annotations", "interface/annotate_tool.rst", Stub),
];

// ------------------------------------------------------------------ editors

static EDITORS: &[BManualNode] = &[
    shot("3dview", "3D Viewport", "editors/3dview/index.rst", BSpaceType::Layout, BHostType::View3D, Stub),
    shot("image", "Image Editor", "editors/image/index.rst", BSpaceType::Rendering, BHostType::Image, Drafted),
    shot("outliner", "Outliner", "editors/outliner/index.rst", BSpaceType::Layout, BHostType::Outliner, Stub),
    shot("properties", "Properties", "editors/properties_editor.rst", BSpaceType::Layout, BHostType::Buts, Stub),
    shot("timeline", "Timeline", "editors/timeline.rst", BSpaceType::Animation, BHostType::Time, Drafted),
    shot("dope_sheet", "Dope Sheet", "editors/dope_sheet/index.rst", BSpaceType::Animation, BHostType::Action, Drafted),
    shot("graph_editor", "Graph Editor", "editors/graph_editor/index.rst", BSpaceType::Animation, BHostType::Ipo, Stub),
    shot("drivers_editor", "Drivers Editor", "editors/drivers_editor.rst", BSpaceType::Animation, BHostType::Drivers, Stub),
    shot("nla", "Nonlinear Animation", "editors/nla/index.rst", BSpaceType::Animation, BHostType::Nla, Stub),
    shot("shader_editor", "Shader Editor", "editors/shader_editor.rst", BSpaceType::Shading, BHostType::Shader, Stub),
    shot("geometry_node", "Geometry Node Editor", "editors/geometry_node.rst", BSpaceType::GeometryNodes, BHostType::GeoNode, Stub),
    shot("compositor", "Compositor", "editors/compositor.rst", BSpaceType::Compositing, BHostType::Compositor, Stub),
    shot("spreadsheet", "Spreadsheet", "editors/spreadsheet.rst", BSpaceType::GeometryNodes, BHostType::Spreadsheet, Stub),
    shot("uv", "UV Editor", "editors/uv/index.rst", BSpaceType::UvEditing, BHostType::Uv, Stub),
    shot("info_editor", "Info Editor", "editors/info_editor.rst", BSpaceType::Scripting, BHostType::Info, Stub),
    shot("text_editor", "Text Editor", "editors/text_editor.rst", BSpaceType::Scripting, BHostType::Text, Stub),
    shot("file_browser", "File Browser", "editors/file_browser.rst", BSpaceType::Layout, BHostType::File, Stub),
    shot("asset_browser", "Asset Browser", "editors/asset_browser.rst", BSpaceType::Layout, BHostType::Asset, Stub),
    shot("preferences", "Preferences", "editors/preferences/index.rst", BSpaceType::Layout, BHostType::Prefs, Stub),
    shot("clip", "Movie Clip Editor", "editors/clip/index.rst", BSpaceType::Layout, BHostType::Clip, Stub),
    shot("video_sequencer", "Video Sequencer", "video_editing/index.rst", BSpaceType::Layout, BHostType::Seq, Stub),
];

// ------------------------------------------------------------- scene layout

static SCENE_LAYOUT: &[BManualNode] = &[
    shot("object", "Objects", "scene_layout/object/index.rst", BSpaceType::Layout, BHostType::View3D, Stub),
    shot("collections", "Collections", "scene_layout/collections/index.rst", BSpaceType::Layout, BHostType::Outliner, Stub),
    page("scene", "Scenes & View Layers", "scene_layout/scene/index.rst", Stub),
];

// --------------------------------------------------------------- animation

static ANIM_EDITORS: &[BManualNode] = &[
    shot("timeline", "The Timeline", "editors/timeline.rst", BSpaceType::Animation, BHostType::Time, Drafted),
    shot("dope_sheet", "The Dope Sheet", "editors/dope_sheet/introduction.rst", BSpaceType::Animation, BHostType::Action, Drafted),
    shot("graph", "The Graph Editor", "editors/graph_editor/introduction.rst", BSpaceType::Animation, BHostType::Ipo, Stub),
    shot("nla", "The NLA Editor", "editors/nla/introduction.rst", BSpaceType::Animation, BHostType::Nla, Stub),
];

static ANIMATION: &[BManualNode] = &[
    page("introduction", "Introduction", "animation/introduction.rst", Drafted),
    page("keyframes", "Keyframes", "animation/keyframes/index.rst", Stub),
    page("actions", "Actions", "animation/actions.rst", Stub),
    page("armatures", "Armatures", "animation/armatures/index.rst", Stub),
    page("constraints", "Constraints", "animation/constraints/index.rst", Stub),
    page("drivers", "Drivers", "animation/drivers/index.rst", Stub),
    page("markers", "Markers", "animation/markers.rst", Drafted),
    page("motion_paths", "Motion Paths", "animation/motion_paths.rst", Stub),
    page("shape_keys", "Shape Keys", "animation/shape_keys/index.rst", Stub),
    part("animation_editors", "Animation Editors", "animation/animation_editors.rst", ANIM_EDITORS),
];

// ------------------------------------------------------------------ render

static RENDER_OUTPUT: &[BManualNode] = &[
    page("introduction", "Output Introduction", "render/output/introduction.rst", Drafted),
    shot("properties", "Output Properties", "render/output/properties/index.rst", BSpaceType::Rendering, BHostType::Buts, Drafted),
    page("animation", "Rendering Animations", "render/output/animation.rst", Drafted),
    page("file_formats", "File Formats", "render/output/properties/output.rst", Stub),
];

static RENDER_ENGINES: &[BManualNode] = &[
    // phiber's own: Blender has no page for an engine it does not ship.
    shot("phiray", "Phiray — phiber's path tracer", "", BSpaceType::Rendering, BHostType::Buts, Drafted),
    page("cycles", "Cycles (read as Phiray)", "render/cycles/index.rst", Stub),
    page("eevee", "EEVEE (read as Phiray)", "render/eevee/index.rst", Stub),
    page("workbench", "Workbench", "render/workbench/index.rst", Stub),
];

static RENDER: &[BManualNode] = &[
    page("introduction", "Introduction", "render/introduction.rst", Drafted),
    shot("render_image", "Rendering an Image", "render/output/introduction.rst", BSpaceType::Rendering, BHostType::Image, Shot),
    part("engines", "Render Engines", "render/render_engines.rst", RENDER_ENGINES),
    part("output", "Output", "render/output/index.rst", RENDER_OUTPUT),
    page("cameras", "Cameras", "render/cameras.rst", Stub),
    page("lights", "Lights", "render/lights/index.rst", Stub),
    page("materials", "Materials", "render/materials/index.rst", Stub),
    page("layers", "View Layers & Passes", "render/layers/index.rst", Stub),
    page("color_management", "Color Management", "render/color_management/index.rst", Stub),
    page("shader_nodes", "Shader Nodes", "render/shader_nodes/index.rst", Stub),
    page("freestyle", "Freestyle", "render/freestyle/index.rst", Stub),
];

// ------------------------------------------------------------------- files

static FILES: &[BManualNode] = &[
    page("blend_files", "Blend Files", "files/blend/index.rst", Stub),
    page("data_blocks", "Data-Blocks", "files/data_blocks.rst", Stub),
    page("linked_libraries", "Linked Libraries", "files/linked_libraries/index.rst", Stub),
    page("import_export", "Import & Export", "files/import_export.rst", Stub),
    page("media", "Media & Textures", "files/media/index.rst", Stub),
];

// ----------------------------------------------------------------- chapters

/// The manual, whole.
pub static MANUAL: &[BManualNode] = &[
    part("getting_started", "Getting Started", "getting_started/index.rst", &[
        page("about", "About phiber", "getting_started/about/index.rst", Stub),
        page("installing", "Installing", "getting_started/installing/index.rst", Stub),
        page("configuration", "Configuration", "getting_started/configuration/index.rst", Stub),
    ]),
    part("interface", "User Interface", "interface/index.rst", INTERFACE),
    part("editors", "Editors", "editors/index.rst", EDITORS),
    part("scene_layout", "Scene Layout", "scene_layout/index.rst", SCENE_LAYOUT),
    part("modeling", "Modeling", "modeling/index.rst", &[
        page("meshes", "Meshes", "modeling/meshes/index.rst", Stub),
        page("modifiers", "Modifiers", "modeling/modifiers/index.rst", Stub),
        page("geometry_nodes", "Geometry Nodes", "modeling/geometry_nodes/index.rst", Stub),
    ]),
    part("animation", "Animation & Rigging", "animation/index.rst", ANIMATION),
    part("render", "Rendering", "render/index.rst", RENDER),
    part("compositing", "Compositing", "compositing/index.rst", &[
        page("introduction", "Introduction", "compositing/introduction.rst", Stub),
        page("types", "Node Types", "compositing/types/index.rst", Stub),
    ]),
    part("files", "Files & Data System", "files/index.rst", FILES),
    part("troubleshooting", "Troubleshooting", "troubleshooting/index.rst", &[
        page("gpu", "GPU & Drivers", "troubleshooting/gpu/index.rst", Stub),
        page("crash", "Crashes", "troubleshooting/crash.rst", Stub),
        // phiber's own: the journal is not a Blender concept.
        page("journal", "The Session Journal", "", Drafted),
    ]),
];
