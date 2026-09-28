//! Image Editor regions, panes and parts — the content of the host.
//!
//! Image Editor host. Blender ref: `space_image/space_image.cc:1343-`.
//! Regions: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER, ASSET_SHELF, ASSET_SHELF_HEADER.

use crate::v2::host::header;
use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

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
    parts: &[BPartType::AssetShelfHeader],
}];
static ASSET_SHELF: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::AssetShelf],
}];

/// `IMAGE_HT_header`: the editor, its mode and menus on the left; the image
/// data-block after the first spacer; the toggles, render layers and display
/// channels after the second.
static HEADER: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::Modebar, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::ImageBlock],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::ImageDisplay],
    },
];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::ToolHeader,
        panes: header::TOOL,
    },
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER,
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
