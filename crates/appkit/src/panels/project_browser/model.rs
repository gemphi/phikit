//! Project Browser model — data types for the 3-pane file browser.

/// A folder entry in the navigator tree.
#[derive(Debug, Clone)]
pub struct NSProjectFolder {
    pub name: String,
    pub path: String,
    pub children: Vec<NSProjectFolder>,
    pub expanded: bool,
}

/// A file entry in the file grid.
#[derive(Debug, Clone)]
pub struct NSProjectFile {
    pub name: String,
    pub path: String,
    pub file_type: NSFileType,
    pub size_bytes: u64,
}

/// File type categories for icon display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSFileType {
    Mesh,
    Material,
    Texture,
    Audio,
    Scene,
    Script,
    Folder,
    Unknown,
}

impl NSFileType {
    /// Display label for the file type.
    pub fn label(self) -> &'static str {
        match self {
            Self::Mesh => "Mesh",
            Self::Material => "Material",
            Self::Texture => "Texture",
            Self::Audio => "Audio",
            Self::Scene => "Scene",
            Self::Script => "Script",
            Self::Folder => "Folder",
            Self::Unknown => "File",
        }
    }
}

/// File inspector metadata shown in the right pane.
#[derive(Debug, Clone, Default)]
pub struct NSFileInspectorData {
    pub name: String,
    pub file_type: String,
    pub size: String,
    pub path: String,
    pub modified: String,
}

/// Full project browser state.
#[derive(Debug, Clone)]
pub struct NSProjectBrowserState {
    /// Root folders in the navigator tree.
    pub folders: Vec<NSProjectFolder>,
    /// Currently selected folder path.
    pub active_folder: String,
    /// Files in the currently active folder.
    pub files: Vec<NSProjectFile>,
    /// Currently selected file index (if any).
    pub selected_file_index: Option<usize>,
    /// File inspector data for the selected file.
    pub file_inspector: Option<NSFileInspectorData>,
    /// Display mode for the file grid.
    pub display_mode: NSBrowserDisplayMode,
    /// Search filter query.
    pub search_query: String,
}

/// Display mode for the file grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSBrowserDisplayMode {
    Grid,
    List,
    Tile,
}

impl Default for NSProjectBrowserState {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            active_folder: String::new(),
            files: Vec::new(),
            selected_file_index: None,
            file_inspector: None,
            display_mode: NSBrowserDisplayMode::Grid,
            search_query: String::new(),
        }
    }
}
