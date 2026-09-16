//! Sidebar set - host-to-sidebar spec association and static tab arrays.

use super::SidebarTab;
use super::def;
use crate::store::hosts::BHostType;

pub static VIEW3D_TABS: &[SidebarTab] = &[
    SidebarTab {
        idx: def::TAB_ITEM_ID,
        label: def::TAB_ITEM_LABEL,
        icon: def::TAB_ITEM_ICON,
    },
    SidebarTab {
        idx: def::TAB_TOOL_ID,
        label: def::TAB_TOOL_LABEL,
        icon: def::TAB_TOOL_ICON,
    },
    SidebarTab {
        idx: def::TAB_VIEW_ID,
        label: def::TAB_VIEW_LABEL,
        icon: def::TAB_VIEW_ICON,
    },
];

pub static NODE_TABS: &[SidebarTab] = &[
    SidebarTab {
        idx: def::TAB_ITEM_ID,
        label: def::TAB_ITEM_LABEL,
        icon: def::TAB_ITEM_ICON,
    },
    SidebarTab {
        idx: def::TAB_TOOL_ID,
        label: def::TAB_TOOL_LABEL,
        icon: def::TAB_TOOL_ICON,
    },
    SidebarTab {
        idx: def::TAB_NODE_ID,
        label: def::TAB_NODE_LABEL,
        icon: def::TAB_NODE_ICON,
    },
];

pub static IMAGE_TABS: &[SidebarTab] = &[
    SidebarTab {
        idx: def::TAB_ITEM_ID,
        label: def::TAB_ITEM_LABEL,
        icon: def::TAB_ITEM_ICON,
    },
    SidebarTab {
        idx: def::TAB_TOOL_ID,
        label: def::TAB_TOOL_LABEL,
        icon: def::TAB_TOOL_ICON,
    },
    SidebarTab {
        idx: def::TAB_VIEW_ID,
        label: def::TAB_VIEW_LABEL,
        icon: def::TAB_VIEW_ICON,
    },
    SidebarTab {
        idx: def::TAB_METADATA_ID,
        label: def::TAB_METADATA_LABEL,
        icon: def::TAB_METADATA_ICON,
    },
];

pub static BUTS_TABS: &[SidebarTab] = &[
    SidebarTab {
        idx: def::TAB_RENDER_ID,
        label: def::TAB_RENDER_LABEL,
        icon: def::TAB_RENDER_ICON,
    },
    SidebarTab {
        idx: def::TAB_OUTPUT_ID,
        label: def::TAB_OUTPUT_LABEL,
        icon: def::TAB_OUTPUT_ICON,
    },
    SidebarTab {
        idx: def::TAB_SCENE_ID,
        label: def::TAB_SCENE_LABEL,
        icon: def::TAB_SCENE_ICON,
    },
    SidebarTab {
        idx: def::TAB_WORLD_ID,
        label: def::TAB_WORLD_LABEL,
        icon: def::TAB_WORLD_ICON,
    },
    SidebarTab {
        idx: def::TAB_OBJECT_ID,
        label: def::TAB_OBJECT_LABEL,
        icon: def::TAB_OBJECT_ICON,
    },
    SidebarTab {
        idx: def::TAB_MODIFIERS_ID,
        label: def::TAB_MODIFIERS_LABEL,
        icon: def::TAB_MODIFIERS_ICON,
    },
    SidebarTab {
        idx: def::TAB_MATERIAL_ID,
        label: def::TAB_MATERIAL_LABEL,
        icon: def::TAB_MATERIAL_ICON,
    },
    SidebarTab {
        idx: def::TAB_TEXTURE_ID,
        label: def::TAB_TEXTURE_LABEL,
        icon: def::TAB_TEXTURE_ICON,
    },
];

pub static DEFAULT_TABS: &[SidebarTab] = &[
    SidebarTab {
        idx: def::TAB_ITEM_ID,
        label: def::TAB_ITEM_LABEL,
        icon: def::TAB_ITEM_ICON,
    },
    SidebarTab {
        idx: def::TAB_VIEW_ID,
        label: def::TAB_VIEW_LABEL,
        icon: def::TAB_VIEW_ICON,
    },
];

/// Sidebar tabs for a given host.
pub fn with_idx(host: BHostType) -> &'static [SidebarTab] {
    match host {
        BHostType::Buts => BUTS_TABS,
        BHostType::View3D | BHostType::Clip => VIEW3D_TABS,
        BHostType::Node | BHostType::Compositor | BHostType::Shader | BHostType::GeoNode => {
            NODE_TABS
        }
        BHostType::Image | BHostType::Uv => IMAGE_TABS,
        _ => DEFAULT_TABS,
    }
}
