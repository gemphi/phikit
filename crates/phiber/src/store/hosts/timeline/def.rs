//! Timeline host definitions matching Blender 5.2 `SpaceTime` / `SpaceAction` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineConfig {
    pub current_frame: i32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub is_playing: bool,
    pub show_seconds: bool,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            current_frame: 1,
            start_frame: 1,
            end_frame: 250,
            is_playing: false,
            show_seconds: false,
        }
    }
}
