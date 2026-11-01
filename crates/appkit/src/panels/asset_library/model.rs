//! Asset library model.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NSAssetCategory {
    All,
    Primitives,
    Lights,
    Materials,
    Audio,
    Custom,
}

#[derive(Debug, Clone)]
pub struct NSAssetItem {
    pub name: &'static str,
    pub icon_id: &'static str,
    pub category: NSAssetCategory,
}

#[derive(Debug, Clone)]
pub struct NSAssetLibraryState {
    pub active_category: NSAssetCategory,
    pub search_query: String,
}

impl NSAssetLibraryState {
    pub fn new() -> Self {
        Self {
            active_category: NSAssetCategory::All,
            search_query: String::new(),
        }
    }
}
