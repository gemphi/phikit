//! Host constants - labels, icons, DNA space types, and space specs.
//!
//! Matches `DNA_space_types.h` and `DNA_space_enums.h` from Blender 5.2.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHostType {
    Empty,
    View3D,
    Outliner,
    Buts, // Properties
    File,
    Image,
    Info,
    Seq,
    Text,
    Action,
    Nla,
    Time,
    Node,
    Logic,
    Console,
    Uv,
    Compositor,
    Shader,
    GeoNode,
    Clip,
    Drivers,
    Asset,
    Spreadsheet,
    Ipo,
    Prefs,
    Topbar,
    Statusbar,
}

// Per-host labels - keep in the same order as `BHostType` for sanity.
pub const EMPTY_LABEL: &str = "Empty";
pub const VIEW3D_LABEL: &str = "3D Viewport";
pub const OUTLINER_LABEL: &str = "Outliner";
pub const BUTS_LABEL: &str = "Properties";
pub const FILE_LABEL: &str = "File Browser";
pub const IMAGE_LABEL: &str = "Image Editor";
pub const INFO_LABEL: &str = "Info";
pub const SEQ_LABEL: &str = "Video Sequencer";
pub const TEXT_LABEL: &str = "Text Editor";
pub const ACTION_LABEL: &str = "Dope Sheet";
pub const NLA_LABEL: &str = "Non-Linear Animation";
pub const TIME_LABEL: &str = "Timeline";
pub const NODE_LABEL: &str = "Node Editor";
pub const LOGIC_LABEL: &str = "Logic Editor";
pub const CONSOLE_LABEL: &str = "Python Console";
pub const UV_LABEL: &str = "UV Editor";
pub const COMPOSITOR_LABEL: &str = "Compositor";
pub const SHADER_LABEL: &str = "Shader Editor";
pub const GEONODE_LABEL: &str = "Geometry Nodes";
pub const CLIP_LABEL: &str = "Clip Editor";
pub const DRIVERS_LABEL: &str = "Drivers";
pub const ASSET_LABEL: &str = "Asset Browser";
pub const SPREADSHEET_LABEL: &str = "Spreadsheet";
pub const IPO_LABEL: &str = "Graph Editor";
pub const PREFS_LABEL: &str = "Preferences";
pub const TOPBAR_LABEL: &str = "Top Bar";
pub const STATUSBAR_LABEL: &str = "Status Bar";

// Per-host icons.
pub const EMPTY_ICON: &str = "empty";
pub const VIEW3D_ICON: &str = "view3d";
pub const OUTLINER_ICON: &str = "outliner";
pub const BUTS_ICON: &str = "properties";
pub const FILE_ICON: &str = "file";
pub const IMAGE_ICON: &str = "image";
pub const INFO_ICON: &str = "info";
pub const SEQ_ICON: &str = "seq";
pub const TEXT_ICON: &str = "text";
pub const ACTION_ICON: &str = "action";
pub const NLA_ICON: &str = "nla";
pub const TIME_ICON: &str = "time";
pub const NODE_ICON: &str = "node";
pub const LOGIC_ICON: &str = "logic";
pub const CONSOLE_ICON: &str = "console";
pub const UV_ICON: &str = "uv";
pub const COMPOSITOR_ICON: &str = "compositor";
pub const SHADER_ICON: &str = "shader";
pub const GEONODE_ICON: &str = "geonode";
pub const CLIP_ICON: &str = "clip";
pub const DRIVERS_ICON: &str = "drivers";
pub const ASSET_ICON: &str = "asset";
pub const SPREADSHEET_ICON: &str = "spreadsheet";
pub const IPO_ICON: &str = "ipo";
pub const PREFS_ICON: &str = "prefs";
pub const TOPBAR_ICON: &str = "topbar";
pub const STATUSBAR_ICON: &str = "statusbar";

impl BHostType {
    /// Blender 5.2 DNA `eSpace_Type` code.
    pub fn dna_space_type(self) -> u8 {
        match self {
            BHostType::Empty => 0,
            BHostType::View3D => 1,
            BHostType::Ipo | BHostType::Drivers => 2,
            BHostType::Outliner => 3,
            BHostType::Buts => 4,
            BHostType::File | BHostType::Asset => 5,
            BHostType::Image | BHostType::Uv => 6,
            BHostType::Info => 7,
            BHostType::Seq => 8,
            BHostType::Text => 9,
            BHostType::Action | BHostType::Time => 12,
            BHostType::Nla => 13,
            BHostType::Node | BHostType::Shader | BHostType::GeoNode | BHostType::Compositor => 16,
            BHostType::Logic => 17,
            BHostType::Console => 18,
            BHostType::Prefs => 19,
            BHostType::Clip => 20,
            BHostType::Topbar => 21,
            BHostType::Statusbar => 22,
            BHostType::Spreadsheet => 23,
        }
    }
}
