//! Graph Editor host definitions matching Blender 5.2 `SpaceGraph` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEditorConfig {
    pub auto_snap: u8,
    pub show_handles: bool,
    pub show_seconds: bool,
}

impl Default for GraphEditorConfig {
    fn default() -> Self {
        Self {
            auto_snap: 1, // Frame snap
            show_handles: true,
            show_seconds: false,
        }
    }
}
