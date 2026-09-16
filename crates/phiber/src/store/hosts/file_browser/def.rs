//! File Browser host definitions matching Blender 5.2 `SpaceFile` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FileDisplayMode {
    Thumbnails = 0,
    ShortList = 1,
    LongList = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileBrowserConfig {
    pub current_directory: String,
    pub display_mode: FileDisplayMode,
    pub thumbnail_size: u8,
}

impl Default for FileBrowserConfig {
    fn default() -> Self {
        Self {
            current_directory: String::new(),
            display_mode: FileDisplayMode::Thumbnails,
            thumbnail_size: 96,
        }
    }
}
