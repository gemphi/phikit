//! Statusbar host definitions matching Blender 5.2 `SpaceStatusBar` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatusbarConfig {
    pub show_key_hints: bool,
    pub show_memory_stats: bool,
    pub show_scene_stats: bool,
    pub show_version: bool,
}
