//! Image / UV editor host definitions matching Blender 5.2 `SpaceImage` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ImageEditorMode {
    View = 0,
    Paint = 1,
    Mask = 2,
    UV = 3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageEditorConfig {
    pub mode: ImageEditorMode,
    pub image_name: Option<String>,
    pub zoom: f32,
}

impl Default for ImageEditorConfig {
    fn default() -> Self {
        Self {
            mode: ImageEditorMode::View,
            image_name: None,
            zoom: 1.0,
        }
    }
}
