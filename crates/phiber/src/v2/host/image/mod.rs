//! Image Editor host. Blender ref: `space_image/space_image.cc:1343-`.
//! Regions: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER, ASSET_SHELF, ASSET_SHELF_HEADER.

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
static ASSET_SHELF_HEADER: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::AssetShelf],
}];
static ASSET_SHELF: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::AssetShelf],
}];

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
    BRegionSpec {
        kind: BRegionType::AssetShelfHeader,
        panes: ASSET_SHELF_HEADER,
    },
    BRegionSpec {
        kind: BRegionType::AssetShelf,
        panes: ASSET_SHELF,
    },
];
