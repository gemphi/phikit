//! Outliner host definitions matching Blender 5.2 `SpaceOutliner` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OutlinerDisplayMode {
    Scenes = 0,
    ViewLayer = 1,
    OrphanedData = 2,
    BlenderFile = 3,
    DataAPI = 4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutlinerConfig {
    pub display_mode: OutlinerDisplayMode,
    pub search_filter: String,
    pub show_restrict_view: bool,
    pub show_restrict_select: bool,
    pub show_restrict_render: bool,
}

impl Default for OutlinerConfig {
    fn default() -> Self {
        Self {
            display_mode: OutlinerDisplayMode::ViewLayer,
            search_filter: String::new(),
            show_restrict_view: true,
            show_restrict_select: true,
            show_restrict_render: true,
        }
    }
}
