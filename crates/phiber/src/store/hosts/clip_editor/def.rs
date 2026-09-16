//! Movie Clip Editor host definitions matching Blender 5.2 `SpaceClip` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ClipEditorMode {
    Tracking = 0,
    Mask = 1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipEditorConfig {
    pub clip_name: Option<String>,
    pub mode: ClipEditorMode,
    pub show_marker_pattern: bool,
    pub show_marker_search: bool,
}

impl Default for ClipEditorConfig {
    fn default() -> Self {
        Self {
            clip_name: None,
            mode: ClipEditorMode::Tracking,
            show_marker_pattern: true,
            show_marker_search: false,
        }
    }
}
