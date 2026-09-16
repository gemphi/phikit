//! UV Editor host definitions matching Blender 5.2 `SpaceImage` (UV mode) in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct UvEditorConfig {
    pub show_stretch: bool,
    pub show_pixel_grid: bool,
    pub zoom: f32,
}

impl Default for UvEditorConfig {
    fn default() -> Self {
        Self {
            show_stretch: false,
            show_pixel_grid: true,
            zoom: 1.0,
        }
    }
}
