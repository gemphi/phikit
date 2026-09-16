//! Custom render-key constants.

pub const RENDER_KEY_VIEWPORT_3D: &str = "viewport_3d";
pub const RENDER_KEY_OUTLINER_TREE: &str = "outliner_tree";
pub const RENDER_KEY_PROPERTY_PANELS: &str = "property_panels";
pub const RENDER_KEY_TIMELINE_RULER: &str = "timeline_ruler";
pub const RENDER_KEY_DOPESHEET_GRID: &str = "dopesheet_grid";
pub const RENDER_KEY_NLA_TRACKS: &str = "nla_tracks";
pub const RENDER_KEY_SEQUENCER_STRIPS: &str = "sequencer_strips";
pub const RENDER_KEY_NODE_GRAPH: &str = "node_graph";
pub const RENDER_KEY_IMAGE_VIEW: &str = "image_view";
pub const RENDER_KEY_TEXT_EDITOR: &str = "text_editor";
pub const RENDER_KEY_PYTHON_CONSOLE: &str = "python_console";
pub const RENDER_KEY_CLIP_VIEW: &str = "clip_view";
pub const RENDER_KEY_SPREADSHEET_TABLE: &str = "spreadsheet_table";
pub const RENDER_KEY_ASSET_GRID: &str = "asset_grid";
pub const RENDER_KEY_FILE_LIST: &str = "file_list";
pub const RENDER_KEY_INFO_LOG: &str = "info_log";
pub const RENDER_KEY_TOPBAR: &str = "topbar";
pub const RENDER_KEY_STATUSBAR: &str = "statusbar";
pub const RENDER_KEY_EMPTY: &str = "empty";

/// Static content node for a host's center canvas — pure data.
/// The `CustomView` leaf loops these and matches them to `BView` primitives;
/// no per-host view code lives outside the store.
#[derive(Clone, Copy, Debug)]
pub enum BContent {
    Empty,
    /// 3D viewport canvas: scene name, object count, vertex count.
    Spatial {
        name: &'static str,
        objects: usize,
        vertices: usize,
    },
    Label {
        text: &'static str,
        icon: Option<&'static str>,
    },
    PropRow {
        label: &'static str,
        value: &'static str,
    },
    Item {
        label: &'static str,
        icon: &'static str,
        active: bool,
    },
    Section {
        label: &'static str,
        icon: Option<&'static str>,
        expanded: bool,
        items: &'static [BContent],
    },
    Separator,
    Scroll(&'static [BContent]),
    Column(&'static [BContent]),
    /// Horizontal row of nodes (e.g. file list columns: name | date | size).
    Row(&'static [BContent]),
}
