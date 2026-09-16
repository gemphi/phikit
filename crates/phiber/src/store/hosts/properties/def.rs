//! Properties host definitions matching Blender 5.2 `SpaceProperties` in `DNA_space_types.h`.

pub use crate::store::parts::panel::BButsContext;

#[derive(Clone, Debug, PartialEq)]
pub struct PropertiesConfig {
    pub active_tab: BButsContext,
    pub pin_id_name: Option<String>,
    pub search_filter: String,
}

impl Default for PropertiesConfig {
    fn default() -> Self {
        Self {
            active_tab: BButsContext::Render,
            pin_id_name: None,
            search_filter: String::new(),
        }
    }
}
