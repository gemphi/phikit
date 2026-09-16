//! Asset Browser host definitions matching Blender 5.2 `SpaceFile` (Asset mode) in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct AssetBrowserConfig {
    pub active_catalog: Option<String>,
    pub search_filter: String,
    pub thumbnail_size: u8,
}

impl Default for AssetBrowserConfig {
    fn default() -> Self {
        Self {
            active_catalog: None,
            search_filter: String::new(),
            thumbnail_size: 96,
        }
    }
}
