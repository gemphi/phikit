//! Default context state - initial values for BContext and BMode.

use super::{BContext, BMode};
use crate::store::hosts::BHostType;
use crate::store::regions::BRegionType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;

pub const DEFAULT_OBJECT_NAME: &str = "Cube";
pub const DEFAULT_TOOL_NAME: &str = "Select Box";
pub const DEFAULT_SCENE_NAME: &str = "Scene";
pub const DEFAULT_VIEW_LAYER_NAME: &str = "ViewLayer";

impl BMode {
    pub fn for_host(host: BHostType) -> Self {
        match host {
            BHostType::View3D => Self::Object,
            BHostType::Image | BHostType::Uv => Self::View,
            BHostType::Text => Self::Edit,
            BHostType::Time => Self::View,
            _ => Self::Default,
        }
    }
}

impl Default for BContext {
    fn default() -> Self {
        Self {
            active_space: BSpaceType::Layout,
            active_zone: BZoneType::Center,
            active_host: BHostType::View3D,
            active_region: BRegionType::Center,
            mode: BMode::Object,
            active_object: Some(DEFAULT_OBJECT_NAME.to_string()),
            selected_objects: vec![DEFAULT_OBJECT_NAME.to_string()],
            active_tool: Some(DEFAULT_TOOL_NAME.to_string()),
            active_scene: DEFAULT_SCENE_NAME.to_string(),
            view_layer: DEFAULT_VIEW_LAYER_NAME.to_string(),
            frame_current: 1,
            playing: false,
        }
    }
}
