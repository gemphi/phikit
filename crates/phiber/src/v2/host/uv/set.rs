//! UV Editor regions, panes and parts — the content of the host.
//!
//! UV Editor host — reuses Blender's `space_image` implementation in UV mode.
//! Same regions as Image: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER, ASSET_SHELF+HEADER.

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

/// `IMAGE_HT_header` in the UV editor: no mode menu (`sima.mode` is UV);
/// while editing, the selection settings before the menus; after the
/// spacer the pivot, snapping and proportional editing, then the image
/// (`template_ID`); at the right end the gizmo and overlay toggles.
static HEADER: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::UvSelect, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::UvTransform, BPartType::ImageBlock],
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
];
