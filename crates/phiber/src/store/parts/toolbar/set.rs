//! Toolbar set - host-to-toolbar association and tool arrays.

use super::ToolEntry;
use super::def;
use crate::store::hosts::BHostType;

pub static VIEW3D_TOOLS: &[ToolEntry] = &[
    ToolEntry {
        idx: def::TOOL_SELECT_BOX_ID,
        label: def::TOOL_SELECT_BOX_LABEL,
        icon: def::TOOL_SELECT_BOX_ICON,
        active: true,
    },
    ToolEntry {
        idx: def::TOOL_SELECT_CIRCLE_ID,
        label: def::TOOL_SELECT_CIRCLE_LABEL,
        icon: def::TOOL_SELECT_CIRCLE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_SELECT_LASSO_ID,
        label: def::TOOL_SELECT_LASSO_LABEL,
        icon: def::TOOL_SELECT_LASSO_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_MOVE_ID,
        label: def::TOOL_MOVE_LABEL,
        icon: def::TOOL_MOVE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_ROTATE_ID,
        label: def::TOOL_ROTATE_LABEL,
        icon: def::TOOL_ROTATE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_SCALE_ID,
        label: def::TOOL_SCALE_LABEL,
        icon: def::TOOL_SCALE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_ANNOTATE_ID,
        label: def::TOOL_ANNOTATE_LABEL,
        icon: def::TOOL_ANNOTATE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_MEASURE_ID,
        label: def::TOOL_MEASURE_LABEL,
        icon: def::TOOL_MEASURE_ICON,
        active: false,
    },
];

pub static IMAGE_TOOLS: &[ToolEntry] = &[
    ToolEntry {
        idx: def::TOOL_UV_SELECT_ID,
        label: def::TOOL_UV_SELECT_LABEL,
        icon: def::TOOL_SELECT_BOX_ICON,
        active: true,
    },
    ToolEntry {
        idx: def::TOOL_UV_MOVE_ID,
        label: def::TOOL_MOVE_LABEL,
        icon: def::TOOL_MOVE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_UV_ROTATE_ID,
        label: def::TOOL_ROTATE_LABEL,
        icon: def::TOOL_ROTATE_ICON,
        active: false,
    },
    ToolEntry {
        idx: def::TOOL_UV_SCALE_ID,
        label: def::TOOL_SCALE_LABEL,
        icon: def::TOOL_SCALE_ICON,
        active: false,
    },
];

/// Tool entries for a given host.
pub fn with_idx(host: BHostType) -> &'static [super::ToolEntry] {
    match host {
        BHostType::View3D | BHostType::Clip => VIEW3D_TOOLS,
        BHostType::Image | BHostType::Uv => IMAGE_TOOLS,
        _ => &[],
    }
}
