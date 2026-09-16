//! Switcher constants — default switchable hosts, groups, and mode definitions.

use super::{ModeEntry, SwitcherGroup};
use crate::store::hosts::BHostType;

pub use crate::store::hosts::SWITCHABLE_HOSTS;

pub const GENERAL_HOSTS: &[BHostType] = &[
    BHostType::View3D,
    BHostType::Image,
    BHostType::Uv,
    BHostType::Compositor,
    BHostType::GeoNode,
    BHostType::Shader,
    BHostType::Seq,
    BHostType::Clip,
];

pub const ANIMATION_HOSTS: &[BHostType] = &[
    BHostType::Action,
    BHostType::Drivers,
    BHostType::Nla,
    BHostType::Time,
];

pub const SCRIPTING_HOSTS: &[BHostType] = &[
    BHostType::Text,
    BHostType::Console,
    BHostType::Info,
    BHostType::Outliner,
    BHostType::Buts,
    BHostType::Spreadsheet,
    BHostType::File,
    BHostType::Asset,
    BHostType::Prefs,
];

pub const HOST_GROUPS: &[SwitcherGroup] = &[
    SwitcherGroup {
        name: "General",
        hosts: GENERAL_HOSTS,
    },
    SwitcherGroup {
        name: "Animation",
        hosts: ANIMATION_HOSTS,
    },
    SwitcherGroup {
        name: "Scripting & Data",
        hosts: SCRIPTING_HOSTS,
    },
];

pub const VIEW3D_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Object Mode",
        icon: "box",
    },
    ModeEntry {
        label: "Edit Mode",
        icon: "pencil",
    },
    ModeEntry {
        label: "Sculpt Mode",
        icon: "sparkles",
    },
    ModeEntry {
        label: "Vertex Paint",
        icon: "brush",
    },
    ModeEntry {
        label: "Weight Paint",
        icon: "gauge",
    },
    ModeEntry {
        label: "Texture Paint",
        icon: "palette",
    },
];

pub const BUTS_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Render Properties",
        icon: "render",
    },
    ModeEntry {
        label: "Output Properties",
        icon: "output",
    },
    ModeEntry {
        label: "View Layer Properties",
        icon: "layers",
    },
    ModeEntry {
        label: "Scene Properties",
        icon: "scene",
    },
    ModeEntry {
        label: "World Properties",
        icon: "world",
    },
    ModeEntry {
        label: "Object Properties",
        icon: "object",
    },
    ModeEntry {
        label: "Modifier Properties",
        icon: "modifier",
    },
    ModeEntry {
        label: "Material Properties",
        icon: "material",
    },
    ModeEntry {
        label: "Physics Properties",
        icon: "physics",
    },
];

pub const OUTLINER_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "View Layer Hierarchy",
        icon: "list-tree",
    },
    ModeEntry {
        label: "Scenes & Collections",
        icon: "layers",
    },
    ModeEntry {
        label: "Selected Objects",
        icon: "check",
    },
    ModeEntry {
        label: "Orphan Data",
        icon: "database",
    },
];

pub const IMAGE_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "View Image",
        icon: "image",
    },
    ModeEntry {
        label: "Paint Image",
        icon: "brush",
    },
    ModeEntry {
        label: "Mask Editing",
        icon: "scissors",
    },
];

pub const UV_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "UV Unwrap & Edit",
        icon: "grid-2x2",
    },
    ModeEntry {
        label: "UV Islands & Seams",
        icon: "grid-3x3",
    },
];

pub const COMPOSITOR_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Compositing Nodes",
        icon: "layers",
    },
    ModeEntry {
        label: "Render Passes",
        icon: "image",
    },
    ModeEntry {
        label: "Color Balance",
        icon: "sun",
    },
];

pub const GEONODE_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Geometry Node Tree",
        icon: "share-2",
    },
    ModeEntry {
        label: "Modifier Node Tree",
        icon: "modifier",
    },
    ModeEntry {
        label: "Tool Node Tree",
        icon: "tool",
    },
];

pub const SHADER_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Object Shader",
        icon: "material",
    },
    ModeEntry {
        label: "World Shader",
        icon: "world",
    },
    ModeEntry {
        label: "Line Style",
        icon: "pen-tool",
    },
];

pub const SEQ_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Sequencer & Preview",
        icon: "film",
    },
    ModeEntry {
        label: "Sequencer Only",
        icon: "layers",
    },
    ModeEntry {
        label: "Preview Only",
        icon: "eye",
    },
];

pub const CLIP_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Track",
        icon: "crosshair",
    },
    ModeEntry {
        label: "Mask",
        icon: "scissors",
    },
    ModeEntry {
        label: "Solve",
        icon: "cpu",
    },
];

pub const ACTION_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Dope Sheet",
        icon: "clock",
    },
    ModeEntry {
        label: "Action Editor",
        icon: "play",
    },
    ModeEntry {
        label: "Shape Key Editor",
        icon: "sliders",
    },
    ModeEntry {
        label: "Grease Pencil",
        icon: "pencil",
    },
    ModeEntry {
        label: "Mask",
        icon: "scissors",
    },
];

pub const DRIVERS_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "F-Curve Editor",
        icon: "activity",
    },
    ModeEntry {
        label: "Drivers",
        icon: "sliders",
    },
];

pub const NLA_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Non-Linear Animation",
        icon: "layers",
    },
    ModeEntry {
        label: "Transition Tracks",
        icon: "git-merge",
    },
];

pub const TEXT_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Python Script Editor",
        icon: "code",
    },
    ModeEntry {
        label: "Documentation",
        icon: "file-text",
    },
];

pub const CONSOLE_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Python Interactive Console",
        icon: "terminal",
    },
    ModeEntry {
        label: "Command History",
        icon: "clock",
    },
];

pub const INFO_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Operator Log",
        icon: "info",
    },
    ModeEntry {
        label: "Command Execution",
        icon: "terminal",
    },
];

pub const PREFS_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Interface",
        icon: "layout",
    },
    ModeEntry {
        label: "Theme",
        icon: "palette",
    },
    ModeEntry {
        label: "Viewport",
        icon: "box",
    },
    ModeEntry {
        label: "Lights",
        icon: "sun",
    },
    ModeEntry {
        label: "Editing",
        icon: "pencil",
    },
    ModeEntry {
        label: "Animation",
        icon: "clock",
    },
    ModeEntry {
        label: "Add-ons",
        icon: "puzzle",
    },
    ModeEntry {
        label: "Input",
        icon: "mouse-pointer",
    },
    ModeEntry {
        label: "Navigation",
        icon: "navigation",
    },
    ModeEntry {
        label: "Keymap",
        icon: "keyboard",
    },
    ModeEntry {
        label: "System",
        icon: "cpu",
    },
    ModeEntry {
        label: "Save & Load",
        icon: "save",
    },
    ModeEntry {
        label: "File Paths",
        icon: "folder",
    },
    ModeEntry {
        label: "Experimental",
        icon: "sparkles",
    },
];

pub const FILE_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Open File",
        icon: "folder",
    },
    ModeEntry {
        label: "Save As",
        icon: "save",
    },
    ModeEntry {
        label: "Import Assets",
        icon: "download",
    },
    ModeEntry {
        label: "Export Assets",
        icon: "upload",
    },
    ModeEntry {
        label: "Link / Append",
        icon: "link",
    },
];

pub const ASSET_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "All Assets",
        icon: "database",
    },
    ModeEntry {
        label: "Current File",
        icon: "file",
    },
    ModeEntry {
        label: "User Library",
        icon: "bookmark",
    },
];

pub const SPREADSHEET_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Point Attributes",
        icon: "grid-2x2",
    },
    ModeEntry {
        label: "Edge Attributes",
        icon: "share-2",
    },
    ModeEntry {
        label: "Face Attributes",
        icon: "box",
    },
    ModeEntry {
        label: "Instance Attributes",
        icon: "copy",
    },
];

pub const DEFAULT_MODES: &[ModeEntry] = &[
    ModeEntry {
        label: "Default Mode",
        icon: "box",
    },
    ModeEntry {
        label: "Settings",
        icon: "sliders",
    },
];
