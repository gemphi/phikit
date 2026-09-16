//! UV Editor host — reuses Blender's `space_image` implementation in UV mode.
//! Same regions as Image: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER, ASSET_SHELF+HEADER.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static TOOL_HEADER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Modebar],
}];
static HEADER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Modebar],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Switcher],
    },
];
static CENTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Toolbar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Custom],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Sidebar],
    },
];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::ToolHeader,
        panes: TOOL_HEADER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
];
