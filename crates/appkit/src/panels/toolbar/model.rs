//! Top Bar model — data types for the editor toolbar.

/// Playback state for the play/pause/stop controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSPlaybackState {
    Stopped,
    Playing,
    Paused,
}

impl Default for NSPlaybackState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Editor mode (driven by the segmented control).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSEditorMode {
    Scene,
    Shaders,
    Timeline,
}

impl Default for NSEditorMode {
    fn default() -> Self {
        Self::Scene
    }
}

impl NSEditorMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Scene => "Scene",
            Self::Shaders => "Shaders",
            Self::Timeline => "Timeline",
        }
    }

    pub const ALL: &'static [Self] = &[Self::Scene, Self::Shaders, Self::Timeline];
}

/// Panel visibility toggles shown in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NSPanelToggles {
    pub sidebar_visible: bool,
    pub bottom_visible: bool,
    pub inspector_visible: bool,
    pub activity_bar_visible: bool,
}

impl Default for NSPanelToggles {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            bottom_visible: true,
            inspector_visible: true,
            activity_bar_visible: true,
        }
    }
}

/// Full toolbar state.
#[derive(Debug, Clone, Default)]
pub struct NSToolbarState {
    pub playback: NSPlaybackState,
    pub editor_mode: NSEditorMode,
    pub panel_toggles: NSPanelToggles,
    pub search_query: String,
}
